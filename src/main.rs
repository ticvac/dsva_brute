mod args;
mod commands;
mod communication;
mod utils;

use commands::process_commands;
use communication::listen;
use args::Args;
use utils::Node;
use utils::Friend;

use clap::Parser;
use std::thread;


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
    node.print_info();

    // Start command processing thread
    let node_clone = node.clone();
    thread::spawn(move || {
        process_commands(&node_clone);
    });

    listen(node);
}
