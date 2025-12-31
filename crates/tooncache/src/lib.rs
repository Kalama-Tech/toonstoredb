//! # tooncache
//!
//! High-performance LRU cache layer for ToonStore.
//!
//! ## Architecture
//! - **HashMap**: AHash for fast lookups (O(1))
//! - **LRU List**: Doubly-linked list for eviction (O(1))
//! - **Integration**: Wraps ToonStore for transparent caching
//!
//! ## Week 2 Goals
//! - LRU eviction policy
//! - Configurable cache size
//! - Hit/miss statistics
//! - Target: 500k+ ops/sec for cached reads

#![warn(missing_docs)]

mod cache;
mod lru;
mod stats;

pub use cache::ToonCache;
pub use stats::CacheStats;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
