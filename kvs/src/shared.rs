use std::{
    cell::RefCell,
    collections::HashMap,
    io::{self, Write},
    rc::Rc,
};

use serde::{Deserialize, Serialize};

use chorus_lib::core::{ChoreographyLocation, LocationSet};

#[derive(ChoreographyLocation)]
pub struct Client;

#[derive(ChoreographyLocation)]
pub struct Primary;

#[derive(ChoreographyLocation)]
pub struct Backup;

// --- Types ---
pub type L = LocationSet!(Client, Primary, Backup);
pub type State = Rc<RefCell<HashMap<String, String>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Put(String, String),
    Get(String),
}

impl Request {
    pub fn is_mutating(&self) -> bool {
        match self {
            Request::Put(_, _) => true,
            Request::Get(_) => false,
        }
    }
}

pub type Response = Option<String>;

// --- Functions ---

pub fn read_request() -> Request {
    loop {
        print!("Command?\n> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        match parse_request(&input) {
            Some(request) => return request,
            None => println!("Invalid command"),
        }
    }
}

fn parse_request(s: &str) -> Option<Request> {
    let l: Vec<&str> = s.trim().split_whitespace().collect();
    match l.as_slice() {
        ["GET", k] => Some(Request::Get(k.to_string())),
        ["PUT", k, v] => Some(Request::Put(k.to_string(), v.to_string())),
        _ => None,
    }
}

pub fn handle_request(state: &Rc<RefCell<HashMap<String, String>>>, request: &Request) -> Response {
    match request {
        Request::Put(k, v) => {
            state.borrow_mut().insert(k.clone(), v.clone());
            None
        }
        Request::Get(k) => {
            let state = state.borrow();
            let v = state.get(k).cloned();
            v
        }
    }
}
