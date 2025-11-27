use std::io::{ BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

mod db;
mod executor;
mod parser;

use db::Db;
use parser::{ parse_one, ParseOneResponse, ParseError};




fn handle_client(mut stream: TcpStream, db: &mut Db) {
    let peer = stream.peer_addr().unwrap();
    println!("Client connected: {}", peer);

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut client_buffer: Vec<u8> = Vec::new();

    let mut byte_buf = [0u8; 1];

    loop {
        let n = match reader.read(&mut byte_buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };

        client_buffer.extend_from_slice(&byte_buf[..n]);
        println!("RAW BYTES = {:?}", client_buffer);

        loop {
            match parse_one(&client_buffer) {
                Ok(ParseOneResponse::RespValue(resp, consumed)) => {
                    println!("Parsed: {:?}", resp);

                    let txt = format!("{:?}\r\n", resp);
                    stream.write_all(txt.as_bytes()).unwrap();

                    // Remove parsed bytes
                    client_buffer.drain(0..consumed);
                }
                
                Err(ParseError::Incomplete) => {
                    break;
                }

                Err(err) => {
                    let txt = format!("ERR {:?}\r\n", err);
                    stream.write_all(txt.as_bytes()).unwrap();
                    client_buffer.clear();
                    break;
                }
            }
        }
    }

    println!("Client disconnected: {}", peer);
}

fn main() {
    let mut db = Db::new();
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    println!("Server listening on port 6379");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_client(stream, &mut db),
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}
