mod args;

use clap::Parser;
use args::Args;
use std::sync::{Arc, Mutex};
use std::net::TcpListener;
use std::thread;
use std::io::{Read, BufRead, self};




fn print_node_info(node: &Node) {
    let friends = node.friends.lock().unwrap();
    let communicating = node.communicating.lock().unwrap();
    let state = node.state.lock().unwrap();
    
    let mut output = String::new();
    output.push_str("=== Node Information ===\n");
    output.push_str(&format!("Node Address: {}\n", node.address));
    output.push_str(&format!("Communicating: {}\n", *communicating));
    output.push_str(&format!("State: {:?}\n", *state));
    output.push_str("Friends:\n");
    for friend in friends.iter() {
        output.push_str(&format!(" - {:?}\n", friend));
    }
    output.push_str("========================\n");
    
    print!("{}", output);
}


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

fn main() {
    let args = Args::parse();
    let my_address = format!("127.0.0.1:{}", args.port);
    
    // loading friends
    let friends: Vec<Friend> = args.friends
        .into_iter()
        .map(|f| {
            let address = if f.contains(':') {
                // Already has IP:port format
                f
            } else {
                // Only port number, assume localhost
                format!("127.0.0.1:{}", f)
            };
            Friend::new(address)
        })
        .collect();

    // create node
    let node = Node::new(my_address, friends);

    // printing node info
    print_node_info(&node);

    // Start command processing thread
    let node_clone = node.clone();
    thread::spawn(move || {
        process_commands(&node_clone);
    });

    // listener on new messages from other nodes
    let listener = TcpListener::bind(&node.address).expect("Failed to bind to port");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let message = read_message(&mut stream.try_clone().unwrap());
                println!("Received message: {:?}", message);

                // Check if communication is enabled
                if !*node.communicating.lock().unwrap() {
                    print!("Ignored message {:?} because communication is off.\n", message);
                    continue;
                }
                
                // Handle the connection in a new thread
                let node_clone = node.clone();
                thread::spawn(move || {
                    // handle connections
                });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}

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
            "info" => {
                print_node_info(_node);
            }
            _ => {
                println!("Unknown command: {}", line);
            }
        }
    }
}

#[derive(Debug)]
pub struct Friend {
    address: String,
    power: u32,
    free_power: u32,
}

impl Friend {
    pub fn new(address: String) -> Self {
        Friend {
            address,
            power: 0,
            free_power: 0,
        }
    }
}

#[derive(Debug)]
pub enum NodeState {
    IDLE,
    COMPUTING {
        parent: String,
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    address: String,
    friends: Arc<Mutex<Vec<Friend>>>,
    communicating: Arc<Mutex<bool>>,
    state: Arc<Mutex<NodeState>>,
}

impl Node {
    pub fn new(address: String, friends: Vec<Friend>) -> Self {
        Node {
            address,
            friends: Arc::new(Mutex::new(friends)),
            communicating: Arc::new(Mutex::new(true)),
            state: Arc::new(Mutex::new(NodeState::IDLE)),
        }
    }
    
}