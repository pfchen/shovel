//! # Shovel
//!
//! Shovel is a declarative, simple, and fast command-line parsing library designed for
//! both standard and no-std environments. It provides a type-safe, zero-allocation
//! approach to building CLI applications with automatic help generation and shell completion.
//!
//! ## Key Features
//!
//! - **Declarative syntax**: Define CLI commands using simple structs and derive macros
//! - **Fast execution**: Minimal runtime overhead with compile-time optimizations
//! - **No-std support**: Works in embedded and resource-constrained environments
//! - **Type safety**: Compile-time validation of command structures and arguments
//! - **Zero allocation**: Stack-based parsing for no-std environments
//! - **Automatic help generation**: Built-in help system with customizable formatting
//! - **Shell completion**: Built-in support for Bash, Zsh, and Fish completion
//! - **Hierarchical commands**: Support for nested subcommands with shared options
//!
//! ## Architecture
//!
//! Shovel applications are structured as hierarchical trees of command nodes ([`Cmd`]).
//! The entry point [`App`] serves as the root of this tree. Each command node can have
//! multiple children ([`Subcmds`]), and leaf nodes must have a user-defined action handler
//! ([`Action`]) that executes when the command is invoked.
//!
//! ### Core Components
//!
//! - **[`App`]**: The root application containing metadata and the main action
//! - **[`Cmd`]**: Individual commands that can be nested to form subcommands
//! - **[`Action`]**: Defines what happens when a command is executed (handler, subcommands, or both)
//! - **[`Opts`]**: Optional arguments (flags and options) parsed from command line
//! - **[`Args`]**: Positional arguments required by commands
//! - **[`Subcmds`]**: Trait for collections of subcommands (implemented for tuples)
//! - **[`Context`]**: Runtime context containing parsed options, arguments, and global state
//!
//! ### Example Structure
//!
//! ```text
//! WebServer [App]
//! |
//! |--- serve [Cmd]
//! |     |--- port: Option<u16> [Opts]          # -p, --port 8080
//! |     |--- host: Option<String> [Opts]       # --host 0.0.0.0
//! |     |--- workers: Option<usize> [Opts]     # --workers 4
//! |     |--- tls_cert: Option<PathBuf> [Opts]  # --tls-cert server.crt
//! |     |--- tls_key: Option<PathBuf> [Opts]   # --tls-key server.key
//! |     `-> serve_handler [Action]
//! |
//! |--- database [Cmd]
//! |     |--- migrate [Subcmd]
//! |     |    |--- steps: Option<u32> [Opts]    # --steps 1
//! |     |    |--- dry_run: bool [Opts]         # --dry-run
//! |     |    |--- direction: String [Args]     # up, down, redo
//! |     |    `-> db_migrate_handler [Action]
//! |     |
//! |     |--- seed [Subcmd]
//! |     |    |--- force: bool [Opts]           # --force
//! |     |    |--- file: Option<PathBuf> [Args] # seed file path
//! |     |    `-> db_seed_handler [Action]
//! |     |
//! |     `--- backup [Subcmd]
//! |          |--- compress: bool [Opts]        # --compress
//! |          |--- output: PathBuf [Args]       # backup file path
//! |          `-> db_backup_handler [Action]
//! |
//! |--- rpc [Cmd]
//! |     |--- server [Subcmd]
//! |     |    |--- port: Option<u16> [Opts]     # -p, --port 9090
//! |     |    |--- max_connections: Option<u32> [Opts] # --max-connections 1000
//! |     |    |--- timeout: Option<u64> [Opts]  # --timeout 30
//! |     |    |--- protocol: String [Args]      # grpc, jsonrpc, msgpack
//! |     |    `-> rpc_server_handler [Action]
//! |     |
//! |     |--- client [Subcmd]
//! |     |    |--- timeout: Option<u64> [Opts]  # --timeout 10
//! |     |    |--- endpoint: String [Args]      # server endpoint
//! |     |    |--- method: String [Args]        # RPC method name
//! |     |    |--- payload: Option<String> [Args] # JSON payload
//! |     |    `-> rpc_client_handler [Action]
//! |     |
//! |     `--- codegen [Subcmd]
//! |          |--- schema: PathBuf [Args]       # schema file path
//! |          |--- output: PathBuf [Args]       # output directory
//! |          |--- language: String [Args]      # rust, python, go, typescript
//! |          `-> rpc_codegen_handler [Action]
//! |
//! |--- logs [Cmd]
//! |     |--- level: Option<String> [Opts]      # --level debug
//! |     |--- follow: bool [Opts]               # -f, --follow
//! |     |--- lines: Option<usize> [Opts]       # -n, --lines 100
//! |     |--- filter: Option<String> [Opts]     # --filter "error"
//! |     `-> logs_handler [Action]
//! |
//! `--- config [Cmd]
//!       |--- validate [Subcmd]
//!       |    |--- strict: bool [Opts]          # --strict
//!       |    |--- file: PathBuf [Args]         # config file path
//!       |    `-> config_validate_handler [Action]
//!       |
//!       |--- generate [Subcmd]
//!       |    |--- output: Option<PathBuf> [Opts] # -o, --output config.toml
//!       |    |--- template: String [Args]      # development, production
//!       |    `-> config_generate_handler [Action]
//!       |
//!       `--- reload [Subcmd]
//!            |--- signal: Option<String> [Opts] # --signal SIGHUP
//!            `-> config_reload_handler [Action]
//! ```
//!
//! This structure demonstrates real-world server application patterns:
//! - **Web server**: `serve` command with network configuration (host, port, TLS, workers)
//! - **Database management**: `database` with migration, seeding, and backup operations
//! - **RPC services**: `rpc` with server/client modes and code generation
//! - **Operational tools**: `logs` for monitoring and `config` for configuration management
//! - **Complex option types**: Network ports, file paths, timeouts, and protocol selection
//! - **Nested command hierarchies**: Commands with subcommands using [`Subcmds`]
//! - **Mixed argument patterns**: Required args, optional flags, and enum-like choices
//!
//! ## Getting Started
//!
//! Add Shovel to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! shovel = "<version>"
//! ```
//!
//! ### Simple Application
//!
//! Create a basic CLI application:
//!
//! ```no_run
//! # #[cfg(feature = "std")] {
//! use shovel::*;
//!
//! fn main() {
//!     App {
//!         name: "hello",
//!         description: "A simple greeting application",
//!         version: "0.1.0",
//!         authors: ["Author Name <author@example.com>"],
//!         copyright: "(c) 2026 Company Name",
//!         action: Action::no_args(|| {
//!             println!("Hello, world!");
//!             println!("Welcome to Shovel CLI framework!");
//!         }),
//!     }
//!     .run()
//!     .expect("Failed to run application");
//! }
//! # }
//! ```
//!
//! ### Application with Options and Arguments
//!
//! ```no_run
//! # #[cfg(feature = "std")] {
//! use shovel::*;
//! use std::path::PathBuf;
//!
//! #[derive(Opts)]
//! struct ProcessOpts {
//!     /// Output file path
//!     #[short = 'o']
//!     output: Option<PathBuf>,
//!     /// Enable verbose logging
//!     #[short = 'v']
//!     verbose: bool,
//!     /// Force overwrite existing files
//!     #[short = 'f']
//!     force: bool,
//! }
//!
//! #[derive(Args)]
//! struct ProcessArgs {
//!     /// Input file to process
//!     input: PathBuf,
//! }
//!
//! fn main() {
//!     App {
//!         name: "fileproc",
//!         description: "A file processing tool",
//!         version: "0.1.0",
//!         authors: ["Author Name <author@example.com>"],
//!         copyright: "(c) 2026 Company Name",
//!         action: Action::opts_args(|opts: ProcessOpts, args: ProcessArgs| {
//!             if opts.verbose {
//!                 println!("Processing file: {}", args.input.display());
//!             }
//!             let output = opts.output.unwrap_or_else(|| {
//!                 args.input.with_extension("processed")
//!             });
//!             if output.exists() && !opts.force {
//!                 eprintln!("Output file exists. Use --force to overwrite.");
//!                 return;
//!             }
//!             println!("Output will be written to: {}", output.display());
//!         }),
//!     }
//!     .run()
//!     .expect("Failed to run application");
//! }
//! # }
//! ```
//!
//! ### Applications with Subcommands
//!
//! For applications with subcommands, use direct construction with the `Action::subcmds()` method:
//!
//! ```no_run
//! # #[cfg(feature = "std")] {
//! use shovel::*;
//! use std::path::PathBuf;
//!
//! #[derive(Opts)]
//! struct ServeOpts {
//!     /// Server port
//!     #[short = 'p']
//!     port: Option<u16>,
//!     /// Number of worker threads
//!     #[short = 'w']
//!     workers: Option<usize>,
//! }
//!
//! fn main() {
//!     App {
//!         name: "webserver",
//!         description: "A web server with database management",
//!         version: "0.1.0",
//!         authors: ["Author Name <author@example.com>"],
//!         copyright: "(c) 2026 Company Name",
//!         action: Action::subcmds((
//!             Cmd {
//!                 name: "serve",
//!                 description: "Start the web server",
//!                 category: None,
//!                 action: Action::opts(|opts: ServeOpts| {
//!                     let port = opts.port.unwrap_or(8080);
//!                     let workers = opts.workers.unwrap_or(4);
//!                     println!("Starting server on port {} with {} workers", port, workers);
//!                 }),
//!             },
//!             Cmd {
//!                 name: "database",
//!                 description: "Database management commands",
//!                 category: None,
//!                 action: Action::subcmds((
//!                     Cmd {
//!                         name: "migrate",
//!                         description: "Run database migrations",
//!                         category: Some("database"),
//!                         action: Action::args(|direction: String| {
//!                             println!("Running migration: {}", direction);
//!                         }),
//!                     },
//!                     Cmd {
//!                         name: "backup",
//!                         description: "Create database backup",
//!                         category: Some("database"),
//!                         action: Action::args(|output: PathBuf| {
//!                             println!("Creating backup at: {}", output.display());
//!                         }),
//!                     },
//!                 )),
//!             },
//!         )),
//!     }
//!     .run()
//!     .expect("Failed to run application");
//! }
//! # }
//! ```
//!
//! ## Attribute Reference
//!
//! Shovel provides several attributes to customize the behavior of derived `Opts` and `Args`:
//!
//! ### `#[derive(Opts)]` Attributes
//!
//! - **`#[long = "custom-name"]`** - Sets the long option name (defaults to kebab-case field name, e.g. `max_connections` -> `max-connections`)
//! - **`#[short = 'c']`** - Sets a short flag (single character) for the option
//! - **`#[format = "VALUE"]`** - Customizes the value name in help text (e.g. showing FILE instead of the field name)
//!
//! ### `#[derive(Args)]` Attributes
//!
//! - **`#[format = "CUSTOM_NAME"]`** - Customizes the argument name in help text
//!
//! ## Builder Pattern
//!
//! Shovel provides `AppBuilder` and `CmdBuilder` for a more fluent API when
//! constructing applications. These builders are available when the `std`
//! feature is enabled and provide a convenient way to set application metadata
//! and actions.
//!
//! ### Builder Example
//!
//! ```no_run
//! # #[cfg(feature = "std")] {
//! use shovel::*;
//!
//! fn main() {
//!     AppBuilder::new("mybuilderapp")
//!         .description("A simple app built with builders")
//!         .version("0.1.0")
//!         .authors(["Author Name <author@example.com>"])
//!         .copyright("(c) 2026 Company Name")
//!         .action(Action::subcmds((
//!             CmdBuilder::new("greet")
//!                 .description("Greets someone")
//!                 .action(Action::args(|name: String| println!("Hello, {}!", name)))
//!                 .build(),
//!             CmdBuilder::new("config")
//!                 .description("Manages configuration")
//!                 .action(Action::subcmds((
//!                     CmdBuilder::new("set")
//!                         .description("Sets a config key")
//!                         .category("configuration")
//!                         .action(Action::args(|key: String| {
//!                             println!("Setting config key: {}", key);
//!                         }))
//!                         .build(),
//!                     CmdBuilder::new("get")
//!                         .description("Gets a config key")
//!                         .category("configuration")
//!                         .action(Action::args(|key: String| println!("Getting {}", key)))
//!                         .build(),
//!                 )))
//!                 .build(),
//!         )))
//!         .build()
//!         .run()
//!         .expect("Failed to run application");
//! }
//! # }
//! ```
//!
//! ## Environment-Specific Usage
//!
//! Shovel is designed to work in both standard and no-std environments, providing
//! flexibility for different deployment scenarios.
//!
//! ### Standard Environment (with `std` feature)
//!
//! For typical CLI applications running on systems with standard library support:
//!
//! ```toml
//! [dependencies]
//! shovel = "<version>"  # Includes std and completion features by default
//! ```
//!
//! The `run()` method provides the most convenient interface:
//!
//! ```no_run
//! # #[cfg(feature = "std")] {
//! # use shovel::*;
//! # fn main() {
//! # let app = App { name: "app", description: "desc", version: "1.0", authors: ["author"], copyright: "copy", action: Action::no_args(|| {}) };
//! app.run().expect("Failed to run application");  // Uses shovel::env::args() and stdout automatically
//! # }
//! # }
//! ```
//!
//! The `shovel::env::args()` function provides a convenient wrapper that converts
//! the result of `std::env::args()` into a format suitable for Shovel's `run_*` methods:
//!
//! ```no_run
//! # #[cfg(feature = "std")] {
//! # fn main() {
//! // Collects std::env::args() into an owned Args struct and convert it to string slice references for run_with_args
//! let args: &[&str] = &shovel::env::args().to_strs();
//! # }
//! # }
//! ```
//!
//! ### No-std Environment
//!
//! For embedded systems, WASM, or other resource-constrained environments:
//!
//! ```toml
//! [dependencies]
//! shovel = { version = "<version>", default-features = false }
//! ```
//!
//! Use stack-based allocation with a fixed-size buffer:
//!
//! ```no_run
//! # #![cfg_attr(feature = "generic_const_exprs", feature(generic_const_exprs))]
//! # #![allow(incomplete_features)]
//! # #[cfg(feature = "std")] {
//! # use shovel::*;
//! # fn main() {
//! # let app = App { name: "app", description: "desc", version: "1.0", authors: ["author"], copyright: "copy", action: Action::no_args(|| {}) };
//! let mut writer = shovel::Stdout::new();
//! // 1. run_heapless<W: Writer>
//! #[cfg(feature = "generic_const_exprs")]
//! app.run_heapless::<_>(&mut writer, &["app", "subcommand"]).expect("Failed to run application");
//!
//! // 2. run_heapless<const H: usize, W: Writer>
//! #[cfg(not(feature = "generic_const_exprs"))]
//! app.run_heapless::<1, _>(&mut writer, &["app", "subcommand"]).expect("Failed to run application");
//! # }
//! # }
//! ```
//!
//! Note: API differs based on `generic_const_exprs` feature availability. The generic parameter `<H>` specifies the maximum stack allocation size.
//!
//! ### Custom I/O and Testing
//!
//! For testing or custom I/O handling, use the flexible `run_with_args` method:
//!
//! ```
//! # #[cfg(feature = "std")] {
//! # use shovel::*;
//! # fn main() {
//! # let app = App { name: "app", description: "desc", version: "1.0", authors: ["author"], copyright: "copy", action: Action::no_args(|| {}) };
//! let mut output = std::io::Cursor::new(Vec::new());
//! let mut writer = IoWriteAdapter::new(&mut output);
//! app.run_with_args(&mut writer, &["app", "--help"]).expect("Failed to run application");
//!
//! // Access the captured output
//! let help_text = String::from_utf8(output.into_inner()).unwrap();
//! assert!(help_text.contains("app"));
//! # }
//! # }
//! ```
//!
//! ## Shell Completion
//!
//! Shovel provides built-in shell completion support for Bash, Zsh, and Fish shells.
//! This feature is enabled by default when using the `std` feature.
//!
//! ### Enabling Completion
//!
//! For custom feature configurations, explicitly enable completion:
//!
//! ```toml
//! [dependencies]
//! shovel = { version = "<version>", features = ["std", "completion"] }
//! ```
//!
//! ### Generating Completion Scripts
//!
//! Shovel automatically adds a `--completion` option to your application. Users can
//! generate completion scripts for their preferred shell:
//!
//! ```bash
//! # For Bash
//! myapp --completion bash > ~/.bash_completion.d/myapp
//! source ~/.bash_completion.d/myapp
//!
//! # For Zsh
//! myapp --completion zsh > ~/.zsh/completions/_myapp
//! # Add ~/.zsh/completions to your fpath in ~/.zshrc
//!
//! # For Fish
//! myapp --completion fish > ~/.config/fish/completions/myapp.fish
//! ```
//!
//! ### Completion Features
//!
//! The generated completion scripts provide:
//! - Command and subcommand completion
//! - Option and flag completion (including short forms)
//! - Context-aware suggestions based on command hierarchy
//! - Help text integration for better user experience
//!
//! Completion works automatically for all commands, subcommands, and options
//! defined in your application structure.
//!
//! ## Scalability Features
//!
//! Shovel supports applications with varying numbers of subcommands through
//! feature-gated compilation limits. By default, applications can have up to
//! 25 subcommands at any single level. For larger applications, enable the
//! appropriate feature:
//!
//! ```toml
//! [dependencies]
//! # Default: up to 25 subcommands
//! shovel = "<version>"
//!
//! # For applications with up to 50 subcommands
//! shovel = { version = "<version>", features = ["subcmds-50"] }
//!
//! # For applications with up to 75 subcommands
//! shovel = { version = "<version>", features = ["subcmds-75"] }
//!
//! # For applications with up to 100 subcommands
//! shovel = { version = "<version>", features = ["subcmds-100"] }
//! ```
//!
//! ### Feature Behavior
//!
//! - **Additive**: Multiple features can be enabled simultaneously
//! - **Highest wins**: When multiple features are enabled, the highest limit is used
//! - **No duplication**: Only one implementation is generated regardless of features enabled
//! - **Compile-time**: All limits are enforced at compile time for zero runtime overhead
//!
//! ### Large Application Design
//!
//! For applications with many commands, consider using hierarchical command structures
//! instead of flat command lists. This provides better user experience and keeps
//! individual command groups manageable:
//!
//! ```no_run
//! # #[cfg(feature = "std")] {
//! # use shovel::*;
//! # fn main() {
//! App {
//!     name: "large-app",
//!     description: "A large application with organized commands",
//!     version: "0.1.0",
//!     authors: ["Author Name <author@example.com>"],
//!     copyright: "(c) 2026 Company Name",
//!     action: Action::subcmds((
//!         // Group 1: File operations (5-10 commands)
//!         Cmd {
//!             name: "file",
//!             description: "File operations",
//!             category: None,
//!             action: Action::subcmds((/* file subcommands */)),
//!         },
//!         // Group 2: Network operations (5-10 commands)
//!         Cmd {
//!             name: "network",
//!             description: "Network operations",
//!             category: None,
//!             action: Action::subcmds((/* network subcommands */)),
//!         },
//!         // Group 3: Database operations (5-10 commands)
//!         Cmd {
//!             name: "database",
//!             description: "Database operations",
//!             category: None,
//!             action: Action::subcmds((/* database subcommands */)),
//!         },
//!     )),
//! }
//! .run()
//! .expect("Failed to run application");
//! # }
//! # }
//! ```
//!
//! This approach scales better than having 30+ commands at the top level.
//!
//!
//! [`App`]: crate::App
//! [`Cmd`]: crate::Cmd
//! [`Subcmds`]: crate::Subcmds
//! [`Args`]: crate::Args
//! [`Opts`]: crate::Opts
//!
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(feature = "generic_const_exprs", feature(generic_const_exprs))]
#![cfg_attr(feature = "generic_const_exprs", allow(incomplete_features))]
#![forbid(warnings, clippy::all)]
#![deny(missing_docs, clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::needless_pass_by_value)]

pub mod action;
pub mod app;
pub mod args;
#[cfg(feature = "std")]
pub mod builder;
pub mod cmd;
#[cfg(feature = "completion")]
pub mod completion;
pub mod context;
#[cfg(feature = "std")]
pub mod env;
pub mod error;
pub mod help;
pub mod opts;
pub mod pool;
pub mod suggestion;
pub mod writer;

/// Exit code returned when a command completes successfully.
///
/// This includes explicit help, version, and completion output.
pub const EXIT_CODE_SUCCESS: i32 = 0;

/// Exit code returned for invalid command-line input or environment fallback.
///
/// The diagnostic has already been written to the configured writer when this
/// code is returned.
pub const EXIT_CODE_USAGE: i32 = 2;

pub use action::{Action, ActionResult, ActionReturn, AppAction};
pub use app::App;
pub use args::{ArgGroup, Args, RawArgs};
pub use cmd::{Cmd, Subcmds};
pub use context::Context;
pub use error::ParseError;
pub use help::{CmdHelp, CmdPath, Help, SubPath, Version};
pub use opts::{Opt, Opts, RawOpts, RawOptsIter};
pub use pool::Pool;
pub use suggestion::Suggestion;
pub use writer::Writer;

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
pub use writer::Stdout;

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
pub use opts::ConfigOpts;

#[cfg_attr(docsrs, doc(cfg(feature = "completion")))]
#[cfg(feature = "completion")]
pub use completion::Shell;

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
pub use writer::IoWriteAdapter;

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
pub use builder::{AppBuilder, CmdBuilder};

#[cfg(feature = "completion")]
pub(crate) use completion::write_completion;
#[cfg(feature = "completion")]
pub(crate) use opts::CompletionOpt;

pub(crate) use help::ActionHelp;
pub(crate) use opts::{BUILTIN_OPTS, HelpOpt, VersionOpt};
pub(crate) use pool::ArrayPool;

pub use shovel_macros::Args;
pub use shovel_macros::Opts;

#[cfg(feature = "std")]
extern crate std;
