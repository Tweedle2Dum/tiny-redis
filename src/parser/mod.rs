#[derive(Debug, PartialEq)]
pub enum Command {
    Ping,
    Get(String),
    Set(String, String),
}

pub fn parse(input: &str) -> Result<Command, String> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();

    match parts.get(0) {
        Some(&"PING") => Ok(Command::Ping),
        Some(&"GET") => {
            if let Some(key) = parts.get(1) {
                Ok(Command::Get(key.to_string()))
            } else {
                Err("GET command requires a key".into())
            }
        }
        Some(&"SET") => {
            if let (Some(key), Some(value)) = (parts.get(1), parts.get(2)) {
                Ok(Command::Set(key.to_string(), value.to_string()))
            } else {
                Err("SET command requires key and value".into())
            }
        }
        _ => Err("Unknown command".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping() {
        assert_eq!(parse("PING").unwrap(), Command::Ping);
    }

    #[test]
    fn test_set_get() {
        let set = parse("SET mykey 123").unwrap();
        match set {
            Command::Set(k, v) => {
                assert_eq!(k, "mykey");
                assert_eq!(v, "123");
            }
            _ => panic!("Expected Set command"),
        }

        let get = parse("GET mykey").unwrap();
        match get {
            Command::Get(k) => assert_eq!(k, "mykey"),
            _ => panic!("Expected Get command"),
        }
    }
}
