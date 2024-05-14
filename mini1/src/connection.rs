use std::{collections::HashMap, error::Error};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use crate::{database::Database, parse_request, execute_command, command::Command};

pub async fn handle_connection(socket: &mut TcpStream, db: &Arc<Mutex<HashMap<u8, Database>>>, tx: mpsc::Sender<String>) -> Result<(), Box<dyn Error>> {
    let mut buf = [0; 1024];
    let mut selected_db = 0; // Default namespace
    let mut cmd_executed = false;

    loop {
        let bytes_read = socket.read(&mut buf).await?;

        // If the client closed the connection, break the loop
        if bytes_read == 0 { break; }

        let request = std::str::from_utf8(&buf[..bytes_read])?;
        println!("Received: {:?}", request);

        let response = match parse_request(request) {
            Ok((cmd, args)) => {
                // Send a message to the main thread whenever a command is executed
                tx.send(format!("Executed command: {}", cmd)).await.unwrap();

                // Push the command into the queue for the selected database
                push_command(&db, &mut selected_db, (cmd.clone(), args.clone()));

                // Process and execute queued commands for the selected database
                execute_queued_commands(db, &mut selected_db, &mut cmd_executed, args).await
            },
            Err(e) => format!("-{}\r\n", e),
        };

        socket.write_all(response.as_bytes()).await?; // Write the response back to the client
    }

    Ok(())
}

// Process and execute queued commands for the selected database
async fn execute_queued_commands(db: &Arc<Mutex<HashMap<u8, Database>>>, selected_db: &mut u8, cmd_executed: &mut bool, args: Vec<String>) -> String {
    println!("Executing queued commands");
    let mut response = String::new();

    while let Some(cmd) = {
        let mut db_guard = db.lock().unwrap();
        let db_entry = db_guard.get_mut(selected_db).unwrap();
        db_entry.queue.pop_front()
    } {
        response += &execute_command(cmd.clone(), db, selected_db, cmd_executed, args.clone()).await;
    }

    response
}

// Push the command into the queue for the selected database
fn push_command(db: &Arc<Mutex<HashMap<u8, Database>>>, selected_db: &mut u8, command: (Command, Vec<String>)) {
    let (cmd, _) = command;
    db.lock().unwrap().get_mut(selected_db).unwrap().queue.push_back(cmd);
}