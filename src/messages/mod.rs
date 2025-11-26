use std::net::TcpStream;
use std::io::Write;
use std::time::Duration;
use crate::Node;

pub trait Message {
    fn from(&self) -> &str;

    fn to(&self) -> &str;

    fn serialize(&self) -> String;

    fn deserialize(s: &str) -> Self where Self: Sized;
}

pub fn send_message<T: Message>(message: &T, node: &Node) {
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
            // Set write timeout as well
            let _ = stream.set_write_timeout(Some(Duration::from_secs(3)));
            let serialized = message.serialize();
            if let Err(e) = stream.write_all(serialized.as_bytes()) {
                eprintln!("Failed to write to {}: {}", message.to(), e);
                node.remove_friend(message.to());
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
    pub command: String,
}

impl Message for PingMessage {
    fn from(&self) -> &str {
        &self.from
    }

    fn to(&self) -> &str {
        &self.to
    }

    fn serialize(&self) -> String {
        format!("PING|{}|{}|{}", self.from, self.to, self.command)
    }

    fn deserialize(s: &str) -> Self {
        let parts: Vec<&str> = s.splitn(4, '|').collect();
        PingMessage {
            from: parts[1].to_string(),
            to: parts[2].to_string(),
            command: parts[3].to_string(),
        }
    }
}