use std::net::{TcpListener};
use std::io::{Read, Write};
use std::thread;
use crate::Node;
use crate::messages::{AckMessage, Message, parse_message, CalculatePowerMessage, CalculateResponseMessage};
use std::thread::sleep;
use std::time::Duration;

mod calc_power;

pub use calc_power::calculate_total_power;

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

pub fn handle_new_connection(_node: &Node, _message: Box<dyn Message>, stream: &mut std::net::TcpStream) {
    // process new connection and return response message
    let response = AckMessage {
        from: _node.address.clone(),
        to: _message.from().to_string(),
    };

    // calculate power message
    if _message.as_any().is::<CalculatePowerMessage>() {
        println!("Handling CalculatePowerMessage from {}", _message.from());
        let power = 1;
        let response = CalculateResponseMessage {
            from: _node.address.clone(),
            to: _message.from().to_string(),
            power,
        };
        let serialized = response.serialize();
        println!("Sending response: {}", serialized);
        let _ = stream.write_all(serialized.as_bytes());
        return;
    }
    // other -> just ack
    let serialized = response.serialize();
    println!("Sending response: {}", serialized);
    let _ = stream.write_all(serialized.as_bytes());

}