fn find_crlf(buffer: &[u8]) -> Option<usize> {
    for i in 0..buffer.len().saturating_sub(1) {
        if buffer[i] == b'\r' && buffer[i + 1] == b'\n' {
            return Some(i);
        }
    }
    None
}

#[derive(Debug)]
pub enum RespValue {
    Simple(String),
    Integer(i64),
    Bulk(String),
    Array(Vec<RespValue>),
    Null,
}

pub enum ParseOneResponse {
    RespValue(RespValue, usize),
}

#[derive(Debug)]
pub enum ParseError {
    Incomplete,
    InvalidType,
    Other(String),
}

pub fn parse_one(buffer: &[u8]) -> Result<ParseOneResponse, ParseError> {
    if buffer.is_empty() {
        return Err(ParseError::Incomplete);
    }

    match buffer[0] as char {
        '+' => parse_simple_string(buffer),
        ':' => parse_integer(buffer),
        '$' => parse_bulk_string(buffer),
        '*' => parse_array(buffer),
        _ => Err(ParseError::InvalidType),
    }
}

fn parse_simple_string(buffer: &[u8]) -> Result<ParseOneResponse, ParseError> {
    let pos = match find_crlf(buffer) {
        Some(p) => p,
        None => return Err(ParseError::Incomplete),
    };

    let content = &buffer[1..pos]; // skip '+' and till pos - 1 

    //try converting byte slice to utf8. if invalid utf8 fuck all
    let s = std::str::from_utf8(content).map_err(|_| ParseError::Other("invalid utf8".into()))?;

    Ok(ParseOneResponse::RespValue(
        RespValue::Simple(s.to_string()),
        pos + 2, // consumed bytes
    ))
}

fn parse_integer(buffer: &[u8]) -> Result<ParseOneResponse, ParseError> {
    // Find CRLF
    let pos = match find_crlf(buffer) {
        Some(p) => p,
        None => return Err(ParseError::Incomplete),
    };

    // Parse the number (skip ':')
    let num_str = std::str::from_utf8(&buffer[1..pos])
        .map_err(|_| ParseError::Other("invalid utf8 in integer".into()))?;

    let num: i64 = num_str
        .parse()
        .map_err(|_| ParseError::Other("invalid integer".into()))?;

    Ok(ParseOneResponse::RespValue(
        RespValue::Integer(num),
        pos + 2, // consumed bytes including \r\n
    ))
}

fn parse_bulk_string(buffer: &[u8]) -> Result<ParseOneResponse, ParseError> {
    // Find first CRLF to get the length
    let pos = match find_crlf(buffer) {
        Some(p) => p,
        None => return Err(ParseError::Incomplete),
    };

    // Parse the length (skip '$')
    let len_str = std::str::from_utf8(&buffer[1..pos])
        .map_err(|_| ParseError::Other("invalid utf8 in length".into()))?;

    let len: i64 = len_str
        .parse()
        .map_err(|_| ParseError::Other("invalid length".into()))?;

    // Handle null bulk string
    if len == -1 {
        return Ok(ParseOneResponse::RespValue(RespValue::Null, pos + 2));
    }

    // Validate length
    if len < 0 {
        return Err(ParseError::Other("negative length".into()));
    }

    let len = len as usize;
    let data_start = pos + 2; // after first CRLF
    let data_end = data_start + len;

    // Check if we have enough bytes
    if buffer.len() < data_end + 2 {
        return Err(ParseError::Incomplete);
    }

    // Verify trailing CRLF
    if buffer[data_end] != b'\r' || buffer[data_end + 1] != b'\n' {
        return Err(ParseError::Other("missing CRLF after bulk string".into()));
    }

    // Extract the actual string
    let content = &buffer[data_start..data_end];
    let s = std::str::from_utf8(content).map_err(|_| ParseError::Other("invalid utf8".into()))?;

    Ok(ParseOneResponse::RespValue(
        RespValue::Bulk(s.to_string()),
        data_end + 2, // consumed all bytes including trailing CRLF
    ))
}

fn parse_array(buffer: &[u8]) -> Result<ParseOneResponse, ParseError> {
    // Find arr length
    let pos = match find_crlf(buffer) {
        Some(p) => p,
        None => return Err(ParseError::Incomplete),
    };

    // Parse the count (skip '*')
    let count_str = std::str::from_utf8(&buffer[1..pos])
        .map_err(|_| ParseError::Other("invalid utf8 in count".into()))?;

    let count: i64 = count_str
        .parse()
        .map_err(|_| ParseError::Other("invalid count".into()))?;

    // Handle null array
    if count == -1 {
        return Ok(ParseOneResponse::RespValue(RespValue::Null, pos + 2));
    }

    // Validate count
    if count < 0 {
        return Err(ParseError::Other("negative count".into()));
    }

    let count = count as usize;
    let mut elements = Vec::with_capacity(count);
    let mut total_consumed = pos + 2; // Skip past *N\r\n

    // Parse each element
    for _ in 0..count {
        let remaining = &buffer[total_consumed..];

        match parse_one(remaining)? {
            ParseOneResponse::RespValue(val, consumed) => {
                elements.push(val);
                total_consumed += consumed;
            }
        }
    }

    Ok(ParseOneResponse::RespValue(
        RespValue::Array(elements),
        total_consumed,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_array_simple() {
        // *2\r\n$3\r\nGET\r\n$3\r\nkey\r\n
        let input = b"*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n";

        match parse_one(input) {
            Ok(ParseOneResponse::RespValue(RespValue::Array(arr), consumed)) => {
                assert_eq!(arr.len(), 2);
                assert_eq!(consumed, 22);

                // Check first element
                match &arr[0] {
                    RespValue::Bulk(s) => assert_eq!(s, "GET"),
                    _ => panic!("Expected bulk string"),
                }

                // Check second element
                match &arr[1] {
                    RespValue::Bulk(s) => assert_eq!(s, "key"),
                    _ => panic!("Expected bulk string"),
                }
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_parse_array_set_command() {
        // *3\r\n$3\r\nSET\r\n$5\r\nmykey\r\n$7\r\nmyvalue\r\n
        let input = b"*3\r\n$3\r\nSET\r\n$5\r\nmykey\r\n$7\r\nmyvalue\r\n";

        match parse_one(input) {
            Ok(ParseOneResponse::RespValue(RespValue::Array(arr), consumed)) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(consumed, 37);

                match &arr[0] {
                    RespValue::Bulk(s) => assert_eq!(s, "SET"),
                    _ => panic!("Expected bulk string"),
                }
                match &arr[1] {
                    RespValue::Bulk(s) => assert_eq!(s, "mykey"),
                    _ => panic!("Expected bulk string"),
                }
                match &arr[2] {
                    RespValue::Bulk(s) => assert_eq!(s, "myvalue"),
                    _ => panic!("Expected bulk string"),
                }
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_parse_array_ping() {
        // *1\r\n$4\r\nPING\r\n
        let input = b"*1\r\n$4\r\nPING\r\n";

        match parse_one(input) {
            Ok(ParseOneResponse::RespValue(RespValue::Array(arr), consumed)) => {
                assert_eq!(arr.len(), 1);
                assert_eq!(consumed, 14);

                match &arr[0] {
                    RespValue::Bulk(s) => assert_eq!(s, "PING"),
                    _ => panic!("Expected bulk string"),
                }
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_parse_array_empty() {
        // *0\r\n
        let input = b"*0\r\n";

        match parse_one(input) {
            Ok(ParseOneResponse::RespValue(RespValue::Array(arr), consumed)) => {
                assert_eq!(arr.len(), 0);
                assert_eq!(consumed, 4);
            }
            _ => panic!("Expected empty array"),
        }
    }

    #[test]
    fn test_parse_array_null() {
        // *-1\r\n
        let input = b"*-1\r\n";

        match parse_one(input) {
            Ok(ParseOneResponse::RespValue(RespValue::Null, consumed)) => {
                assert_eq!(consumed, 5);
            }
            _ => panic!("Expected null array"),
        }
    }

    #[test]
    fn test_parse_array_incomplete() {
        // *2\r\n$3\r\nGET\r\n (missing second element)
        let input = b"*2\r\n$3\r\nGET\r\n";

        match parse_one(input) {
            Err(ParseError::Incomplete) => {} // Expected
            _ => panic!("Expected Incomplete error"),
        }
    }

    #[test]
    fn test_parse_array_mixed_types() {
        // *3\r\n+OK\r\n:42\r\n$5\r\nhello\r\n
        let input = b"*3\r\n+OK\r\n:42\r\n$5\r\nhello\r\n";

        match parse_one(input) {
            Ok(ParseOneResponse::RespValue(RespValue::Array(arr), _)) => {
                assert_eq!(arr.len(), 3);

                match &arr[0] {
                    RespValue::Simple(s) => assert_eq!(s, "OK"),
                    _ => panic!("Expected simple string"),
                }
                match &arr[1] {
                    RespValue::Integer(n) => assert_eq!(*n, 42),
                    _ => panic!("Expected integer"),
                }
                match &arr[2] {
                    RespValue::Bulk(s) => assert_eq!(s, "hello"),
                    _ => panic!("Expected bulk string"),
                }
            }
            _ => panic!("Expected array with mixed types"),
        }
    }

    #[test]
    fn test_parse_bulk_string() {
        // $5\r\nhello\r\n
        let input = b"$5\r\nhello\r\n";

        match parse_one(input) {
            Ok(ParseOneResponse::RespValue(RespValue::Bulk(s), consumed)) => {
                assert_eq!(s, "hello");
                assert_eq!(consumed, 11);
            }
            _ => panic!("Expected bulk string"),
        }
    }

    #[test]
    fn test_parse_bulk_string_null() {
        // $-1\r\n
        let input = b"$-1\r\n";

        match parse_one(input) {
            Ok(ParseOneResponse::RespValue(RespValue::Null, consumed)) => {
                assert_eq!(consumed, 5);
            }
            _ => panic!("Expected null bulk string"),
        }
    }

    #[test]
    fn test_parse_integer() {
        // :42\r\n
        let input = b":42\r\n";

        match parse_one(input) {
            Ok(ParseOneResponse::RespValue(RespValue::Integer(n), consumed)) => {
                assert_eq!(n, 42);
                assert_eq!(consumed, 5);
            }
            _ => panic!("Expected integer"),
        }
    }

    #[test]
    fn test_parse_integer_negative() {
        // :-100\r\n
        let input = b":-100\r\n";

        match parse_one(input) {
            Ok(ParseOneResponse::RespValue(RespValue::Integer(n), consumed)) => {
                assert_eq!(n, -100);
                assert_eq!(consumed, 7);
            }
            _ => panic!("Expected negative integer"),
        }
    }
}
