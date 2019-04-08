// Input/output
use std::io::{Read, Write, ErrorKind};

// Networking
use std::net::{TcpListener, TcpStream};

// Time
use std::time::{SystemTime};

/// Entry point
fn main() {
    println!("Hi, I'm the new Redis! Waiting for connections...");
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

    // Accept new connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => connections_manager.handle_new_client(stream),
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => continue,
            Err(_e) => panic!("IO error in listener.incoming() ! Catastrophic failure. Shutting down."),
        }
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
        };

        // Let someone else handle this later
        self.connections.push(connection);
    }
}

/// Connection abstraction
struct Connection {
    stream: TcpStream,
    last_active: SystemTime,
}