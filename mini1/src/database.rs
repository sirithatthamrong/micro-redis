use std::collections::{HashMap, VecDeque};
use crate::command::Command;
use std::fmt;

#[derive(Debug)]
#[derive(Clone)]
pub enum Data {
    Scalar(String),
    List(VecDeque<String>),
}

// Implement the Display trait for Data (Debugging purposes)
impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Data::Scalar(s) => write!(f, "{}", s),
            Data::List(l) => write!(f, "{:?}", l),
        }
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Database {
    pub data: HashMap<String, Data>,
    pub queue: VecDeque<Command>,
}

#[allow(dead_code)]
impl Database {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            queue: VecDeque::new(),
        }
    }
}