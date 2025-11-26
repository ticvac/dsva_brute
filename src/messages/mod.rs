use std::net::TcpStream;
use std::io::{Write, Read};
use std::time::Duration;
use crate::Node;

pub fn parse_message(s: &str) -> Option<Box<dyn Message>> {
    let parts: Vec<&str> = s.splitn(4, '|').collect();
    match parts[0] {
        "PING" => Some(Box::new(PingMessage {
            from: parts[1].to_string(),
            to: parts[2].to_string(),
        })),
        "ACK" => Some(Box::new(AckMessage {
            from: parts[1].to_string(),
            to: parts[2].to_string(),
        })),
        _ => None,
    }
}

pub trait Message {
    fn from(&self) -> &str;
    fn to(&self) -> &str;
    fn serialize(&self) -> String;
}

pub fn send_message<T: Message>(message: &T, node: &Node) {
    // Check if communication is enabled
    if !node.is_communicating() {
        eprintln!("Cannot send message, communication is off");
        return;
    }

    // Prevent sending to same node
    if message.to() == node.address {
        eprintln!("Cannot send message to self: {}", message.to());
        return;
    }

    // Prevent sending to non-friends
    if !node.is_friend(message.to()) {
        eprintln!("Cannot send message to non-friend: {}", message.to());
        return;
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
                return;
            }

            // Wait for a response message and parse it
            let mut buffer = [0u8; 1024];
            match stream.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    let response = String::from_utf8_lossy(&buffer[..n]);
                    if parse_message(&response).is_some() {
                        println!("Received valid message from {}", message.to());
                    } else {
                        eprintln!("Failed to parse response from {}: {}", message.to(), response);
                        node.remove_friend(message.to());
                    }
                }
                Ok(_) | Err(_) => {
                    eprintln!("No response received from {}", message.to());
                    node.remove_friend(message.to());
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to connect to {}: {}", message.to(), e);
            node.remove_friend(message.to());
        }
    }
}

#[derive(Debug)]
pub struct PingMessage {
    pub from: String,
    pub to: String,
}

impl Message for PingMessage {
    fn from(&self) -> &str {
        &self.from
    }

    fn to(&self) -> &str {
        &self.to
    }

    fn serialize(&self) -> String {
        format!("PING|{}|{}", self.from, self.to)
    }
}

pub struct AckMessage {
    pub from: String,
    pub to: String,
}

impl Message for AckMessage {
    fn from(&self) -> &str {
        &self.from
    }

    fn to(&self) -> &str {
        &self.to
    }

    fn serialize(&self) -> String {
        format!("ACK|{}|{}", self.from, self.to)
    }
}