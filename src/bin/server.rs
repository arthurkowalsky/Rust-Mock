// Legacy RustMock binary (backward compatible)
// This maintains the old CLI interface for existing users

use clap::Parser;
use RustMock::{init_logger, start_server, ServerConfig};

#[derive(Parser)]
#[command(name = "RustMock", version, about = "Mock API server")]
struct Args {
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    #[arg(long, default_value = "8090")]
    port: u16,

    #[arg(long)]
    default_proxy_url: Option<String>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logger();

    let args = Args::parse();

    let config = ServerConfig {
        host: args.host,
        port: args.port,
        default_proxy_url: args.default_proxy_url,
    };

    start_server(config).await
}
