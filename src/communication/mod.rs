use std::net::{TcpListener};
use std::io::{Read, Write};
use std::thread;
use crate::Node;
use crate::messages::{AckMessage, CalculatePowerMessage, CalculateResponseMessage, Message, PingMessage, parse_message, SolveProblemMessage};
use std::thread::sleep;
use std::time::Duration;
use crate::problem::{merge_parts, Problem, Combinable};


mod calc_power;
mod send_parts;

pub use calc_power::calculate_total_power;
pub use send_parts::send_parts_to_friends;


fn read_message(_stream: &mut std::net::TcpStream) -> Option<String> {
    let mut buffer = [0; 1024];
    match _stream.read(&mut buffer) {
        Ok(size) if size > 0 => {
            let message = String::from_utf8_lossy(&buffer[..size]).to_string();
            Some(message)
        }
        _ => None,
    }
}

pub fn listen(node: Node) {
    // listener on new messages from other nodes
    let listener = TcpListener::bind(&node.address).expect("Failed to bind to port");
    for stream in listener.incoming() {
        // Check if communication is enabled before accepting connections
        if !*node.communicating.lock().unwrap() {
            println!("Communication is off, not accepting connections");
            sleep(Duration::from_millis(100));
            continue;
        }

        match stream {
            Ok(mut stream) => {
                // Read the incoming message
                if let Some(message) = read_message(&mut stream) {
                    println!("Received message: {:?}", message);
                    let node_clone = node.clone();
                    let mut stream_clone = stream.try_clone().unwrap();
                    let message_string = message.clone();
                    // Handle the connection in a new thread
                    thread::spawn(move || {
                        println!("Handling new connection...");
                        if let Some(msg) = parse_message(&message_string) {
                            handle_new_connection(&node_clone, msg, &mut stream_clone);
                        } else {
                            eprintln!("Failed to parse incoming message: {}", message_string);
                        }
                    });
                } else {
                    eprintln!("Failed to read message from stream");
                }
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}

// every new connection in separate thread...
fn handle_new_connection(_node: &Node, _message: Box<dyn Message>, stream: &mut std::net::TcpStream) {
    // process new connection and return response message
    
    // calculate power message
    if _message.as_any().is::<CalculatePowerMessage>() {
        handle_calculate_connection(_node, _message, stream);
        return;
    } else if _message.as_any().is::<PingMessage>() {
        _node.add_friend(_message.from().to_string());
    } else if _message.as_any().is::<SolveProblemMessage>() {
        handle_solve_message(_node, _message.clone_box(), stream);
    }
    // always send ack at the end
    send_acknowledgment(_node, _message, stream);
}

fn send_acknowledgment(_node: &Node, _message: Box<dyn Message>, stream: &mut std::net::TcpStream) {
    let response = AckMessage {
        from: _node.address.clone(),
        to: _message.from().to_string(),
    };
    let serialized = response.serialize();
    println!("Sending acknowledgment: {}", serialized);
    let _ = stream.write_all(serialized.as_bytes());
}


fn handle_calculate_connection(_node: &Node, _message: Box<dyn Message>, stream: &mut std::net::TcpStream) {
    // if not idle -> will not work
    if !_node.is_idle() {
        send_acknowledgment(_node, _message, stream);
        return;
    }
    _node.set_state_worker();
    // set parent
    _node.set_parent(_message.from());

    let power = calculate_total_power(_node);

    let response = CalculateResponseMessage {
        from: _node.address.clone(),
        to: _message.from().to_string(),
        power,
    };
    let serialized = response.serialize();
    println!("Sending response: {}", serialized);
    let _ = stream.write_all(serialized.as_bytes());
}


fn handle_solve_message(_node: &Node, _message: Box<dyn Message>, _stream: &mut std::net::TcpStream) {
    let problem_message = _message.as_any().downcast_ref::<SolveProblemMessage>().unwrap();
    println!("Received solve problem message: {:?}", problem_message);
    let problem = Problem::new(
        problem_message.alphabet.clone(),
        problem_message.start.clone(),
        problem_message.end.clone(),
        problem_message.hash.clone(),
    );
    let mut available_power = _node.friends.lock().unwrap().iter().filter(|friend| friend.is_child()).map(|friend| friend.power).sum::<u32>();
    available_power += _node.power;
    let parts = problem.divide_into_n(available_power as usize);
    println!("Divided into {} parts.", parts.len());
    for (i, part) in parts.iter().enumerate() {
        println!("Part {}: {:?}, combinations: {}", i, part, part.total_combinations());
    }
    assign_parts_to_self_and_friends(_node, parts);
    send_parts_to_friends(_node);
    // (Solving own part happens in this thread)
    println!("WORKER started solving problem...");
}


// Assign parts to self and friends, shared for both commands and communication
pub fn assign_parts_to_self_and_friends(_node: &crate::Node, parts: Vec<crate::problem::PartOfAProblem>) {
    // Assign my part
    if let Some(my_part) = parts.get(0) {
        _node.solving_part_of_a_problem.lock().unwrap().replace(my_part.clone());
    }
    // Assign parts to friends
    let mut part_index = 1; // 0 is for myself
    let mut friends = _node.friends.lock().unwrap();
    for friend in friends.iter_mut() {
        if friend.is_child() && friend.power > 0 {
            let take_n = friend.power as usize;
            if part_index + take_n > parts.len() + 1 {
                break;
            }
            let merged = merge_parts(&parts[part_index..part_index+take_n].to_vec());
            println!("Assigning to friend {:?} part: {:?}, total {:?}", friend, merged, merged.total_combinations());
            friend.solving_part_of_a_problem.replace(merged);
            part_index += take_n;
        }
    }
}