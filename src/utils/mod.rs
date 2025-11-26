use std::sync::{Arc, Mutex};

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

    pub fn address(&self) -> &str {
        &self.address
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
    pub address: String,
    friends: Arc<Mutex<Vec<Friend>>>,
    pub communicating: Arc<Mutex<bool>>,
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
    
    pub fn print_info(&self) {
        let friends = self.friends.lock().unwrap();
        let communicating = self.communicating.lock().unwrap();
        let state = self.state.lock().unwrap();

        let mut output = String::new();
        output.push_str("=== Node Information ===\n");
        output.push_str(&format!("Node Address: {}\n", self.address));
        output.push_str(&format!("Communicating: {}\n", *communicating));
        output.push_str(&format!("State: {:?}\n", *state));
        output.push_str("Friends:\n");
        for friend in friends.iter() {
            output.push_str(&format!(" - {:?}\n", friend));
        }

        output.push_str("========================\n");

        print!("{}", output);
    }

    pub fn is_friend(&self, address: &str) -> bool {
        let friends = self.friends.lock().unwrap();
        friends.iter().any(|f| f.address() == address)
    }

    pub fn remove_friend(&self, address: &str) {
        let mut friends = self.friends.lock().unwrap();
        friends.retain(|f| f.address() != address);
        println!("Removed friend: {}", address);
    }

    pub fn is_communicating(&self) -> bool {
        *self.communicating.lock().unwrap()
    }
}


pub fn parse_address(input: &str) -> String {
    if input.contains(':') {
        input.to_string()
    } else {
        format!("127.0.0.1:{}", input)
    }
}