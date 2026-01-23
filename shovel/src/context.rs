//! Runtime context for command execution.
//!
//! This module provides the [`Context`] type, which encapsulates all parsed command-line
//! arguments and options for a specific command execution. The context is passed to
//! action handlers, providing type-safe access to all command parameters.
//!
//! # Core Type
//!
//! ## [`Context<O, A, G>`] - Execution Context
//!
//! The main context type that holds parsed command-line data:
//! - **`O: Opts`** - Command-specific options (e.g., `--verbose`, `--output`)
//! - **`A: Args`** - Positional arguments (e.g., file names, URLs)
//! - **`G: Opts`** - Global options inherited from parent commands
//!
//! # Context Fields
//!
//! ## `opts: O` - Command Options
//! Options specific to the current command, parsed according to the command's
//! option specification.
//!
//! ## `args: A` - Positional Arguments
//! Required positional arguments for the command, parsed and validated
//! according to the argument specification.
//!
//! ## `global_opts: G` - Global Options
//! Options that are available to all commands in the application hierarchy,
//! typically defined at the application level.
//!
//! # Usage Patterns
//!
//! ## Basic Context Usage
//! ```
//! # use shovel::*;
//! # #[derive(Opts)] struct MyOpts { verbose: bool }
//! # #[derive(Args)] struct MyArgs { file: String }
//! let action: Action<MyOpts, MyArgs, (), ()> = Action::handler(|ctx: Context<MyOpts, MyArgs, ()>| {
//!     if ctx.opts.verbose {
//!         println!("Processing file: {}", ctx.args.file);
//!     }
//! });
//! ```
//!
//! ## Destructuring Context
//! ```
//! # use shovel::*;
//! # #[derive(Opts)] struct MyOpts { verbose: bool }
//! # #[derive(Args)] struct MyArgs { file: String }
//! let action: Action<MyOpts, MyArgs, (), ()> = Action::handler(|ctx: Context<MyOpts, MyArgs, ()>| {
//!     let Context { opts, args, global_opts } = ctx;
//!     if opts.verbose {
//!         println!("Processing: {}", args.file);
//!     }
//! });
//! ```
//!
//! ## With Global Options
//! ```
//! # use shovel::*;
//! # #[derive(Opts)] struct GlobalOpts { debug: bool }
//! # #[derive(Opts)] struct CmdOpts { force: bool }
//! # #[derive(Args)] struct CmdArgs { target: String }
//! let action: Action<CmdOpts, CmdArgs, GlobalOpts, ()> = Action::handler(|ctx: Context<CmdOpts, CmdArgs, GlobalOpts>| {
//!     if ctx.global_opts.debug {
//!         println!("Debug: Processing {} (force: {})", ctx.args.target, ctx.opts.force);
//!     }
//! });
//! ```
//!
//! # Type Safety
//!
//! The context system provides compile-time type safety:
//! - **Option Types**: Only defined options are accessible
//! - **Argument Types**: Arguments are parsed to their declared types
//! - **Global Consistency**: Global options are consistent across all commands
//! - **Validation**: Invalid combinations are caught at compile time
//!
//! # Parsing Process
//!
//! Context creation involves several steps:
//! 1. **Option Parsing**: Command-line flags and options are parsed according to `O::specs()`
//! 2. **Argument Parsing**: Remaining arguments are parsed according to `A::specs()`
//! 3. **Validation**: All parsed values are validated for type and constraint compliance
//! 4. **Context Creation**: A new `Context` instance is created with the parsed data
//!
//! # Error Handling
//!
//! Context parsing can fail with [`ParseError`] in several cases:
//! - Unknown options provided
//! - Invalid option values
//! - Incorrect number of arguments
//! - Type conversion failures
//! - Constraint violations
//!
//! # Performance Characteristics
//!
//! - **Zero Allocation**: Context creation uses stack-based parsing
//! - **Type Erasure**: No runtime type information overhead
//! - **Compile-time Optimization**: Parser selection resolved at compile time
//! - **Minimal Copying**: Arguments are moved into the context structure
//!
//! # Examples
//!
//! ## File Processing Command
//! ```
//! # use shovel::*;
//! #[derive(Opts)]
//! struct ProcessOpts {
//!     /// Enable verbose output
//!     verbose: bool,
//!     /// Output format
//!     format: Option<String>,
//! }
//!
//! #[derive(Args)]
//! struct ProcessArgs {
//!     /// Input file path
//!     input: String,
//!     /// Output file path
//!     output: String,
//! }
//!
//! let action: Action<ProcessOpts, ProcessArgs, (), ()> = Action::handler(|ctx: Context<ProcessOpts, ProcessArgs, ()>| {
//!     let format = ctx.opts.format.unwrap_or_else(|| "json".to_string());
//!
//!     if ctx.opts.verbose {
//!         println!("Processing {} -> {} (format: {})",
//!                  ctx.args.input, ctx.args.output, format);
//!     }
//!
//!     // Process the files...
//! });
//! ```
//!
//! ## Command with Global Debug Option
//! ```
//! # use shovel::*;
//! #[derive(Opts)]
//! struct GlobalOpts {
//!     /// Enable debug mode
//!     debug: bool,
//! }
//!
//! #[derive(Opts)]
//! struct DeployOpts {
//!     /// Deployment environment
//!     env: String,
//! }
//!
//! #[derive(Args)]
//! struct DeployArgs {
//!     /// Application name
//!     app: String,
//! }
//!
//! let action: Action<DeployOpts, DeployArgs, GlobalOpts, ()> = Action::handler(|ctx: Context<DeployOpts, DeployArgs, GlobalOpts>| {
//!     if ctx.global_opts.debug {
//!         println!("Debug: Deploying {} to {}", ctx.args.app, ctx.opts.env);
//!     }
//!
//!     // Perform deployment...
//! });
//! ```
//!
//! See also: [`Action`](crate::Action), [`Opts`], [`Args`], [`ParseError`]

use crate::Opts;
use crate::ParseError;
use crate::{Args, RawArgs};

/// `Context` encapsulates the parsed command-line arguments for a `shovel` [`Cmd`].
///
/// It provides access to:
/// - Command-specific options (`O`)
/// - Positional arguments (`A`)
/// - Global options (`G`)
///
/// This struct is passed to each action handler, allowing easy access to all parsed arguments.
///
/// [`Cmd`]: crate::Cmd
#[derive(Debug)]
pub struct Context<O: Opts, A: Args, G: Opts> {
    /// Command-specific optional arguments
    pub opts: O,
    /// Positional arguments
    pub args: A,
    /// Global optional arguments
    pub global_opts: G,
}

impl<O: Opts, A: Args, G: Opts> Context<O, A, G> {
    pub(crate) fn parse<'a>(
        raw_args: &'a RawArgs<'a>,
        global_opts: G,
    ) -> Result<Self, ParseError<'a>> {
        let (opts, size) = O::parse(&raw_args.opts(O::specs()))?;
        let args = A::parse(raw_args.args(size))?;
        Ok(Self {
            opts,
            args,
            global_opts,
        })
    }
}
