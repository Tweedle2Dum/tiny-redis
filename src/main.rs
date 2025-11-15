use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

mod db;
mod executor;
mod parser;

use db::Db;
use executor::Executor;
use parser::parse;

fn handle_client(mut stream: TcpStream, db: &mut Db) {
    let mut executor = Executor::new(db);
    let peer = stream.peer_addr().unwrap();
    println!("Client connected: {}", peer);

    let reader = BufReader::new(stream.try_clone().unwrap());

    for line in reader.lines() {
        let line = line.unwrap();
        if line.trim().eq_ignore_ascii_case("QUIT") {
            break;
        }

        let response = match parse(&line) {
            Ok(cmd) => match executor.execute(cmd) {
                Ok(out) => out,
                Err(err) => format!("ERR {:?}", err),
            },
            Err(err) => format!("Parse error: {:?}", err),
        };

        let resp = format!("{}\r\n", response);
        stream.write_all(resp.as_bytes()).unwrap();
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
