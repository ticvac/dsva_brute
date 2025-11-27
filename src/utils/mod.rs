use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub enum FriendType {
    Parent,
    Child,
    NotSpecified,
}


#[derive(Debug)]
pub struct Friend {
    pub address: String,
    pub power: u32,
    pub friend_type: FriendType,
}

impl Friend {
    pub fn new(address: String) -> Self {
        Friend {
            address,
            power: 0,
            friend_type: FriendType::NotSpecified,
        }
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn set_type(&mut self, friend_type: FriendType) {
        self.friend_type = friend_type;
    }
}

#[derive(Debug)]
pub enum NodeState {
    IDLE,
    LEADER,
    WORKER,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub address: String,
    pub friends: Arc<Mutex<Vec<Friend>>>,
    pub communicating: Arc<Mutex<bool>>,
    pub state: Arc<Mutex<NodeState>>,
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

    pub fn add_friend(&self, address: String) {
        let mut friends = self.friends.lock().unwrap();
        if !friends.iter().any(|f| f.address() == address) {
            friends.push(Friend::new(address.clone()));
            println!("Added friend: {}", address);
        } else {
            println!("Friend {} already exists", address);
        }
    }

    pub fn is_communicating(&self) -> bool {
        *self.communicating.lock().unwrap()
    }

    pub fn is_idle(&self) -> bool {
        matches!(*self.state.lock().unwrap(), NodeState::IDLE)
    }

    pub fn is_leader(&self) -> bool {
        matches!(*self.state.lock().unwrap(), NodeState::LEADER)
    }

    pub fn set_state_leader(&self) {
        let mut state = self.state.lock().unwrap();
        *state = NodeState::LEADER;
    }

    pub fn set_state_worker(&self) {
        let mut state = self.state.lock().unwrap();
        *state = NodeState::WORKER;
    }

    pub fn set_parent(&self, address: &str) {
        let mut friends = self.friends.lock().unwrap();
        for friend in friends.iter_mut() {
            if friend.address().trim() == address.trim() {
                friend.set_type(FriendType::Parent);
                return;
            }
        }
        panic!("Parent friend with address '{}' not found", address);
    }
}


pub fn parse_address(input: &str) -> String {
    if input.contains(':') {
        input.to_string()
    } else {
        format!("127.0.0.1:{}", input)
    }
}