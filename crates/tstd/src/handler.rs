//! Command handler for RESP server

use crate::resp::RespValue;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tooncache::ToonCache;

pub struct CommandHandler {
    cache: Arc<ToonCache>,
    key_map: Arc<RwLock<HashMap<String, u64>>>,
}

impl CommandHandler {
    pub fn new(cache: Arc<ToonCache>) -> Self {
        Self {
            cache,
            key_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn handle(&self, cmd: RespValue) -> RespValue {
        let arr = match cmd {
            RespValue::Array(Some(arr)) if !arr.is_empty() => arr,
            _ => return RespValue::Error("ERR invalid command format".to_string()),
        };

        let command = match &arr[0] {
            RespValue::BulkString(Some(cmd)) => String::from_utf8_lossy(cmd).to_uppercase(),
            _ => return RespValue::Error("ERR invalid command".to_string()),
        };

        match command.as_str() {
            "PING" => self.handle_ping(&arr[1..]),
            "ECHO" => self.handle_echo(&arr[1..]),
            "GET" => self.handle_get(&arr[1..]),
            "SET" => self.handle_set(&arr[1..]),
            "DEL" => self.handle_del(&arr[1..]),
            "EXISTS" => self.handle_exists(&arr[1..]),
            "KEYS" => self.handle_keys(&arr[1..]),
            "DBSIZE" => self.handle_dbsize(),
            "FLUSHDB" => self.handle_flushdb(),
            "INFO" => self.handle_info(&arr[1..]),
            "COMMAND" => self.handle_command(&arr[1..]),
            _ => RespValue::Error(format!("ERR unknown command '{}'", command)),
        }
    }

    fn handle_ping(&self, args: &[RespValue]) -> RespValue {
        if args.is_empty() {
            RespValue::SimpleString("PONG".to_string())
        } else if args.len() == 1 {
            // PING with message
            args[0].clone()
        } else {
            RespValue::Error("ERR wrong number of arguments for 'ping' command".to_string())
        }
    }

    fn handle_echo(&self, args: &[RespValue]) -> RespValue {
        if args.len() != 1 {
            return RespValue::Error(
                "ERR wrong number of arguments for 'echo' command".to_string(),
            );
        }
        args[0].clone()
    }

    fn handle_get(&self, args: &[RespValue]) -> RespValue {
        if args.len() != 1 {
            return RespValue::Error("ERR wrong number of arguments for 'get' command".to_string());
        }

        let key = match &args[0] {
            RespValue::BulkString(Some(k)) => match String::from_utf8(k.clone()) {
                Ok(s) => s,
                Err(_) => return RespValue::Error("ERR invalid key".to_string()),
            },
            _ => return RespValue::Error("ERR invalid key type".to_string()),
        };

        // Look up row_id from key_map
        let key_map = self.key_map.read().unwrap();
        let row_id = match key_map.get(&key) {
            Some(id) => *id,
            None => return RespValue::BulkString(None), // Key not found
        };

        match self.cache.get(row_id) {
            Ok(data) => RespValue::BulkString(Some(data)),
            Err(_) => RespValue::BulkString(None),
        }
    }

    fn handle_set(&self, args: &[RespValue]) -> RespValue {
        if args.len() < 2 {
            return RespValue::Error("ERR wrong number of arguments for 'set' command".to_string());
        }

        let key = match &args[0] {
            RespValue::BulkString(Some(k)) => match String::from_utf8(k.clone()) {
                Ok(s) => s,
                Err(_) => return RespValue::Error("ERR invalid key".to_string()),
            },
            _ => return RespValue::Error("ERR invalid key type".to_string()),
        };

        let value = match &args[1] {
            RespValue::BulkString(Some(v)) => v,
            _ => return RespValue::Error("ERR invalid value type".to_string()),
        };

        // Check if key already exists
        let mut key_map = self.key_map.write().unwrap();

        if let Some(&existing_row_id) = key_map.get(&key) {
            // Update existing key - delete old value first
            let _ = self.cache.delete(existing_row_id);
        }

        // Insert new value and map key to row_id
        match self.cache.put(value) {
            Ok(row_id) => {
                key_map.insert(key, row_id);
                RespValue::SimpleString("OK".to_string())
            }
            Err(e) => RespValue::Error(format!("ERR {}", e)),
        }
    }

    fn handle_del(&self, args: &[RespValue]) -> RespValue {
        if args.is_empty() {
            return RespValue::Error("ERR wrong number of arguments for 'del' command".to_string());
        }

        let mut deleted = 0i64;
        let mut key_map = self.key_map.write().unwrap();

        for arg in args {
            if let RespValue::BulkString(Some(k)) = arg {
                if let Ok(key) = String::from_utf8(k.clone()) {
                    if let Some(row_id) = key_map.remove(&key) {
                        if self.cache.delete(row_id).is_ok() {
                            deleted += 1;
                        }
                    }
                }
            }
        }

        RespValue::Integer(deleted)
    }

    fn handle_exists(&self, args: &[RespValue]) -> RespValue {
        if args.is_empty() {
            return RespValue::Error(
                "ERR wrong number of arguments for 'exists' command".to_string(),
            );
        }

        let mut count = 0i64;
        let key_map = self.key_map.read().unwrap();

        for arg in args {
            if let RespValue::BulkString(Some(k)) = arg {
                if let Ok(key) = String::from_utf8(k.clone()) {
                    if key_map.contains_key(&key) {
                        count += 1;
                    }
                }
            }
        }

        RespValue::Integer(count)
    }

    fn handle_keys(&self, args: &[RespValue]) -> RespValue {
        let pattern = if args.is_empty() {
            "*".to_string()
        } else {
            match &args[0] {
                RespValue::BulkString(Some(p)) => match String::from_utf8(p.clone()) {
                    Ok(s) => s,
                    Err(_) => return RespValue::Error("ERR invalid pattern".to_string()),
                },
                _ => return RespValue::Error("ERR invalid pattern type".to_string()),
            }
        };

        let key_map = self.key_map.read().unwrap();
        let mut matching_keys = Vec::new();

        for key in key_map.keys() {
            if matches_pattern(key, &pattern) {
                matching_keys.push(RespValue::BulkString(Some(key.as_bytes().to_vec())));
            }
        }

        RespValue::Array(Some(matching_keys))
    }

    fn handle_dbsize(&self) -> RespValue {
        let key_map = self.key_map.read().unwrap();
        RespValue::Integer(key_map.len() as i64)
    }

    fn handle_flushdb(&self) -> RespValue {
        let mut key_map = self.key_map.write().unwrap();
        key_map.clear();
        self.cache.clear_cache();
        RespValue::SimpleString("OK".to_string())
    }

    fn handle_info(&self, _args: &[RespValue]) -> RespValue {
        let stats = self.cache.stats();
        let key_map = self.key_map.read().unwrap();
        let info = format!(
            "# Server\r\n\
             toonstore_version:0.1.0\r\n\
             \r\n\
             # Stats\r\n\
             total_keys:{}\r\n\
             cache_size:{}\r\n\
             cache_capacity:{}\r\n\
             cache_hits:{}\r\n\
             cache_misses:{}\r\n\
             cache_hit_ratio:{:.2}\r\n",
            key_map.len(),
            self.cache.cache_len(),
            self.cache.capacity(),
            stats.hits(),
            stats.misses(),
            stats.hit_ratio(),
        );
        RespValue::BulkString(Some(info.into_bytes()))
    }

    fn handle_command(&self, _args: &[RespValue]) -> RespValue {
        // Return empty array for COMMAND (redis-cli compatibility)
        RespValue::Array(Some(vec![]))
    }
}

/// Simple glob pattern matching for Redis KEYS command
/// Supports: * (matches any sequence), ? (matches single char)
fn matches_pattern(key: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    let key_chars: Vec<char> = key.chars().collect();
    let pattern_chars: Vec<char> = pattern.chars().collect();

    let mut key_idx = 0;
    let mut pattern_idx = 0;
    let mut star_idx = None;
    let mut match_idx = 0;

    while key_idx < key_chars.len() {
        if pattern_idx < pattern_chars.len() {
            match pattern_chars[pattern_idx] {
                '*' => {
                    star_idx = Some(pattern_idx);
                    match_idx = key_idx;
                    pattern_idx += 1;
                    continue;
                }
                '?' => {
                    key_idx += 1;
                    pattern_idx += 1;
                    continue;
                }
                c if c == key_chars[key_idx] => {
                    key_idx += 1;
                    pattern_idx += 1;
                    continue;
                }
                _ => {}
            }
        }

        // No match, backtrack to last star if exists
        if let Some(star) = star_idx {
            pattern_idx = star + 1;
            match_idx += 1;
            key_idx = match_idx;
        } else {
            return false;
        }
    }

    // Check remaining pattern chars are all stars
    while pattern_idx < pattern_chars.len() && pattern_chars[pattern_idx] == '*' {
        pattern_idx += 1;
    }

    pattern_idx == pattern_chars.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_ping() {
        let dir = TempDir::new().unwrap();
        let cache = Arc::new(ToonCache::new(dir.path(), 100).unwrap());
        let handler = CommandHandler::new(cache);

        let cmd = RespValue::Array(Some(vec![RespValue::BulkString(Some(b"PING".to_vec()))]));

        let resp = handler.handle(cmd);
        assert_eq!(resp, RespValue::SimpleString("PONG".to_string()));
    }

    #[test]
    fn test_echo() {
        let dir = TempDir::new().unwrap();
        let cache = Arc::new(ToonCache::new(dir.path(), 100).unwrap());
        let handler = CommandHandler::new(cache);

        let cmd = RespValue::Array(Some(vec![
            RespValue::BulkString(Some(b"ECHO".to_vec())),
            RespValue::BulkString(Some(b"hello".to_vec())),
        ]));

        let resp = handler.handle(cmd);
        assert_eq!(resp, RespValue::BulkString(Some(b"hello".to_vec())));
    }

    #[test]
    fn test_set_and_get() {
        let dir = TempDir::new().unwrap();
        let cache = Arc::new(ToonCache::new(dir.path(), 100).unwrap());
        let handler = CommandHandler::new(cache);

        // SET key value
        let set_cmd = RespValue::Array(Some(vec![
            RespValue::BulkString(Some(b"SET".to_vec())),
            RespValue::BulkString(Some(b"mykey".to_vec())),
            RespValue::BulkString(Some(b"myvalue".to_vec())),
        ]));

        let resp = handler.handle(set_cmd);
        assert_eq!(resp, RespValue::SimpleString("OK".to_string()));
    }
}
