use crate::utils::Node;

use crate::messages::{CalculatePowerMessage, send_message, CalculateResponseMessage};
use crate::utils::FriendType;

pub fn calculate_total_power(node: &Node) -> u32 {
    println!("Calculating total power...");
    let mut total_power = 1; // my power is 1
    let mut handles = vec![];
    let mut friend_addresses = vec![];

    // Collect friend addresses (and their indices) before spawning threads
    let friends = node.friends.lock().unwrap();
    let friends_addresses: Vec<String> = {
        friends.iter().map(|f| f.address.clone()).collect()
    };
    drop(friends);
    for friend in friends_addresses.iter() {
        let friend_address_clone = friend.to_string();
        let node_clone = node.clone();
        let handle = std::thread::spawn(move || {
            println!("Querying power from friend: {}", friend_address_clone);
            let message = CalculatePowerMessage {
                from: node_clone.address.clone(),
                to: friend_address_clone.clone(),
            };
            let response = send_message(&message, &node_clone);
            if let Some(response_message) = response {
                if let Some(calc_msg) = response_message.as_any().downcast_ref::<CalculateResponseMessage>() {
                    println!("Received power {} from {}", calc_msg.power, friend_address_clone);
                    return calc_msg.power;
                }
            }
            println!("Failed to get power from {}", friend_address_clone);
            0
        });
        friend_addresses.push(friend.to_string());
        handles.push(handle);
    }

    // Wait for all threads to finish and collect results first
    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        results.push(handle.join());
    }

    // Now lock and update friends after all threads are done
    let mut friends = node.friends.lock().unwrap();
    for (i, result) in results.into_iter().enumerate() {
        let address = &friend_addresses[i];
        if let Some(friend) = friends.iter_mut().find(|f| f.address() == *address) {
            match result {
                Ok(power) => {
                    if power > 0 {
                        total_power += power;
                        friend.friend_type = FriendType::Child;
                        friend.power = power;
                    }
                }
                Err(_) => {
                    println!("Thread panicked while querying friend: {}", friend.address());
                    node.remove_friend(&friend.address());
                }
            }
        }
    }
    total_power
}