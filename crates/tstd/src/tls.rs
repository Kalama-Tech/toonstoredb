//! TLS/SSL support for ToonStore

use anyhow::{Context, Result};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::ServerConfig;
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use tracing::info;

/// TLS configuration mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsMode {
    /// TLS disabled (plain TCP)
    Disabled,
    /// TLS optional (support both plain and TLS connections)
    Prefer,
    /// TLS required (reject plain TCP connections)
    Require,
}

impl TlsMode {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "disable" | "disabled" => Ok(TlsMode::Disabled),
            "prefer" | "optional" => Ok(TlsMode::Prefer),
            "require" | "required" => Ok(TlsMode::Require),
            _ => anyhow::bail!(
                "Invalid TLS mode: {}. Use 'disable', 'prefer', or 'require'",
                s
            ),
        }
    }

    pub fn is_enabled(&self) -> bool {
        !matches!(self, TlsMode::Disabled)
    }

    pub fn is_required(&self) -> bool {
        matches!(self, TlsMode::Require)
    }
}

/// TLS certificate and key configuration
#[allow(dead_code)]
pub struct TlsConfig {
    pub mode: TlsMode,
    pub server_config: Option<Arc<ServerConfig>>,
}

impl TlsConfig {
    /// Create disabled TLS config
    pub fn disabled() -> Self {
        Self {
            mode: TlsMode::Disabled,
            server_config: None,
        }
    }

    /// Load TLS configuration from certificate and key files
    pub fn from_files<P: AsRef<Path>>(cert_path: P, key_path: P, mode: TlsMode) -> Result<Self> {
        if mode == TlsMode::Disabled {
            return Ok(Self::disabled());
        }

        let cert_path = cert_path.as_ref();
        let key_path = key_path.as_ref();

        info!("Loading TLS certificate from: {:?}", cert_path);
        info!("Loading TLS private key from: {:?}", key_path);

        // Load certificate chain
        let cert_file = File::open(cert_path)
            .context(format!("Failed to open certificate file: {:?}", cert_path))?;
        let mut cert_reader = BufReader::new(cert_file);
        let cert_chain: Vec<CertificateDer> = certs(&mut cert_reader)
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to parse certificate file")?;

        if cert_chain.is_empty() {
            anyhow::bail!("No certificates found in certificate file");
        }

        // Load private key
        let key_file = File::open(key_path)
            .context(format!("Failed to open private key file: {:?}", key_path))?;
        let mut key_reader = BufReader::new(key_file);
        let keys: Vec<_> = pkcs8_private_keys(&mut key_reader)
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to parse private key file")?;

        if keys.is_empty() {
            anyhow::bail!("No private keys found in key file");
        }

        let private_key = PrivateKeyDer::Pkcs8(keys.into_iter().next().unwrap());

        // Create server configuration
        let server_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, private_key)
            .context("Failed to create TLS server configuration")?;

        info!("TLS configuration loaded successfully (mode: {:?})", mode);

        Ok(Self {
            mode,
            server_config: Some(Arc::new(server_config)),
        })
    }

    /// Check if TLS is enabled
    #[allow(dead_code)]
    pub fn is_enabled(&self) -> bool {
        self.mode.is_enabled()
    }

    /// Check if TLS is required
    #[allow(dead_code)]
    pub fn is_required(&self) -> bool {
        self.mode.is_required()
    }

    /// Get the server config (if TLS is enabled)
    #[allow(dead_code)]
    pub fn server_config(&self) -> Option<Arc<ServerConfig>> {
        self.server_config.clone()
    }
}

/// Helper to generate self-signed certificate for testing
///
/// Note: This requires the `rcgen` crate. For production, use proper certificates
/// from a certificate authority (e.g., Let's Encrypt).
///
/// To enable: Add `rcgen = "0.11"` to Cargo.toml and uncomment this code
#[allow(dead_code)]
fn _example_generate_self_signed_cert() -> Result<(Vec<u8>, Vec<u8>)> {
    // Requires rcgen dependency - uncomment to use:
    // use rcgen::{generate_simple_self_signed, CertifiedKey};
    // let subject_alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string()];
    // let CertifiedKey { cert, key_pair } = generate_simple_self_signed(subject_alt_names)
    //     .context("Failed to generate self-signed certificate")?;
    // Ok((cert.pem().into_bytes(), key_pair.serialize_pem().into_bytes()))
    unimplemented!("Add rcgen dependency to Cargo.toml to use this function")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_mode_from_str() {
        assert_eq!(TlsMode::from_str("disable").unwrap(), TlsMode::Disabled);
        assert_eq!(TlsMode::from_str("prefer").unwrap(), TlsMode::Prefer);
        assert_eq!(TlsMode::from_str("require").unwrap(), TlsMode::Require);
        assert!(TlsMode::from_str("invalid").is_err());
    }

    #[test]
    fn test_tls_config_disabled() {
        let config = TlsConfig::disabled();
        assert!(!config.is_enabled());
        assert!(!config.is_required());
        assert!(config.server_config().is_none());
    }
}
