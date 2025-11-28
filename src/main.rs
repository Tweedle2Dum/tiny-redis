use crate::server::TcpServer;

mod db;
mod executor;
mod parser;
mod server;

fn main() {
    let server = TcpServer::new("127.0.0.1:6379".into())
        .expect("Unable to create a new tcp stream listener socket");
    server.run();
}
