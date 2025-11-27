use std::io::{self, BufRead};
use std::sync::atomic::{AtomicBool};
use crate::Node;
use crate::messages;
use crate::utils::parse_address;

use messages::{PingMessage};
use messages::send_message;
use crate::communication::calculate_total_power;

use crate::problem::{Problem, merge_parts};
use crate::problem::Combinable;


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
    // return if not leader
    if !_node.is_leader() {
        println!("Only leader can initiate solving.");
        return;
    }
    // input parsing
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
    // problem definition
    let mut problem = Problem::new(
        alphabet,
        start,
        end,
        hash,
    );
    println!("Problem defined: {:?}", problem);
    println!("Total combinations to try: {}", problem.total_combinations());

    let mut available_power = _node.friends.lock().unwrap().iter()
        .map(|friend| friend.power)
        .sum::<u32>();
    available_power += _node.power;

    let total_pieces = available_power as usize;
    let parts = problem.divide_into_n(total_pieces);
    println!("Divided into {} parts.", parts.len());
    for (i, part) in parts.iter().enumerate() {
        println!("Part {}: {:?}, combinations: {}", i, part, part.total_combinations());
    }

    // set my part...
    let my_part = parts.get(0);
    _node.solving_part_of_a_problem.lock().unwrap().replace((*my_part.unwrap()).clone());

    // setting parts to friends
    let mut part_index = 1; // 0 is for myself
    let parts = parts; // make mutable for draining
    let friends = _node.friends.lock().unwrap();
    for friend in friends.iter() {
        if friend.is_child() && friend.power > 0 {
            let take_n = friend.power as usize;
            if part_index + take_n > parts.len() + 1 {
                // Not enough parts left
                break;
            }
            let merged = merge_parts(&parts[part_index..part_index+take_n].to_vec());
            // Here you would send the merged part to the friend, e.g. via a message
            println!("Assigning to friend {}: {:?}", friend.address, merged);
            part_index += take_n;
        }
    }
    drop(friends);

    // sending parts...
    send_parts_to_friends(_node);

    
    // solve my part in separate thread
    

    return;
    let stop_flag = AtomicBool::new(false);
    match problem.brute_force(stop_flag) {
        Some(solution) => {
            println!("Solution found: {}", solution);
        }
        None => {
            println!("No solution found.");
        }
    }

}