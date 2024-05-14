use std::{collections::{HashMap, VecDeque}, sync::{Arc, Mutex}};
use tokio::time::{timeout, Duration};

use crate::database::{Database, Data};

// SELECT command
pub async fn execute_select_cmd(index: u8, db: &Arc<Mutex<HashMap<u8, Database>>>, selected_db: &mut u8) -> Result<String, &'static str> {
    *selected_db = index;
    println!("Selected DB: {:?}", selected_db);

    let mut db = db.lock().unwrap();

    if db.contains_key(selected_db) {
        Ok("+OK\r\n".to_string())
    } else {
        db.insert(*selected_db, Database { data: HashMap::new(), queue: VecDeque::new() });
        Ok("+OK\r\n".to_string())
    }
}

// GET command
pub fn execute_get_cmd(key: String, db: &Arc<Mutex<HashMap<u8, Database>>>, selected_db: &mut u8) -> Result<String, &'static str> {
    let db = db.lock().unwrap();
    let cur_db = db.get(selected_db).ok_or("Database not found")?;
    println!("Current DB: {:?}", cur_db);
    println!("Key: {:?}", key);

    match cur_db.data.get(&key) {
        Some(value) => Ok(format!("+{}\r\n", value)),
        None => Ok("$-1\r\n".to_string()), // RESP2 Null Bulk String for non-existent key
    }
}

// SET command
pub fn execute_set_cmd(key: String, value: String, db: &Arc<Mutex<HashMap<u8, Database>>>, selected_db: &mut u8) -> Result<String, &'static str> {
    let mut db = db.lock().unwrap();
    let cur_db = db.get_mut(selected_db).ok_or("Database not found")?;
    println!("Current DB: {:?}", cur_db);

    cur_db.data.insert(key, Data::Scalar(value));
    println!("Updated DB: {:?}", cur_db);

    Ok("+OK\r\n".to_string())
}

// PING command
pub fn execute_ping_cmd(message: Option<String>) -> Result<String, &'static str> {
    let response = match message {
        Some(msg) => format!("+{}\r\n", msg),
        None => "+PONG\r\n".to_string(),
    };

    Ok(response)
}

// EXISTS command
pub fn execute_exists_cmd(keys: Vec<String>, db: &Arc<Mutex<HashMap<u8, Database>>>, selected_db: &mut u8) -> Result<String, &'static str> {
    let db = db.lock().unwrap();
    let cur_db = db.get(selected_db).ok_or("Database not found")?;

    let mut count = 0;
    for key in keys {
        if cur_db.data.contains_key(&key) {
            count += 1;
        }
    }
    println!("Count: {:?}", count);

    Ok(format!(":{}\r\n", count))
}

// RPUSH command
#[allow(unreachable_patterns)]
pub fn execute_rpush_cmd(key: String, values: Vec<String>, db: &Arc<Mutex<HashMap<u8, Database>>>, selected_db: &mut u8) -> Result<String, &'static str> {
    let mut db = db.lock().unwrap();
    let cur_db = db.get_mut(selected_db).ok_or("Database not found")?;
    println!("Current DB: {:?}", cur_db);

    let list_len;
    {
        let list = cur_db.data.entry(key.clone()).or_insert(Data::List(VecDeque::new()));
        if let Data::List(list) = list {
            // If it's a list, push the values into it
            for value in values {
                list.push_back(value);
            }
            list_len = list.len();
        } else {
            // If it's not a list, return an error
            return Err("Key is not a list");
        }
    }
    println!("Updated DB: {:?}", cur_db);

    Ok(format!(":{}\r\n", list_len))
}

// LPUSH command
#[allow(unreachable_patterns)]
pub fn execute_lpush_cmd(key: String, values: Vec<String>, db: &Arc<Mutex<HashMap<u8, Database>>>, selected_db: &mut u8) -> Result<String, &'static str> {
    let mut db = db.lock().unwrap();
    let cur_db = db.get_mut(selected_db).ok_or("Database not found")?;
    println!("Current DB: {:?}", cur_db);

    let list_len;
    {
        let list = cur_db.data.entry(key.clone()).or_insert(Data::List(VecDeque::new()));
        if let Data::List(list) = list {
            // If it's a list, push the values into it
            for value in values {
                list.push_front(value);
            }
            list_len = list.len();
        } else {
            // If it's not a list, return an error
            return Err("Key is not a list");
        }
    }
    println!("Updated DB: {:?}", cur_db);

    Ok(format!(":{}\r\n", list_len))
}

// BLPOP command
pub async fn execute_blpop_cmd(keys: Vec<String>, timeout_duration: f64, db: &Arc<Mutex<HashMap<u8, Database>>>, selected_db: &mut u8) -> Result<String, &'static str> {
    let timeout_duration = Duration::from_secs_f64(timeout_duration);

    for key in &keys {
        let mut db_lock = db.lock().unwrap();
        let cur_db = db_lock.get_mut(selected_db).ok_or("Database not found")?;

        if let Some(data) = cur_db.data.get_mut(key) {
            if let Data::List(list) = data {
                if let Some(value) = list.pop_front() {
                    drop(db_lock);
                    return Ok(format!("*2\r\n${}\r\n{}\r\n${}\r\n{}\r\n", key.len(), key, value.len(), value));
                }
            }
        }
        drop(db_lock);
    }

    let result = timeout(timeout_duration, async {
        loop {
            for key in &keys {
                let mut db_lock = db.lock().unwrap();
                let cur_db = db_lock.get_mut(selected_db).ok_or("Database not found")?;

                if let Some(data) = cur_db.data.get_mut(key) {
                    if let Data::List(list) = data {
                        if !list.is_empty() {
                            if let Some(value) = list.pop_front() {
                                drop(db_lock);
                                return Ok(format!("*2\r\n${}\r\n{}\r\n${}\r\n{}\r\n", key.len(), key, value.len(), value));
                            }
                        }
                    }
                }
                drop(db_lock);
            }
            tokio::time::sleep(Duration::from_secs_f64(0.1)).await;
        }
    }).await;

    match result {
        Ok(response) => response,
        Err(_) => Err("Timeout"),
    }
}

// BRPOP command
pub async fn execute_brpop_cmd(keys: Vec<String>, timeout_duration: f64, db: &Arc<Mutex<HashMap<u8, Database>>>, selected_db: &mut u8) -> Result<String, &'static str> {
    let timeout_duration = Duration::from_secs_f64(timeout_duration);

    for key in &keys {
        let mut db_lock = db.lock().unwrap();
        let cur_db = db_lock.get_mut(selected_db).ok_or("Database not found")?;

        if let Some(data) = cur_db.data.get_mut(key) {
            if let Data::List(list) = data {
                if let Some(value) = list.pop_back() {
                    drop(db_lock);
                    return Ok(format!("*2\r\n${}\r\n{}\r\n${}\r\n{}\r\n", key.len(), key, value.len(), value));
                }
            }
        }
        drop(db_lock);
    }

    let result = timeout(timeout_duration, async {
        loop {
            for key in &keys {
                let mut db_lock = db.lock().unwrap();
                let cur_db = db_lock.get_mut(selected_db).ok_or("Database not found")?;

                if let Some(data) = cur_db.data.get_mut(key) {
                    if let Data::List(list) = data {
                        if !list.is_empty() {
                            if let Some(value) = list.pop_back() {
                                drop(db_lock);
                                return Ok(format!("*2\r\n${}\r\n{}\r\n${}\r\n{}\r\n", key.len(), key, value.len(), value));
                            }
                        }
                    }
                }
                drop(db_lock);
            }
            tokio::time::sleep(Duration::from_secs_f64(0.1)).await;
        }
    }).await;

    match result {
        Ok(response) => response,
        Err(_) => Err("Timeout"),
    }
}