
use std::io::{Write, Read};
use std::net::{TcpListener, TcpStream};
use std::thread;

const OP_MAX_LEN: usize = 24;
const CHAR_SPACE: char = ' ';


fn main() {
    println!("Hello, world!");
}

fn server(url: &str) {
    let listener = TcpListener::bind(url)
        .expect(&format!("Cannot bind on {}", url));

    for stream in listener.incoming() {
        thread::spawn(|| {
            match stream {
                Ok(stream) => handle_client(stream),
                Err(err) => println!("Accept error: {}", err)
            }
            // let mut buffer = [0u8; 4096];
            // let buffer = stream.read(&mut buffer).unwrap();
        });
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buf = [0u8; 4096];
    let mut op = [0u8; OP_MAX_LEN];

    stream.read(&mut buf)
        .expect("Could not read stream info buffer.");

    for (c, i) in buf.iter().enumerate() {

    }
}

// fn determine_op(cmd: &str) -> Op {

// }