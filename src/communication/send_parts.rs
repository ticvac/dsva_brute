use crate::utils::{Node, FriendType};
use crate::messages::{send_message, SolveProblemMessage};
use crate::problem::PartOfAProblemState;
use std::thread;

/// Sends parts of a problem to friends: for each friend of type Child with a not distributed part, send it.
pub fn send_parts_to_friends(node: &Node) {
    // Collect work to do while holding the lock
    let mut to_send = Vec::new();
    {
        let mut friends = node.friends.lock().unwrap();
        for friend in friends.iter_mut() {
            if friend.friend_type == FriendType::Child {
                if let Some(part) = &mut friend.solving_part_of_a_problem {
                    if matches!(part.state, PartOfAProblemState::NotDistributed) {
                        let node_clone = node.clone();
                        let friend_address = friend.address.clone();
                        part.state = PartOfAProblemState::Distributed;
                        let part = part.clone();
                        to_send.push((node_clone, friend_address, part));
                    }
                }
            }
        }
    }

    // Now send messages outside the lock
    let mut handles = Vec::new();
    for (node_clone, friend_address, part) in to_send {
        let handle = thread::spawn(move || {
            let message = SolveProblemMessage {
                from: node_clone.address.clone(),
                to: friend_address,
                alphabet: part.alphabet.clone(),
                start: part.start.clone(),
                end: part.end.clone(),
                hash: part.hash.clone(),
            };
            let _ = send_message(&message, &node_clone);
        });
        handles.push(handle);
    }
    for handle in handles {
        let _ = handle.join();
    }
}
