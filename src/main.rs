#![feature(bind_by_move_pattern_guards)]

// Input/output
use std::io::{Read, Write, ErrorKind};

// Networking
use std::net::{TcpListener, TcpStream, Shutdown};

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
        closed_connections: vec![],
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
    closed_connections: Vec<usize>,
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
            offset: 0,
        };

        // Let someone else handle this later
        self.connections.push(connection);
    }

    /// Service all connections
    fn service_connections(&mut self) {
        for (idx, mut connection) in self.connections.iter_mut().enumerate() {
            match Self::service_connection(idx, &mut connection) {
                Ok(_n) => (),
                Err(err) if err == ConnectionState::Closed => {
                    self.closed_connections.push(idx);
                },
                Err(_) => (),
            };
        }

        for idx in self.closed_connections.iter() {
            self.connections[*idx].stream.shutdown(Shutdown::Both);
            self.connections.remove(*idx);

            log(&format!("Shutdown connection {}", *idx));
        }

        self.closed_connections.clear();
    }

    /// Service specific connection
    fn service_connection(idx: usize, connection: &mut Connection) -> Result<usize, ConnectionState> {
        match connection.state {
            ConnectionState::WaitForRead => Self::read_connection(connection),
            ConnectionState::WaitForWrite => Self::write_connection(connection),
            ConnectionState::WaitForOp => {
                Ok(0)
            },
            ConnectionState::Closed => Err(ConnectionState::Closed),
        }
    }

    fn read_connection(connection: &mut Connection) -> Result<usize, ConnectionState> {
        match connection.stream.read(&mut connection.buf[connection.offset..]) {
            // Read successful
            Ok(n) if n == 0 => {
                // connection.state = ConnectionState::Closed;
                Ok(0)
            },

            Ok(n) => {
                //
                connection.offset = n;
                log(&format!("Read {} bytes", n));

                if Self::end_of_message(&connection.buf) {
                    connection.state = ConnectionState::WaitForOp;
                    connection.offset = 0;
                }

                Ok(n)
            },

            // Socket not ready
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => Ok(0),

            // I/O error
            Err(e) => Err(ConnectionState::Closed),
        }
    }

    fn write_connection(connection: &mut Connection) -> Result<usize, ConnectionState> {
        match connection.stream.write(&connection.buf[connection.offset..]) {
            Ok(n) if n == 0 => {
                connection.offset = 0;
                connection.state = ConnectionState::WaitForRead;
                Ok(0)
            },
            Ok(n) => {
                connection.offset = n;
                Ok(n)
            },
            Err(_err) => Err(ConnectionState::Closed),
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
    offset: usize,
}

#[derive(PartialEq)]
enum ConnectionState {
    WaitForRead,
    WaitForOp,
    WaitForWrite,
    Closed,
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



