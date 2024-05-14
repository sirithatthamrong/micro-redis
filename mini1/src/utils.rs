use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::{command::Command, database::Database, execute_cmd::{*}};

static MAX_DATABASES: u8 = 15;
static MAX_KEYS: usize = 5;

// Parse the incoming request
pub(crate) fn parse_request(request: &str) -> Result<(Command, Vec<String>), &'static str> {
    println!("----------------- Parse Request -----------------");

    let mut iter = request.split("\r\n");
    // println!("Iter: {:?}", iter);
    let header = iter.next().ok_or("Empty request")?;
    // println!("Header: {:?}", header);

    if !header.starts_with("*") {
        return Err("Invalid request format");
    }

    let count: usize = header[1..].parse().map_err(|_| "Invalid request format")?;
    let mut parts = Vec::with_capacity(count);

    for _ in 0..count {
        let length_header = iter.next().ok_or("Invalid request format")?;

        if !length_header.starts_with("$") {
            return Err("Invalid request format");
        }

        let length: usize = length_header[1..].parse().map_err(|_| "Invalid request format")?;
        let arg = iter.next().ok_or("Invalid request format")?;

        if arg.len() != length {
            return Err("Invalid request format");
        }

        parts.push(arg.to_string());
    }

    let cmd = parts.remove(0);
    println!("Command: {:?}", cmd);
    println!("Parts: {:?}", parts);

    let command = match cmd.to_uppercase().as_str() {
        "SELECT" => {
            if parts.len() != 1 { return Err("Syntax error. Usage: SELECT <index>"); }
            let index: u8 = parts[0].parse().map_err(|_| "Invalid database index")?;
            if index > MAX_DATABASES { return Err("Invalid database index"); }
            Command::Select(index)
        },
        "GET" => {
            if parts.len() != 1 { return Err("Syntax error. Usage: GET <key>"); }
            Command::Get(parts[0].clone())
        },
        "SET" => {
            if parts.len() != 2 { return Err("Syntax error. Usage: SET <key> <value>"); }
            Command::Set(parts[0].clone(), parts[1].clone())
        },
        "PING" => {
            Command::Ping(parts.first().map(|s| s.clone()))
        },
        "EXISTS" => {
            if parts.is_empty() { return Err("Syntax error. Usage: EXISTS <key> [<key> ...]"); }
            Command::Exists(parts.clone())
        },   
        "RPUSH" => {
            if parts.len() < 2 { return Err("Syntax error. Usage: RPUSH <key> <value> [<value> ...]"); }
            Command::Rpush(parts[0].clone(), parts[1..].to_vec())
        },
        "LPUSH" => {
            if parts.len() < 2 { return Err("Syntax error. Usage: LPUSH <key> <value> [<value> ...]"); }
            Command::Lpush(parts[0].clone(), parts[1..].to_vec())
        },
        "BLPOP" => {
            if parts.len() < 2 { return Err("Syntax error. Usage: BLPOP <key> <value> [<value> ...]"); }
            if parts.len() > MAX_KEYS { return Err("Exceeded maximum number of keys (5)"); }
            Command::BLPOP(parts[..parts.len() - 1].to_vec(), parts[parts.len() - 1].parse().map_err(|_| "Invalid timeout")?)
        },
        "BRPOP" => {
            if parts.len() < 2 { return Err("Syntax error. Usage: BRPOP <key> <value> [<value> ...]"); }
            if parts.len() > MAX_KEYS { return Err("Exceeded maximum number of keys (5)"); }
            Command::BRPOP(parts[..parts.len() - 1].to_vec(), parts[parts.len() - 1].parse().map_err(|_| "Invalid timeout")?)
        },
        _ => return Err("Unsupported command"),
    };

    Ok((command, parts))
}

// Execute the parsed command
#[allow(unreachable_patterns)]
#[allow(unused_variables)]
pub async fn execute_command(cmd: Command, db: &Arc<Mutex<HashMap<u8, Database>>>, selected_db: &mut u8, cmd_executed: &mut bool, args: Vec<String>) -> String {
    match cmd {
        Command::Select(index) => {
            if *cmd_executed {
                "-SELECT can only be called at the start of the session\r\n".to_string()
            } else {
                *cmd_executed = true;
                *selected_db = index;
                match execute_select_cmd(index, db, selected_db).await {
                    Ok(response) => response,
                    Err(e) => format!("-{}\r\n", e),
                }
            }
        }
        Command::Get(key) => {
            *cmd_executed = true;
            match execute_get_cmd(key, db, selected_db) {
                Ok(response) => response,
                Err(e) => format!("-{}\r\n", e),
            }
        }
        Command::Set(key, value) => {
            *cmd_executed = true;
            match execute_set_cmd(key, value, db, selected_db) {
                Ok(response) => response,
                Err(e) => format!("-{}\r\n", e),
            }
        }
        Command::Ping(message) => {
            *cmd_executed = true;
            match execute_ping_cmd(message) {
                Ok(response) => response,
                Err(e) => format!("-{}\r\n", e)
            }
        }
        Command::Exists(key) => {
            *cmd_executed = true;
            match execute_exists_cmd(key, db, selected_db) {
                Ok(response) => response,
                Err(e) => format!("-{}\r\n", e),
            }
        },
        Command::Rpush(key, values) => {
            *cmd_executed = true;
            match execute_rpush_cmd(key, values, db, selected_db) {
                Ok(response) => response,
                Err(e) => format!("-{}\r\n", e),
            }
        }
        Command::Lpush(key, values) => {
            *cmd_executed = true;
            match execute_lpush_cmd(key, values, db, selected_db) {
                Ok(response) => response,
                Err(e) => format!("-{}\r\n", e),
            }
        }
        Command::BLPOP(keys, timeout) => {
            *cmd_executed = true;
            let result = execute_blpop_cmd(keys, timeout, db, selected_db).await;
                match result {
                    Ok(response) => response,
                    Err(e) => format!("-{}\r\n", e),
                }
        }
        Command::BRPOP(keys, timeout) => {
            *cmd_executed = true;
            let result = execute_brpop_cmd(keys, timeout, db, selected_db).await;
            match result {
                Ok(response) => response,
                Err(e) => format!("-{}\r\n", e),
            }
        }
        _ => "Unsupported command".to_string(),
    }
}

