// Input/output
use std::io::{Read, Write, ErrorKind};

// Networking
use std::net::{TcpListener, TcpStream};

// Time
use std::time::{SystemTime, Duration};

// Sleep
use std::thread::sleep;

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
            ConnectionState::WaitForRead => {
                match connection.stream.read_to_end(&mut connection.buf) {
                    Ok(n) => {
                        log(&format!("Read {} bytes", n));
                        Ok(n)
                    },
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => Ok(0),
                    Err(e) => Err(()),
                }
            },
            ConnectionState::WaitForWrite => {
                connection.stream.write(&connection.buf);
                connection.state = ConnectionState::WaitForRead;

                Ok(0)
            },
        }
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
    WaitForWrite,
}
