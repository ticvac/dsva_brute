use std::net::{TcpListener};
use std::io::{Read, Write};
use std::thread;
use crate::Node;


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

pub fn listen(node: Node) {
    // listener on new messages from other nodes
    let listener = TcpListener::bind(&node.address).expect("Failed to bind to port");
    for stream in listener.incoming() {
        // Check if communication is enabled before accepting connections
        if !*node.communicating.lock().unwrap() {
            println!("Communication is off, not accepting connections");
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }

        match stream {
            Ok(mut stream) => {
                let message = read_message(&mut stream);
                println!("Received message: {:?}", message);
                
                // Send ACK response
                if let Err(e) = stream.write_all(b"ACK") {
                    eprintln!("Failed to send ACK: {}", e);
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