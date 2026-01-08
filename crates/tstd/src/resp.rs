//! RESP (REdis Serialization Protocol) parser and serializer
//!
//! Implements RESP2 protocol for Redis compatibility

use bytes::{Buf, BytesMut};
use std::io::Cursor;

/// Maximum bulk string size (512MB) - prevents DoS via memory exhaustion
const MAX_BULK_STRING_SIZE: usize = 512 * 1024 * 1024;

/// Maximum array size (1M elements) - prevents DoS via array bomb
const MAX_ARRAY_SIZE: usize = 1024 * 1024;

/// RESP data types
#[derive(Debug, Clone, PartialEq)]
pub enum RespValue {
    /// Simple string: +OK\r\n
    SimpleString(String),
    /// Error: -Error message\r\n
    Error(String),
    /// Integer: :1000\r\n
    Integer(i64),
    /// Bulk string: $6\r\nfoobar\r\n
    BulkString(Option<Vec<u8>>),
    /// Array: *2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n
    Array(Option<Vec<RespValue>>),
}

impl RespValue {
    /// Serialize to RESP format
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            RespValue::SimpleString(s) => format!("+{}\r\n", s).into_bytes(),
            RespValue::Error(e) => format!("-{}\r\n", e).into_bytes(),
            RespValue::Integer(i) => format!(":{}\r\n", i).into_bytes(),
            RespValue::BulkString(None) => b"$-1\r\n".to_vec(),
            RespValue::BulkString(Some(data)) => {
                let mut result = format!("${}\r\n", data.len()).into_bytes();
                result.extend_from_slice(data);
                result.extend_from_slice(b"\r\n");
                result
            }
            RespValue::Array(None) => b"*-1\r\n".to_vec(),
            RespValue::Array(Some(arr)) => {
                let mut result = format!("*{}\r\n", arr.len()).into_bytes();
                for val in arr {
                    result.extend_from_slice(&val.serialize());
                }
                result
            }
        }
    }

    /// Parse RESP from buffer
    pub fn parse(buf: &mut BytesMut) -> Result<Option<RespValue>, String> {
        if buf.is_empty() {
            return Ok(None);
        }

        let mut cursor = Cursor::new(&buf[..]);
        match parse_value(&mut cursor) {
            Ok(Some(value)) => {
                let pos = cursor.position() as usize;
                buf.advance(pos);
                Ok(Some(value))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

fn parse_value(cursor: &mut Cursor<&[u8]>) -> Result<Option<RespValue>, String> {
    if !cursor.has_remaining() {
        return Ok(None);
    }

    let type_byte = cursor.get_u8();

    match type_byte {
        b'+' => parse_simple_string(cursor),
        b'-' => parse_error(cursor),
        b':' => parse_integer(cursor),
        b'$' => parse_bulk_string(cursor),
        b'*' => parse_array(cursor),
        _ => Err(format!("Unknown RESP type: {}", type_byte as char)),
    }
}

fn parse_simple_string(cursor: &mut Cursor<&[u8]>) -> Result<Option<RespValue>, String> {
    match read_line(cursor)? {
        Some(line) => Ok(Some(RespValue::SimpleString(
            String::from_utf8(line).map_err(|e| e.to_string())?,
        ))),
        None => Ok(None),
    }
}

fn parse_error(cursor: &mut Cursor<&[u8]>) -> Result<Option<RespValue>, String> {
    match read_line(cursor)? {
        Some(line) => Ok(Some(RespValue::Error(
            String::from_utf8(line).map_err(|e| e.to_string())?,
        ))),
        None => Ok(None),
    }
}

fn parse_integer(cursor: &mut Cursor<&[u8]>) -> Result<Option<RespValue>, String> {
    match read_line(cursor)? {
        Some(line) => {
            let s = String::from_utf8(line).map_err(|e| e.to_string())?;
            let num = s.parse::<i64>().map_err(|e| e.to_string())?;
            Ok(Some(RespValue::Integer(num)))
        }
        None => Ok(None),
    }
}

fn parse_bulk_string(cursor: &mut Cursor<&[u8]>) -> Result<Option<RespValue>, String> {
    let len_line = match read_line(cursor)? {
        Some(line) => line,
        None => return Ok(None),
    };

    let len_str = String::from_utf8(len_line).map_err(|e| e.to_string())?;
    let len = len_str.parse::<i64>().map_err(|e| e.to_string())?;

    if len == -1 {
        return Ok(Some(RespValue::BulkString(None)));
    }

    let len = len as usize;

    // Security: Prevent DoS via large bulk string allocation
    if len > MAX_BULK_STRING_SIZE {
        return Err(format!(
            "ERR bulk string too large: {} bytes (max: {} bytes)",
            len, MAX_BULK_STRING_SIZE
        ));
    }

    // Check if we have enough data
    let remaining = cursor.remaining();
    if remaining < len + 2 {
        return Ok(None); // Need more data
    }

    let mut data = vec![0u8; len];
    cursor.copy_to_slice(&mut data);

    // Read \r\n
    if cursor.remaining() < 2 {
        return Ok(None);
    }
    let cr = cursor.get_u8();
    let lf = cursor.get_u8();
    if cr != b'\r' || lf != b'\n' {
        return Err("Expected \\r\\n after bulk string".to_string());
    }

    Ok(Some(RespValue::BulkString(Some(data))))
}

fn parse_array(cursor: &mut Cursor<&[u8]>) -> Result<Option<RespValue>, String> {
    let len_line = match read_line(cursor)? {
        Some(line) => line,
        None => return Ok(None),
    };

    let len_str = String::from_utf8(len_line).map_err(|e| e.to_string())?;
    let len = len_str.parse::<i64>().map_err(|e| e.to_string())?;

    if len == -1 {
        return Ok(Some(RespValue::Array(None)));
    }

    let len = len as usize;

    // Security: Prevent DoS via large array allocation
    if len > MAX_ARRAY_SIZE {
        return Err(format!(
            "ERR array too large: {} elements (max: {} elements)",
            len, MAX_ARRAY_SIZE
        ));
    }

    let mut arr = Vec::with_capacity(len);

    for _ in 0..len {
        match parse_value(cursor)? {
            Some(val) => arr.push(val),
            None => return Ok(None), // Need more data
        }
    }

    Ok(Some(RespValue::Array(Some(arr))))
}

fn read_line(cursor: &mut Cursor<&[u8]>) -> Result<Option<Vec<u8>>, String> {
    let start = cursor.position() as usize;
    let slice = &cursor.get_ref()[start..];

    // Find \r\n
    for (i, window) in slice.windows(2).enumerate() {
        if window == b"\r\n" {
            let end = start + i;
            cursor.set_position((end + 2) as u64);
            return Ok(Some(cursor.get_ref()[start..end].to_vec()));
        }
    }

    Ok(None) // Need more data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_string() {
        let data = b"+OK\r\n";
        let mut buf = BytesMut::from(&data[..]);
        let val = RespValue::parse(&mut buf).unwrap().unwrap();
        assert_eq!(val, RespValue::SimpleString("OK".to_string()));
        assert_eq!(val.serialize(), data);
    }

    #[test]
    fn test_error() {
        let data = b"-Error message\r\n";
        let mut buf = BytesMut::from(&data[..]);
        let val = RespValue::parse(&mut buf).unwrap().unwrap();
        assert_eq!(val, RespValue::Error("Error message".to_string()));
        assert_eq!(val.serialize(), data);
    }

    #[test]
    fn test_integer() {
        let data = b":1000\r\n";
        let mut buf = BytesMut::from(&data[..]);
        let val = RespValue::parse(&mut buf).unwrap().unwrap();
        assert_eq!(val, RespValue::Integer(1000));
        assert_eq!(val.serialize(), data);
    }

    #[test]
    fn test_bulk_string() {
        let data = b"$6\r\nfoobar\r\n";
        let mut buf = BytesMut::from(&data[..]);
        let val = RespValue::parse(&mut buf).unwrap().unwrap();
        assert_eq!(val, RespValue::BulkString(Some(b"foobar".to_vec())));
        assert_eq!(val.serialize(), data);
    }

    #[test]
    fn test_null_bulk_string() {
        let data = b"$-1\r\n";
        let mut buf = BytesMut::from(&data[..]);
        let val = RespValue::parse(&mut buf).unwrap().unwrap();
        assert_eq!(val, RespValue::BulkString(None));
        assert_eq!(val.serialize(), data);
    }

    #[test]
    fn test_array() {
        let data = b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n";
        let mut buf = BytesMut::from(&data[..]);
        let val = RespValue::parse(&mut buf).unwrap().unwrap();
        assert_eq!(
            val,
            RespValue::Array(Some(vec![
                RespValue::BulkString(Some(b"foo".to_vec())),
                RespValue::BulkString(Some(b"bar".to_vec())),
            ]))
        );
        assert_eq!(val.serialize(), data);
    }

    #[test]
    fn test_command_array() {
        let data = b"*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n";
        let mut buf = BytesMut::from(&data[..]);
        let val = RespValue::parse(&mut buf).unwrap().unwrap();

        if let RespValue::Array(Some(arr)) = val {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], RespValue::BulkString(Some(b"SET".to_vec())));
            assert_eq!(arr[1], RespValue::BulkString(Some(b"key".to_vec())));
            assert_eq!(arr[2], RespValue::BulkString(Some(b"value".to_vec())));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_incomplete_data() {
        let data = b"$6\r\nfoo"; // Incomplete
        let mut buf = BytesMut::from(&data[..]);
        let val = RespValue::parse(&mut buf).unwrap();
        assert!(val.is_none()); // Should return None, not error
    }
}
