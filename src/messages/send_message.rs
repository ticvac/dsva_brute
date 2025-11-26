use std::net::TcpStream;
use std::io::{Write, Read};
use std::time::Duration;
use crate::Node;
use crate::messages::parse_message;
use crate::messages::Message;

pub fn send_message<T: Message>(message: &T, node: &Node) -> Option<Box<dyn Message>> {
    // Check if communication is enabled
    if !node.is_communicating() {
        eprintln!("Cannot send message, communication is off");
        return None;
    }

    // Prevent sending to same node
    if message.to() == node.address {
        eprintln!("Cannot send message to self: {}", message.to());
        return None;
    }

    // Prevent sending to non-friends
    if !node.is_friend(message.to()) {
        eprintln!("Cannot send message to non-friend: {}", message.to());
        return None;
    }

    println!("Sending message to {}", message.to());
    
    // Set connection timeout to prevent stalling
    match TcpStream::connect_timeout(
        &message.to().parse().unwrap(),
        Duration::from_secs(3)
    ) {
        Ok(mut stream) => {
            // Set timeouts
            let _ = stream.set_write_timeout(Some(Duration::from_secs(3)));
            let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
            
            let serialized = message.serialize();
            if let Err(e) = stream.write_all(serialized.as_bytes()) {
                eprintln!("Failed to write to {}: {}", message.to(), e);
                node.remove_friend(message.to());
                return None;
            }

            // Wait for a response message and parse it
            let mut buffer = [0u8; 1024];
            match stream.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    let response = String::from_utf8_lossy(&buffer[..n]).to_string();
                    if let Some(parsed_msg) = parse_message(&response) {
                        println!("Received valid response {}", parsed_msg.serialize());
                        Some(parsed_msg)
                    } else {
                        eprintln!("Failed to parse response from {}: {}", message.to(), response);
                        node.remove_friend(message.to());
                        None
                    }
                }
                Ok(_) | Err(_) => {
                    eprintln!("No response received from {}", message.to());
                    node.remove_friend(message.to());
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to connect to {}: {}", message.to(), e);
            node.remove_friend(message.to());
            None
        }
    }
}