//! # toonstoredb
//!
//! Embedded database engine using TOON (Token-Oriented Object Notation) format.
//!
//! ## Week 1 Goals
//! - Single-threaded writer, unlimited readers
//! - Memory-mapped storage
//! - Basic operations: GET, PUT, DELETE, SCAN
//! - 1 MB max value, 1 GB max DB size

#![warn(missing_docs)]

mod parser;
mod storage;
mod error;

pub use error::{Error, Result};
pub use storage::ToonStore;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
