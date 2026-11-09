//! # Shovel Macros
//!
//! This crate provides macros for the [Shovel](https://crates.io/crates/shovel) command-line argument parsing library.
//! These macros automatically generate implementations of the `Opts`, `Args`, and `Subcmds` traits for your data structures.
//!
//! ## Available Macros
//!
//! - **`#[derive(Opts)]`** - Automatically implements the `Opts` trait for parsing optional arguments (flags and named arguments)
//! - **`#[derive(Args)]`** - Automatically implements the `Args` trait for parsing positional arguments
//! - **`impl_subcmds!`** - Macro for implementing the `Subcmds` trait for tuples (internal use only)
//!
//! ## Quick Start
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! shovel = "0.1"
//! ```
//!
//! ## Examples
//!
//! ### Deriving `Opts` for Optional Arguments
//!
//! ```rust,ignore
//! use shovel::Opts;
//!
//! #[derive(Opts)]
//! struct ServerOpts {
//!     /// The port to listen on
//!     #[short = 'p']
//!     port: u16,
//!
//!     /// Enable verbose logging
//!     verbose: bool,
//!
//!     /// Configuration file path
//!     #[long = "config"]
//!     #[format = "CONFIG"]
//!     config: Option<String>,
//! }
//! ```
//!
//! This generates code equivalent to manually implementing:
//! ```rust,ignore
//! use shovel::Opts;
//!
//! impl Opts for ServerOpts {
//!     fn parse(opts: &RawOpts) -> Result<(Self, usize), ParseError> {
//!         // Generated parsing logic
//!     }
//!
//!     fn specs() -> &'static [Opt] {
//!         // Generated option specifications
//!     }
//! }
//! ```
//!
//! ### Deriving `Args` for Positional Arguments
//!
//! ```rust,ignore
//! use shovel::Args;
//!
//! #[derive(Args)]
//! struct FileArgs {
//!     /// Input file path
//!     input: String,
//!
//!     /// Output file path
//!     output: String,
//! }
//! ```
//!
//! ### Complete Application Example
//!
//! ```rust,ignore
//! use shovel::*;
//!
//! #[derive(Opts)]
//! struct CopyOpts {
//!     /// Preserve file permissions
//!     #[short = 'p']
//!     preserve: bool,
//!
//!     /// Enable verbose output
//!     #[short = 'v']
//!     verbose: bool,
//! }
//!
//! #[derive(Args)]
//! struct CopyArgs {
//!     source: String,
//!     destination: String,
//! }
//!
//! fn main() {
//!     App {
//!         name: "copy",
//!         description: "Copy files with options",
//!         version: "0.1.0",
//!         authors: ["Author Name <author@example.com>"],
//!         copyright: "(c) 2026 Company Name",
//!         action: Action::opts_args(|opts: CopyOpts, args: CopyArgs| {
//!             if opts.verbose {
//!                 println!("Copying {} to {}", args.source, args.destination);
//!             }
//!             // Copy logic here
//!         }),
//!     }
//!     .run()
//!     .expect("Failed to run application");
//! }
//! ```
//!
//! ## Attribute Reference
//!
//! ### `#[derive(Opts)]` Attributes
//!
//! - **`#[long = "custom-name"]`** - Sets the long option name (defaults to kebab-case field name, e.g. `max_connections` -> `max-connections`)
//! - **`#[short = 'c']`** - Sets a short flag (single character) for the option
//! - **`#[format = "VALUE"]`** - Customizes the value name in help text (e.g. showing FILE instead of the field name)
//! - **`#[env = "NAME"]`** - Uses `NAME` as a fallback when the option is absent from the command line.
//!   Names must be non-empty ASCII without `=`, NUL, or surrounding whitespace.
//!   Values are parsed like command-line values; lists are comma-separated and
//!   booleans accept `true`/`false`. CLI values take precedence, required fields
//!   are checked after fallback, and the first matching environment pair wins.
//!   Non-Unicode process environment entries are ignored.
//!
//! ### `#[derive(Args)]` Attributes
//!
//! - **`#[format = "CUSTOM_NAME"]`** - Customizes the argument name in help text
//!
//! ## Field Type Support
//!
//! ### `#[derive(Opts)]` supports any type that implements `FromStr`:
//!
//! - **Primitives**: `bool`, `i32`, `u64`, `f64`, `char`, etc.
//! - **Strings**: `String`, `&str` (in some contexts)
//! - **Collections**: `Vec<T>` where `T: FromStr`
//! - **Options**: `Option<T>` where `T: FromStr`
//! - **Network types**: `IpAddr`, `SocketAddr`, etc.
//! - **Path types**: `PathBuf`
//! - **Custom types**: Any type implementing `FromStr`
//!
//! ### `#[derive(Args)]` supports any type that implements `FromStr`:
//!
//! - **Primitives**: `i32`, `u64`, `f64`, `char`, etc.
//! - **Strings**: `String`, `&str` (in some contexts)
//! - **Collections**: `Vec<T>` where `T: FromStr` (consumes remaining arguments)
//! - **Network types**: `IpAddr`, `SocketAddr`, etc.
//! - **Path types**: `PathBuf`
//! - **Custom types**: Any type implementing `FromStr`
//!
//! Note: `#[derive(Args)]` does NOT support `Option<T>` types. Use `#[derive(Opts)]` for optional arguments.
//!
//! ## Error Handling
//!
//! The generated code automatically handles parsing errors and provides descriptive error messages:
//!
//! - Invalid option values
//! - Missing required options/arguments
//! - Unknown options
//! - Argument count mismatches
//!
//! ## Performance
//!
//! The derive macros generate efficient parsing code with:
//!
//! - Zero runtime overhead compared to manual implementations
//! - Compile-time validation of field types
//! - Minimal memory allocations during parsing
//! - Stack-based parsing for better performance

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use proc_macro::TokenStream;

mod args;
mod attrs;
mod field;
mod opts;
mod subcmds;

/// Derives the `Opts` trait for parsing optional command-line arguments.
///
/// This macro automatically generates an implementation of the `Opts` trait for structs,
/// enabling them to parse optional command-line arguments like `--verbose`, `--port 8080`, etc.
///
/// # Supported Field Types
///
/// Any type that implements `FromStr` can be used as a field type:
/// - Primitives: `bool`, `i32`, `u64`, `f64`, `char`
/// - Strings: `String`
/// - Collections: `Vec<T>` where `T: FromStr`
/// - Options: `Option<T>` where `T: FromStr` (makes the option optional)
/// - Network types: `IpAddr`, `SocketAddr`
/// - Path types: `PathBuf`
///
/// # Boolean Fields
///
/// Boolean fields are treated as flags (no value required):
/// ```rust,ignore
/// use shovel::Opts;
///
/// #[derive(Opts)]
/// struct MyOpts {
///     verbose: bool,  // Becomes --verbose flag
/// }
/// ```
///
/// # Optional Fields
///
/// Fields wrapped in `Option<T>` become optional:
/// ```rust,ignore
/// use shovel::Opts;
///
/// #[derive(Opts)]
/// struct MyOpts {
///     port: u16,              // Required: --port <VALUE>
///     config: Option<String>, // Optional: --config <VALUE>
/// }
/// ```
///
/// # Attributes
///
/// - `#[long = "custom-name"]` - Sets the long option name (defaults to kebab-case field name)
/// - `#[short = 'c']` - Sets a short flag for the option (e.g., `-p` for `--port`)
/// - `#[format = "VALUE"]` - Customizes the value name in help text (e.g. showing FILE instead of the field name)
/// - `#[env = "NAME"]` - Uses `NAME` as a fallback when the option is absent
///   from the command line. Names must be non-empty ASCII without `=`, NUL, or
///   surrounding whitespace. Values are parsed like command-line values; lists
///   are comma-separated and booleans accept `true`/`false`. CLI values take
///   precedence, required fields are checked after fallback, and the first
///   matching environment pair wins. Non-Unicode process environment entries
///   are ignored.
///
/// # Example
///
/// ```rust,ignore
/// use std::path::PathBuf;
///
/// use shovel::Opts;
///
/// #[derive(Opts)]
/// struct ServerOpts {
///     /// Listen port
///     #[short = 'p']
///     port: u16,
///
///     /// Verbose mode
///     #[short = 'v']
///     verbose: bool,
///
///     /// Configuration file path
///     #[long = "config"]
///     #[format = "FILE"]
///     config_file: Option<PathBuf>,
/// }
/// ```
///
/// This generates parsing logic that accepts:
/// - `--port 8080` or `-p 8080`
/// - `--verbose` or `-v`
/// - `--config /path/to/config` (optional)
///
/// # Enum Example
///
/// ```rust,ignore
/// use std::fmt;
/// use std::str::FromStr;
///
/// use shovel::Opts;
///
/// #[derive(Debug)]
/// pub enum LogLevel {
///     Error,
///     Warn,
///     Info,
///     Debug,
///     Trace,
/// }
///
/// impl FromStr for LogLevel {
///     type Err = String;
///
///     fn from_str(s: &str) -> Result<Self, Self::Err> {
///         match s.to_lowercase().as_str() {
///             "error" => Ok(LogLevel::Error),
///             "warn" => Ok(LogLevel::Warn),
///             "info" => Ok(LogLevel::Info),
///             "debug" => Ok(LogLevel::Debug),
///             "trace" => Ok(LogLevel::Trace),
///             _ => Err(format!("Invalid log level: {}", s)),
///         }
///     }
/// }
///
/// impl fmt::Display for LogLevel {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         match self {
///             LogLevel::Error => write!(f, "error"),
///             LogLevel::Warn => write!(f, "warn"),
///             LogLevel::Info => write!(f, "info"),
///             LogLevel::Debug => write!(f, "debug"),
///             LogLevel::Trace => write!(f, "trace"),
///         }
///     }
/// }
///
/// #[derive(Opts)]
/// struct ServerOpts {
///     /// Logging level
///     #[short = 'L']
///     log_level: LogLevel,
///
///     /// Port to listen on
///     #[short = 'p']
///     port: u16,
/// }
/// ```
#[proc_macro_derive(Opts, attributes(short, long, format, env))]
pub fn opts_derive(input: TokenStream) -> TokenStream {
    let input = match syn::parse(input) {
        Err(e) => return e.to_compile_error().into(),
        Ok(input) => input,
    };
    opts::derive_opts(&input)
}

/// Derives the `Args` trait for parsing positional command-line arguments.
///
/// This macro automatically generates an implementation of the `Args` trait for structs,
/// enabling them to parse positional arguments like `file1.txt file2.txt`.
///
/// # Argument Order
///
/// Arguments are parsed in the order they appear in the struct definition:
/// ```rust,ignore
/// use shovel::Args;
///
/// #[derive(Args)]
/// struct CopyArgs {
///     source: String,      // First argument
///     destination: String, // Second argument
/// }
/// ```
///
/// # Supported Field Types
///
/// Any type that implements `FromStr` can be used:
/// - Primitives: `i32`, `u64`, `f64`, `char`
/// - Strings: `String`
/// - Collections: `Vec<T>` where `T: FromStr` (consumes remaining arguments)
/// - Network types: `IpAddr`, `SocketAddr`
/// - Path types: `PathBuf`
///
/// # Optional Arguments
///
/// Note: `#[derive(Args)]` does NOT support optional arguments with `Option<T>`.
/// Use `#[derive(Opts)]` for optional arguments. All arguments in `Args` are required
/// except for `Vec<T>` fields which consume remaining arguments.
///
/// # Variadic Arguments
///
/// `Vec<T>` fields consume all remaining arguments:
/// ```rust,ignore
/// use shovel::Args;
///
/// #[derive(Args)]
/// struct MultiFileArgs {
///     command: String,     // First argument
///     files: Vec<String>,  // All remaining arguments
/// }
/// ```
///
/// # Multiple Groups
///
/// Enums can be used to define mutually exclusive argument sets. Each variant
/// represents a different argument pattern, and users must match exactly one:
/// ```rust,ignore
/// use shovel::Args;
///
/// #[derive(Args)]
/// pub enum SumArgs {
///     /// Sum two numbers
///     #[format = "<NUM1> <NUM2>"]
///     Two(f64, f64),
///     /// Sum three numbers
///     #[format = "<NUM1> <NUM2> <NUM3>"]
///     Three(f64, f64, f64),
///     /// Sum four numbers
///     #[format = "<NUM1> <NUM2> <NUM3> <NUM4>"]
///     Four(f64, f64, f64, f64),
///     /// Sum a variable number of numbers
///     #[format = "<NUM>..."]
///     Many(Vec<f64>),
/// }
/// ```
///
/// # Attributes
///
/// - `#[format = "CUSTOM_NAME"]` - Customizes the argument name in help text
///
/// # Example
///
/// ```rust,ignore
/// use shovel::Args;
///
/// #[derive(Args)]
/// #[format = "<INPUT> <OUTPUT> ..."]
/// struct ProcessArgs {
///     input: String,
///     output: String,
///     extras: Vec<String>,
/// }
/// ```
///
/// # Enum Example
///
/// ```rust,ignore
/// use std::fmt;
/// use std::path::PathBuf;
/// use std::str::FromStr;
///
/// use shovel::Args;
///
/// #[derive(Debug)]
/// pub enum FileType {
///     Json,
///     Xml,
///     Csv,
///     Txt,
/// }
///
/// impl FromStr for FileType {
///     type Err = String;
///
///     fn from_str(s: &str) -> Result<Self, Self::Err> {
///         match s.to_lowercase().as_str() {
///             "json" => Ok(FileType::Json),
///             "xml" => Ok(FileType::Xml),
///             "csv" => Ok(FileType::Csv),
///             "txt" => Ok(FileType::Txt),
///             _ => Err(format!("Invalid file type: {}", s)),
///         }
///     }
/// }
///
/// impl fmt::Display for FileType {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         match self {
///             FileType::Json => write!(f, "json"),
///             FileType::Xml => write!(f, "xml"),
///             FileType::Csv => write!(f, "csv"),
///             FileType::Txt => write!(f, "txt"),
///         }
///     }
/// }
///
/// #[derive(Args)]
/// struct ConvertArgs {
///     /// Input file path
///     input: PathBuf,
///
///     /// Output file path
///     output: PathBuf,
///
///     /// Output file type
///     #[format = "TYPE"]
///     output_type: FileType,
/// }
/// ```
#[proc_macro_derive(Args, attributes(format))]
pub fn args_derive(input: TokenStream) -> TokenStream {
    let input = match syn::parse(input) {
        Err(e) => return e.to_compile_error().into(),
        Ok(input) => input,
    };
    args::derive_args(&input)
}

/// Implements the `Subcmds` trait for tuples of commands.
///
/// This is an internal macro used by the `shovel` crate to generate `Subcmds` implementations
/// for tuples of different sizes. It enables commands to have subcommands organized as tuples.
///
/// # Internal Use Only
///
/// **Note**: This macro is intended for internal use within the `shovel` crate only.
/// End users should not call this macro directly. Instead, use the high-level APIs
/// provided by the `shovel` crate.
///
/// # Purpose
///
/// This macro generates implementations that allow tuples of `Cmd` structs to be used
/// as subcommands. For example, it enables patterns like:
///
/// ```rust,ignore
/// Action::subcmds((
///     Cmd { name: "start", ... },
///     Cmd { name: "stop", ... },
///     Cmd { name: "status", ... },
/// ))
/// ```
///
/// # Generated Code
///
/// The macro generates:
/// - Parsing logic to match command names against tuple elements
/// - Help text generation for all subcommands
/// - Error handling for unknown subcommands
/// - Command suggestion logic for typos
///
/// # Tuple Size Limits
///
/// The macro supports tuples up to a certain size limit (typically 25 elements),
/// which should be sufficient for most command-line applications.
///
/// # Example Usage (Internal)
///
/// ```rust,ignore
/// // This is called internally by the shovel crate:
/// impl_subcmds!(number=25);
/// ```
#[proc_macro]
pub fn impl_subcmds(tokens: TokenStream) -> TokenStream {
    subcmds::impl_subcmds(tokens)
}
