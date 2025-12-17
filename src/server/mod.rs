use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

#[derive(Debug)]
pub enum HandlerResult {
    /// Not enough bytes yet, keep reading
    Incomplete,
    /// Fatal error, with message
    Error(String),
    /// Successfully handled, optionally return bytes to write back
    Ok(Vec<u8>),
}

pub struct TcpServer {
    listener: TcpListener,
}

impl TcpServer {
    pub fn new(addr: &str) -> std::io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        println!("Server listening on {}", addr);
        Ok(TcpServer { listener })
    }

    pub fn run<F>(&self, mut handler: F)
    where
        F: FnMut(&mut Vec<u8>) -> HandlerResult,
    {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    Self::handle_connection(stream, &mut handler);
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }
    }

    fn handle_connection<F>(mut stream: TcpStream, handler: &mut F)
    where
        F: FnMut(&mut Vec<u8>) -> HandlerResult,
    {
        let peer = stream.peer_addr().unwrap();
        println!("Client connected: {}", peer);

        let mut scratch = [0u8; 4096];
        let mut client_buffer: Vec<u8> = Vec::new();

        loop {
            match stream.read(&mut scratch) {
                Ok(0) => {
                    println!("Client {} disconnected", peer);
                    break;
                }
                Ok(n) => {
                    client_buffer.extend_from_slice(&scratch[..n]);
                    println!("RAW BYTES = {:?}", &client_buffer);

                    match handler(&mut client_buffer) {
                        HandlerResult::Incomplete => {
                            // need more bytes
                            continue;
                        }
                        HandlerResult::Error(err) => {
                            // send error message and break
                            let _ = stream.write_all(format!("ERR {}\r\n", err).as_bytes());
                            client_buffer.clear();
                            break;
                        }
                        HandlerResult::Ok(response_bytes) => {
                            // send response back
                            if let Err(e) = stream.write_all(&response_bytes) {
                                println!("Write error: {}", e);
                                break;
                            }
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    println!("Error: {}", e);
                    break;
                }
            }
        }
    }
}