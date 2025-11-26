pub mod send_message;


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