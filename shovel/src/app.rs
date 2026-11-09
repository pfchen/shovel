//! Application root structure and execution methods.
//!
//! This module provides the [`App`] struct, which serves as the root of a Shovel CLI application.
//! The `App` contains metadata about the application (name, version, authors, etc.) and the main
//! action that defines the application's behavior.
//!
//! # Core Type
//!
//! ## [`App<'a, const N: usize, O, A, C>`] - Application Root
//!
//! The main application structure with compile-time configuration:
//! - **`'a`** - Lifetime for string references (name, description, etc.)
//! - **`N`** - Number of authors in the authors array (compile-time constant)
//! - **`O: Opts`** - Global options available to all commands
//! - **`A: Args`** - Arguments for the root application command
//! - **`C: Subcmds<O>`** - Subcommands available at the root level
//!
//! # Execution Methods
//!
//! The `App` provides three execution methods with different performance characteristics:
//!
//! ## 1. `run()` - Standard Execution (std feature)
//! Convenient method for typical CLI applications:
//! ```no_run
//! # #[cfg(feature = "std")] {
//! # use shovel::*;
//! # fn main() {
//! App {
//!     name: "myapp",
//!     description: "A simple app",
//!     version: "0.1.0",
//!     authors: ["Author Name <author@example.com>"],
//!     copyright: "(c) 2026 Company Name",
//!     action: Action::no_args(|| println!("Hello!")),
//! }
//! .run()
//! .expect("Failed to run application");
//! # }
//! # }
//! ```
//!
//! ## 2. `run_with_args()` - Custom I/O
//! For testing or custom output handling:
//! ```
//! # #[cfg(feature = "std")] {
//! # use shovel::*;
//! # fn main() {
//! # let app = App { name: "app", description: "desc", version: "1.0", authors: ["author"], copyright: "copy", action: Action::no_args(|| {}) };
//! let mut output = std::io::Cursor::new(Vec::new());
//! let mut writer = IoWriteAdapter::new(&mut output);
//! app.run_with_args(&mut writer, &["app", "--help"]).expect("Failed to run application");
//! # }
//! # }
//! ```
//!
//! ## 3. `run_heapless()` - Zero Allocation
//! High-performance execution with compile-time memory allocation.
//! Note: This method behavior differs based on the `generic_const_exprs` feature:
//! ```no_run
//! # #![cfg_attr(feature = "generic_const_exprs", feature(generic_const_exprs))]
//! # #![allow(incomplete_features)]
//! # #[cfg(feature = "std")] {
//! # use shovel::*;
//! # fn main() {
//! # let app = App { name: "app", description: "desc", version: "1.0", authors: ["author"], copyright: "copy", action: Action::no_args(|| {}) };
//! let mut writer = Stdout::new();
//! // With `generic_const_exprs` (auto-inference of command count):
//! #[cfg(feature = "generic_const_exprs")]
//! app.run_heapless::<_>(&mut writer, &["app", "command"]).expect("Failed to run application");
//! // Without `generic_const_exprs` (explicit command count required):
//! #[cfg(not(feature = "generic_const_exprs"))]
//! app.run_heapless::<1, _>(&mut writer, &["app", "command"]).expect("Failed to run application");
//! # }
//! # }
//! ```
//!
//! # Performance Characteristics
//!
//! - **`run()`**: Small heap allocation (for help storage + args collection), convenient API
//! - **`run_with_args()`**: Small heap allocation (for help storage), custom I/O support
//! - **`run_heapless()`**: Zero heap allocation, maximum performance. Note: API differs based on `generic_const_exprs` feature availability.
//!
//! # Application Structure
//!
//! Applications are structured as hierarchical trees:
//! ```text
//! App (root)
//! ├── Global Options
//! ├── Root Action/Arguments
//! └── Subcommands
//!     ├── Command A
//!     │   ├── Options
//!     │   ├── Arguments
//!     │   └── Action/Subcommands
//!     └── Command B
//!         └── ...
//! ```
//!
//! # Examples
//!
//! ## Simple Application
//! ```no_run
//! # #[cfg(feature = "std")] {
//! # use shovel::*;
//! # fn main() {
//! App {
//!     name: "greet",
//!     description: "A greeting application",
//!     version: "0.1.0",
//!     authors: ["Author Name <author@example.com>"],
//!     copyright: "(c) 2026 Company Name",
//!     action: Action::args(|name: String| {
//!         println!("Hello, {}!", name);
//!     }),
//! }
//! .run()
//! .expect("Failed to run application");
//! # }
//! # }
//! ```
//!
//! ## Application with Subcommands
//! ```no_run
//! # #[cfg(feature = "std")] {
//! # use shovel::*;
//! # fn main() {
//! App {
//!     name: "tool",
//!     description: "A multi-command tool",
//!     version: "2.0.0",
//!     authors: ["Author Name <author@example.com>"],
//!     copyright: "(c) 2026 Company Name",
//!     action: AppAction::<()>::subcmds((
//!         Cmd {
//!             name: "build",
//!             description: "Build the project",
//!             category: None,
//!             action: Action::no_args(|| println!("Building...")),
//!         },
//!         Cmd {
//!             name: "test",
//!             description: "Run tests",
//!             category: None,
//!             action: Action::no_args(|| println!("Testing...")),
//!         },
//!     )),
//! }
//! .run()
//! .expect("Failed to run application");
//! # }
//! # }
//! ```
//!
//! See also: [`Action`], [`Cmd`](crate::Cmd), [`Opts`], [`Args`]

use core::fmt;

use crate::BUILTIN_OPTS;
#[cfg(feature = "completion")]
use crate::CompletionOpt;
use crate::ParseError;
use crate::Subcmds;
use crate::Writer;
use crate::action::{Action, ActionResult};
use crate::{Args, RawArgs};
use crate::{ArrayPool, Pool};
use crate::{CmdHelp, Version};
use crate::{EXIT_CODE_SUCCESS, EXIT_CODE_USAGE, HelpOpt, Opts, VersionOpt};

/// `App` represents the root of a command-line application.
///
/// This struct is the entry point for defining and running a CLI application.
/// It encapsulates all the necessary information and behavior for the application.
///
/// # Type Parameters
///
/// - `'a`: Lifetime of string references used in the struct.
/// - `N`: Const generic parameter for the number of authors.
/// - `O`: Type implementing the `Opts` trait for application options.
/// - `A`: Type implementing the `Args` trait for application arguments.
/// - `C`: Type implementing the `Subcmds<O>` trait for subcommands.
///
/// # Fields
///
/// - `name`: The name of the application.
/// - `description`: A brief description of what the application does.
/// - `version`: The version string of the application.
/// - `authors`: An array of strings representing the authors of the application.
/// - `copyright`: The copyright notice for the application.
/// - `action`: The main action of the application, defined using the `Action` type.
///
/// # Example
///
/// ```edition2021
/// # #[cfg(feature = "std")] {
/// use shovel::*;
///
/// # fn main() {
/// App {
///     name: "myapp",
///     description: "A simple application",
///     version: "0.1.0",
///     authors: ["Author Name <author@example.com>"],
///     copyright: "(c) 2026 Company Name",
///     action: Action::no_args(|| {
///         println!("Hello, world!");
///     }),
/// }
/// .run()
/// .expect("Failed to run application");
/// # }
/// # }
/// ```
///
/// This example creates a simple "Hello, world!" application using the `App` struct.
///
/// # Usage Guidelines & Performance Characteristics
///
/// Shovel provides three execution methods with different performance and ergonomic trade-offs:
///
/// ## High Performance: `run_heapless()` (Recommended)
/// **Zero heap allocation** - Uses compile-time sized stack arrays
/// ```no_run
/// # #![cfg_attr(feature = "generic_const_exprs", feature(generic_const_exprs))]
/// # #![allow(incomplete_features)]
/// # #[cfg(feature = "std")] {
/// # use shovel::*;
/// # let app = App { name: "app", description: "desc", version: "1.0", authors: ["author"], copyright: "copy", action: Action::no_args(|| {}) };
/// # let mut writer = shovel::Stdout::new();
/// #[cfg(feature = "generic_const_exprs")]
/// app.run_heapless::<_>(&mut writer, &["app"]).expect("Failed to run the application");
/// #[cfg(not(feature = "generic_const_exprs"))]
/// app.run_heapless::<1, _>(&mut writer, &["app"]).expect("Failed to run the application");
/// # }
/// ```
/// - **Zero heap allocation** (`ArrayPool`)
/// - **Optimal performance** for CLI parsing
/// - **Works in no-std** environments
/// - **Requires const generic** parameter (command count) unless `generic_const_exprs` is enabled
/// - **Use for:** Performance-critical applications, embedded systems, no-std environments
///
/// ## Convenience: `run_with_args()`
/// **Small heap allocation** - Uses dynamically sized vectors
/// ```no_run
/// # #[cfg(feature = "std")] {
/// # use shovel::*;
/// # let app = App { name: "app", description: "desc", version: "1.0", authors: ["author"], copyright: "copy", action: Action::no_args(|| {}) };
/// # let mut writer = shovel::Stdout::new();
/// app.run_with_args(&mut writer, &["app"]).expect("Failed to run the application");
/// # }
/// ```
/// - **Small heap allocation** (`VecPool`: ~`Self::cmd_count()` * 64 bytes)
/// - **Simple API** - no const generics required
/// - **Custom I/O** support for testing
/// - **Use for:** Testing, custom I/O, when ergonomics matter more than performance
///
/// ## Standard: `run()`
/// **Small heap allocation** - Convenience wrapper using stdout
/// ```no_run
/// # #[cfg(feature = "std")] {
/// # use shovel::*;
/// # let app = App { name: "app", description: "desc", version: "1.0", authors: ["author"], copyright: "copy", action: Action::no_args(|| {}) };
/// app.run().expect("Failed to run the application");
/// # }
/// ```
/// - **Small heap allocation** (same as `run_with_args`)
/// - **Simplest API** - uses stdout and `std::env::args()`
/// - **Use for:** Simple CLI applications where convenience is prioritized
///
/// # Performance Comparison
///
/// | Method | Stack Usage | Heap Usage | API Complexity | Performance |
/// |--------|-------------|------------|-----------------|-------------|
/// | `run_heapless()` | ~`Self::cmd_count()` * sizeof(CmdHelp) | **0 bytes** | Medium (const generic) | **Optimal** |
/// | `run_with_args()` | Minimal (local vars only) | ~`Self::cmd_count()` * 64 bytes | Low (simple) | Good |
/// | `run()` | Minimal (local vars only) | ~`Self::cmd_count()` * 64 bytes + args | Lowest (none) | Good |
///
/// ## Allocation Behavior Details
///
/// ### `ArrayPool` (Stack Allocation)
/// ```text
/// Memory Layout: [CmdHelp; H] on stack (where H = `Self::cmd_count()`)
/// Allocation: Compile-time, zero runtime cost
/// Deallocation: Automatic (stack unwinding)
/// Cache Performance: Optimal (contiguous memory)
/// ```
///
/// ### `VecPool` (Heap Allocation)
/// ```text
/// Memory Layout: Vec<CmdHelp> on heap
/// Allocation: Runtime allocation (typically 1 allocation for help storage)
/// Deallocation: Runtime free (Drop trait)
/// Cache Performance: Good (may be fragmented)
/// ```
///
/// ## Recommendation
///
/// **For most applications**: Start with `run()` for simplicity, then optimize to
/// `run_heapless()` if performance profiling shows CLI parsing overhead.
///
/// See also: [`Cmd`](crate::Cmd), [`Action`], [`Opts`], [`Args`], [`Subcmds`]
#[derive(Debug)]
pub struct App<
    'a,
    const N: usize,
    O: Opts,
    A: Args,
    C: Subcmds<O>,
    R: crate::ActionReturn = (),
> {
    /// The name of the application.
    pub name: &'a str,
    /// Description of the application.
    pub description: &'a str,
    /// Version of the application.
    pub version: &'a str,
    /// All authors of the application.
    pub authors: [&'a str; N],
    /// Copyright of the application.
    pub copyright: &'a str,
    /// The action of the application.
    pub action: Action<O, A, O, C, R>,
}

impl<
    'a,
    const N: usize,
    O: Opts,
    A: Args,
    C: Subcmds<O>,
    R: crate::ActionReturn,
> App<'a, N, O, A, C, R>
{
    const CMD_COUNT: usize = C::CMD_COUNT + 1;

    /// **🚀 High Performance**: Parse and run with zero heap allocation.
    ///
    /// This method provides **optimal performance** by using compile-time sized stack arrays
    /// instead of heap allocation. It's the **recommended method** for performance-critical
    /// applications and is required for no-std environments.
    ///
    /// ## Performance Characteristics
    /// - **Zero heap allocation** - Uses stack-based memory management
    /// - **Compile-time sizing** - Memory layout determined at compile time
    /// - **Cache-friendly** - Better memory locality than heap allocation
    /// - **No-std compatible** - Works in embedded and constrained environments
    ///
    /// ## Memory Usage
    /// ```text
    /// Stack allocation: ~Self::cmd_count() * sizeof(CmdHelp) bytes
    /// Heap allocation: 0 bytes
    /// ```
    ///
    /// # Type Parameters
    ///
    /// - `W`: The type implementing the [`Writer`] trait for output.
    ///
    /// # Arguments
    ///
    /// - `writer`: A mutable reference to the output writer.
    /// - `args`: A slice of command-line arguments (including the program name).
    ///
    /// # Returns
    ///
    /// Returns the exit code, or `fmt::Error` if writing output fails.
    ///
    /// # Errors
    ///
    /// - Parse failures are written to the writer and return
    ///   [`EXIT_CODE_USAGE`].
    /// - Returns an error only if writing output fails.
    ///
    /// # When to Use
    ///
    /// **Prefer this method when:**
    /// - Performance is critical
    /// - Working in no-std environments
    /// - Memory allocation should be minimized
    /// - Building embedded or real-time applications
    ///
    /// **Consider alternatives when:**
    /// - API ergonomics are more important than performance
    /// - Building simple CLI tools where allocation overhead is negligible
    /// - Working with dynamic command structures (rare)
    ///
    /// ## Feature Differences
    /// - **With `generic_const_exprs` enabled**: The command count is inferred automatically at compile time using `Self::cmd_count()`.
    /// - **Without `generic_const_exprs`**: You must provide the command count `H` explicitly as a const generic parameter.
    ///   This is required for older Rust versions that don't support `generic_const_exprs`.
    ///
    /// # Examples
    ///
    /// With `generic_const_exprs` (command count inferred automatically):
    /// ```
    /// # #![cfg_attr(feature = "generic_const_exprs", feature(generic_const_exprs))]
    /// # #![allow(incomplete_features)]
    /// # use shovel::*;
    /// # fn main() {
    /// App {
    ///     name: "myapp",
    ///     description: "A CLI with two subcommands",
    ///     version: "0.1.0",
    ///     authors: ["Author Name <author@example.com>"],
    ///     copyright: "(c) 2026 Company Name",
    ///     action: Action::subcmds((
    ///         Cmd {
    ///             name: "greet",
    ///             description: "Greet someone",
    ///             category: None,
    ///             action: Action::no_args(|| println!("Hello!")),
    ///         },
    ///         Cmd {
    ///             name: "farewell",
    ///             description: "Say goodbye",
    ///             category: None,
    ///             action: Action::no_args(|| println!("Goodbye!")),
    ///         },
    ///     )),
    /// }
    /// .run_heapless::<_>(&mut Stdout::new(), &["myapp", "greet"])
    /// .expect("Failed to run application");
    /// # }
    /// ```
    ///
    /// Without `generic_const_exprs` (explicit command count required):
    /// ```
    /// # #![cfg_attr(feature = "generic_const_exprs", feature(generic_const_exprs))]
    /// # #![allow(incomplete_features)]
    /// # #[cfg(not(feature = "generic_const_exprs"))] {
    /// # use shovel::*;
    /// # fn main() {
    /// App {
    ///     name: "myapp",
    ///     description: "A CLI with two subcommands",
    ///     version: "0.1.0",
    ///     authors: ["Author Name <author@example.com>"],
    ///     copyright: "(c) 2026 Company Name",
    ///     action: Action::subcmds((
    ///         Cmd {
    ///             name: "greet",
    ///             description: "Greet someone",
    ///             category: None,
    ///             action: Action::no_args(|| println!("Hello!")),
    ///         },
    ///         Cmd {
    ///             name: "farewell",
    ///             description: "Say goodbye",
    ///             category: None,
    ///             action: Action::no_args(|| println!("Goodbye!")),
    ///         },
    ///     )),
    /// }
    /// .run_heapless::<3, _>(&mut Stdout::new(), &["myapp", "greet"])
    /// .expect("Failed to run application");
    /// # }
    /// # }
    /// ```
    ///
    /// These examples create a CLI app with two subcommands and run it with the "greet" subcommand.
    #[cfg(feature = "generic_const_exprs")]
    pub fn run_heapless<W: Writer>(
        self,
        writer: &mut W,
        args: &'a [&'a str],
    ) -> Result<i32, fmt::Error>
    where
        [(); C::CMD_COUNT + 1]: Sized,
    {
        self.run_heapless_impl::<{ C::CMD_COUNT + 1 }, W>(writer, args, &[])
    }

    /// Execute without internal allocation, with explicit environment fallback.
    ///
    /// # Errors
    /// Returns an error if writing output fails.
    #[cfg(feature = "generic_const_exprs")]
    pub fn run_heapless_with_env<W: Writer>(
        self,
        writer: &mut W,
        args: &'a [&'a str],
        env: &'a [(&'a str, &'a str)],
    ) -> Result<i32, fmt::Error>
    where
        [(); C::CMD_COUNT + 1]: Sized,
    {
        self.run_heapless_impl::<{ C::CMD_COUNT + 1 }, W>(writer, args, env)
    }

    /// **🚀 High Performance**: Parse and run with zero heap allocation.
    ///
    /// This method provides **optimal performance** by using compile-time sized stack arrays
    /// instead of heap allocation. It's the **recommended method** for performance-critical
    /// applications and is required for no-std environments.
    ///
    /// ## Performance Characteristics
    /// - **Zero heap allocation** - Uses stack-based memory management
    /// - **Compile-time sizing** - Memory layout determined at compile time
    /// - **Cache-friendly** - Better memory locality than heap allocation
    /// - **No-std compatible** - Works in embedded and constrained environments
    ///
    /// ## Memory Usage
    /// ```text
    /// Stack allocation: ~`Self::cmd_count()` * sizeof(CmdHelp) bytes
    /// Heap allocation: 0 bytes
    /// ```
    ///
    /// # Type Parameters
    ///
    /// - `H`: The exact number of commands in the application. Must equal [`cmd_count()`](Self::cmd_count).
    /// - `W`: The type implementing the [`Writer`] trait for output.
    ///
    /// # Arguments
    ///
    /// - `writer`: A mutable reference to the output writer.
    /// - `args`: A slice of command-line arguments (including the program name).
    ///
    /// # Returns
    ///
    /// Returns the exit code, or `fmt::Error` if writing output fails.
    ///
    /// # Errors
    ///
    /// - Parse failures are written to the writer and return
    ///   [`EXIT_CODE_USAGE`].
    /// - Returns an error only if writing output fails.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if `H` does not equal the command count. This is checked
    /// via `debug_assert!` to ensure memory safety without runtime overhead in release builds.
    ///
    /// # When to Use
    ///
    /// **Prefer this method when:**
    /// - Performance is critical
    /// - Working in no-std environments
    /// - Memory allocation should be minimized
    /// - Building embedded or real-time applications
    ///
    /// **Consider alternatives when:**
    /// - API ergonomics are more important than performance
    /// - Building simple CLI tools where allocation overhead is negligible
    /// - Working with dynamic command structures (rare)
    ///
    /// ## Feature Differences
    /// - **With `generic_const_exprs` enabled**: The command count is inferred automatically at compile time using `Self::cmd_count()`.
    /// - **Without `generic_const_exprs`**: You must provide the command count `H` explicitly as a const generic parameter.
    ///   This is required for older Rust versions that don't support `generic_const_exprs`.
    ///
    /// # Examples
    ///
    /// With `generic_const_exprs` (command count inferred automatically):
    /// ```
    /// # #![cfg_attr(feature = "generic_const_exprs", feature(generic_const_exprs))]
    /// # #![allow(incomplete_features)]
    /// # #[cfg(feature = "generic_const_exprs")] {
    /// # use shovel::*;
    /// # fn main() {
    /// App {
    ///     name: "myapp",
    ///     description: "A CLI with two subcommands",
    ///     version: "0.1.0",
    ///     authors: ["Author Name <author@example.com>"],
    ///     copyright: "(c) 2026 Company Name",
    ///     action: Action::subcmds((
    ///         Cmd {
    ///             name: "greet",
    ///             description: "Greet someone",
    ///             category: None,
    ///             action: Action::no_args(|| println!("Hello!")),
    ///         },
    ///         Cmd {
    ///             name: "farewell",
    ///             description: "Say goodbye",
    ///             category: None,
    ///             action: Action::no_args(|| println!("Goodbye!")),
    ///         },
    ///     )),
    /// }
    /// .run_heapless::<_>(&mut Stdout::new(), &["myapp", "greet"])
    /// .expect("Failed to run application");
    /// # }
    /// # }
    /// ```
    ///
    /// Without `generic_const_exprs` (explicit command count required):
    /// ```
    /// # #![cfg_attr(feature = "generic_const_exprs", feature(generic_const_exprs))]
    /// # #![allow(incomplete_features)]
    /// # use shovel::*;
    /// # fn main() {
    /// App {
    ///     name: "myapp",
    ///     description: "A CLI with two subcommands",
    ///     version: "0.1.0",
    ///     authors: ["Author Name <author@example.com>"],
    ///     copyright: "(c) 2026 Company Name",
    ///     action: Action::subcmds((
    ///         Cmd {
    ///             name: "greet",
    ///             description: "Greet someone",
    ///             category: None,
    ///             action: Action::no_args(|| println!("Hello!")),
    ///         },
    ///         Cmd {
    ///             name: "farewell",
    ///             description: "Say goodbye",
    ///             category: None,
    ///             action: Action::no_args(|| println!("Goodbye!")),
    ///         },
    ///     )),
    /// }
    /// .run_heapless::<3, _>(&mut Stdout::new(), &["myapp", "greet"])
    /// .expect("Failed to run application");
    /// # }
    /// ```
    ///
    /// These examples create a CLI app with two subcommands and run it with the "greet" subcommand.
    ///
    #[cfg(not(feature = "generic_const_exprs"))]
    pub fn run_heapless<const H: usize, W: Writer>(
        self,
        writer: &mut W,
        args: &'a [&'a str],
    ) -> Result<i32, fmt::Error> {
        self.run_heapless_impl::<H, W>(writer, args, &[])
    }

    /// Execute without internal allocation, with explicit environment fallback.
    ///
    /// # Errors
    /// Returns an error if writing output fails.
    #[cfg(not(feature = "generic_const_exprs"))]
    pub fn run_heapless_with_env<const H: usize, W: Writer>(
        self,
        writer: &mut W,
        args: &'a [&'a str],
        env: &'a [(&'a str, &'a str)],
    ) -> Result<i32, fmt::Error> {
        self.run_heapless_impl::<H, W>(writer, args, env)
    }

    fn run_heapless_impl<const H: usize, W: Writer>(
        self,
        writer: &mut W,
        args: &'a [&'a str],
        env: &'a [(&'a str, &'a str)],
    ) -> Result<i32, fmt::Error> {
        debug_assert!(Self::CMD_COUNT == H, "Mismatched command count");
        debug_assert!(!args.is_empty(), "No arguments provided");

        // Create RawArgs
        let raw_args = RawArgs::with_env(&args[1..], env);

        // Creating the helps pool
        let mut helps = ArrayPool::<CmdHelp<'_>, H>::new();

        // Parsing help option
        if HelpOpt::parse(raw_args.opts(&BUILTIN_OPTS[..1])) {
            return writer
                .write_help(self.help(helps.as_pool()).into_help(
                    self.name,
                    &[],
                    None,
                    None,
                ))
                .map(|()| EXIT_CODE_SUCCESS);
        }

        // Parsing version option
        if VersionOpt::parse(raw_args.opts(&BUILTIN_OPTS[1..2])) {
            return writer
                .write_version(self.version())
                .map(|()| EXIT_CODE_SUCCESS);
        }

        #[cfg(feature = "completion")]
        {
            // Parse completion option
            if let Some(completion) =
                CompletionOpt::parse(raw_args.opts(&BUILTIN_OPTS[2..3]))
            {
                match completion {
                    Err(e) => {
                        return writer.write_error(e).map(|()| EXIT_CODE_USAGE);
                    }
                    Ok(shell) => {
                        return writer
                            .write_completion(
                                self.help(helps.as_pool()).into_help(
                                    self.name,
                                    &[],
                                    None,
                                    None,
                                ),
                                shell,
                            )
                            .map(|()| EXIT_CODE_SUCCESS);
                    }
                }
            }
        }

        // Parsing global options
        let (global_opts, start_pos) =
            match O::parse_with_env(&raw_args.opts(O::specs()), env) {
                Err(e) => {
                    return writer.write_error(e).map(|()| EXIT_CODE_USAGE);
                }
                Ok((opts, size)) => {
                    raw_args.consume(size);
                    (opts, size)
                }
            };

        match self.action.run(self.name, &raw_args, global_opts, true) {
            ActionResult::Error(e) => {
                writer.write_error(e).map(|()| EXIT_CODE_USAGE)
            }
            ActionResult::ShowHelp { cmd, pos } => {
                let subpath = if cmd == self.name && pos == start_pos {
                    &[]
                } else {
                    raw_args.cmd_path(start_pos, pos)
                };
                let Some(help) = self.help(helps.as_pool()).find(subpath, cmd)
                else {
                    // This should not happen if the command structure is valid
                    return writer
                        .write_error(ParseError::UnknownOption(cmd))
                        .map(|()| EXIT_CODE_USAGE);
                };

                writer
                    .write_help(help.into_help(
                        self.name,
                        subpath,
                        Some(O::specs()),
                        Some(BUILTIN_OPTS),
                    ))
                    .map(|()| EXIT_CODE_SUCCESS)
            }
            ActionResult::Suggestion(suggestion) => writer
                .write_suggestion(suggestion)
                .map(|()| EXIT_CODE_USAGE),
            ActionResult::Success(code) => Ok(code),
        }
    }

    /// **🔧 Convenience**: Parse and run with custom I/O and small heap allocation.
    ///
    /// This method provides a **simple API** for custom I/O scenarios while using
    /// a small amount of heap allocation for internal memory management. It's ideal
    /// for testing, custom writers, and when API ergonomics are more important than
    /// optimal performance.
    ///
    /// ## Performance Characteristics
    /// - **Small heap allocation** - Uses `VecPool<T>` for dynamic memory management
    /// - **Runtime sizing** - Memory allocated based on actual command count
    /// - **Simple API** - No const generic parameters required
    /// - **Flexible I/O** - Supports custom writers for testing and integration
    ///
    /// ## Memory Usage
    /// ```text
    /// Stack allocation: Minimal (local variables only)
    /// Heap allocation: ~`Self::cmd_count()` * 64 bytes (VecPool for help system)
    /// ```
    /// For typical CLI apps: **Variable heap allocation based on command count**
    ///
    /// # Type Parameters
    ///
    /// - `W`: A type that implements the [`Writer`] trait for output.
    ///
    /// # Arguments
    ///
    /// - `writer`: A mutable reference to the output writer.
    /// - `args`: A slice of command-line arguments (including the program name).
    ///
    /// # Returns
    ///
    /// Returns the exit code, or `fmt::Error` if writing output fails.
    ///
    /// # Errors
    ///
    /// - Parse failures are written to the writer and return
    ///   [`EXIT_CODE_USAGE`].
    /// - Returns an error only if writing output fails.
    ///
    /// # When to Use
    ///
    /// **Prefer this method when:**
    /// - Writing unit tests with custom output capture
    /// - Integrating with custom I/O systems
    /// - Building CLI tools where ergonomics matter more than performance
    /// - The small allocation overhead is acceptable
    ///
    /// **Consider alternatives when:**
    /// - Performance is critical → use [`run_heapless()`](Self::run_heapless)
    /// - Using standard I/O → use [`run()`](Self::run)
    /// - Working in no-std environments → use [`run_heapless()`](Self::run_heapless)
    ///
    /// See also: [`run()`](Self::run), [`run_heapless()`](Self::run_heapless)
    ///
    /// # Examples
    ///
    /// ```edition2021
    /// use shovel::*;
    /// use std::io::Cursor;
    /// use std::io::Write;
    ///
    /// # fn main() {
    /// let cursor = Cursor::new(Vec::<u8>::new());
    /// let mut output = IoWriteAdapter::new(cursor);
    ///
    /// App {
    ///     name: "myapp",
    ///     description: "A simple app",
    ///     version: "0.1.0",
    ///     authors: ["Author Name <author@example.com>"],
    ///     copyright: "(c) 2026 Company Name",
    ///     action: Action::no_args(|| println!("Hello, world!")),
    /// }.run_with_args(
    ///     &mut output,
    ///     &["myapp", "--help"]
    /// )
    /// .expect("Failed to run application");
    ///
    /// let inner_cursor = output.into_inner();
    /// assert!(String::from_utf8_lossy(&inner_cursor.into_inner()).contains("Usage:"));
    /// # }
    /// ```
    ///
    /// This example demonstrates running the app with a custom writer (`Cursor`) and arguments.
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    #[cfg(feature = "std")]
    pub fn run_with_args<W: Writer>(
        self,
        writer: &mut W,
        args: &'a [&'a str],
    ) -> Result<i32, fmt::Error> {
        self.run_with_args_and_env(writer, args, &[])
    }

    /// Execute with explicitly supplied environment fallback values.
    ///
    /// # Errors
    /// Returns an error if writing output fails.
    #[cfg(feature = "std")]
    pub fn run_with_args_and_env<W: Writer>(
        self,
        writer: &mut W,
        args: &'a [&'a str],
        env: &'a [(&'a str, &'a str)],
    ) -> Result<i32, fmt::Error> {
        // Create RawArgs
        let raw_args = RawArgs::with_env(&args[1..], env);

        // Create the helps pool
        let mut helps = crate::pool::VecPool::new(Self::CMD_COUNT);

        // Parse help option
        if HelpOpt::parse(raw_args.opts(&BUILTIN_OPTS[..1])) {
            return writer
                .write_help(self.help(helps.as_pool()).into_help(
                    self.name,
                    &[],
                    None,
                    None,
                ))
                .map(|()| EXIT_CODE_SUCCESS);
        }

        // Parse version option
        if VersionOpt::parse(raw_args.opts(&BUILTIN_OPTS[1..2])) {
            return writer
                .write_version(self.version())
                .map(|()| EXIT_CODE_SUCCESS);
        }

        #[cfg(feature = "completion")]
        {
            // Parse completion option
            if let Some(completion) =
                CompletionOpt::parse(raw_args.opts(&BUILTIN_OPTS[2..3]))
            {
                match completion {
                    Err(e) => {
                        return writer.write_error(e).map(|()| EXIT_CODE_USAGE);
                    }
                    Ok(shell) => {
                        return writer
                            .write_completion(
                                self.help(helps.as_pool()).into_help(
                                    self.name,
                                    &[],
                                    None,
                                    None,
                                ),
                                shell,
                            )
                            .map(|()| EXIT_CODE_SUCCESS);
                    }
                }
            }
        }

        // Parse global options
        let (global_opts, start_pos) =
            match O::parse_with_env(&raw_args.opts(O::specs()), env) {
                Err(e) => {
                    return writer.write_error(e).map(|()| EXIT_CODE_USAGE);
                }
                Ok((opts, size)) => {
                    raw_args.consume(size);
                    (opts, size)
                }
            };

        match self.action.run(self.name, &raw_args, global_opts, true) {
            ActionResult::Error(e) => {
                writer.write_error(e).map(|()| EXIT_CODE_USAGE)
            }
            ActionResult::ShowHelp { cmd, pos } => {
                let subpath = if cmd == self.name && pos == start_pos {
                    &[]
                } else {
                    raw_args.cmd_path(start_pos, pos)
                };
                let Some(help) = self.help(helps.as_pool()).find(subpath, cmd)
                else {
                    // This should not happen if the command structure is valid
                    return writer
                        .write_error(ParseError::UnknownOption(cmd))
                        .map(|()| EXIT_CODE_USAGE);
                };

                writer
                    .write_help(help.into_help(
                        self.name,
                        subpath,
                        Some(O::specs()),
                        Some(BUILTIN_OPTS),
                    ))
                    .map(|()| EXIT_CODE_SUCCESS)
            }
            ActionResult::Suggestion(suggestion) => writer
                .write_suggestion(suggestion)
                .map(|()| EXIT_CODE_USAGE),
            ActionResult::Success(code) => Ok(code),
        }
    }

    /// **✨ Standard**: Parse and run with standard I/O and small heap allocation.
    ///
    /// This is the **simplest method** for typical CLI applications. It's a convenience
    /// wrapper around [`run_with_args_and_env()`](Self::run_with_args_and_env) that uses
    /// standard I/O and automatically collects command-line arguments and process
    /// environment variables.
    ///
    /// ## Performance Characteristics
    /// - **Small heap allocation** - Same as `run_with_args()` (uses `VecPool<T>`)
    /// - **Automatic I/O** - Uses stdout, `std::env::args()`, and process environment variables
    /// - **Simplest API** - No parameters required
    /// - **Production ready** - Suitable for most CLI applications
    ///
    /// ## Memory Usage
    /// ```text
    /// Stack allocation: Minimal (local variables only)
    /// Heap allocation: ~`Self::cmd_count()` * 64 bytes + args collection
    /// ```
    /// For typical CLI apps: **Variable heap allocation based on command count + argument strings**
    ///
    /// # Returns
    ///
    /// Returns the exit code, or `fmt::Error` if writing output fails.
    ///
    /// # Errors
    ///
    /// - Parse failures are written to the writer and return
    ///   [`EXIT_CODE_USAGE`].
    /// - Returns an error only if writing output fails.
    ///
    /// # When to Use
    ///
    /// **Prefer this method when:**
    /// - Building typical CLI applications
    /// - Simplicity is more important than performance
    /// - Using standard I/O (stdout/stderr)
    /// - The small allocation overhead is acceptable
    ///
    /// **Consider alternatives when:**
    /// - Performance is critical → use [`run_heapless()`](Self::run_heapless)
    /// - Need custom I/O → use [`run_with_args()`](Self::run_with_args)
    /// - Working in no-std environments → use [`run_heapless()`](Self::run_heapless)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use shovel::*;
    ///
    /// fn main() {
    ///     let code = App {
    ///         name: "myapp",
    ///         description: "A simple app",
    ///         version: "0.1.0",
    ///         authors: ["Author Name <author@example.com>"],
    ///         copyright: "(c) 2026 Company Name",
    ///         action: Action::no_args(|| println!("Hello, world!")),
    ///     }
    ///     .run()
    ///     .expect("Failed to run application");
    ///     std::process::exit(code);
    /// }
    /// ```
    ///
    /// This example shows how to use the `run` method in a typical main function.
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    #[cfg(feature = "std")]
    pub fn run(self) -> Result<i32, fmt::Error> {
        self.run_with_args_and_env(
            &mut crate::writer::Stdout::new(),
            &crate::env::args().to_strs(),
            &crate::env::vars_os().to_strs(),
        )
    }

    /// Run using process arguments and explicitly supplied environment values.
    ///
    /// # Errors
    /// Returns an error if writing output fails.
    #[cfg(feature = "std")]
    pub fn run_with_env(self, env: &[(&str, &str)]) -> Result<i32, fmt::Error> {
        self.run_with_args_and_env(
            &mut crate::writer::Stdout::new(),
            &crate::env::args().to_strs(),
            env,
        )
    }

    /// Returns the total number of commands available in the application.
    ///
    /// This includes both the main application command and any subcommands defined.
    ///
    /// # Examples
    ///
    /// ```
    /// use shovel::*;
    ///
    /// # fn main() {
    /// let count = App {
    ///     name: "print",
    ///     description: "Print ping/pong",
    ///     version: "0.1.0",
    ///     authors: ["Author Name <author@example.com>"],
    ///     copyright: "(c) 2026 Company Name",
    ///     action: AppAction::<()>::subcmds((
    ///         Cmd {
    ///             name: "ping",
    ///             description: "Print a ping",
    ///             category: None,
    ///             action: Action::no_args(|| {
    ///                 println!("ping");
    ///             }),
    ///         },
    ///         Cmd {
    ///             name: "pong",
    ///             description: "Print a pong",
    ///             category: None,
    ///             action: Action::no_args(|| {
    ///                 println!("pong");
    ///             }),
    ///         },
    ///     )),
    /// }.cmd_count();
    /// assert_eq!(count, 3);
    /// # }
    /// ```
    pub const fn cmd_count(&self) -> usize {
        Self::CMD_COUNT
    }

    fn help<'b>(&'b self, pool: Pool<'b, CmdHelp<'b>>) -> CmdHelp<'b> {
        CmdHelp::new(
            self.name,
            self.description,
            None,
            self.action.help(pool),
            BUILTIN_OPTS,
        )
    }

    fn version(&'_ self) -> Version<'_> {
        Version {
            name: self.name,
            version: self.version,
            authors: &self.authors,
            copyright: self.copyright,
        }
    }
}
