use sha2::{Sha256, Digest};
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

pub trait Combinable {
    fn total_combinations(&self) -> usize;
}

#[derive(Debug)]
pub enum PartOfAProblemState {
    NotDistributed,
    Distributed,
}

#[derive(Debug)]
pub struct PartOfAProblem {
    pub start: String,
    pub end: String,
    pub alphabet: String,
    pub hash: String,
    pub state: PartOfAProblemState,
}

impl Combinable for PartOfAProblem {
    fn total_combinations(&self) -> usize {
        let alphabet_size = self.alphabet.len();
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

#[derive(Debug)]
pub struct Problem {
    pub alphabet: String,
    pub start: String,
    pub end: String,
    pub hash: String,
    pub current: String,
}

impl Combinable for Problem {
    fn total_combinations(&self) -> usize {
        let alphabet_size = self.alphabet.len();
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

impl Problem {
    pub fn new(
        alphabet: String,
        start: String,
        end: String,
        hash: String,
    ) -> Self {
        Problem {
            alphabet,
            start: start.clone(),
            end: end.clone(),
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
            if let None = self.next() {
                break;
            }
        }
        None
    }

    /// Helper: convert a string to its index in the given alphabet base
    fn str_to_index(&self, s: &str) -> usize {
        let alphabet_size = self.alphabet.len();
        s.chars().fold(0, |acc, c| {
            acc * alphabet_size + self.alphabet.find(c).unwrap()
        })
    }

    /// Helper: convert an index to a string in the given alphabet base, with minimum length
    fn index_to_str(&self, mut idx: usize, min_len: usize) -> String {
        let alphabet: Vec<char> = self.alphabet.chars().collect();
        let base = alphabet.len();
        let mut chars = Vec::new();
        while idx > 0 {
            chars.push(alphabet[idx % base]);
            idx /= base;
        }
        while chars.len() < min_len {
            chars.push(alphabet[0]);
        }
        chars.reverse();
        chars.iter().collect()
    }

    /// Divide the problem into n parts, each with roughly the same number of combinations
    pub fn divide_into_n(&self, n: usize) -> Vec<PartOfAProblem> {
        let total = self.total_combinations();
        if n == 0 || total == 0 {
            return vec![];
        }
        let num_parts = n.min(total); // never create more parts than total combinations
        let min_len = self.start.len().max(self.end.len());
        let start_idx = self.str_to_index(&self.start);
        let end_idx = self.str_to_index(&self.end);
        let mut parts = Vec::new();
        let mut prev_start = start_idx;
        let mut remaining = total;
        for i in 0..num_parts {
            let part_size = if i == num_parts - 1 {
                remaining
            } else {
                (remaining + (num_parts - i) - 1) / (num_parts - i) // ceil division for fair split
            };
            let part_end = if i == num_parts - 1 {
                end_idx
            } else {
                prev_start + part_size - 1
            };
            if part_end > end_idx {
                break;
            }
            let part = PartOfAProblem {
                start: self.index_to_str(prev_start, min_len),
                end: self.index_to_str(part_end, min_len),
                alphabet: self.alphabet.clone(),
                hash: self.hash.clone(),
                state: PartOfAProblemState::NotDistributed,
            };
            parts.push(part);
            prev_start = part_end + 1;
            if remaining < part_size { break; }
            remaining -= part_size;
            if prev_start > end_idx { break; }
        }
        parts
    }

    pub fn next(&mut self) -> Option<String> {
        if self.current == self.end {
            return None;
        }
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
        // All characters wrapped, increase length by one
        chars.insert(0, self.alphabet.chars().nth(0).unwrap());
        self.current = chars.iter().collect();
        Some(self.current.clone())
    }

    pub fn check_hash(&self, candidate: &str) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(candidate.as_bytes());
        let result = hasher.finalize();
        let hash_string = format!("{:x}", result);
        hash_string == self.hash
    }

    // total_combinations is now in Combinable trait




}