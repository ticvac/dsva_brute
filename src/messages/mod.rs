mod send_message;

pub use send_message::send_message;
use core::str;
use std::any::Any;

pub fn parse_message(s: &str) -> Option<Box<dyn Message>> {
    let parts: Vec<&str> = s.splitn(4, '|').collect();
    match parts[0] {
        "PING" => Some(Box::new(PingMessage {
            from: parts[1].to_string(),
            to: parts[2].to_string(),
        })),
        "ACK" => Some(Box::new(AckMessage {
            from: parts[1].to_string(),
            to: parts[2].to_string(),
        })),
        "CALC" => Some(Box::new(CalculatePowerMessage {
            from: parts[1].to_string(),
            to: parts[2].to_string(),
        })),
        "CALC_RESPONSE" => Some(Box::new(CalculateResponseMessage {
            from: parts[1].to_string(),
            to: parts[2].to_string(),
            power: parts[3].parse().unwrap_or(0),
        })),
        "SOLVE" => Some(Box::new(SolveProblemMessage {
            from: parts[1].to_string(),
            to: parts[2].to_string(),
            alphabet: parts[3].to_string(),
            start: parts[4].to_string(),
            end: parts[5].to_string(),
            hash: parts[6].to_string(),
        })),
        _ => None,
    }
}

pub trait Message {
    fn from(&self) -> &str;
    fn to(&self) -> &str;
    fn serialize(&self) -> String;

    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn Message>;
}

#[derive(Debug, Clone)]
pub struct PingMessage {
    pub from: String,
    pub to: String,
}

impl Message for PingMessage {
    fn from(&self) -> &str {
        &self.from
    }

    fn to(&self) -> &str {
        &self.to
    }

    fn serialize(&self) -> String {
        format!("PING|{}|{}", self.from, self.to)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Message> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct AckMessage {
    pub from: String,
    pub to: String,
}

impl Message for AckMessage {
    fn from(&self) -> &str {
        &self.from
    }

    fn to(&self) -> &str {
        &self.to
    }

    fn serialize(&self) -> String {
        format!("ACK|{}|{}", self.from, self.to)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Message> {
        Box::new(self.clone())
    }
}

#[derive(Clone, Debug)]
pub struct CalculatePowerMessage {
    pub from: String,
    pub to: String,
}

impl Message for CalculatePowerMessage {
    fn from(&self) -> &str {
        &self.from
    }

    fn to(&self) -> &str {
        &self.to
    }

    fn serialize(&self) -> String {
        format!("CALC|{}|{}", self.from, self.to)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Message> {
        Box::new(self.clone())
    }
}

#[derive(Clone, Debug)]
pub struct CalculateResponseMessage {
    pub from: String,
    pub to: String,
    pub power: u32,
}

impl Message for CalculateResponseMessage {
    fn from(&self) -> &str {
        &self.from
    }

    fn to(&self) -> &str {
        &self.to
    }

    fn serialize(&self) -> String {
        format!("CALC_RESPONSE|{}|{}|{}", self.from, self.to, self.power)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Message> {
        Box::new(self.clone())
    }
}

#[derive(Clone, Debug)]
pub struct SolveProblemMessage {
    pub from: String,
    pub to: String,
    pub alphabet: String,
    pub start: String,
    pub end: String,
    pub hash: String,
}

impl Message for SolveProblemMessage {
    fn from(&self) -> &str {
        &self.from
    }

    fn to(&self) -> &str {
        &self.to
    }

    fn serialize(&self) -> String {
        format!(
            "SOLVE|{}|{}|{}|{}|{}|{}",
            self.from, self.to, self.alphabet, self.start, self.end, self.hash
        )
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Message> {
        Box::new(self.clone())
    }
}

// Implement Clone for Box<dyn Message>
impl Clone for Box<dyn Message> {
    fn clone(&self) -> Box<dyn Message> {
        self.clone_box()
    }
}