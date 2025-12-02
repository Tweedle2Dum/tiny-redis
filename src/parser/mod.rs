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

#[derive(Debug)]
pub enum ParseError {
    Incomplete,
    InvalidType,
    Other(String),
}

/// Parses a single RESP value from the buffer.
/// Returns (RespValue, bytes_consumed) on success.
pub fn parse_one(buffer: &[u8]) -> Result<(RespValue, usize), ParseError> {
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

fn parse_simple_string(buffer: &[u8]) -> Result<(RespValue, usize), ParseError> {
    let pos = find_crlf(buffer).ok_or(ParseError::Incomplete)?;

    let content = &buffer[1..pos];
    let s = std::str::from_utf8(content).map_err(|_| ParseError::Other("invalid utf8".into()))?;

    Ok((RespValue::Simple(s.to_string()), pos + 2))
}

fn parse_integer(buffer: &[u8]) -> Result<(RespValue, usize), ParseError> {
    let pos = find_crlf(buffer).ok_or(ParseError::Incomplete)?;

    let num_str = std::str::from_utf8(&buffer[1..pos])
        .map_err(|_| ParseError::Other("invalid utf8 in integer".into()))?;

    let num: i64 = num_str
        .parse()
        .map_err(|_| ParseError::Other("invalid integer".into()))?;

    Ok((RespValue::Integer(num), pos + 2))
}

fn parse_bulk_string(buffer: &[u8]) -> Result<(RespValue, usize), ParseError> {
    let pos = find_crlf(buffer).ok_or(ParseError::Incomplete)?;

    let len_str = std::str::from_utf8(&buffer[1..pos])
        .map_err(|_| ParseError::Other("invalid utf8 in length".into()))?;

    let len: i64 = len_str
        .parse()
        .map_err(|_| ParseError::Other("invalid length".into()))?;

    if len == -1 {
        return Ok((RespValue::Null, pos + 2));
    }

    if len < 0 {
        return Err(ParseError::Other("negative length".into()));
    }

    let len = len as usize;
    let data_start = pos + 2;
    let data_end = data_start + len;

    if buffer.len() < data_end + 2 {
        return Err(ParseError::Incomplete);
    }

    if buffer[data_end] != b'\r' || buffer[data_end + 1] != b'\n' {
        return Err(ParseError::Other("missing CRLF after bulk string".into()));
    }

    let content = &buffer[data_start..data_end];
    let s = std::str::from_utf8(content).map_err(|_| ParseError::Other("invalid utf8".into()))?;

    Ok((RespValue::Bulk(s.to_string()), data_end + 2))
}

fn parse_array(buffer: &[u8]) -> Result<(RespValue, usize), ParseError> {
    let pos = find_crlf(buffer).ok_or(ParseError::Incomplete)?;

    let count_str = std::str::from_utf8(&buffer[1..pos])
        .map_err(|_| ParseError::Other("invalid utf8 in count".into()))?;

    let count: i64 = count_str
        .parse()
        .map_err(|_| ParseError::Other("invalid count".into()))?;

    if count == -1 {
        return Ok((RespValue::Null, pos + 2));
    }

    if count < 0 {
        return Err(ParseError::Other("negative count".into()));
    }

    let count = count as usize;
    let mut elements = Vec::with_capacity(count);
    let mut total_consumed = pos + 2;

    for _ in 0..count {
        let remaining = &buffer[total_consumed..];
        let (val, consumed) = parse_one(remaining)?;
        elements.push(val);
        total_consumed += consumed;
    }

    Ok((RespValue::Array(elements), total_consumed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_array_simple() {
        let input = b"*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n";

        let (value, consumed) = parse_one(input).unwrap();

        match value {
            RespValue::Array(arr) => {
                assert_eq!(arr.len(), 2);
                assert_eq!(consumed, 22);

                match &arr[0] {
                    RespValue::Bulk(s) => assert_eq!(s, "GET"),
                    _ => panic!("Expected bulk string"),
                }

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
        let input = b"*3\r\n$3\r\nSET\r\n$5\r\nmykey\r\n$7\r\nmyvalue\r\n";

        let (value, consumed) = parse_one(input).unwrap();

        match value {
            RespValue::Array(arr) => {
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
        let input = b"*1\r\n$4\r\nPING\r\n";

        let (value, consumed) = parse_one(input).unwrap();

        match value {
            RespValue::Array(arr) => {
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
        let input = b"*0\r\n";

        let (value, consumed) = parse_one(input).unwrap();

        match value {
            RespValue::Array(arr) => {
                assert_eq!(arr.len(), 0);
                assert_eq!(consumed, 4);
            }
            _ => panic!("Expected empty array"),
        }
    }

    #[test]
    fn test_parse_array_null() {
        let input = b"*-1\r\n";

        let (value, consumed) = parse_one(input).unwrap();

        match value {
            RespValue::Null => {
                assert_eq!(consumed, 5);
            }
            _ => panic!("Expected null array"),
        }
    }

    #[test]
    fn test_parse_array_incomplete() {
        let input = b"*2\r\n$3\r\nGET\r\n";

        match parse_one(input) {
            Err(ParseError::Incomplete) => {}
            _ => panic!("Expected Incomplete error"),
        }
    }

    #[test]
    fn test_parse_array_mixed_types() {
        let input = b"*3\r\n+OK\r\n:42\r\n$5\r\nhello\r\n";

        let (value, _) = parse_one(input).unwrap();

        match value {
            RespValue::Array(arr) => {
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
        let input = b"$5\r\nhello\r\n";

        let (value, consumed) = parse_one(input).unwrap();

        match value {
            RespValue::Bulk(s) => {
                assert_eq!(s, "hello");
                assert_eq!(consumed, 11);
            }
            _ => panic!("Expected bulk string"),
        }
    }

    #[test]
    fn test_parse_bulk_string_null() {
        let input = b"$-1\r\n";

        let (value, consumed) = parse_one(input).unwrap();

        match value {
            RespValue::Null => {
                assert_eq!(consumed, 5);
            }
            _ => panic!("Expected null bulk string"),
        }
    }

    #[test]
    fn test_parse_integer() {
        let input = b":42\r\n";

        let (value, consumed) = parse_one(input).unwrap();

        match value {
            RespValue::Integer(n) => {
                assert_eq!(n, 42);
                assert_eq!(consumed, 5);
            }
            _ => panic!("Expected integer"),
        }
    }

    #[test]
    fn test_parse_integer_negative() {
        let input = b":-100\r\n";

        let (value, consumed) = parse_one(input).unwrap();

        match value {
            RespValue::Integer(n) => {
                assert_eq!(n, -100);
                assert_eq!(consumed, 7);
            }
            _ => panic!("Expected negative integer"),
        }
    }
}
