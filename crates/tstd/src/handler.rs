//! Command handler for RESP server

use crate::auth::{AuthConfig, SessionState};
use crate::backup::BackupConfig;
use crate::resp::RespValue;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::sync::{Arc, RwLock};
use tooncache::ToonCache;
use tracing::{error, info, warn};

pub struct CommandHandler {
    cache: Arc<ToonCache>,
    key_map: Arc<RwLock<HashMap<String, u64>>>,
    keymap_path: String,
    auth_config: Arc<AuthConfig>,
    backup_config: Arc<BackupConfig>,
}

impl CommandHandler {
    pub fn new(
        cache: Arc<ToonCache>,
        data_dir: &str,
        auth_config: Arc<AuthConfig>,
        backup_config: Arc<BackupConfig>,
    ) -> Self {
        let keymap_path = format!("{}/keymap.txt", data_dir);
        let mut key_map = Self::load_keymap(&keymap_path);

        // If keymap is empty, rebuild it from the database
        if key_map.is_empty() {
            info!("Keymap is empty, rebuilding from database...");
            key_map = Self::rebuild_keymap(&cache);
            info!("Rebuilt {} keys from database", key_map.len());

            // Save the rebuilt keymap
            if !key_map.is_empty() {
                Self::save_keymap_static(&keymap_path, &key_map);
            }
        } else {
            info!("Loaded {} keys from persistent storage", key_map.len());
        }

        Self {
            cache,
            key_map: Arc::new(RwLock::new(key_map)),
            keymap_path,
            auth_config,
            backup_config,
        }
    }

    /// Load key mapping from disk
    fn load_keymap(path: &str) -> HashMap<String, u64> {
        let mut map = HashMap::new();

        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            for line in reader.lines().map_while(Result::ok) {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() == 2 {
                    if let Ok(row_id) = parts[1].parse::<u64>() {
                        map.insert(parts[0].to_string(), row_id);
                    }
                }
            }
        }

        map
    }

    /// Rebuild keymap by scanning the database
    fn rebuild_keymap(cache: &Arc<ToonCache>) -> HashMap<String, u64> {
        let mut map = HashMap::new();

        // Scan through all database entries
        for result in cache.scan() {
            match result {
                Ok((row_id, data)) => {
                    // Parse the TOON line to extract the "id" field
                    if let Ok(json_str) = String::from_utf8(data) {
                        // Try to parse as JSON to extract the "id" field
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json_str) {
                            if let Some(id) = value.get("id") {
                                if let Some(id_str) = id.as_str() {
                                    info!("Rebuilding key: {} -> {}", id_str, row_id);
                                    map.insert(id_str.to_string(), row_id);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error scanning database: {}", e);
                }
            }
        }

        map
    }

    /// Save keymap to disk (static version for use without self)
    fn save_keymap_static(path: &str, key_map: &HashMap<String, u64>) {
        match OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
        {
            Ok(file) => {
                let mut writer = BufWriter::new(file);
                for (key, row_id) in key_map.iter() {
                    if let Err(e) = writeln!(writer, "{}\t{}", key, row_id) {
                        error!("Failed to write keymap entry: {}", e);
                    }
                }
                if let Err(e) = writer.flush() {
                    error!("Failed to flush keymap: {}", e);
                }
            }
            Err(e) => error!("Failed to open keymap file: {}", e),
        }
    }

    /// Save key mapping to disk
    fn save_keymap(&self) {
        let key_map = self.key_map.read().unwrap();

        match OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.keymap_path)
        {
            Ok(file) => {
                let mut writer = BufWriter::new(file);
                for (key, row_id) in key_map.iter() {
                    if let Err(e) = writeln!(writer, "{}\t{}", key, row_id) {
                        error!("Failed to write keymap entry: {}", e);
                    }
                }
                if let Err(e) = writer.flush() {
                    error!("Failed to flush keymap: {}", e);
                }
            }
            Err(e) => error!("Failed to open keymap file: {}", e),
        }
    }

    pub fn handle(&self, cmd: RespValue, session: &mut SessionState) -> RespValue {
        let arr = match cmd {
            RespValue::Array(Some(arr)) if !arr.is_empty() => arr,
            _ => return RespValue::Error("ERR invalid command format".to_string()),
        };

        let command = match &arr[0] {
            RespValue::BulkString(Some(cmd)) => String::from_utf8_lossy(cmd).to_uppercase(),
            _ => return RespValue::Error("ERR invalid command".to_string()),
        };

        // AUTH command can be used without authentication
        if command.as_str() == "AUTH" {
            return self.handle_auth(&arr[1..], session);
        }

        // Check authentication for all other commands
        if self.auth_config.is_required() && !session.is_authenticated() {
            return RespValue::Error("NOAUTH Authentication required".to_string());
        }

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
            "SAVE" | "BGSAVE" => self.handle_save(&arr[1..]),
            "BGREWRITEAOF" | "BACKUP" => self.handle_backup(&arr[1..]),
            "RESTORE" => self.handle_restore(&arr[1..]),
            "LASTSAVE" => self.handle_lastsave(),
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
        info!(
            "GET: Looking for key '{}', keymap has {} keys",
            key,
            key_map.len()
        );
        let row_id = match key_map.get(&key) {
            Some(id) => {
                info!("GET: Found key '{}' -> row_id {}", key, id);
                *id
            }
            None => {
                info!("GET: Key '{}' not found in keymap", key);
                return RespValue::BulkString(None);
            } // Key not found
        };

        match self.cache.get(row_id) {
            Ok(data) => {
                info!("GET: Successfully retrieved data for row_id {}", row_id);
                RespValue::BulkString(Some(data))
            }
            Err(e) => {
                error!("GET: Failed to retrieve data for row_id {}: {}", row_id, e);
                RespValue::BulkString(None)
            }
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
                drop(key_map); // Release lock before save
                self.save_keymap(); // Persist to disk
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

        drop(key_map); // Release lock
        if deleted > 0 {
            self.save_keymap(); // Persist to disk
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
        drop(key_map); // Release lock
        self.cache.clear_cache();
        self.save_keymap(); // Persist empty keymap
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

    fn handle_auth(&self, args: &[RespValue], session: &mut SessionState) -> RespValue {
        if args.len() != 1 {
            return RespValue::Error(
                "ERR wrong number of arguments for 'auth' command".to_string(),
            );
        }

        let password = match &args[0] {
            RespValue::BulkString(Some(p)) => match String::from_utf8(p.clone()) {
                Ok(s) => s,
                Err(_) => return RespValue::Error("ERR invalid password".to_string()),
            },
            _ => return RespValue::Error("ERR invalid password type".to_string()),
        };

        if !self.auth_config.is_required() {
            return RespValue::Error("ERR Client sent AUTH, but no password is set".to_string());
        }

        if self.auth_config.verify(&password) {
            session.authenticate();
            RespValue::SimpleString("OK".to_string())
        } else {
            RespValue::Error("WRONGPASS invalid username-password pair".to_string())
        }
    }

    fn handle_save(&self, _args: &[RespValue]) -> RespValue {
        match self.backup_config.create_backup(Some("manual")) {
            Ok(path) => {
                info!("Manual backup created: {:?}", path);
                RespValue::SimpleString("OK".to_string())
            }
            Err(e) => {
                error!("Failed to create backup: {}", e);
                RespValue::Error(format!("ERR Failed to create backup: {}", e))
            }
        }
    }

    fn handle_backup(&self, args: &[RespValue]) -> RespValue {
        let backup_name = if args.is_empty() {
            "backup"
        } else {
            match &args[0] {
                RespValue::BulkString(Some(n)) => match std::str::from_utf8(n) {
                    Ok(s) => s,
                    Err(_) => return RespValue::Error("ERR invalid backup name".to_string()),
                },
                _ => return RespValue::Error("ERR invalid backup name type".to_string()),
            }
        };

        match self.backup_config.create_backup(Some(backup_name)) {
            Ok(path) => {
                info!("Named backup created: {:?}", path);
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                RespValue::BulkString(Some(filename.as_bytes().to_vec()))
            }
            Err(e) => {
                error!("Failed to create backup: {}", e);
                RespValue::Error(format!("ERR Failed to create backup: {}", e))
            }
        }
    }

    fn handle_restore(&self, args: &[RespValue]) -> RespValue {
        if args.is_empty() {
            return RespValue::Error(
                "ERR wrong number of arguments for 'restore' command".to_string(),
            );
        }

        let backup_file = match &args[0] {
            RespValue::BulkString(Some(f)) => match String::from_utf8(f.clone()) {
                Ok(s) => s,
                Err(_) => return RespValue::Error("ERR invalid backup filename".to_string()),
            },
            _ => return RespValue::Error("ERR invalid backup filename type".to_string()),
        };

        let backup_path = if std::path::Path::new(&backup_file).is_absolute() {
            std::path::PathBuf::from(&backup_file)
        } else {
            self.backup_config.backup_dir.join(&backup_file)
        };

        if !backup_path.exists() {
            return RespValue::Error(format!("ERR Backup file not found: {:?}", backup_path));
        }

        warn!("Restoring from backup: {:?}", backup_path);

        match self.backup_config.restore_backup(&backup_path) {
            Ok(_) => {
                info!("Database restored successfully from {:?}", backup_path);
                RespValue::SimpleString("OK - Server restart recommended".to_string())
            }
            Err(e) => {
                error!("Failed to restore backup: {}", e);
                RespValue::Error(format!("ERR Failed to restore backup: {}", e))
            }
        }
    }

    fn handle_lastsave(&self) -> RespValue {
        match self.backup_config.list_backups() {
            Ok(backups) => {
                let mut result = Vec::new();
                result.push(RespValue::BulkString(Some(
                    "Recent Backups:".as_bytes().to_vec(),
                )));

                for (i, backup) in backups.iter().take(10).enumerate() {
                    let info = format!("{}. {} ({} bytes)", i + 1, backup.filename, backup.size);
                    result.push(RespValue::BulkString(Some(info.as_bytes().to_vec())));
                }

                if result.len() == 1 {
                    result.push(RespValue::BulkString(Some(
                        "No backups found".as_bytes().to_vec(),
                    )));
                }

                RespValue::Array(Some(result))
            }
            Err(e) => {
                error!("Failed to list backups: {}", e);
                RespValue::Error(format!("ERR Failed to list backups: {}", e))
            }
        }
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
        let auth = Arc::new(AuthConfig::disabled());
        let backup = Arc::new(BackupConfig::new(dir.path(), None::<&str>));
        let handler =
            CommandHandler::new(cache, dir.path().to_str().unwrap(), auth.clone(), backup);
        let mut session = SessionState::new(false);

        let cmd = RespValue::Array(Some(vec![RespValue::BulkString(Some(b"PING".to_vec()))]));

        let resp = handler.handle(cmd, &mut session);
        assert_eq!(resp, RespValue::SimpleString("PONG".to_string()));
    }

    #[test]
    fn test_echo() {
        let dir = TempDir::new().unwrap();
        let cache = Arc::new(ToonCache::new(dir.path(), 100).unwrap());
        let auth = Arc::new(AuthConfig::disabled());
        let backup = Arc::new(BackupConfig::new(dir.path(), None::<&str>));
        let handler =
            CommandHandler::new(cache, dir.path().to_str().unwrap(), auth.clone(), backup);
        let mut session = SessionState::new(false);

        let cmd = RespValue::Array(Some(vec![
            RespValue::BulkString(Some(b"ECHO".to_vec())),
            RespValue::BulkString(Some(b"hello".to_vec())),
        ]));

        let resp = handler.handle(cmd, &mut session);
        assert_eq!(resp, RespValue::BulkString(Some(b"hello".to_vec())));
    }

    #[test]
    fn test_set_and_get() {
        let dir = TempDir::new().unwrap();
        let cache = Arc::new(ToonCache::new(dir.path(), 100).unwrap());
        let auth = Arc::new(AuthConfig::disabled());
        let backup = Arc::new(BackupConfig::new(dir.path(), None::<&str>));
        let handler =
            CommandHandler::new(cache, dir.path().to_str().unwrap(), auth.clone(), backup);
        let mut session = SessionState::new(false);

        // SET key value
        let set_cmd = RespValue::Array(Some(vec![
            RespValue::BulkString(Some(b"SET".to_vec())),
            RespValue::BulkString(Some(b"mykey".to_vec())),
            RespValue::BulkString(Some(b"myvalue".to_vec())),
        ]));

        let resp = handler.handle(set_cmd, &mut session);
        assert_eq!(resp, RespValue::SimpleString("OK".to_string()));
    }
}
