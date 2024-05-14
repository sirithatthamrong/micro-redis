use std::fmt;

#[derive(Debug, Clone)]
pub enum Command {
    Select(u8),
    Get(String),
    Set(String, String),
    Ping(Option<String>),
    Exists(Vec<String>),
    Rpush(String, Vec<String>),
    Lpush(String, Vec<String>),
    BLPOP(Vec<String>, f64),
    BRPOP(Vec<String>, f64),
}

// Implement the Display trait for Command (Debugging purposes)
impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Command::Select(db) => write!(f, "Select database {}", db),
            Command::Get(key) => write!(f, "Get value for key {}", key),
            Command::Set(key, value) => write!(f, "Set value for key {} to {}", key, value),
            Command::Ping(message) => match message {
                Some(msg) => write!(f, "Ping with message {}", msg),
                None => write!(f, "Ping"),
            },
            Command::Exists(key) => write!(f, "Check if key {:?} exists", key),
            Command::Rpush(key, values) => write!(f, "Push values {:?} to key {}", values, key),
            Command::Lpush(key, values) => write!(f, "Push values {:?} to key {}", values, key),
            Command::BLPOP(keys, timeout) => write!(f, "BLPOP on keys {:?} with timeout {}", keys, timeout),
            Command::BRPOP(keys, timeout) => write!(f, "BRPOP on keys {:?} with timeout {}", keys, timeout),
        }
    }
}
