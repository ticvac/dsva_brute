use std::net::{TcpListener};
use std::io::{Read, Write};
use std::thread;
use crate::Node;
use crate::messages::{AckMessage, CalculatePowerMessage, CalculateResponseMessage, Message, PingMessage, parse_message, SolveProblemMessage, SolveResponseMessage, send_message};
use std::thread::sleep;
use std::time::Duration;
use crate::problem::{Combinable, Problem, merge_parts, update_state_of_parts};
use crate::problem::PartOfAProblemState;
use crate::utils::{NodeState};

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
        handle_solve_message(_node, _message.clone_box());
    } else if _message.as_any().is::<SolveResponseMessage>() {
        handle_solve_response_message(_node, _message.clone_box());
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


fn handle_solve_message(_node: &Node, _message: Box<dyn Message>) {
    let problem_message = _message.as_any().downcast_ref::<SolveProblemMessage>().unwrap();
    println!("Received solve problem message: {:?}", problem_message);
    let problem = Problem::new_from_solve_message(problem_message);
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
    // need thread to send ack...
    let problem_part = _node.solving_part_of_a_problem.lock().unwrap().as_ref().unwrap().clone();
    _node.stop_flag.store(false, std::sync::atomic::Ordering::SeqCst);
    let stop_flag = _node.stop_flag.clone();
    let my_address = _node.address.to_string();
    let parent_address = _node.get_parent_address();
    let node_clone = _node.clone();
    std::thread::spawn(move || {
        let mut problem = Problem::new_from_part(&problem_part);
        match problem.brute_force(&stop_flag) {
            Some(solution) => {
                println!("Solution found: {}", solution);
                let response = SolveResponseMessage {
                    from: my_address,
                    to: parent_address,
                    start: problem_part.start.clone(),
                    end: problem_part.end.clone(),
                    solution: Some(solution),
                    space_searched: true,
                };
                send_message(&response, &node_clone);
            }
            None => {
                println!("No solution found in my part.");
                let space_searched = !stop_flag.load(std::sync::atomic::Ordering::SeqCst);
                let response = SolveResponseMessage {
                    from: my_address,
                    to: parent_address,
                    start: problem_part.start.clone(),
                    end: problem_part.end.clone(),
                    solution: None,
                    space_searched,
                };
                send_message(&response, &node_clone);
            }
        }
        stop_flag.store(true, std::sync::atomic::Ordering::SeqCst);
    });
}


// Assign parts to self and friends, shared for both commands and communication
pub fn assign_parts_to_self_and_friends(_node: &Node, parts: Vec<crate::problem::PartOfAProblem>) {
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

pub fn handle_solve_response_message(node: &Node, _message: Box<dyn Message>) {
    let solve_response = _message.as_any().downcast_ref::<SolveResponseMessage>().unwrap();
    if !node.is_leader() {
        // Forward the message to the parent (who will forward to leader)
        let parent_address = node.get_parent_address();
        let mut forward_message = solve_response.clone();
        // Set 'from' to this node, 'to' to parent
        forward_message.from = node.address.clone();
        forward_message.to = parent_address.clone();
        send_message(&forward_message, node);
        return;
    }
    
    print!("Leader handling solve response message...\n");
    println!("Received solve response: {:?}", solve_response);

    if solve_response.solution.is_some() {
        println!("Solution found by a worker: {}", solve_response.solution.as_ref().unwrap());
        node.stop_flag.store(true, std::sync::atomic::Ordering::SeqCst);
        // TODO inform all nodes to stop...
        return;
    }
    
    let updated_part = crate::problem::PartOfAProblem {
        alphabet: String::new(), // not needed here
        start: solve_response.start.clone(),
        end: solve_response.end.clone(),
        hash: String::new(), // not needed here
        state: if solve_response.space_searched {
            PartOfAProblemState::SearchedAndNotFound
        } else {
            PartOfAProblemState::NotDistributed
        },
    };
    

    {
        let mut state = node.state.lock().unwrap();
        if let NodeState::LEADER { problem: _, parts: leader_parts } = &mut *state {
            println!("Updating leader's parts with response...");
            println!("Before update: {:?}", leader_parts);
            update_state_of_parts(leader_parts, &updated_part);
            println!("After update: {:?}", leader_parts);
        }
    }
}