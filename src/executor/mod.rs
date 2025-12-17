use crate::db::Db;
use crate::parser::{ParseError, RespValue, parse_one};
use crate::server::HandlerResult;

#[derive(Debug)]
enum Command {
    PING,
    SET(String, String),
    GET(String),
    ECHO(String),
}

pub struct RedisHandler {
    db: Db,
}

impl RedisHandler {
    pub fn new() -> Self {
        Self {
            db: Db::new(),
        }
    }

    pub fn handle(&mut self, buffer: &mut Vec<u8>) -> HandlerResult {
        let mut commands: Vec<Command> = Vec::new();
        let mut total_consumed = 0;

        // Parse all complete commands from the buffer
        loop {
            let remaining = &buffer[total_consumed..];

            if remaining.is_empty() {
                break;
            }

            match parse_one(remaining) {
                Ok((value, consumed)) => {
                    // Try to convert RespValue into Command
                    match parse_command(&value) {
                        Ok(cmd) => {
                            commands.push(cmd);
                            total_consumed += consumed;
                        }
                        Err(err) => {
                            buffer.clear();
                            return HandlerResult::Error(err);
                        }
                    }
                }
                Err(ParseError::Incomplete) => {
                    // Need more data - keep what we haven't processed yet
                    break;
                }
                Err(ParseError::InvalidType) => {
                    buffer.clear();
                    return HandlerResult::Error("Invalid RESP type".to_string());
                }
                Err(ParseError::Other(msg)) => {
                    buffer.clear();
                    return HandlerResult::Error(msg);
                }
            }
        }

        // Remove consumed bytes from buffer
        buffer.drain(..total_consumed);

        // If we didn't parse any complete commands, wait for more data
        if commands.is_empty() {
            return HandlerResult::Incomplete;
        }

        // Execute commands and build response
        let mut response_bytes = Vec::new();

        for cmd in commands {
            let response = self.execute_command(cmd);
            response_bytes.extend_from_slice(&response);
        }

        HandlerResult::Ok(response_bytes)
    }

    fn execute_command(&mut self, cmd: Command) -> Vec<u8> {
        match cmd {
            Command::PING => {
                b"+PONG\r\n".to_vec()
            }

            Command::ECHO(msg) => {
                format!("${}\r\n{}\r\n", msg.len(), msg).into_bytes()
            }

            Command::SET(key, value) => {
                self.db.set(key, value);
                b"+OK\r\n".to_vec()
            }

            Command::GET(key) => {
                match self.db.get(&key) {
                    Some(value) => {
                        format!("${}\r\n{}\r\n", value.len(), value).into_bytes()
                    }
                    None => {
                        b"$-1\r\n".to_vec()
                    }
                }
            }
        }
    }
}

fn parse_command(value: &RespValue) -> Result<Command, String> {
    match value {
        RespValue::Array(arr) if !arr.is_empty() => {
            // Get command name
            let cmd_name = match &arr[0] {
                RespValue::Bulk(s) => s.to_uppercase(),
                RespValue::Simple(s) => s.to_uppercase(),
                _ => return Err("Invalid command format".to_string()),
            };

            // Parse based on command name
            match cmd_name.as_str() {
                "PING" => Ok(Command::PING),

                "ECHO" if arr.len() == 2 => {
                    let msg = extract_string(&arr[1])?;
                    Ok(Command::ECHO(msg))
                }

                "SET" if arr.len() == 3 => {
                    let key = extract_string(&arr[1])?;
                    let value = extract_string(&arr[2])?;
                    Ok(Command::SET(key, value))
                }

                "GET" if arr.len() == 2 => {
                    let key = extract_string(&arr[1])?;
                    Ok(Command::GET(key))
                }

                _ => Err(format!("Unknown or invalid command: {}", cmd_name)),
            }
        }
        _ => Err("Expected array command".to_string()),
    }
}

fn extract_string(value: &RespValue) -> Result<String, String> {
    match value {
        RespValue::Bulk(s) => Ok(s.clone()),
        RespValue::Simple(s) => Ok(s.clone()),
        _ => Err("Expected string value".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_command() {
        let mut handler = RedisHandler::new();
        let mut buffer = b"*1\r\n$4\r\nPING\r\n".to_vec();

        match handler.handle(&mut buffer) {
            HandlerResult::Ok(response) => {
                assert_eq!(response, b"+PONG\r\n");
                assert!(buffer.is_empty());
            }
            _ => panic!("Expected Ok response"),
        }
    }

    #[test]
    fn test_echo_command() {
        let mut handler = RedisHandler::new();
        let mut buffer = b"*2\r\n$4\r\nECHO\r\n$5\r\nhello\r\n".to_vec();

        match handler.handle(&mut buffer) {
            HandlerResult::Ok(response) => {
                assert_eq!(response, b"$5\r\nhello\r\n");
                assert!(buffer.is_empty());
            }
            _ => panic!("Expected Ok response"),
        }
    }

    #[test]
    fn test_incomplete_command() {
        let mut handler = RedisHandler::new();
        let mut buffer = b"*2\r\n$3\r\nGET".to_vec();

        match handler.handle(&mut buffer) {
            HandlerResult::Incomplete => {
                // Buffer should still contain the incomplete data
                assert_eq!(buffer, b"*2\r\n$3\r\nGET");
            }
            _ => panic!("Expected Incomplete"),
        }
    }

    #[test]
    fn test_multiple_commands() {
        let mut handler = RedisHandler::new();
        let mut buffer = b"*1\r\n$4\r\nPING\r\n*2\r\n$4\r\nECHO\r\n$2\r\nhi\r\n".to_vec();

        match handler.handle(&mut buffer) {
            HandlerResult::Ok(response) => {
                assert_eq!(response, b"+PONG\r\n$2\r\nhi\r\n");
                assert!(buffer.is_empty());
            }
            _ => panic!("Expected Ok response"),
        }
    }

    #[test]
    fn test_set_get_commands() {
        let mut handler = RedisHandler::new();
        
        // SET command
        let mut buffer = b"*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n".to_vec();
        match handler.handle(&mut buffer) {
            HandlerResult::Ok(response) => {
                assert_eq!(response, b"+OK\r\n");
            }
            _ => panic!("Expected Ok response"),
        }

        // GET command - should now return the value
        let mut buffer = b"*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n".to_vec();
        match handler.handle(&mut buffer) {
            HandlerResult::Ok(response) => {
                assert_eq!(response, b"$5\r\nvalue\r\n");
            }
            _ => panic!("Expected Ok response"),
        }
    }

    #[test]
    fn test_get_nonexistent_key() {
        let mut handler = RedisHandler::new();
        let mut buffer = b"*2\r\n$3\r\nGET\r\n$7\r\nmissing\r\n".to_vec();

        match handler.handle(&mut buffer) {
            HandlerResult::Ok(response) => {
                assert_eq!(response, b"$-1\r\n");
            }
            _ => panic!("Expected Ok response"),
        }
    }
}