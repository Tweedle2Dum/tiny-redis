#[derive(Debug, PartialEq)]
pub enum Command {
    Ping,
    Get(String),
    Set(String, String),
    Del(String),
    Exists(String),
    Inc(String)
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnknownCommand,
    MissingKey,
    MissingValue,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnknownCommand => write!(f, "Unknown command"),
            ParseError::MissingKey => write!(f, "Key is missing"),
            ParseError::MissingValue => write!(f, "Value is missing"),
        }
    }
}

pub fn parse(input: &str) -> Result<Command, ParseError> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();

    match parts.get(0) {
        Some(&"PING") => Ok(Command::Ping),
        Some(&"GET") => {
            if let Some(key) = parts.get(1) {
                Ok(Command::Get(key.to_string()))
            } else {
                Err(ParseError::MissingKey)
            }
        }
        Some(&"SET") => match (parts.get(1), parts.get(2)) {
            (Some(key), Some(value)) => Ok(Command::Set(key.to_string(), value.to_string())),
            (None, _) => Err(ParseError::MissingKey),
            (_, None) => Err(ParseError::MissingValue),
        },
        Some(&"DEL") => {
            if let Some(key) = parts.get(1) {
                Ok(Command::Del(key.to_string()))
            } else {
                Err(ParseError::MissingKey)
            }
        },
        Some(&"EXISTS") => {
            if let Some(key) = parts.get(1){
                Ok(Command::Exists(key.to_string()))
            } else {
                Err(ParseError::MissingKey)
            }
        },
        Some(&"INC") => {
            if let Some(key) = parts.get(1){
                Ok(Command::Inc(key.to_string()))
            } else {
                Err(ParseError::MissingKey)
            }
        }
        _ => Err(ParseError::UnknownCommand),
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

    #[test]
    fn test_unknown_command() {
        let unknown = parse("cute doom turtle");
        match unknown {
            Ok(_) => panic!("Test unknown command failed"),
            Err(err) => assert_eq!(err, ParseError::UnknownCommand),
        }
    }

    #[test]
    fn test_missing_key() {
        let missing_key = parse("GET");
        match missing_key {
            Ok(_) => panic!("Get missing key failed"),
            Err(err) => assert_eq!(err, ParseError::MissingKey),
        }
    }
    
    #[test]
    fn test_increment() {
        let inc = parse("INC 23");
        match inc {
            Ok(Command::Inc(key)) => assert_eq!(key,"23"),
            _ => panic!("INC test failed") 
        }
    }
}
