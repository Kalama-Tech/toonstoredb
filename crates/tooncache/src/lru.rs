//! LRU (Least Recently Used) cache implementation
//!
//! Uses intrusive linked list for O(1) eviction.

use std::collections::HashMap;
use std::hash::Hash;
use ahash::RandomState;

/// Node in the LRU doubly-linked list
struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,
    next: Option<usize>,
}

/// LRU cache with fixed capacity
pub struct LruCache<K, V> {
    map: HashMap<K, usize, RandomState>,
    nodes: Vec<Option<Node<K, V>>>,
    head: Option<usize>,
    tail: Option<usize>,
    free_list: Vec<usize>,
    capacity: usize,
}

impl<K, V> LruCache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    /// Create a new LRU cache with the given capacity
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity must be greater than 0");
        
        Self {
            map: HashMap::with_capacity_and_hasher(capacity, RandomState::new()),
            nodes: Vec::with_capacity(capacity),
            head: None,
            tail: None,
            free_list: Vec::new(),
            capacity,
        }
    }

    /// Get a value from the cache
    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some(&idx) = self.map.get(key) {
            self.move_to_front(idx);
            self.nodes[idx].as_ref().map(|node| &node.value)
        } else {
            None
        }
    }

    /// Insert a key-value pair into the cache
    pub fn put(&mut self, key: K, value: V) {
        if let Some(&idx) = self.map.get(&key) {
            // Update existing
            if let Some(node) = &mut self.nodes[idx] {
                node.value = value;
            }
            self.move_to_front(idx);
        } else {
            // Insert new
            if self.map.len() >= self.capacity {
                self.evict();
            }
            
            let idx = self.alloc_node();
            self.nodes[idx] = Some(Node {
                key: key.clone(),
                value,
                prev: None,
                next: self.head,
            });
            
            if let Some(head_idx) = self.head {
                if let Some(head) = &mut self.nodes[head_idx] {
                    head.prev = Some(idx);
                }
            }
            
            self.head = Some(idx);
            if self.tail.is_none() {
                self.tail = Some(idx);
            }
            
            self.map.insert(key, idx);
        }
    }

    /// Remove a key from the cache
    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(idx) = self.map.remove(key) {
            self.unlink(idx);
            self.free_node(idx);
            self.nodes[idx].take().map(|node| node.value)
        } else {
            None
        }
    }

    /// Get the current size of the cache
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.map.clear();
        self.nodes.clear();
        self.free_list.clear();
        self.head = None;
        self.tail = None;
    }

    fn move_to_front(&mut self, idx: usize) {
        if self.head == Some(idx) {
            return; // Already at front
        }

        self.unlink(idx);
        
        if let Some(node) = &mut self.nodes[idx] {
            node.prev = None;
            node.next = self.head;
        }
        
        if let Some(head_idx) = self.head {
            if let Some(head) = &mut self.nodes[head_idx] {
                head.prev = Some(idx);
            }
        }
        
        self.head = Some(idx);
    }

    fn unlink(&mut self, idx: usize) {
        let (prev, next) = if let Some(node) = &self.nodes[idx] {
            (node.prev, node.next)
        } else {
            return;
        };

        match prev {
            Some(prev_idx) => {
                if let Some(prev_node) = &mut self.nodes[prev_idx] {
                    prev_node.next = next;
                }
            }
            None => {
                self.head = next;
            }
        }

        match next {
            Some(next_idx) => {
                if let Some(next_node) = &mut self.nodes[next_idx] {
                    next_node.prev = prev;
                }
            }
            None => {
                self.tail = prev;
            }
        }
    }

    fn evict(&mut self) {
        if let Some(tail_idx) = self.tail {
            if let Some(node) = self.nodes[tail_idx].take() {
                self.map.remove(&node.key);
                self.unlink(tail_idx);
                self.free_node(tail_idx);
            }
        }
    }

    fn alloc_node(&mut self) -> usize {
        if let Some(idx) = self.free_list.pop() {
            idx
        } else {
            let idx = self.nodes.len();
            self.nodes.push(None);
            idx
        }
    }

    fn free_node(&mut self, idx: usize) {
        self.free_list.push(idx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_basic() {
        let mut cache = LruCache::new(2);
        
        cache.put(1, "a");
        cache.put(2, "b");
        
        assert_eq!(cache.get(&1), Some(&"a"));
        assert_eq!(cache.get(&2), Some(&"b"));
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_lru_eviction() {
        let mut cache = LruCache::new(2);
        
        cache.put(1, "a");
        cache.put(2, "b");
        cache.put(3, "c"); // Should evict 1
        
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some(&"b"));
        assert_eq!(cache.get(&3), Some(&"c"));
    }

    #[test]
    fn test_lru_update() {
        let mut cache = LruCache::new(2);
        
        cache.put(1, "a");
        cache.put(2, "b");
        cache.get(&1); // Move 1 to front
        cache.put(3, "c"); // Should evict 2
        
        assert_eq!(cache.get(&1), Some(&"a"));
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&3), Some(&"c"));
    }

    #[test]
    fn test_lru_remove() {
        let mut cache = LruCache::new(3);
        
        cache.put(1, "a");
        cache.put(2, "b");
        cache.put(3, "c");
        
        assert_eq!(cache.remove(&2), Some("b"));
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(&2), None);
    }

    #[test]
    fn test_lru_clear() {
        let mut cache = LruCache::new(3);
        
        cache.put(1, "a");
        cache.put(2, "b");
        cache.clear();
        
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_lru_overwrite() {
        let mut cache = LruCache::new(2);
        
        cache.put(1, "a");
        cache.put(1, "b"); // Overwrite
        
        assert_eq!(cache.get(&1), Some(&"b"));
        assert_eq!(cache.len(), 1);
    }
}
