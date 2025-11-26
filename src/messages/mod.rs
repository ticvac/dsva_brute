mod send_message;

pub use send_message::send_message;

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
        "CALC_RESPONSE" => Some(Box::new(CalculationResponseMessage {
            from: parts[1].to_string(),
            to: parts[2].to_string(),
            result: parts[3].parse().unwrap_or(0),
        })),
        _ => None,
    }
}

pub trait Message {
    fn from(&self) -> &str;
    fn to(&self) -> &str;
    fn serialize(&self) -> String;
}

#[derive(Debug)]
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
}

#[derive(Debug)]
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
}

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
}

pub struct CalculationResponseMessage {
    pub from: String,
    pub to: String,
    pub result: usize,
}

impl Message for CalculationResponseMessage {
    fn from(&self) -> &str {
        &self.from
    }

    fn to(&self) -> &str {
        &self.to
    }

    fn serialize(&self) -> String {
        format!("CALC_RESPONSE|{}|{}|{}", self.from, self.to, self.result)
    }
}