# Shovel

`shovel` is a declarative, simple, and fast package for building command line tools in Rust.

## Key Features

- **Declarative syntax**: Define CLI commands using simple structs and derive macros
- **Fast execution**: Minimal runtime overhead with compile-time optimizations
- **No-std support**: Works in embedded and resource-constrained environments
- **Type safety**: Compile-time validation of command structures and arguments
- **Zero allocation**: Stack-based parsing for no-std environments
- **Automatic help generation**: Built-in help system with customizable formatting
- **Shell completion**: Built-in support for Bash, Zsh, and Fish completion
- **Hierarchical commands**: Support for nested subcommands with shared options

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage](#usage)
  - [Simple Application](#simple-application)
  - [Application with Options and Arguments](#application-with-options-and-arguments)
  - [Applications with Subcommands](#applications-with-subcommands)
- [Builder Pattern](#builder-pattern)
- [Derive Macros and Traits](#derive-macros-and-traits)
  - [#[derive(Opts)] - Optional Arguments](#deriveopts---optional-arguments)
  - [#[derive(Args)] - Positional Arguments](#deriveargs---positional-arguments)
- [Environment-Specific Usage](#environment-specific-usage)
  - [Standard Environment](#standard-environment-with-std-feature)
  - [No-std Environment](#no-std-environment)
  - [Custom I/O and Testing](#custom-io-and-testing)
- [Shell Completion](#shell-completion)
- [Scalability Features](#scalability-features)
- [Contributing](#contributing)
- [License](#license)

## Installation

Add `shovel` to your `Cargo.toml`:

```toml
[dependencies]
shovel = "<version>"
```

## Usage

### Simple Application

Create a basic CLI application:

```rust
use shovel::*;

fn main() -> std::fmt::Result {
    App {
        name: "hello",
        description: "A simple greeting application",
        version: "0.1.0",
        authors: ["Author Name <author@example.com>"],
        copyright: "(c) 2026 Company Name",
        action: Action::no_args(|| {
            println!("Hello, world!");
            println!("Welcome to Shovel CLI framework!");
        }),
    }
    .run()
}
```

### Application with Options and Arguments

```rust
use shovel::*;
use std::path::PathBuf;

#[derive(Opts)]
struct ProcessOpts {
    /// Output file path
    #[short = 'o']
    output: Option<PathBuf>,
    /// Enable verbose logging
    #[short = 'v']
    verbose: bool,
    /// Force overwrite existing files
    #[short = 'f']
    force: bool,
}

#[derive(Args)]
struct ProcessArgs {
    /// Input file to process
    input: PathBuf,
}

fn main() -> std::fmt::Result {
    App {
        name: "fileproc",
        description: "A file processing tool",
        version: "0.1.0",
        authors: ["Author Name <author@example.com>"],
        copyright: "(c) 2026 Company Name",
        action: Action::opts_args(|opts: ProcessOpts, args: ProcessArgs| {
            if opts.verbose {
                println!("Processing file: {}", args.input.display());
            }
            let output = opts.output.unwrap_or_else(|| {
                args.input.with_extension("processed")
            });
            if output.exists() && !opts.force {
                eprintln!("Output file exists. Use --force to overwrite.");
                return;
            }
            println!("Output will be written to: {}", output.display());
        }),
    }
    .run()
}
```

### Applications with Subcommands

For applications with subcommands, use direct construction with the `Action::subcmds()` method:

```rust
use shovel::*;
use std::path::PathBuf;

#[derive(Opts)]
struct ServeOpts {
    /// Server port
    #[short = 'p']
    port: Option<u16>,
    /// Number of worker threads
    #[short = 'w']
    workers: Option<usize>,
}

fn main() {
    App {
        name: "webserver",
        description: "A web server with database management",
        version: "0.1.0",
        authors: ["Author Name <author@example.com>"],
        copyright: "(c) 2026 Company Name",
        action: Action::subcmds((
            Cmd {
                name: "serve",
                description: "Start the web server",
                category: None,
                action: Action::opts(|opts: ServeOpts| {
                    let port = opts.port.unwrap_or(8080);
                    let workers = opts.workers.unwrap_or(4);
                    println!("Starting server on port {} with {} workers", port, workers);
                }),
            },
            Cmd {
                name: "database",
                description: "Database management commands",
                category: None,
                action: Action::subcmds((
                    Cmd {
                        name: "migrate",
                        description: "Run database migrations",
                        category: Some("database"),
                        action: Action::args(|direction: String| {
                            println!("Running migration: {}", direction);
                        }),
                    },
                    Cmd {
                        name: "backup",
                        description: "Create database backup",
                        category: Some("database"),
                        action: Action::args(|output: PathBuf| {
                            println!("Creating backup at: {}", output.display());
                        }),
                    },
                )),
            },
        )),
    }
    .run()
    .expect("Failed to run application");
}
```

## Builder Pattern

Shovel provides [`AppBuilder`] and [`CmdBuilder`] for a more fluent API when
constructing applications. These builders are available when the `std`
feature is enabled and provide a convenient way to set application metadata
and actions.

### Builder Example

```rust
use shovel::*;
use std::path::PathBuf;

fn main() {
    AppBuilder::new("rpcserver")
        .description("An RPC server with management commands")
        .version("0.1.0")
        .authors(["Author Name <author@example.com>"])
        .copyright("(c) 2026 Company Name")
        .action(Action::subcmds((
            CmdBuilder::new("server")
                .description("Start the RPC server")
                .action(Action::args(|port: u16| {
                    println!("Starting RPC server on port {}", port);
                }))
                .build(),
            CmdBuilder::new("client")
                .description("RPC client operations")
                .action(Action::subcmds((
                    CmdBuilder::new("call")
                        .description("Make an RPC call")
                        .category("client")
                        .action(Action::args(|(endpoint, method): (String, String)| {
                            println!("Calling {} on {}", method, endpoint);
                        }))
                        .build(),
                    CmdBuilder::new("test")
                        .description("Test RPC connectivity")
                        .category("client")
                        .action(Action::args(|endpoint: String| {
                            println!("Testing connection to {}", endpoint);
                        }))
                        .build(),
                )))
                .build(),
            CmdBuilder::new("codegen")
                .description("Generate RPC client code")
                .action(Action::args(|(schema, output): (PathBuf, PathBuf)| {
                    println!("Generating code from {} to {}", schema.display(), output.display());
                }))
                .build(),
        )))
        .build()
        .run()
        .expect("Failed to run application");
}
```

## Derive Macros and Traits

Shovel provides derive macros to automatically implement the required traits for your data structures:

### `#[derive(Opts)]` - Optional Arguments

Define command-line options (flags and valued options):

```rust
use shovel::*;
use std::path::PathBuf;

#[derive(Opts)]
pub struct ServerOpts {
    /// Server port
    #[short = 'p']
    pub port: Option<u16>,
    /// Custom long name (defaults to kebab-case field name, e.g. max_connections -> max-connections)
    #[long = "bind-host"]
    /// Host address to bind
    #[short = 'h']
    pub host: Option<String>,
    /// Number of worker threads
    #[short = 'w']
    pub workers: Option<usize>,
    /// Enable TLS encryption
    #[short = 't']
    pub tls: bool,
}

#[derive(Opts)]
pub struct DatabaseOpts {
    /// Database connection URL
    #[short = 'u']
    pub url: Option<String>,
    /// Connection timeout in seconds
    #[short = 't']
    pub timeout: Option<u64>,
    /// Enable connection pooling
    #[short = 'p']
    pub pool: bool,
}

#[derive(Opts)]
struct LogOpts {
    /// Log level (debug, info, warn, error)
    #[short = 'l']
    pub level: Option<String>,
    /// Follow log output
    #[short = 'f']
    pub follow: bool,
    /// Number of lines to show
    #[short = 'n']
    pub lines: Option<usize>,
}
```

### `#[derive(Args)]` - Positional Arguments

Define positional arguments for commands:

```rust
use shovel::*;
use std::path::PathBuf;

#[derive(Args)]
pub struct DeployArgs {
    /// Target environment (staging, production)
    pub environment: String,
    /// Configuration file path
    pub config: PathBuf,
}

#[derive(Args)]
pub struct BackupArgs {
    /// Database name to backup
    pub database: String,
    /// Output file path
    pub output: PathBuf,
}

#[derive(Args)]
pub enum RpcArgs {
    /// Start RPC server
    #[format = "<PORT>"]
    Server(u16),
    /// Make RPC call
    #[format = "<ENDPOINT> <METHOD>"]
    Call(String, String),
    /// Generate client code
    #[format = "<SCHEMA> <OUTPUT> <LANGUAGE>"]
    Codegen(PathBuf, PathBuf, String),
    /// Batch process multiple files
    #[format = "<FILE>..."]
    Batch(Vec<PathBuf>),
}
```

## Environment-Specific Usage

Shovel is designed to work in both standard and no-std environments, providing
flexibility for different deployment scenarios.

### Standard Environment (with `std` feature)

For typical CLI applications running on systems with standard library support:

```toml
[dependencies]
shovel = "<version>"  # Includes std and completion features by default
```

The `run()` method provides the most convenient interface:

```rust
use shovel::*;

fn main() {
    let app = App {
        name: "app",
        description: "desc",
        version: "0.1.0",
        authors: ["Author Name <author@example.com>"],
        copyright: "(c) 2026 Company Name",
        action: Action::no_args(|| {})
    };
    // Uses std::env::args() and stdout automatically
    app.run().expect("Failed to run application");
}
```

### No-std Environment

For embedded systems, WASM, or other resource-constrained environments:

```toml
[dependencies]
shovel = { version = "<version>", default-features = false }
```

Use stack-based allocation with a fixed-size buffer:

```rust
use shovel::*;

fn main() {
    App {
        name: "app",
        description: "desc",
        version: "0.1.0",
        authors: ["Author Name <author@example.com>"],
        copyright: "(c) 2026 Company Name",
        action: Action::no_args(|| {})
    }
    .run_heapless::<1, _>(&mut shovel::Stdout::new(), &["app", "subcommand"])
    .expect("Failed to run application");
}
```

The generic parameter `<1, _>` must match the exact command count (`app.cmd_count()`), unless
the `generic_const_exprs` feature is enabled.

### Custom I/O and Testing

For testing or custom I/O handling, use the flexible `run_with_args` method:

```rust
use shovel::*;

fn main() {
    let app = App {
        name: "app",
        description: "desc",
        version: "0.1.0",
        authors: ["Author Name <author@example.com>"],
        copyright: "(c) 2026 Company Name",
        action: Action::no_args(|| {})
    };
    let mut output = std::io::Cursor::new(Vec::new());
    let mut writer = IoWriteAdapter::new(&mut output);
    app.run_with_args(&mut writer, &["app", "--help"]).expect("Failed to run application");

    // Access the captured output
    let help_text = String::from_utf8(output.into_inner()).unwrap();
    assert!(help_text.contains("app"));
}
```

## Shell Completion

Shovel provides built-in shell completion support for Bash, Zsh, and Fish shells.
This feature is enabled by default when using the `std` feature.

### Enabling Completion

For custom feature configurations, explicitly enable completion:

```toml
[dependencies]
shovel = { version = "<version>", features = ["std", "completion"] }
```

### Generating Completion Scripts

Shovel automatically adds a `--completion` option to your application. Users can
generate completion scripts for their preferred shell:

```bash
# For Bash
myapp --completion bash > ~/.bash_completion.d/myapp
source ~/.bash_completion.d/myapp

# For Zsh
myapp --completion zsh > ~/.zsh/completions/_myapp
# Add ~/.zsh/completions to your fpath in ~/.zshrc

# For Fish
myapp --completion fish > ~/.config/fish/completions/myapp.fish
```

### Completion Features

The generated completion scripts provide:

- Command and subcommand completion
- Option and flag completion (including short forms)
- Context-aware suggestions based on command hierarchy
- Help text integration for better user experience

Completion works automatically for all commands, subcommands, and options
defined in your application structure.

## Scalability Features

Shovel supports applications with varying numbers of subcommands through feature-gated compilation limits. By default, applications can have up to 25 subcommands at any single level.

### Enabling Large Subcommand Support

For applications that need more subcommands, enable the appropriate feature:

```toml
[dependencies]
# Default: up to 25 subcommands
shovel = "<version>"

# For applications with up to 50 subcommands
shovel = { version = "<version>", features = ["subcmds-50"] }

# For applications with up to 75 subcommands
shovel = { version = "<version>", features = ["subcmds-75"] }

# For applications with up to 100 subcommands
shovel = { version = "<version>", features = ["subcmds-100"] }
```

### Feature Behavior

- **Additive**: Multiple features can be enabled simultaneously
- **Highest wins**: When multiple features are enabled, the highest limit is used
- **No duplication**: Only one implementation is generated regardless of features enabled
- **Compile-time**: All limits are enforced at compile time for zero runtime overhead

### Design Recommendations

For applications with many commands, consider using hierarchical command structures instead of flat command lists:

```rust
use shovel::*;

fn main() {
    App {
        name: "devops-tool",
        description: "A comprehensive DevOps management tool",
        version: "0.1.0",
        authors: ["Author Name <author@example.com>"],
        copyright: "(c) 2026 Company Name",
        action: Action::subcmds((
            // Group 1: Server management (serve, status, restart, logs, etc.)
            Cmd {
                name: "server",
                description: "Server management operations",
                category: None,
                action: Action::subcmds((/* server subcommands */)),
            },
            // Group 2: Database operations (migrate, backup, restore, seed, etc.)
            Cmd {
                name: "database",
                description: "Database management operations",
                category: None,
                action: Action::subcmds((/* database subcommands */)),
            },
            // Group 3: Deployment operations (deploy, rollback, status, etc.)
            Cmd {
                name: "deploy",
                description: "Deployment management operations",
                category: None,
                action: Action::subcmds((/* deployment subcommands */)),
            },
            // Group 4: Monitoring operations (metrics, alerts, health, etc.)
            Cmd {
                name: "monitor",
                description: "Monitoring and observability operations",
                category: None,
                action: Action::subcmds((/* monitoring subcommands */)),
            },
        )),
    }
    .run()
    .expect("Failed to run application");
}
```

This hierarchical approach provides better user experience and keeps individual command groups manageable.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
