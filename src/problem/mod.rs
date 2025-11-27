use sha2::{Sha256, Digest};
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};


#[derive(Debug)]
pub struct Problem {
    pub alphabet: String,
    pub start: String,
    pub end: String,
    pub max_length: usize,
    pub hash: String,
    pub current: String,
}

impl Problem {
    pub fn new(
        alphabet: String,
        start: String,
        end: String,
        max_length: usize,
        hash: String,
    ) -> Self {
        Problem {
            alphabet,
            start: start.clone(),
            end: end.clone(),
            max_length,
            hash,
            current: start,
        }
    }

    pub fn brute_force(&mut self, stop_flag: AtomicBool) -> Option<String> {
        loop {
            println!("Trying candidate: {}", self.current);
            if stop_flag.load(Relaxed) {
                return None;
            }
            if self.check_hash(&self.current) {
                return Some(self.current.clone());
            }
            if self.current == self.end {
                break;
            }
            if let None = self.next() {
                break;
            }
        }
        None
    }

    // end checked in brute_force...
    pub fn next(&mut self) -> Option<String> {
        let mut chars: Vec<char> = self.current.chars().collect();
        for i in (0..chars.len()).rev() {
            if let Some(pos) = self.alphabet.find(chars[i]) {
                if pos + 1 < self.alphabet.len() {
                    chars[i] = self.alphabet.chars().nth(pos + 1).unwrap();
                    self.current = chars.iter().collect();
                    return Some(self.current.clone());
                } else {
                    chars[i] = self.alphabet.chars().nth(0).unwrap();
                }
            }
        }
        // All characters wrapped, try to increase length if possible
        if chars.len() < self.max_length {
            chars.insert(0, self.alphabet.chars().nth(0).unwrap());
            self.current = chars.iter().collect();
            return Some(self.current.clone());
        }
        None
    }

    pub fn check_hash(&self, candidate: &str) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(candidate.as_bytes());
        let result = hasher.finalize();
        let hash_string = format!("{:x}", result);
        hash_string == self.hash
    }

    pub fn total_combinations(&self) -> usize {
        let alphabet_size = self.alphabet.len();
        // Helper to convert a string to its index in the given alphabet base
        let str_to_index = |s: &str| -> usize {
            s.chars().fold(0, |acc, c| {
                acc * alphabet_size + self.alphabet.find(c).unwrap()
            })
        };
        let start_index = str_to_index(&self.start);
        let end_index = str_to_index(&self.end);
        if end_index >= start_index {
            end_index - start_index + 1
        } else {
            0
        }
    }

}