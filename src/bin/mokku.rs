// Mokku - Modern CLI for RustMock server
// Provides an intuitive interface for mocking APIs with OpenAPI support

use clap::{Parser, Subcommand};
use colored::Colorize;
use inquire::{Select, Text};
use std::path::PathBuf;
use RustMock::{init_logger, load_openapi_from_file, start_server, ServerConfig, EndpointConfig};

#[derive(Parser)]
#[command(
    name = "mokku",
    version,
    about = "🚀 Mock API server with OpenAPI support",
    long_about = "A powerful mock server for API development with OpenAPI/Swagger import, dynamic endpoints, and proxy mode"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Server port (default: 8090)
    #[arg(long, short = 'p', global = true)]
    port: Option<u16>,

    /// Server host (default: 0.0.0.0)
    #[arg(long, global = true)]
    host: Option<String>,

    /// Default proxy URL for unmatched requests
    #[arg(long, global = true)]
    proxy: Option<String>,

    /// Auto-open dashboard in browser
    #[arg(long, short = 'o', global = true)]
    open: bool,
}

#[derive(Clone, Subcommand)]
enum Commands {
    /// Start the mock server (default)
    Server {
        /// Server port
        #[arg(long, short = 'p')]
        port: Option<u16>,

        /// Auto-open dashboard
        #[arg(long, short = 'o')]
        open: bool,
    },

    /// Import OpenAPI/Swagger spec from file
    Import {
        /// Path to OpenAPI file (YAML or JSON)
        file: PathBuf,

        /// Auto-start server after import
        #[arg(long, short = 's')]
        start: bool,

        /// Auto-open dashboard
        #[arg(long, short = 'o')]
        open: bool,

        /// Server port (if --start is used)
        #[arg(long, short = 'p')]
        port: Option<u16>,
    },

    /// Create a quick mock endpoint
    Mock {
        /// HTTP method (GET, POST, PUT, DELETE, etc.)
        method: Option<String>,

        /// Endpoint path (e.g., /users or /api/users)
        path: Option<String>,

        /// HTTP status code (default: 200)
        status: Option<u16>,

        /// Response body as JSON string
        body: Option<String>,

        /// Server URL (default: http://localhost:8090)
        #[arg(long, default_value = "http://localhost:8090")]
        server: String,
    },

    /// Replay a recorded session (coming soon)
    Replay {
        /// Name of the recording to replay
        name: String,
    },
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    init_logger();

    let cli = Cli::parse();

    // If no subcommand provided, show interactive menu
    if cli.command.is_none() {
        return run_interactive_mode(cli).await;
    }

    match cli.command.clone().unwrap() {
        Commands::Server { port, open } => {
            let config = build_server_config(&cli, port);
            let should_open = open || cli.open;
            start_server_with_browser(config, should_open).await?;
        }

        Commands::Import { file, start, open, port } => {
            handle_import(file, start, open || cli.open, port, &cli).await?;
        }

        Commands::Mock { method, path, status, body, server } => {
            handle_mock(method, path, status, body, server).await?;
        }

        Commands::Replay { name } => {
            handle_replay(name)?;
        }
    }

    Ok(())
}

// Interactive mode when no command is specified
async fn run_interactive_mode(cli: Cli) -> anyhow::Result<()> {
    println!("{}", "🎯 Mokku Interactive Mode".bright_cyan().bold());
    println!();

    let options = vec![
        "Start server",
        "Import OpenAPI spec",
        "Create quick mock",
        "Exit",
    ];

    let choice = Select::new("What would you like to do?", options)
        .prompt()
        .map_err(|e| anyhow::anyhow!("Selection cancelled: {}", e))?;

    match choice {
        "Start server" => {
            let port_input = Text::new("Server port:")
                .with_default("8090")
                .prompt()?;
            let port: u16 = port_input.parse().unwrap_or(8090);

            let open_browser = inquire::Confirm::new("Open browser?")
                .with_default(true)
                .prompt()
                .unwrap_or(true);

            let config = ServerConfig {
                host: cli.host.unwrap_or_else(|| "0.0.0.0".to_string()),
                port,
                default_proxy_url: cli.proxy,
            };

            start_server_with_browser(config, open_browser).await?;
        }

        "Import OpenAPI spec" => {
            let file_path = Text::new("Path to OpenAPI file:")
                .with_placeholder("./openapi.yaml")
                .prompt()?;

            let start = inquire::Confirm::new("Start server after import?")
                .with_default(true)
                .prompt()
                .unwrap_or(true);

            let open = if start {
                inquire::Confirm::new("Open browser?")
                    .with_default(true)
                    .prompt()
                    .unwrap_or(true)
            } else {
                false
            };

            handle_import(
                PathBuf::from(file_path),
                start,
                open,
                cli.port,
                &cli,
            )
            .await?;
        }

        "Create quick mock" => {
            let method = Text::new("HTTP method:")
                .with_default("GET")
                .prompt()?
                .to_uppercase();

            let path = Text::new("Endpoint path:")
                .with_placeholder("/api/users")
                .prompt()?;

            let status_input = Text::new("Status code:")
                .with_default("200")
                .prompt()?;
            let status: u16 = status_input.parse().unwrap_or(200);

            let body = Text::new("Response body (JSON):")
                .with_default(r#"{"message": "OK"}"#)
                .prompt()?;

            let server = Text::new("Server URL:")
                .with_default("http://localhost:8090")
                .prompt()?;

            handle_mock(
                Some(method),
                Some(path),
                Some(status),
                Some(body),
                server,
            )
            .await?;
        }

        "Exit" => {
            println!("{}", "👋 Goodbye!".bright_green());
            return Ok(());
        }

        _ => unreachable!(),
    }

    Ok(())
}

// Handle import command
async fn handle_import(
    file: PathBuf,
    start: bool,
    open: bool,
    port: Option<u16>,
    cli: &Cli,
) -> anyhow::Result<()> {
    println!("{} {}", "📥 Importing OpenAPI spec from".bright_blue(), file.display());

    // Load and validate OpenAPI spec
    let spec = load_openapi_from_file(&file)
        .map_err(|e| anyhow::anyhow!("Failed to load OpenAPI spec: {}", e))?;

    println!("{} OpenAPI spec loaded successfully", "✓".bright_green());

    if start {
        let config = build_server_config(cli, port);

        // Import the spec by setting OPENAPI_FILE env variable
        std::env::set_var("OPENAPI_FILE", file.to_string_lossy().to_string());

        println!("{} Starting server with imported endpoints...", "🚀".bright_cyan());
        start_server_with_browser(config, open).await?;
    } else {
        // Just validate and show info
        let mut endpoint_count = 0;
        for (_path, item) in &spec.paths.paths {
            if let openapiv3::ReferenceOr::Item(path_item) = item {
                if path_item.get.is_some() { endpoint_count += 1; }
                if path_item.post.is_some() { endpoint_count += 1; }
                if path_item.put.is_some() { endpoint_count += 1; }
                if path_item.patch.is_some() { endpoint_count += 1; }
                if path_item.delete.is_some() { endpoint_count += 1; }
            }
        }

        println!(
            "{} Found {} endpoints in OpenAPI spec",
            "✓".bright_green(),
            endpoint_count.to_string().bright_yellow()
        );
        println!("{}", "Tip: Use --start to launch server with these endpoints".bright_black());
    }

    Ok(())
}

// Handle mock command
async fn handle_mock(
    method: Option<String>,
    path: Option<String>,
    status: Option<u16>,
    body: Option<String>,
    server: String,
) -> anyhow::Result<()> {
    // Interactive mode if arguments missing
    let method = if let Some(m) = method {
        m.to_uppercase()
    } else {
        Text::new("HTTP method:")
            .with_default("GET")
            .prompt()?
            .to_uppercase()
    };

    let path = if let Some(p) = path {
        p
    } else {
        Text::new("Endpoint path:")
            .with_placeholder("/api/users")
            .prompt()?
    };

    let status = if let Some(s) = status {
        s
    } else {
        let status_input = Text::new("Status code:")
            .with_default("200")
            .prompt()?;
        status_input.parse().unwrap_or(200)
    };

    let body = if let Some(b) = body {
        b
    } else {
        Text::new("Response body (JSON):")
            .with_default(r#"{"message": "OK"}"#)
            .prompt()?
    };

    // Parse response body as JSON
    let response: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| anyhow::anyhow!("Invalid JSON response body: {}", e))?;

    // Create endpoint config
    let endpoint = EndpointConfig {
        method: method.clone(),
        path: path.clone(),
        response,
        status: Some(status),
        headers: None,
        proxy_url: None,
    };

    // Send to server API
    let client = reqwest::Client::new();
    let url = format!("{}/__mock/endpoints", server.trim_end_matches('/'));

    println!(
        "{} Creating mock: {} {} → {}",
        "🔨".bright_yellow(),
        method.bright_cyan(),
        path.bright_white(),
        status.to_string().bright_green()
    );

    match client.post(&url).json(&endpoint).send().await {
        Ok(response) => {
            if response.status().is_success() {
                println!("{} Mock endpoint created successfully!", "✓".bright_green());
                println!();
                println!("{}", "Test it:".bright_black());
                println!("  curl -X {} {}{}", method, server.trim_end_matches('/'), path);
            } else {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                return Err(anyhow::anyhow!(
                    "Server returned error {}: {}",
                    status,
                    error_text
                ));
            }
        }
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Failed to connect to server at {}: {}\nIs the server running? Try: mokku server",
                server,
                e
            ));
        }
    }

    Ok(())
}

// Handle replay command (placeholder)
fn handle_replay(name: String) -> anyhow::Result<()> {
    println!(
        "{} Replay mode is not yet implemented",
        "⚠️".bright_yellow()
    );
    println!();
    println!("This feature is coming soon! It will allow you to:");
    println!("  • Record API traffic");
    println!("  • Save sessions with name: {}", name.bright_cyan());
    println!("  • Replay them later for testing");
    println!();
    println!("Track progress: https://github.com/arthurkowalsky/Rust-Mock/issues");

    std::process::exit(1);
}

// Build server config from CLI args
fn build_server_config(cli: &Cli, port_override: Option<u16>) -> ServerConfig {
    ServerConfig {
        host: cli.host.clone().unwrap_or_else(|| "0.0.0.0".to_string()),
        port: port_override.or(cli.port).unwrap_or(8090),
        default_proxy_url: cli.proxy.clone(),
    }
}

// Start server and optionally open browser
async fn start_server_with_browser(config: ServerConfig, open_browser: bool) -> anyhow::Result<()> {
    let url = format!("http://localhost:{}", config.port);

    println!();
    println!("{}", "🚀 Starting Mokku Server...".bright_cyan().bold());
    println!();
    println!("  {} {}", "Dashboard:".bright_black(), url.bright_white().underline());
    println!("  {} http://{}:{}", "Bind:".bright_black(), config.host, config.port);

    if let Some(ref proxy) = config.default_proxy_url {
        println!("  {} {}", "Proxy:".bright_black(), proxy.bright_yellow());
    }

    println!();
    println!("{}", "Press Ctrl+C to stop".bright_black());
    println!();

    // Open browser if requested
    if open_browser {
        let url_clone = url.clone();
        actix_web::rt::spawn(async move {
            // Wait a bit for server to start
            actix_web::rt::time::sleep(std::time::Duration::from_millis(1500)).await;
            if let Err(e) = open::that(&url_clone) {
                eprintln!("Failed to open browser: {}", e);
            }
        });
    }

    // Start server
    start_server(config).await?;

    Ok(())
}
