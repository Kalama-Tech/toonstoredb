//! Authentication module for ToonStore
//!
//! Provides password-based authentication similar to Redis AUTH command.
//! Passwords are stored as bcrypt hashes for security.

use anyhow::{Context, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use std::fs;
use std::path::Path;
use tracing::{info, warn};

/// Authentication configuration
#[derive(Clone)]
pub struct AuthConfig {
    /// BCrypt password hash (if authentication is enabled)
    password_hash: Option<String>,
    /// Whether authentication is required
    pub required: bool,
}

impl AuthConfig {
    /// Create auth config with no authentication
    pub fn disabled() -> Self {
        Self {
            password_hash: None,
            required: false,
        }
    }

    /// Create auth config from password file
    pub fn from_password_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            info!(
                "No password file found at {:?}, authentication disabled",
                path
            );
            return Ok(Self::disabled());
        }

        let password_hash = fs::read_to_string(path)
            .context("Failed to read password file")?
            .trim()
            .to_string();

        if password_hash.is_empty() {
            warn!("Password file is empty, authentication disabled");
            return Ok(Self::disabled());
        }

        info!("Authentication enabled from password file");
        Ok(Self {
            password_hash: Some(password_hash),
            required: true,
        })
    }

    /// Create auth config from plain password
    pub fn from_password(password: &str) -> Result<Self> {
        if password.is_empty() {
            return Ok(Self::disabled());
        }

        let password_hash = hash(password, DEFAULT_COST).context("Failed to hash password")?;

        info!("Authentication enabled with provided password");
        Ok(Self {
            password_hash: Some(password_hash),
            required: true,
        })
    }

    /// Verify a password against the stored hash
    pub fn verify(&self, password: &str) -> bool {
        match &self.password_hash {
            Some(hash) => verify(password, hash).unwrap_or(false),
            None => true, // No auth required
        }
    }

    /// Check if authentication is required
    pub fn is_required(&self) -> bool {
        self.required
    }
}

/// Session state for tracking client authentication
#[derive(Clone)]
pub struct SessionState {
    pub authenticated: bool,
}

impl SessionState {
    pub fn new(auth_required: bool) -> Self {
        Self {
            // If auth not required, start authenticated
            authenticated: !auth_required,
        }
    }

    pub fn authenticate(&mut self) {
        self.authenticated = true;
    }

    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }
}

/// Helper to create a password hash for the password file
#[allow(dead_code)]
pub fn create_password_hash(password: &str) -> Result<String> {
    let hash = hash(password, DEFAULT_COST).context("Failed to hash password")?;
    Ok(hash)
}

/// Helper to save password hash to file
#[allow(dead_code)]
pub fn save_password_hash<P: AsRef<Path>>(path: P, password: &str) -> Result<()> {
    let hash = create_password_hash(password)?;
    fs::write(path, hash).context("Failed to write password file")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_disabled() {
        let auth = AuthConfig::disabled();
        assert!(!auth.is_required());
        assert!(auth.verify("any_password"));
    }

    #[test]
    fn test_auth_with_password() {
        let auth = AuthConfig::from_password("mysecret").unwrap();
        assert!(auth.is_required());
        assert!(auth.verify("mysecret"));
        assert!(!auth.verify("wrongpassword"));
    }

    #[test]
    fn test_session_state() {
        let mut session = SessionState::new(true);
        assert!(!session.is_authenticated());

        session.authenticate();
        assert!(session.is_authenticated());
    }
}
