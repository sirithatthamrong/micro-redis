mod command;
mod database;
mod connection;
mod utils;
mod execute_cmd;

use tokio::net::TcpListener;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use crate::{database::Database, connection::handle_connection, utils::{parse_request, execute_command}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    println!("Listening on port {}", listener.local_addr()?.port());

    let db = Arc::new(Mutex::new(HashMap::new()));
    db.lock().unwrap().insert(0, Database { data: HashMap::new(), queue: VecDeque::new() }); // Default namespace

    // Create a channel for sending messages from the connection handlers to the main thread
    let (tx, mut rx) = mpsc::channel(32);

    // Spawn a new task for handling messages from the connection handlers
    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            println!("Received message from connection handler: {}", message);
        }
    });

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("Accepted connection from {}", addr);

        let db_clone = Arc::clone(&db);
        let tx_clone = tx.clone(); // Clone the sender for use in this connection handler
        
        tokio::spawn(async move {
            println!("Spawning a new task for handling the connection");
            handle_connection(&mut socket, &db_clone, tx_clone).await
                .map_err(|e| { eprintln!("Error: {}", e); })
                .ok();
        });
    }
}