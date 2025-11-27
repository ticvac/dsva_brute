use std::io::{self, BufRead};
use crate::Node;
use crate::messages;
use crate::utils::parse_address;

use messages::{PingMessage};
use messages::send_message;
use crate::communication::calculate_total_power;

use crate::problem::{Problem};
use crate::problem::Combinable;

use crate::communication::{send_parts_to_friends, assign_parts_to_self_and_friends};


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
            "connect" => {
                handle_connect_command(_node, parts);
            }
            "cal" => {
                handle_calculate_command(_node);
            }
            "solve" => {
                handle_solve_command(_node, parts);
            }
            _ => {
                println!("Unknown command: {}", line);
            }
        }
    }
}

// can block thread up to timeout in send_message !!!
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
    };
    send_message(&message, _node);
}

fn handle_connect_command(_node: &Node, parts: Vec<&str>) {
    if parts.len() < 2 {
        println!("Usage: connect <address/port>");
        return;
    }
    let address = parse_address(parts[1]);
    println!("Connecting to {}", address);
    // add to friends
    _node.add_friend(address.clone());
    // now ping it
    let address_str = address.to_string();
    handle_ping_command(_node, vec!["ping", &address_str]);
}

fn handle_calculate_command(_node: &Node) {
    if !_node.is_idle() {
        println!("Node is not idle.");
        return;
    }
    println!("Starting calculation...");

    // set to leader
    _node.set_state_leader();
    // calculate power
    let total_power = calculate_total_power(_node);
    println!("Total calculated power: {}", total_power);
}



fn handle_solve_command(_node: &Node, parts: Vec<&str>) {
    if parts.len() < 4 {
        println!("Usage: solve <alphabet> <min_len> <max_len> <target_hash>");
        println!("Example: solve abc 2 3 ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb");
        return;
    }
    if !_node.is_leader() {
        println!("Only leader can initiate solving.");
        return;
    }
    let alphabet = parts[1].to_string();
    let min_length = match parts[2].parse::<usize>() {
        Ok(n) => n,
        Err(_) => {
            println!("Invalid min_length: {}", parts[2]);
            return;
        }
    };
    let max_length = match parts[3].parse::<usize>() {
        Ok(n) => n,
        Err(_) => {
            println!("Invalid max_length: {}", parts[3]);
            return;
        }
    };
    let hash = parts[4].to_string();
    let start = alphabet.chars().next().unwrap().to_string().repeat(min_length);
    let end = alphabet.chars().last().unwrap().to_string().repeat(max_length);
    let problem = Problem::new(alphabet, start, end, hash);
    println!("Problem defined: {:?}", problem);
    println!("Total combinations to try: {}", problem.total_combinations());
    let mut available_power = _node.friends.lock().unwrap().iter().filter(|friend | friend.is_child()).map(|friend| friend.power).sum::<u32>();
    available_power += _node.power;
    let parts = problem.divide_into_n(available_power as usize);
    println!("Divided into {} parts.", parts.len());
    for (i, part) in parts.iter().enumerate() {
        println!("Part {}: {:?}, combinations: {}", i, part, part.total_combinations());
    }
    assign_parts_to_self_and_friends(_node, parts);
    send_parts_to_friends(_node);
    // solve my part in separate thread (not implemented here)
    println!("LEADER started solving problem...");
}