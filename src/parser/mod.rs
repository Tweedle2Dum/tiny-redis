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
    Array(Vec<RespValue>)
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
        _   => Err(ParseError::InvalidType),
    }
}

fn parse_simple_string(buffer: &[u8]) -> Result<ParseOneResponse, ParseError> {
    let pos = match find_crlf(buffer) {
        Some(p) => p,
        None => return Err(ParseError::Incomplete),
    };

    let content = &buffer[1..pos]; // skip '+' and till pos - 1 

    //try converting byte slice to utf8. if invalid utf8 fuck all
    let s = std::str::from_utf8(content)
        .map_err(|_| ParseError::Other("invalid utf8".into()))?;

    Ok(ParseOneResponse::RespValue(
        RespValue::Simple(s.to_string()),
        pos + 2, // consumed bytes
    ))
}

fn parse_integer(buffer: &[u8]) -> Result<ParseOneResponse, ParseError> {
    println!("TODO: integer");
    Err(ParseError::Incomplete)
}

fn parse_bulk_string(buffer: &[u8]) -> Result<ParseOneResponse, ParseError> {
    println!("TODO: bulk string");
    Err(ParseError::Incomplete)
}

fn parse_array(buffer: &[u8]) -> Result<ParseOneResponse, ParseError> {
    println!("TODO: array");
    Err(ParseError::Incomplete)
}
