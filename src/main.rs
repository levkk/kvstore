// Input/output
use std::io::{Read, Write, ErrorKind};

// Networking
use std::net::{TcpListener, TcpStream};

// Time
use std::time::{SystemTime, Duration};

// Sleep
use std::thread::sleep;

// Hash
use std::collections::HashMap;

/// Entry point
fn main() {
    log("Hi, I'm the new Redis! Waiting for connections...");
    server("localhost:9999");
}

/// Controls logging (can adjust level globally, i.e. debug, info, etc.)
fn log(msg: &str) {
    println!("{}", msg);
}

/// Listen for connections and handle new clients
fn server(url: &str) {
    // TCP socket
    let listener = TcpListener::bind(url)
        .expect(&format!("Cannot bind on {}. Maybe the port is already in use?", url));

    // Do not block on accept()
    listener.set_nonblocking(true)
        .expect("The OS does not support non-blocking sockets which are required.");

    // Store all connections in here
    let mut connections_manager = ConnectionManager{
        connections: vec![],
    };

    // Main loop
    loop {
        // Async, so it won't block if no new connections are waiting
        match listener.accept() {
            Ok((stream, _addr)) => connections_manager.handle_new_client(stream),
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => (),
            Err(_e) => panic!("IO error in listener.incoming() ! Catastrophic failure. Shutting down."),
        };

        // All connections are async here as well, so won't block unless it has to read/write
        connections_manager.service_connections();

        // Not waste wild CPU cycles
        sleep(Duration::from_millis(1));
    }
}


/// Manages all connections
struct ConnectionManager {
    connections: Vec<Connection>,
}

impl ConnectionManager {

    /// Handle the new client
    fn handle_new_client(&mut self, mut stream: TcpStream) {
        // Set the connection as non-blocking
        stream.set_nonblocking(true)
            .expect("Could not set incoming stream to non-blocking.");

        log("New client");

        // Abstract the connection
        let connection = Connection{
            stream: stream,
            last_active: SystemTime::now(),
            state: ConnectionState::WaitForWrite,
            buf: vec![],
        };

        // Let someone else handle this later
        self.connections.push(connection);
    }

    /// Service all connections
    fn service_connections(&mut self) {
        for mut connection in self.connections.iter_mut() {
            Self::service_connection(&mut connection);
        }
    }

    /// Service specific connection
    fn service_connection(connection: &mut Connection) -> Result<usize, ()> {
        match connection.state {
            ConnectionState::WaitForRead => Self::read_connection(connection),
            ConnectionState::WaitForWrite => Self::write_connection(connection),
            ConnectionState::WaitForOp => {
                Ok(0)
            },
        }
    }

    fn read_connection(connection: &mut Connection) -> Result<usize, ()> {
        match connection.stream.read_to_end(&mut connection.buf) {
            // Read successful
            Ok(n) => {
                log(&format!("Read {} bytes", n));

                if Self::end_of_message(&connection.buf) {
                    connection.state = ConnectionState::WaitForOp;
                }

                Ok(n)
            },

            // Socket not ready
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => Ok(0),

            // I/O error
            Err(e) => Err(()),
        }
    }

    fn write_connection(connection: &mut Connection) -> Result<usize, ()> {
        match connection.stream.write_all(&connection.buf) {
            Ok(_) => {
                let len = connection.buf.len();
                connection.buf.clear();
                connection.state = ConnectionState::WaitForRead;
                Ok(len)
            },
            Err(_err) => Err(()),
        }
    }

    fn parse_op(buf: &Vec<u8>) {
        
    }

    fn end_of_message(buf: &Vec<u8>) -> bool {
        buf.contains(&('\r' as u8))
    }
}

/// Connection abstraction
struct Connection {
    stream: TcpStream,
    last_active: SystemTime,
    state: ConnectionState,
    buf: Vec<u8>,
}

enum ConnectionState {
    WaitForRead,
    WaitForOp,
    WaitForWrite,
}

struct Store {
    dict: HashMap<String, Value>,
}

impl Store {
    fn set(&mut self, key: &str, value: &str) -> Result<(), String> {
        match value.chars().nth(0).unwrap() {
            ':' => {},
            _ => {},
        };

        Ok(())
    }
}

struct Value {
    value_type: ValueType,
    integer_value: u64,
    raw_string_value: String,
    array: Vec<u64>,
}

enum ValueType {
    Integer,
    RawString,
    // Array,
}



