use crate::server::HandlerResult;
use std::any::Any;

enum Command {
    PING,
    SET(String, String),
    GET(String),
    PIPE(Box<dyn Any>),
}

pub fn redis_handler(buffer: &mut Vec<u8>) -> HandlerResult {
    let mut commands: Vec<Command> = Vec::new();

    HandlerResult::Ok((vec![]))
}
