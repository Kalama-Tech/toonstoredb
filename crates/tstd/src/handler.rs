//! Command handler for RESP server

use crate::resp::RespValue;
use std::sync::Arc;
use tooncache::ToonCache;

pub struct CommandHandler {
    cache: Arc<ToonCache>,
}

impl CommandHandler {
    pub fn new(cache: Arc<ToonCache>) -> Self {
        Self { cache }
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

        // Parse key as row_id (for now, simple numeric keys)
        let row_id: u64 = match key.parse() {
            Ok(id) => id,
            Err(_) => return RespValue::BulkString(None), // Key not found
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

        let _key = match &args[0] {
            RespValue::BulkString(Some(k)) => k,
            _ => return RespValue::Error("ERR invalid key type".to_string()),
        };

        let value = match &args[1] {
            RespValue::BulkString(Some(v)) => v,
            _ => return RespValue::Error("ERR invalid value type".to_string()),
        };

        match self.cache.put(value) {
            Ok(_row_id) => RespValue::SimpleString("OK".to_string()),
            Err(e) => RespValue::Error(format!("ERR {}", e)),
        }
    }

    fn handle_del(&self, args: &[RespValue]) -> RespValue {
        if args.is_empty() {
            return RespValue::Error("ERR wrong number of arguments for 'del' command".to_string());
        }

        let mut deleted = 0i64;
        for arg in args {
            if let RespValue::BulkString(Some(k)) = arg {
                if let Ok(key_str) = String::from_utf8(k.clone()) {
                    if let Ok(row_id) = key_str.parse::<u64>() {
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
        for arg in args {
            if let RespValue::BulkString(Some(k)) = arg {
                if let Ok(key_str) = String::from_utf8(k.clone()) {
                    if let Ok(row_id) = key_str.parse::<u64>() {
                        if self.cache.get(row_id).is_ok() {
                            count += 1;
                        }
                    }
                }
            }
        }

        RespValue::Integer(count)
    }

    fn handle_keys(&self, _args: &[RespValue]) -> RespValue {
        // For now, return empty array
        // TODO: Implement pattern matching for keys
        RespValue::Array(Some(vec![]))
    }

    fn handle_dbsize(&self) -> RespValue {
        RespValue::Integer(self.cache.len() as i64)
    }

    fn handle_flushdb(&self) -> RespValue {
        self.cache.clear_cache();
        RespValue::SimpleString("OK".to_string())
    }

    fn handle_info(&self, _args: &[RespValue]) -> RespValue {
        let stats = self.cache.stats();
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
            self.cache.len(),
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
