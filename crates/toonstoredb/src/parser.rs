//! TOON format parser using nom
//!
//! File format:
//! ```text
//! TOON001
//! [version: u32]
//! [row_count: u32]
//! ...TOON lines (\n-terminated)...
//! ```
//!
//! TOON line format (from toondb spec):
//! ```text
//! collection[count]{field1,field2,...}:
//!   value1,value2,...
//!   value1,value2,...
//! ```

use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{char, digit1, multispace0, multispace1},
    combinator::{map_res, opt},
    multi::separated_list0,
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

use crate::error::{Error, Result};

/// Magic header for TOON files
pub const TOON_MAGIC: &[u8] = b"TOON001\n";

/// Magic header for TOON index files
pub const TOON_IDX_MAGIC: &[u8] = b"TOONIDX1";

/// TOON file header
#[derive(Debug, Clone, PartialEq)]
pub struct ToonHeader {
    /// File format version
    pub version: u32,
    /// Number of rows in the file
    pub row_count: u32,
}

/// Parse TOON file header
///
/// Format:
/// ```text
/// TOON001\n
/// [4 bytes: version u32 little-endian]
/// [4 bytes: row_count u32 little-endian]
/// ```
pub fn parse_header(input: &[u8]) -> Result<ToonHeader> {
    if input.len() < TOON_MAGIC.len() + 8 {
        return Err(Error::Parse("Input too short for header".to_string()));
    }

    // Check magic
    if &input[0..TOON_MAGIC.len()] != TOON_MAGIC {
        return Err(Error::Parse("Invalid TOON magic header".to_string()));
    }

    // Parse version and row_count (little-endian u32)
    let version_bytes = &input[TOON_MAGIC.len()..TOON_MAGIC.len() + 4];
    let row_count_bytes = &input[TOON_MAGIC.len() + 4..TOON_MAGIC.len() + 8];

    let version = u32::from_le_bytes([
        version_bytes[0],
        version_bytes[1],
        version_bytes[2],
        version_bytes[3],
    ]);

    let row_count = u32::from_le_bytes([
        row_count_bytes[0],
        row_count_bytes[1],
        row_count_bytes[2],
        row_count_bytes[3],
    ]);

    Ok(ToonHeader { version, row_count })
}

/// Create a TOON file header
pub fn create_header(version: u32, row_count: u32) -> Vec<u8> {
    let mut header = Vec::with_capacity(TOON_MAGIC.len() + 8);
    header.extend_from_slice(TOON_MAGIC);
    header.extend_from_slice(&version.to_le_bytes());
    header.extend_from_slice(&row_count.to_le_bytes());
    header
}

/// Parse a single TOON line (raw, no interpretation)
///
/// Returns the line content without the trailing newline
pub fn parse_line(input: &[u8]) -> IResult<&[u8], &[u8]> {
    terminated(take_until("\n"), char('\n'))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header() {
        let header = create_header(1, 42);
        let parsed = parse_header(&header).unwrap();
        
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.row_count, 42);
    }

    #[test]
    fn test_parse_header_invalid_magic() {
        let mut header = create_header(1, 0);
        header[0] = b'X'; // Corrupt magic
        
        let result = parse_header(&header);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_header_too_short() {
        let header = b"TOON001\n";
        let result = parse_header(header);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_line() {
        let input = b"users[2]{id,name}:\nmore data";
        let (remaining, line) = parse_line(input).unwrap();
        
        assert_eq!(line, b"users[2]{id,name}:");
        assert_eq!(remaining, b"more data");
    }

    #[test]
    fn test_create_header_format() {
        let header = create_header(1, 100);
        
        // Check magic
        assert_eq!(&header[0..8], TOON_MAGIC);
        
        // Check version (little-endian)
        assert_eq!(u32::from_le_bytes([header[8], header[9], header[10], header[11]]), 1);
        
        // Check row_count (little-endian)
        assert_eq!(u32::from_le_bytes([header[12], header[13], header[14], header[15]]), 100);
    }
}
