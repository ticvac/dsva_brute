mod args;
mod commands;
mod communication;
mod utils;
mod messages;

use commands::process_commands;
use communication::listen;
use args::Args;
use utils::Node;
use utils::Friend;
use utils::parse_address;

use clap::Parser;
use std::thread;


fn main() {
    let args = Args::parse();
    let my_address = parse_address(&format!("{}", args.port));
    
    // loading friends
    let friends: Vec<Friend> = args.friends
        .into_iter()
        .map(|f| {
            let address = parse_address(&f);
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
