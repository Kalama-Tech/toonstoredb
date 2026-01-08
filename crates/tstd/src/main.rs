//! ToonStore Daemon - Redis-compatible RESP server with Auth, TLS, and Backup support

mod auth;
mod backup;
mod handler;
mod resp;
mod tls;
mod users;

use anyhow::Result;
use auth::{AuthConfig, SessionState};
use backup::BackupConfig;
use bytes::BytesMut;
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tls::{TlsConfig, TlsMode};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tooncache::ToonCache;
use tracing::{error, info, warn};

use crate::handler::CommandHandler;
use crate::resp::RespValue;

/// Maximum concurrent connections - prevents DoS via connection flooding
const MAX_CONNECTIONS: usize = 10000;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Bind address
    #[arg(short, long, default_value = "127.0.0.1:6379")]
    bind: String,

    /// Data directory
    #[arg(short, long, default_value = "./data")]
    data: String,

    /// Cache capacity (number of items)
    #[arg(short, long, default_value_t = 10000)]
    capacity: usize,

    /// Health check mode (for Docker)
    #[arg(long)]
    health: bool,

    /// Password for authentication (or path to password file with @)
    #[arg(long)]
    password: Option<String>,

    /// Enable multi-user authentication
    #[arg(long)]
    multi_user: bool,

    /// TLS/SSL mode: disable, prefer, require
    #[arg(long, default_value = "disable")]
    tls_mode: String,

    /// Path to TLS certificate file (PEM format)
    #[arg(long)]
    tls_cert: Option<PathBuf>,

    /// Path to TLS private key file (PEM format)
    #[arg(long)]
    tls_key: Option<PathBuf>,

    /// Backup directory
    #[arg(long)]
    backup_dir: Option<PathBuf>,

    /// Auto-backup interval in minutes (0 to disable)
    #[arg(long, default_value_t = 0)]
    auto_backup: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let args = Args::parse();

    // Health check
    if args.health {
        // Try to connect to the server
        match TcpStream::connect(&args.bind).await {
            Ok(_) => {
                println!("OK");
                std::process::exit(0);
            }
            Err(_) => {
                eprintln!("FAILED");
                std::process::exit(1);
            }
        }
    }

    info!("Starting ToonStore Daemon v{}", env!("CARGO_PKG_VERSION"));
    info!("Binding to {}", args.bind);
    info!("Data directory: {}", args.data);
    info!("Cache capacity: {}", args.capacity);

    // Create data directory if it doesn't exist
    std::fs::create_dir_all(&args.data)?;

    // Initialize authentication
    let (auth_config, user_manager) = if args.multi_user {
        // Multi-user mode
        info!("🔐 Multi-user authentication enabled");
        let user_manager = match crate::users::UserManager::new(&args.data) {
            Ok(mgr) => Arc::new(mgr),
            Err(e) => {
                error!("Failed to initialize user manager: {}", e);
                return Err(e);
            }
        };
        (Arc::new(AuthConfig::disabled()), Some(user_manager))
    } else {
        // Single-password mode
        let auth_config = if let Some(password) = &args.password {
            if password.starts_with('@') {
                let path = password.trim_start_matches('@');
                Arc::new(AuthConfig::from_password_file(path)?)
            } else {
                Arc::new(AuthConfig::from_password(password)?)
            }
        } else {
            Arc::new(AuthConfig::disabled())
        };

        if auth_config.is_required() {
            info!("✅ Single-password authentication: ENABLED");
        } else {
            warn!("⚠️  Authentication: DISABLED (use --password or --multi-user to enable)");
        }

        (auth_config, None)
    };

    // Initialize TLS
    let tls_mode = TlsMode::from_str(&args.tls_mode)?;
    let _tls_config = if tls_mode.is_enabled() {
        let cert = args
            .tls_cert
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("--tls-cert required when TLS is enabled"))?;
        let key = args
            .tls_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("--tls-key required when TLS is enabled"))?;
        Arc::new(TlsConfig::from_files(cert, key, tls_mode)?)
    } else {
        Arc::new(TlsConfig::disabled())
    };

    // Initialize backup configuration
    let backup_config = Arc::new(BackupConfig::new(
        args.data.as_str(),
        args.backup_dir.as_deref(),
    ));
    info!("📦 Backup directory: {:?}", backup_config.backup_dir);

    // Initialize cache
    let cache = Arc::new(ToonCache::new(&args.data, args.capacity)?);
    info!("Database opened successfully");

    // Initialize shared command handler (loads keymap once)
    let handler = Arc::new(CommandHandler::new(
        cache,
        &args.data,
        auth_config.clone(),
        backup_config.clone(),
        user_manager.clone(),
    ));

    // Start auto-backup task if enabled
    if args.auto_backup > 0 {
        let backup_config_clone = backup_config.clone();
        let interval_minutes = args.auto_backup;
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_secs(interval_minutes * 60));
            loop {
                interval.tick().await;
                info!("Running automatic backup...");
                match backup_config_clone.create_backup(Some("auto")) {
                    Ok(path) => {
                        info!("Auto-backup created: {:?}", path);
                        if let Err(e) = backup_config_clone.cleanup_old_backups(10) {
                            error!("Failed to cleanup old backups: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Auto-backup failed: {}", e);
                    }
                }
            }
        });
        info!("✅ Auto-backup: Every {} minutes", interval_minutes);
    }

    // Bind TCP listener
    let listener = TcpListener::bind(&args.bind).await?;
    info!("Server listening on {}", args.bind);

    // Connection limiter to prevent DoS attacks
    let connection_limiter = Arc::new(Semaphore::new(MAX_CONNECTIONS));
    info!(
        "Connection limit: {} concurrent connections",
        MAX_CONNECTIONS
    );

    // Print connection info
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║          ToonStore Server Ready!                            ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    println!("\n📡 NETWORK MODE (Redis-compatible RESP Protocol):");

    let auth_part = if auth_config.is_required() {
        ":<password>@"
    } else {
        ""
    };

    println!(
        "   Connection String: toonstore://{}{}",
        auth_part, args.bind
    );
    println!("   (Also compatible:  redis://{}{})", auth_part, args.bind);
    println!(
        "   redis-cli Command: redis-cli -h {} -p {}{}",
        args.bind.split(':').next().unwrap_or("127.0.0.1"),
        args.bind.split(':').nth(1).unwrap_or("6379"),
        if auth_config.is_required() {
            " -a <password>"
        } else {
            ""
        }
    );
    println!("   Protocol:          RESP (works with any Redis client)");

    println!("\n🔒 SECURITY:");
    println!(
        "   Authentication:    {}",
        if auth_config.is_required() {
            "✅ ENABLED"
        } else {
            "⚠️  DISABLED"
        }
    );
    println!("   TLS/SSL:           ⚠️  DISABLED (use --tls-mode to enable)");

    println!("\n💾 EMBEDDED MODE (Direct Database Access):");
    println!("   ┌─────────────────────────────────────────────────────────┐");
    println!("   │ Layer           │ Connection String                    │");
    println!("   ├─────────────────────────────────────────────────────────┤");
    println!(
        "   │ toonstoredb     │ file://{}                  │",
        args.data
    );
    println!(
        "   │ (storage)       │ ToonStore::open(\"{}\")        │",
        args.data
    );
    println!("   ├─────────────────────────────────────────────────────────┤");
    println!(
        "   │ tooncache       │ file://{}?capacity={}   │",
        args.data, args.capacity
    );
    println!(
        "   │ (cache+storage) │ ToonCache::new(\"{}\", {}) │",
        args.data, args.capacity
    );
    println!("   └─────────────────────────────────────────────────────────┘");
    println!("   Performance:       66x faster than network mode");
    println!("\n📊 CONFIGURATION:");
    println!("   Data Directory:  {}", args.data);
    println!("   Cache Capacity:  {} items", args.capacity);
    println!("   Cache Hit Rate:  Will be shown in INFO command");

    println!("\n💡 USAGE EXAMPLES:");
    println!("   Network Mode:");

    let auth_example = if auth_config.is_required() {
        "password@"
    } else {
        ""
    };

    println!(
        "     Python:  redis.from_url('toonstore://{}{}'))",
        auth_example, args.bind
    );
    println!(
        "     Node.js: redis.createClient({{ url: 'toonstore://{}{}' }})",
        auth_example, args.bind
    );
    println!(
        "     CLI:     redis-cli -h {} -p {}{}",
        args.bind.split(':').next().unwrap_or("127.0.0.1"),
        args.bind.split(':').nth(1).unwrap_or("6379"),
        if auth_config.is_required() {
            " -a <password>"
        } else {
            ""
        }
    );

    println!("\n   Embedded Mode (Rust):");
    println!("     Database: ToonStore::open(\"{}\")?", args.data);
    println!(
        "     Cached:   ToonCache::new(\"{}\", {})?",
        args.data, args.capacity
    );

    if auth_config.is_required() {
        println!("\n   Authentication:");
        println!("     redis-cli -a <password> PING");
        println!("     Or: AUTH <password> after connecting");
    }

    println!("\n📦 BACKUP COMMANDS:");
    println!("   SAVE / BGSAVE      - Create immediate backup");
    println!("   BACKUP [name]      - Create named backup");
    println!("   RESTORE <file>     - Restore from backup");
    println!("   LASTSAVE           - List recent backups");

    println!("\n🛑 Press Ctrl+C to stop\n");

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                info!("New connection from {}", addr);

                // Acquire connection permit (blocks if at limit)
                let permit = match connection_limiter.clone().try_acquire_owned() {
                    Ok(permit) => permit,
                    Err(_) => {
                        warn!(
                            "Connection limit reached, rejecting connection from {}",
                            addr
                        );
                        continue;
                    }
                };

                let handler = Arc::clone(&handler);
                let auth_config = Arc::clone(&auth_config);

                tokio::spawn(async move {
                    // Permit is automatically released when dropped
                    let _permit = permit;

                    if let Err(e) = handle_client(stream, handler, auth_config).await {
                        error!("Error handling client {}: {}", addr, e);
                    }
                    info!("Connection closed: {}", addr);
                });
            }
            Err(e) => {
                error!("Error accepting connection: {}", e);
            }
        }
    }
}

async fn handle_client(
    mut stream: TcpStream,
    handler: Arc<CommandHandler>,
    auth_config: Arc<AuthConfig>,
) -> Result<()> {
    let mut buffer = BytesMut::with_capacity(4096);
    let mut session = SessionState::new(auth_config.is_required());

    loop {
        // Read data from client
        let n = stream.read_buf(&mut buffer).await?;
        info!(
            "Read {} bytes from client, buffer total: {}",
            n,
            buffer.len()
        );

        if n == 0 {
            // Connection closed
            info!("Client closed connection");
            return Ok(());
        }

        // Parse and handle commands
        loop {
            match RespValue::parse(&mut buffer) {
                Ok(Some(cmd)) => {
                    info!("Parsed command: {:?}", cmd);
                    // Handle command with session state
                    let response = handler.handle(cmd, &mut session);
                    info!("Response: {:?}", response);

                    // Send response
                    stream.write_all(&response.serialize()).await?;

                    // Check for QUIT command
                    if matches!(response, RespValue::SimpleString(ref s) if s == "OK") {
                        // Check if this was a QUIT command by looking at the original command
                        // For now, we'll just continue - proper QUIT handling would close connection
                    }
                }
                Ok(None) => {
                    // Need more data
                    info!("Need more data, buffer size: {}", buffer.len());
                    break;
                }
                Err(e) => {
                    warn!("Parse error: {}", e);
                    let error_resp = RespValue::Error(format!("ERR {}", e));
                    stream.write_all(&error_resp.serialize()).await?;
                    buffer.clear();
                    break;
                }
            }
        }
    }
}
