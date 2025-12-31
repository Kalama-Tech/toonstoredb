//! ToonCache: LRU cache wrapping ToonStore

use std::path::Path;
use std::sync::Arc;
use parking_lot::RwLock;
use toonstoredb::{ToonStore, Result, Error};

use crate::lru::LruCache;
use crate::stats::CacheStats;

/// Cached storage layer combining LRU cache with ToonStore backend
pub struct ToonCache {
    /// Underlying persistent storage
    store: Arc<ToonStore>,
    
    /// LRU cache for hot data
    cache: Arc<RwLock<LruCache<u64, Vec<u8>>>>,
    
    /// Cache statistics
    stats: Arc<CacheStats>,
    
    /// Cache capacity
    capacity: usize,
}

impl ToonCache {
    /// Create a new ToonCache with the given capacity
    ///
    /// # Arguments
    /// * `path` - Database directory path
    /// * `capacity` - Maximum number of items in cache
    ///
    /// # Returns
    /// * `Result<ToonCache>` - Cache-enabled database handle
    pub fn new<P: AsRef<Path>>(path: P, capacity: usize) -> Result<Self> {
        let store = ToonStore::open(path)?;
        
        Ok(Self {
            store: Arc::new(store),
            cache: Arc::new(RwLock::new(LruCache::new(capacity))),
            stats: Arc::new(CacheStats::new()),
            capacity,
        })
    }

    /// Put a value into the database and cache
    ///
    /// # Arguments
    /// * `line` - Raw TOON line data
    ///
    /// # Returns
    /// * `Result<u64>` - Row ID of inserted line
    pub fn put(&self, line: &[u8]) -> Result<u64> {
        let row_id = self.store.put(line)?;
        
        // Cache the value
        let mut cache = self.cache.write();
        cache.put(row_id, line.to_vec());
        self.stats.record_insert();
        
        Ok(row_id)
    }

    /// Get a value from cache or storage
    ///
    /// # Arguments
    /// * `row_id` - Row ID to retrieve
    ///
    /// # Returns
    /// * `Result<Vec<u8>>` - Raw TOON line data
    pub fn get(&self, row_id: u64) -> Result<Vec<u8>> {
        // Try cache first
        {
            let mut cache = self.cache.write();
            if let Some(value) = cache.get(&row_id) {
                self.stats.record_hit();
                return Ok(value.clone());
            }
        }
        
        // Cache miss - fetch from storage
        self.stats.record_miss();
        let value = self.store.get(row_id)?;
        
        // Update cache
        let mut cache = self.cache.write();
        cache.put(row_id, value.clone());
        
        Ok(value)
    }

    /// Delete a value from cache and storage
    ///
    /// # Arguments
    /// * `row_id` - Row ID to delete
    ///
    /// # Returns
    /// * `Result<()>` - Ok if deleted
    pub fn delete(&self, row_id: u64) -> Result<()> {
        // Remove from cache
        let mut cache = self.cache.write();
        cache.remove(&row_id);
        
        // Delete from storage
        self.store.delete(row_id)
    }

    /// Scan all non-deleted rows (bypasses cache)
    ///
    /// # Returns
    /// * Iterator over (row_id, data) pairs
    pub fn scan(&self) -> impl Iterator<Item = Result<(u64, Vec<u8>)>> + '_ {
        self.store.scan()
    }

    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Get current cache size
    pub fn cache_len(&self) -> usize {
        self.cache.read().len()
    }

    /// Get cache capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Clear the cache (storage remains unchanged)
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write();
        cache.clear();
        self.stats.reset();
    }

    /// Get the number of rows in storage
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// Check if the database is empty
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// Close the database and sync to disk
    pub fn close(self) -> Result<()> {
        // Cache is dropped automatically
        // Extract store from Arc
        match Arc::try_unwrap(self.store) {
            Ok(mut store) => store.close(),
            Err(_) => Err(Error::Closed), // Still has references
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_basic() {
        let dir = TempDir::new().unwrap();
        let cache = ToonCache::new(dir.path(), 10).unwrap();

        let row_id = cache.put(b"test data").unwrap();
        let data = cache.get(row_id).unwrap();

        assert_eq!(data, b"test data");
        // Put() adds to cache, so first get() is a hit
        assert_eq!(cache.stats().hits(), 1);
        assert_eq!(cache.stats().misses(), 0);
    }

    #[test]
    fn test_cache_hit() {
        let dir = TempDir::new().unwrap();
        let cache = ToonCache::new(dir.path(), 10).unwrap();

        let row_id = cache.put(b"test data").unwrap();
        
        // First get - cache hit (put cached it)
        cache.get(row_id).unwrap();
        assert_eq!(cache.stats().hits(), 1);
        
        // Second get - cache hit
        cache.get(row_id).unwrap();
        assert_eq!(cache.stats().hits(), 2);
    }

    #[test]
    fn test_cache_eviction() {
        let dir = TempDir::new().unwrap();
        let cache = ToonCache::new(dir.path(), 2).unwrap();

        let id0 = cache.put(b"data 0").unwrap();
        let id1 = cache.put(b"data 1").unwrap();
        
        // Cache now: [id1 (head), id0 (tail)]
        assert_eq!(cache.cache_len(), 2);
        
        let id2 = cache.put(b"data 2").unwrap();
        
        // Cache should evict id0 (LRU), now: [id2 (head), id1]
        assert_eq!(cache.cache_len(), 2);
        
        // Verify id1 and id2 are cached
        cache.get(id1).unwrap();
        cache.get(id2).unwrap();
        assert_eq!(cache.stats().hits(), 2);
        
        // id0 should be evicted (cache miss)
        cache.get(id0).unwrap();
        assert_eq!(cache.stats().misses(), 1);
    }

    #[test]
    fn test_cache_delete() {
        let dir = TempDir::new().unwrap();
        let cache = ToonCache::new(dir.path(), 10).unwrap();

        let row_id = cache.put(b"test data").unwrap();
        cache.delete(row_id).unwrap();

        let result = cache.get(row_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_stats() {
        let dir = TempDir::new().unwrap();
        let cache = ToonCache::new(dir.path(), 10).unwrap();

        let id0 = cache.put(b"data 0").unwrap();
        let id1 = cache.put(b"data 1").unwrap();

        // Both are cached from put, so all gets are hits
        cache.get(id0).unwrap(); // hit
        cache.get(id0).unwrap(); // hit
        cache.get(id1).unwrap(); // hit
        cache.get(id1).unwrap(); // hit

        assert_eq!(cache.stats().hits(), 4);
        assert_eq!(cache.stats().misses(), 0);
        assert_eq!(cache.stats().hit_ratio(), 1.0);
    }

    #[test]
    fn test_cache_clear() {
        let dir = TempDir::new().unwrap();
        let cache = ToonCache::new(dir.path(), 10).unwrap();

        cache.put(b"data 0").unwrap();
        cache.put(b"data 1").unwrap();

        assert_eq!(cache.cache_len(), 2);
        
        cache.clear_cache();
        
        assert_eq!(cache.cache_len(), 0);
        assert_eq!(cache.stats().hits(), 0);
    }

    #[test]
    fn test_cache_scan() {
        let dir = TempDir::new().unwrap();
        let cache = ToonCache::new(dir.path(), 10).unwrap();

        cache.put(b"line 0").unwrap();
        cache.put(b"line 1").unwrap();
        cache.put(b"line 2").unwrap();

        let results: Vec<_> = cache.scan().collect();
        assert_eq!(results.len(), 3);
    }
}
