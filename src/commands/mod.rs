use std::io::{self, BufRead};
use crate::Node;
use crate::messages;
use crate::utils::parse_address;

use messages::{PingMessage, send_message};


pub fn process_commands(_node: &Node) {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l.trim().to_string(),
            Err(_) => continue,
        };
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();

        match parts[0] {
            "die" => {
                println!("Shutting down node...");
                std::process::exit(0);
            }
            "info" => {
                _node.print_info();
            }
            "ping" => {
                handle_ping_command(_node, parts);
            }
            "comm" => {
                let mut comm = _node.communicating.lock().unwrap();
                *comm = !*comm;
                println!("Communicating set to {}", *comm);
            }
            _ => {
                println!("Unknown command: {}", line);
            }
        }
    }
}

fn handle_ping_command(_node: &Node, parts: Vec<&str>) {
    if parts.len() < 2 {
        println!("Usage: ping <address/port>");
        return;
    }
    // construct address
    let address = parse_address(parts[1]);
    // send ping message
    let message = PingMessage {
        from: _node.address.clone(),
        to: address,
        command: "ping".to_string(),
    };
    send_message(&message, _node);
}