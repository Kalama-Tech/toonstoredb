//! User management for ToonStore
//!
//! Provides multi-user authentication with roles and permissions

use anyhow::{Context, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::RwLock;
use tracing::{info, warn};

/// User role with specific permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    /// Admin - full access to all commands including FLUSHDB, user management
    Admin,
    /// ReadWrite - can read and write data, but not manage users or flush database
    ReadWrite,
    /// ReadOnly - can only read data
    ReadOnly,
}

impl UserRole {
    /// Check if role can execute a command
    pub fn can_execute(&self, command: &str) -> bool {
        let cmd = command.to_uppercase();

        match self {
            UserRole::Admin => true, // Admin can do everything
            UserRole::ReadWrite => {
                // ReadWrite cannot flush DB or manage users
                !matches!(
                    cmd.as_str(),
                    "FLUSHDB" | "FLUSHALL" | "USER" | "ACL" | "CONFIG"
                )
            }
            UserRole::ReadOnly => {
                // ReadOnly can only read
                matches!(
                    cmd.as_str(),
                    "GET" | "MGET" | "EXISTS" | "KEYS" | "DBSIZE" | "INFO" | "PING" | "ECHO"
                )
            }
        }
    }
}

/// User account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Username
    pub username: String,
    /// BCrypt password hash
    pub password_hash: String,
    /// User role
    pub role: UserRole,
    /// Whether user is active
    pub active: bool,
    /// Optional database restriction (None = all databases)
    pub database: Option<String>,
}

impl User {
    /// Create a new user
    pub fn new(username: String, password: &str, role: UserRole) -> Result<Self> {
        let password_hash = hash(password, DEFAULT_COST).context("Failed to hash password")?;

        Ok(Self {
            username,
            password_hash,
            role,
            active: true,
            database: None,
        })
    }

    /// Verify password
    pub fn verify_password(&self, password: &str) -> bool {
        verify(password, &self.password_hash).unwrap_or(false)
    }

    /// Check if user can execute a command
    #[allow(dead_code)]
    pub fn can_execute(&self, command: &str) -> bool {
        self.active && self.role.can_execute(command)
    }
}

/// User database manager
pub struct UserManager {
    users: RwLock<HashMap<String, User>>,
    users_file: String,
}

impl UserManager {
    /// Create a new user manager
    pub fn new(data_dir: &str) -> Result<Self> {
        let users_file = format!("{}/users.json", data_dir);
        let users = Self::load_users(&users_file)?;

        let manager = Self {
            users: RwLock::new(users),
            users_file,
        };

        // Create default admin user if no users exist
        if manager.users.read().unwrap().is_empty() {
            info!("No users found, creating default admin user");
            manager.create_user("admin", "admin", UserRole::Admin)?;
            warn!("⚠️  Default admin user created with password 'admin' - CHANGE THIS!");
        }

        Ok(manager)
    }

    /// Load users from file
    fn load_users(path: &str) -> Result<HashMap<String, User>> {
        if !Path::new(path).exists() {
            info!(
                "No users file found at {}, starting with empty user database",
                path
            );
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(path).context("Failed to read users file")?;
        let users: Vec<User> =
            serde_json::from_str(&content).context("Failed to parse users file")?;

        let mut map = HashMap::new();
        for user in users {
            map.insert(user.username.clone(), user);
        }

        info!("Loaded {} users from {}", map.len(), path);
        Ok(map)
    }

    /// Save users to file
    fn save_users(&self) -> Result<()> {
        let users = self.users.read().unwrap();
        let users_vec: Vec<&User> = users.values().collect();

        let content =
            serde_json::to_string_pretty(&users_vec).context("Failed to serialize users")?;

        fs::write(&self.users_file, content).context("Failed to write users file")?;

        Ok(())
    }

    /// Create a new user
    pub fn create_user(&self, username: &str, password: &str, role: UserRole) -> Result<()> {
        let mut users = self.users.write().unwrap();

        if users.contains_key(username) {
            return Err(anyhow::anyhow!("User '{}' already exists", username));
        }

        let user = User::new(username.to_string(), password, role)?;
        users.insert(username.to_string(), user);
        drop(users);

        self.save_users()?;
        info!("Created user: {}", username);

        Ok(())
    }

    /// Authenticate a user
    pub fn authenticate(&self, username: &str, password: &str) -> Option<User> {
        let users = self.users.read().unwrap();

        if let Some(user) = users.get(username) {
            if user.active && user.verify_password(password) {
                return Some(user.clone());
            }
        }

        None
    }

    /// Delete a user
    pub fn delete_user(&self, username: &str) -> Result<()> {
        let mut users = self.users.write().unwrap();

        if username == "admin" {
            return Err(anyhow::anyhow!("Cannot delete admin user"));
        }

        if users.remove(username).is_none() {
            return Err(anyhow::anyhow!("User '{}' not found", username));
        }

        drop(users);
        self.save_users()?;
        info!("Deleted user: {}", username);

        Ok(())
    }

    /// List all users
    pub fn list_users(&self) -> Vec<String> {
        let users = self.users.read().unwrap();
        users.keys().cloned().collect()
    }

    /// Change user password
    pub fn change_password(&self, username: &str, new_password: &str) -> Result<()> {
        let mut users = self.users.write().unwrap();

        let user = users
            .get_mut(username)
            .ok_or_else(|| anyhow::anyhow!("User '{}' not found", username))?;

        user.password_hash = hash(new_password, DEFAULT_COST).context("Failed to hash password")?;

        drop(users);
        self.save_users()?;
        info!("Changed password for user: {}", username);

        Ok(())
    }

    /// Update user role
    #[allow(dead_code)] // Available for future use
    pub fn update_role(&self, username: &str, role: UserRole) -> Result<()> {
        let mut users = self.users.write().unwrap();

        let user = users
            .get_mut(username)
            .ok_or_else(|| anyhow::anyhow!("User '{}' not found", username))?;

        user.role = role;

        drop(users);
        self.save_users()?;
        info!("Updated role for user: {}", username);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_user_creation() {
        let user = User::new("testuser".to_string(), "password123", UserRole::ReadWrite).unwrap();
        assert_eq!(user.username, "testuser");
        assert!(user.verify_password("password123"));
        assert!(!user.verify_password("wrongpassword"));
    }

    #[test]
    fn test_user_permissions() {
        let admin = User::new("admin".to_string(), "pass", UserRole::Admin).unwrap();
        let readwrite = User::new("rw".to_string(), "pass", UserRole::ReadWrite).unwrap();
        let readonly = User::new("ro".to_string(), "pass", UserRole::ReadOnly).unwrap();

        // Admin can do everything
        assert!(admin.can_execute("GET"));
        assert!(admin.can_execute("SET"));
        assert!(admin.can_execute("FLUSHDB"));

        // ReadWrite can read/write but not flush
        assert!(readwrite.can_execute("GET"));
        assert!(readwrite.can_execute("SET"));
        assert!(!readwrite.can_execute("FLUSHDB"));

        // ReadOnly can only read
        assert!(readonly.can_execute("GET"));
        assert!(!readonly.can_execute("SET"));
        assert!(!readonly.can_execute("FLUSHDB"));
    }

    #[test]
    fn test_user_manager() {
        let dir = TempDir::new().unwrap();
        let manager = UserManager::new(dir.path().to_str().unwrap()).unwrap();

        // Default admin user should exist
        assert!(manager.authenticate("admin", "admin").is_some());

        // Create new user
        manager
            .create_user("testuser", "testpass", UserRole::ReadWrite)
            .unwrap();

        // Authenticate
        let user = manager.authenticate("testuser", "testpass").unwrap();
        assert_eq!(user.username, "testuser");
        assert_eq!(user.role, UserRole::ReadWrite);

        // Wrong password
        assert!(manager.authenticate("testuser", "wrongpass").is_none());

        // Change password
        manager.change_password("testuser", "newpass").unwrap();
        assert!(manager.authenticate("testuser", "newpass").is_some());
        assert!(manager.authenticate("testuser", "testpass").is_none());
    }
}
