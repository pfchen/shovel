// This example demonstrates a comprehensive webserver CLI application showcasing
// Shovel's key features: hierarchical commands, options, arguments, enums, and
// the builder pattern.
//
// To run this example:
// 1. Navigate to the project root directory in your terminal.
// 2. Execute: `cargo run --example webserver -- --help`
//
// Try these commands:
//
// Basic commands:
//   cargo run --example webserver -- serve -p 8080
//   cargo run --example webserver -- serve --host 0.0.0.0 --port 3000
//   cargo run --example webserver -- logs --level debug
//   cargo run --example webserver -- logs -f
//
// Database commands:
//   cargo run --example webserver -- db migrate up
//   cargo run --example webserver -- db migrate --dry-run down  # options before args
//   cargo run --example webserver -- db seed users.toml
//   cargo run --example webserver -- db backup /backups/dump.sql
//
// RPC commands:
//   cargo run --example webserver -- rpc server -p 9090 grpc
//   cargo run --example webserver -- rpc client --timeout 5 localhost:9090 get_user '{"id":1}'
//   cargo run --example webserver -- rpc codegen schema.proto ./output rust
//
// Config commands:
//   cargo run --example webserver -- config validate config.toml
//   cargo run --example webserver -- config generate --output prod.toml production
//   cargo run --example webserver -- config reload SIGHUP  # signal is required
//
// Global options:
//   cargo run --example webserver -- --verbose serve -p 8080
//   cargo run --example webserver -- --help
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use shovel::*;

// ============= Global Options =============

#[derive(Opts)]
pub struct GlobalOpts {
    /// Enable verbose logging
    #[short]
    pub verbose: bool,
}

// ============= Server Options =============

#[derive(Opts)]
pub struct ServeOpts {
    /// Server port
    #[short = 'p']
    pub port: Option<u16>,
    /// Bind address
    #[short = 'H']
    pub host: Option<String>,
    /// Number of worker threads
    #[short = 'w']
    pub workers: Option<usize>,
    /// TLS certificate file
    #[short]
    pub tls_cert: Option<PathBuf>,
    /// TLS private key file
    #[short = 'k']
    pub tls_key: Option<PathBuf>,
}

// ============= Database Options =============

#[derive(Opts)]
pub struct MigrateOpts {
    /// Number of migration steps to run (0 for all)
    #[short = 'n']
    pub steps: Option<u32>,
    /// Preview changes without applying
    #[short]
    pub dry_run: bool,
}

#[derive(Opts)]
pub struct SeedOpts {
    /// Overwrite existing data
    #[short]
    pub force: bool,
}

#[derive(Opts)]
pub struct BackupOpts {
    /// Compress the backup with gzip
    #[short]
    pub compress: bool,
}

// ============= Database Arguments =============

pub enum MigrationDirection {
    Up,
    Down,
    Redo,
}

impl FromStr for MigrationDirection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "up" => Ok(MigrationDirection::Up),
            "down" => Ok(MigrationDirection::Down),
            "redo" => Ok(MigrationDirection::Redo),
            _ => Err(format!(
                "Invalid direction: {}. Use 'up', 'down', or 'redo'",
                s
            )),
        }
    }
}

/// up, down, or redo
#[derive(Args)]
pub struct MigrateArgs {
    /// Migration direction: up, down, or redo
    #[format = "DIRECTION"]
    pub direction: MigrationDirection,
}

#[derive(Args)]
pub struct SeedArgs {
    /// Path to the seed data file
    pub file: PathBuf,
}

#[derive(Args)]
pub struct BackupArgs {
    /// Output path for the backup file
    pub output: PathBuf,
}

// ============= RPC Options =============

#[derive(Opts)]
pub struct RpcServerOpts {
    /// Server port
    #[short = 'p']
    pub port: Option<u16>,
    /// Maximum concurrent connections
    #[short = 'm']
    pub max_connections: Option<u32>,
    /// Request timeout in seconds
    #[short = 't']
    pub timeout: Option<u64>,
}

#[derive(Opts)]
pub struct RpcClientOpts {
    /// Request timeout in seconds
    #[short = 't']
    pub timeout: Option<u64>,
}

#[derive(Opts)]
pub struct RpcCodegenOpts {
    /// Generate additional documentation
    #[short]
    pub docs: bool,
}

// ============= RPC Arguments =============

pub enum RpcProtocol {
    Grpc,
    JsonRpc,
    MsgPack,
}

impl FromStr for RpcProtocol {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "grpc" => Ok(RpcProtocol::Grpc),
            "jsonrpc" => Ok(RpcProtocol::JsonRpc),
            "msgpack" => Ok(RpcProtocol::MsgPack),
            _ => Err(format!("Invalid protocol: {}", s)),
        }
    }
}

impl fmt::Display for RpcProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpcProtocol::Grpc => write!(f, "grpc"),
            RpcProtocol::JsonRpc => write!(f, "jsonrpc"),
            RpcProtocol::MsgPack => write!(f, "msgpack"),
        }
    }
}

/// grpc, jsonrpc, or msgpack
#[derive(Args)]
pub struct RpcServerArgs {
    /// grpc, jsonrpc, or msgpack
    #[format = "PROTOCOL"]
    pub protocol: RpcProtocol,
}

#[derive(Args)]
pub struct RpcClientArgs {
    /// RPC server endpoint
    #[format = "ENDPOINT"]
    pub endpoint: String,
    /// RPC method name to call
    #[format = "METHOD"]
    pub method: String,
    /// JSON payload
    pub payload: String,
}

pub enum CodegenLanguage {
    Rust,
    Go,
    TypeScript,
    Python,
    Java,
}

impl FromStr for CodegenLanguage {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rust" | "rs" => Ok(CodegenLanguage::Rust),
            "go" | "golang" => Ok(CodegenLanguage::Go),
            "typescript" | "ts" => Ok(CodegenLanguage::TypeScript),
            "python" | "py" => Ok(CodegenLanguage::Python),
            "java" => Ok(CodegenLanguage::Java),
            _ => Err(format!(
                "Invalid language: {}. Use 'rust', 'go', 'typescript', 'python', or 'java'",
                s
            )),
        }
    }
}

#[derive(Args)]
pub struct RpcCodegenArgs {
    /// Path to the schema file
    #[format = "SCHEMA"]
    pub schema: PathBuf,
    /// Output directory for generated code
    #[format = "OUTPUT"]
    pub output: PathBuf,
    /// Target language
    #[format = "LANGUAGE"]
    pub language: CodegenLanguage,
}

// ============= Logs Options =============

#[derive(Opts)]
pub struct LogsOpts {
    /// Log level: error, warn, info, debug, trace
    #[short = 'l']
    pub level: Option<String>,
    /// Follow the log file (tail -f)
    #[short = 'f']
    pub follow: bool,
    /// Number of lines to show
    #[short = 'n']
    pub lines: Option<usize>,
    /// Filter logs by pattern
    #[short = 'i']
    pub filter: Option<String>,
}

// ============= Config Options =============

#[derive(Opts)]
pub struct ConfigValidateOpts {
    /// Strict validation (fail on warnings)
    #[short]
    pub strict: bool,
}

#[derive(Opts)]
pub struct ConfigGenerateOpts {
    /// Output file path
    #[short = 'o']
    pub output: Option<PathBuf>,
}

// ============= Config Arguments =============

pub enum ConfigSignal {
    Sighup,
    Sigterm,
    Sigint,
}

impl FromStr for ConfigSignal {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "SIGHUP" | "HUP" => Ok(ConfigSignal::Sighup),
            "SIGTERM" | "TERM" => Ok(ConfigSignal::Sigterm),
            "SIGINT" | "INT" => Ok(ConfigSignal::Sigint),
            _ => Err(format!(
                "Invalid signal: {}. Use 'SIGHUP', 'SIGTERM', or 'SIGINT'",
                s
            )),
        }
    }
}

pub enum ConfigTemplate {
    Development,
    Staging,
    Production,
}

impl FromStr for ConfigTemplate {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(ConfigTemplate::Development),
            "staging" | "stage" => Ok(ConfigTemplate::Staging),
            "production" | "prod" => Ok(ConfigTemplate::Production),
            _ => Err(format!(
                "Invalid template: {}. Use 'development', 'staging', or 'production'",
                s
            )),
        }
    }
}

/// Path to the config file
#[derive(Args)]
pub struct ConfigValidateArgs {
    /// Path to the config file
    #[format = "FILE"]
    pub file: PathBuf,
}

#[derive(Args)]
pub struct ConfigGenerateArgs {
    /// Template name: development, staging, or production
    #[format = "TEMPLATE"]
    pub template: ConfigTemplate,
}

#[derive(Args)]
pub struct ConfigReloadArgs {
    /// Signal to send: SIGHUP, SIGTERM, or SIGINT
    #[format = "SIGNAL"]
    pub signal: ConfigSignal,
}

fn main() {
    let code = App {
        name: "webserver",
        description: "A web server management CLI with database, RPC, and configuration tools",
        version: "0.1.0",
        authors: ["Author Name <author@example.com>"],
        copyright: "(c) 2026 Company Name",
        action: AppAction::subcmds((
            Cmd {
                name: "serve",
                description: "Start the web server",
                category: None,
                action: Action::opts_with_global_opts(|opts: ServeOpts, global: GlobalOpts| {
                    if global.verbose {
                        println!("[DEBUG] Starting web server...");
                    }

                    let port = opts.port.unwrap_or(8080);
                    let host = opts.host.as_deref().unwrap_or("127.0.0.1");
                    let workers = opts.workers.unwrap_or(4);

                    println!("Starting web server on {}:{}", host, port);
                    println!("Worker threads: {}", workers);

                    if let (Some(cert), Some(key)) = (&opts.tls_cert, &opts.tls_key) {
                        println!("TLS enabled: {} / {}", cert.display(), key.display());
                    }

                    0
                }),
            },
            Cmd {
                name: "db",
                description: "Database management operations",
                category: None,
                action: Action::subcmds((
                    Cmd {
                        name: "migrate",
                        description: "Run database migrations",
                        category: Some("database"),
                        action: Action::all(
                            |opts: MigrateOpts, args: MigrateArgs, global: GlobalOpts| {
                                if global.verbose {
                                    println!("[DEBUG] Running database migrations...");
                                }

                                match args.direction {
                                    MigrationDirection::Up => {
                                        println!("Running migrations forward");
                                        if let Some(n) = opts.steps {
                                            println!("  Steps: {}", n);
                                        }
                                        if opts.dry_run {
                                            println!("  [DRY RUN - no changes applied]");
                                        }
                                    }
                                    MigrationDirection::Down => {
                                        println!("Rolling back migrations");
                                        if let Some(n) = opts.steps {
                                            println!("  Steps: {}", n);
                                        }
                                    }
                                    MigrationDirection::Redo => {
                                        println!("Re-running last migration");
                                    }
                                }

                                0
                            },
                        ),
                    },
                    Cmd {
                        name: "seed",
                        description: "Seed database with initial data",
                        category: Some("database"),
                        action: Action::all(
                            |opts: SeedOpts, args: SeedArgs, global: GlobalOpts| {
                                if global.verbose {
                                    println!("[DEBUG] Seeding database from: {}", args.file.display());
                                }
                                println!("Seeding database from: {}", args.file.display());
                                if opts.force {
                                    println!("  [Overwriting existing data]");
                                }

                                0
                            },
                        ),
                    },
                    Cmd {
                        name: "backup",
                        description: "Create a database backup",
                        category: Some("database"),
                        action: Action::all(
                            |opts: BackupOpts, args: BackupArgs, global: GlobalOpts| {
                                if global.verbose {
                                    println!("[DEBUG] Creating backup at: {}", args.output.display());
                                }
                                println!("Creating backup at: {}", args.output.display());
                                if opts.compress {
                                    println!("  [Compressing with gzip]");
                                }

                                0
                            },
                        ),
                    },
                )),
            },
            Cmd {
                name: "rpc",
                description: "RPC service management",
                category: None,
                action: Action::subcmds((
                    Cmd {
                        name: "server",
                        description: "Start an RPC server",
                        category: Some("rpc"),
                        action: Action::all(
                            |opts: RpcServerOpts, args: RpcServerArgs, global: GlobalOpts| {
                                if global.verbose {
                                    println!("[DEBUG] Starting RPC server...");
                                }

                                match args.protocol {
                                    RpcProtocol::Grpc => {
                                        let port = opts.port.unwrap_or(50051);
                                        println!("Starting gRPC server on port {}", port);
                                        if let Some(m) = opts.max_connections {
                                            println!("  Max connections: {}", m);
                                        }
                                    }
                                    RpcProtocol::JsonRpc => {
                                        let port = opts.port.unwrap_or(8080);
                                        println!("Starting JSON-RPC server on port {}", port);
                                    }
                                    RpcProtocol::MsgPack => {
                                        let port = opts.port.unwrap_or(9000);
                                        println!("Starting MessagePack RPC server on port {}", port);
                                    }
                                }

                                0
                            },
                        ),
                    },
                    Cmd {
                        name: "client",
                        description: "Call an RPC method",
                        category: Some("rpc"),
                        action: Action::all(
                            |opts: RpcClientOpts, args: RpcClientArgs, global: GlobalOpts| {
                                if global.verbose {
                                    println!("[DEBUG] Calling RPC method '{}' on {}", args.method, args.endpoint);
                                }

                                println!("Calling RPC method '{}' on {}", args.method, args.endpoint);
                                if let Some(timeout) = opts.timeout {
                                    println!("  Timeout: {}s", timeout);
                                }
                                if !args.payload.is_empty() {
                                    println!("  Payload: {}", args.payload);
                                }

                                0
                            },
                        ),
                    },
                    Cmd {
                        name: "codegen",
                        description: "Generate RPC client/server code from schema",
                        category: Some("rpc"),
                        action: Action::all(
                            |opts: RpcCodegenOpts, args: RpcCodegenArgs, global: GlobalOpts| {
                                if global.verbose {
                                    println!("[DEBUG] Generating RPC code from {}", args.schema.display());
                                }

                                let lang_name = match args.language {
                                    CodegenLanguage::Rust => "Rust",
                                    CodegenLanguage::Go => "Go",
                                    CodegenLanguage::TypeScript => "TypeScript",
                                    CodegenLanguage::Python => "Python",
                                    CodegenLanguage::Java => "Java",
                                };
                                println!(
                                    "Generating {} RPC code from {}",
                                    lang_name,
                                    args.schema.display()
                                );
                                println!("  Output: {}", args.output.display());
                                if opts.docs {
                                    println!("  [Including documentation]");
                                }

                                0
                            },
                        ),
                    },
                )),
            },
            Cmd {
                name: "logs",
                description: "View and filter server logs",
                category: None,
                action: Action::opts_with_global_opts(|opts: LogsOpts, global: GlobalOpts| {
                    if global.verbose {
                        println!("[DEBUG] Opening log viewer...");
                    }

                    println!("Log viewer");

                    if let Some(level) = &opts.level {
                        println!("  Level: {}", level);
                    }
                    if opts.follow {
                        println!("  [Following log file]");
                    }
                    if let Some(n) = opts.lines {
                        println!("  Lines: {}", n);
                    }
                    if let Some(filter) = &opts.filter {
                        println!("  Filter: {}", filter);
                    }

                    0
                }),
            },
            Cmd {
                name: "config",
                description: "Configuration management",
                category: None,
                action: Action::subcmds((
                    Cmd {
                        name: "validate",
                        description: "Validate a configuration file",
                        category: Some("configuration"),
                        action: Action::all(
                            |opts: ConfigValidateOpts, args: ConfigValidateArgs, global: GlobalOpts| {
                                if global.verbose {
                                    println!("[DEBUG] Validating configuration: {}", args.file.display());
                                }
                                println!("Validating configuration: {}", args.file.display());
                                if opts.strict {
                                    println!("  [Strict mode - warnings fail]");
                                }

                                0
                            },
                        ),
                    },
                    Cmd {
                        name: "generate",
                        description: "Generate a configuration file from template",
                        category: Some("configuration"),
                        action: Action::all(
                            |opts: ConfigGenerateOpts, args: ConfigGenerateArgs, global: GlobalOpts| {
                                if global.verbose {
                                    println!("[DEBUG] Generating configuration from template...");
                                }

                                let template_name = match args.template {
                                    ConfigTemplate::Development => "development",
                                    ConfigTemplate::Staging => "staging",
                                    ConfigTemplate::Production => "production",
                                };
                                println!("Generating configuration from template: {}", template_name);
                                if let Some(output) = &opts.output {
                                    println!("  Output: {}", output.display());
                                }

                                0
                            },
                        ),
                    },
                    Cmd {
                        name: "reload",
                        description: "Reload the server configuration",
                        category: Some("configuration"),
                        action: Action::args_with_global_opts(|args: ConfigReloadArgs, global: GlobalOpts| {
                            if global.verbose {
                                println!("[DEBUG] Reloading configuration with signal...");
                            }

                            let signal_name = match args.signal {
                                ConfigSignal::Sighup => "SIGHUP",
                                ConfigSignal::Sigterm => "SIGTERM",
                                ConfigSignal::Sigint => "SIGINT",
                            };
                            println!("Sending {} to reload configuration", signal_name);

                            0
                        }),
                    },
                )),
            },
        )),
    }
    .run()
    .expect("Failed to run the application");
    std::process::exit(code);
}
