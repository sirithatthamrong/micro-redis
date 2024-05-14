# Micro Redis Server

This mini project was completed as part of the term assignment for ICCS492: The Guts of Modern System by Kanladaporn Sirithatthamrong

## Contents
- [Code](#Code)
    - [main.rs](#main-rs)
    - [connections.rs](#connections-rs)
    - [utils.rs](#utils-rs)
    - [execute_cmd.rs](#execute_cmd-rs)
    - [command.rs](#utils-rs)
    - [database.rs](#database-rs)

## Code

### main-rs
- Binds the server to `127.0.0.1:6379`
- Initializes a HashMap wrapped in a Mutex and Arc, representing the database. The database is initially empty, with one default namespace (`0`). 
- Sets up a channel (`tx` and `rx`) for communication between connection handlers and the main thread.
- For each connection:
    - Spawns a new task to handle the connection asynchronously.

### connections-rs
1. `handle_connection` :
    - Handling incoming client connections asynchronously.
    - Initializes a buffer to read data from the socket and tracks the currently selected database namespace and whether a command has been executed.
    - Parses the request using the `parse_request` function. If parsing is successful, it sends a message to the main thread indicating the executed command, pushes the command into the queue for the selected database, and executes queued commands for the database.
2. `execute_queued_commands` :
    - Processes and executes queued commands for the selected database.
    - Pop commands from the queue of the selected database and execute them.
    - Executes each command using the `execute_command` function.
3. `push_command` :
    - Pushes a command into the queue for the selected database.
    - It acquires a lock on the database, retrieves the queue for the selected database, and pushes the command into the queue.

### utils-rs
1. `parse_request` : Parse the request
2. `execute_command` : Matches the command variant and calls corresponding execution functions, passing relevant parameters.

### execute_cmd-rs
*These commands are implemented based on the instructions.*
1. `execute_select_cmd` : Changes the currently selected database to the specified index. 
2. `execute_get_cmd` : Retrieves the value associated with the given key from the selected database.
3. `execute_set_cmd` : Sets the value of the specified key in the selected database.
4. `execute_ping_cmd`
5. `execute_exists_cmd` : Checks if the specified keys exist in the selected database.
6. `execute_rpush_cmd` and `execute_lpush_cmd` : Appends or prepends values to a list associated with the given key.
7. `execute_blpop_cmd` and `execute_brpop_cmd` : Blocks until a value is available in one of the specified lists, or until a timeout occurs.

### command-rs
An enumeration `Command` representing various Redis-like commands along with their associated parameters.

### database-rs
`Data` and `Database` are used to model the data stored in a Redis-like database.
1. `Data` : Represent the different types of data that can be stored in the database.
    - `Scalar(String)` and `List(VecDeque<String>)`
2. `Database` : 
    - `data` : A HashMap that stores key-value pairs, where the key is a string representing the name of the data and the value is of type `Data`.
    - `queue` :  A VecDeque that stores commands queued for execution to guarantee linearization, and each database will have their own queue.
