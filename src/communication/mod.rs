use std::net::{TcpListener};
use std::io::Read;
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