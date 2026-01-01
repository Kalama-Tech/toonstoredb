//! ToonStore Daemon - Redis-compatible RESP server

mod handler;
mod resp;

use anyhow::Result;
use bytes::BytesMut;
use clap::Parser;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tooncache::ToonCache;
use tracing::{error, info, warn};

use crate::handler::CommandHandler;
use crate::resp::RespValue;

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

    // Initialize cache
    let cache = Arc::new(ToonCache::new(&args.data, args.capacity)?);
    info!("Database opened successfully");

    // Bind TCP listener
    let listener = TcpListener::bind(&args.bind).await?;
    info!("Server listening on {}", args.bind);

    // Print connection info
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║          ToonStore Server Ready!                            ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!("\n📡 NETWORK MODE (Redis-compatible RESP Protocol):");
    println!("   Connection String: redis://{}", args.bind);
    println!(
        "   redis-cli Command: redis-cli -h {} -p {}",
        args.bind.split(':').next().unwrap_or("127.0.0.1"),
        args.bind.split(':').nth(1).unwrap_or("6379")
    );
    println!("   Protocol:          RESP (works with any Redis client)");
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
    println!("     Python:  redis.from_url('redis://{}')", args.bind);
    println!(
        "     Node.js: redis.createClient({{ url: 'redis://{}' }})",
        args.bind
    );
    println!(
        "     CLI:     redis-cli -h {} -p {}",
        args.bind.split(':').next().unwrap_or("127.0.0.1"),
        args.bind.split(':').nth(1).unwrap_or("6379")
    );
    println!("\n   Embedded Mode (Rust):");
    println!("     Database: ToonStore::open(\"{}\")?", args.data);
    println!(
        "     Cached:   ToonCache::new(\"{}\", {})?",
        args.data, args.capacity
    );
    println!("\n🛑 Press Ctrl+C to stop\n");

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                info!("New connection from {}", addr);
                let cache = Arc::clone(&cache);

                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream, cache).await {
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

async fn handle_client(mut stream: TcpStream, cache: Arc<ToonCache>) -> Result<()> {
    let handler = CommandHandler::new(cache);
    let mut buffer = BytesMut::with_capacity(4096);

    loop {
        // Read data from client
        let n = stream.read_buf(&mut buffer).await?;

        if n == 0 {
            // Connection closed
            return Ok(());
        }

        // Parse and handle commands
        loop {
            match RespValue::parse(&mut buffer) {
                Ok(Some(cmd)) => {
                    // Handle command
                    let response = handler.handle(cmd);

                    // Send response
                    stream.write_all(&response.serialize()).await?;
                }
                Ok(None) => {
                    // Need more data
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
