//! Command definitions and hierarchical command structures.
//!
//! This module provides the core abstractions for defining commands and organizing them
//! into hierarchical structures. Commands can have options, arguments, and subcommands,
//! all with compile-time type safety and zero-allocation memory management.
//!
//! # Core Types
//!
//! ## [`Cmd<O, A, G, C>`] - Type-Safe Command Definition
//!
//! Individual command with compile-time verified options and arguments:
//! - **`O: Opts`** - Command-specific options (e.g., `--verbose`, `--output`)
//! - **`A: Args`** - Command arguments (e.g., file names, URLs)
//! - **`G: Opts`** - Global options inherited from parent commands
//! - **`C: Subcmds<G>`** - Subcommands with shared global options
//!
//! ## [`Subcmds<G>`] - Unified Command Collection Interface
//!
//! Trait that enables elegant command composition through:
//! - **Tuple-based syntax**: `(cmd1, cmd2, cmd3)` for natural grouping
//! - **Recursive nesting**: Commands can contain other command collections
//! - **Compile-time counting**: `CMD_COUNT` computed automatically
//! - **Zero-allocation help**: Memory-pool-based help generation
//!
//! # Key Features
//!
//! ## Zero-Allocation Design
//! ```text
//! Command Structure:     Memory Allocation:
//! ┌─ App                ┌─ Stack (compile-time sized)
//! ├─ cmd1               ├─ ArrayPool<CmdHelp, N>
//! ├─ cmd2               ├─ Help metadata (static)
//! └─ cmd3               └─ Zero heap allocation
//! ```
//!
//! ## Compile-Time Command Counting
//! ```rust
//! // Automatically computed at compile time
//! use shovel::{Subcmds, Cmd, Action};
//!
//! assert_eq!(<() as Subcmds<()>>::CMD_COUNT, 0);                    // Empty
//!
//! // Single command count
//! type SingleCmd = Cmd<'static, (), (), (), ()>;
//! assert_eq!(<SingleCmd as Subcmds<()>>::CMD_COUNT, 1);
//!
//! // Tuple command count
//! type TupleCmd = (Cmd<'static, (), (), (), ()>, Cmd<'static, (), (), (), ()>);
//! assert_eq!(<TupleCmd as Subcmds<()>>::CMD_COUNT, 2);
//! ```
//!
//! ## Scalability Features
//! Feature-gated command limits:
//! ```toml
//! [features]
//! default = []                    # Up to 25 commands
//! subcmds-50 = []                # Up to 50 commands
//! subcmds-75 = []                # Up to 75 commands
//! subcmds-100 = []               # Up to 100 commands
//! ```
//!
//! The library automatically generates `Subcmds` implementations for tuples
//! of different sizes, allowing natural command grouping syntax.
//!
//! # Usage Patterns
//!
//! ## Simple Command
//! ```rust
//! use shovel::{Cmd, Action};
//!
//! let cmd: Cmd<'_, (), (), (), ()> = Cmd {
//!     name: "hello",
//!     description: "Say hello",
//!     category: None,
//!     action: Action::no_args(|| println!("Hello, world!")),
//! };
//! ```
//!
//! ## Hierarchical Commands
//! ```rust
//! use shovel::{Cmd, Action};
//!
//! // Leaf commands
//! let start_cmd: Cmd<'_, (), (), (), ()> = Cmd {
//!     name: "start",
//!     description: "Start the service",
//!     category: Some("control"),
//!     action: Action::no_args(|| println!("Starting...")),
//! };
//!
//! let stop_cmd: Cmd<'_, (), (), (), ()> = Cmd {
//!     name: "stop",
//!     description: "Stop the service",
//!     category: Some("control"),
//!     action: Action::no_args(|| println!("Stopping...")),
//! };
//!
//! // Parent command with subcommands
//! let service_cmd: Cmd<'_, (), (), (), _> = Cmd {
//!     name: "service",
//!     description: "Service management",
//!     category: None,
//!     action: Action::subcmds((start_cmd, stop_cmd)),
//! };
//! ```
//!
//! ## Category-Based Organization
//! ```rust
//! # use shovel::{Cmd, Action};
//! let file_cmds = (
//!     Cmd { name: "read", description: "Read file", category: Some("file"), action: Action::<(), (), (), ()>::no_args(|| {}) },
//!     Cmd { name: "write", description: "Write file", category: Some("file"), action: Action::<(), (), (), ()>::no_args(|| {}) },
//! );
//! ```
//!
//! ## Nested Command Hierarchies
//! ```rust
//! # use shovel::{Cmd, Action};
//! let database_cmds = (
//!     Cmd { name: "migrate", description: "Run migrations", category: None, action: Action::<(), (), (), ()>::no_args(|| {}) },
//!     Cmd { name: "seed", description: "Seed database", category: None, action: Action::<(), (), (), ()>::no_args(|| {}) },
//! );
//!
//! let db_cmd: Cmd<'_, (), (), (), _> = Cmd {
//!     name: "db",
//!     description: "Database operations",
//!     category: None,
//!     action: Action::subcmds(database_cmds),
//! };
//! ```
//!
//! # Performance Characteristics
//!
//! ## Type System Integration
//! - **Compile-time verification** of option/argument compatibility
//! - **Automatic help generation** from type information
//! - **Zero-cost abstractions** with no runtime overhead
//!
//! ## Command Resolution Algorithm
//! 1. **Exact name matching** for command lookup
//! 2. **Fuzzy matching** with edit distance for suggestions
//! 3. **Hierarchical traversal** for nested command structures
//!
//! ## Memory Pool Integration
//! - **Stack-allocated pools** for help generation
//! - **Hierarchical allocation** matching command structure
//! - **Lifetime-safe** memory management without heap allocation

use shovel_macros::impl_subcmds;

use crate::BUILTIN_OPTS;
use crate::CmdHelp;
use crate::EXIT_CODE_SUCCESS;
use crate::HelpOpt;
use crate::Opts;
use crate::Pool;
use crate::Suggestion;
use crate::action::{Action, ActionResult};
use crate::{Args, RawArgs};

/// Command of `shovel` application.
///
/// A [`Cmd`] represents a single command in your CLI application hierarchy.
/// It can be a subcommand of an [`App`](crate::App) or another [`Cmd`], allowing for
/// nested command structures like `git commit --message "..."` or `cargo build --release`.
///
/// # Usage Guidelines
///
/// ## Simple Command
/// ```
/// # use shovel::*;
/// let cmd: Cmd<'_, (), (), (), ()> = Cmd {
///     name: "hello",
///     description: "Say hello",
///     category: None,
///     action: Action::no_args(|| println!("Hello!")),
/// };
/// ```
///
/// ## Command with Options and Arguments
/// ```
/// # use shovel::*;
/// # #[derive(Opts)] struct MyOpts { verbose: bool }
/// # #[derive(Args)] struct MyArgs { name: String }
/// let cmd: Cmd<'_, MyOpts, MyArgs, (), ()> = Cmd {
///     name: "greet",
///     description: "Greet someone",
///     category: Some("social"),
///     action: Action::opts_args(|opts: MyOpts, args: MyArgs| {
///         if opts.verbose {
///             println!("Greeting {} verbosely!", args.name);
///         } else {
///             println!("Hello, {}!", args.name);
///         }
///     }),
/// };
/// ```
///
/// See also: [`App`](crate::App), [`Action`], [`Opts`], [`Args`]
#[derive(Debug)]
pub struct Cmd<
    'a,
    O: Opts,
    A: Args,
    G: Opts,
    C: Subcmds<G>,
    R: crate::ActionReturn = (),
> {
    /// The name of the command.
    pub name: &'a str,
    /// A long description of the command.
    pub description: &'a str,
    /// The category the command is part of
    pub category: Option<&'a str>,
    /// The action of the command.
    pub action: Action<O, A, G, C, R>,
}

/// A trait for objects which can be used as the children of a command.
pub trait Subcmds<G: Opts> {
    /// The number of commands.
    const CMD_COUNT: usize;

    /// Runs the subcommand matching the given name.
    ///
    /// # Arguments
    /// * `name` - The name of the subcommand to run
    /// * `raw_args` - The raw arguments passed to the subcommand
    /// * `global_opts` - The global options shared across all commands
    ///
    /// # Returns
    /// An `ActionResult` indicating the outcome of running the subcommand
    fn run<'b>(
        &'b self,
        name: &'b str,
        raw_args: &'b RawArgs<'_>,
        global_opts: G,
    ) -> ActionResult<'b>;

    /// Retrieves help information for all subcommands.
    ///
    /// # Arguments
    /// * `pool` - A memory pool for allocating `CmdHelp` instances
    ///
    /// # Returns
    /// An optional slice of `CmdHelp`s, one for each subcommand,
    /// or `None` if there are no subcommands
    fn get_help_list<'b>(
        &'b self,
        pool: Pool<'b, CmdHelp<'b>>,
    ) -> Option<&'b [CmdHelp<'b>]>;
}

impl<G: Opts> Subcmds<G> for () {
    const CMD_COUNT: usize = 0;

    fn run(
        &'_ self,
        _name: &str,
        _raw_args: &RawArgs<'_>,
        _global_opts: G,
    ) -> ActionResult<'_> {
        ActionResult::Success(EXIT_CODE_SUCCESS)
    }

    fn get_help_list<'b>(
        &'b self,
        _pool: Pool<'b, CmdHelp<'b>>,
    ) -> Option<&'b [CmdHelp<'b>]> {
        None
    }
}

impl<O: Opts, A: Args, G: Opts, C: Subcmds<G>, R: crate::ActionReturn>
    Cmd<'_, O, A, G, C, R>
{
    /// Run the command.
    pub fn run<'b>(
        &'b self,
        raw_args: &'b RawArgs<'_>,
        global_opts: G,
    ) -> ActionResult<'b> {
        let pos = raw_args.current();
        if HelpOpt::parse(raw_args.opts(O::specs())) {
            return ActionResult::ShowHelp {
                cmd: self.name,
                pos,
            };
        }
        self.action.run(self.name, raw_args, global_opts, false)
    }

    /// Returns this help of command.
    pub fn help<'b>(&'b self, pool: Pool<'b, CmdHelp<'b>>) -> CmdHelp<'b> {
        CmdHelp::new(
            self.name,
            self.description,
            self.category,
            self.action.help(pool),
            &BUILTIN_OPTS[..1],
        )
    }
}

impl<O1: Opts, A1: Args, G: Opts, C1: Subcmds<G>, R1: crate::ActionReturn>
    Subcmds<G> for Cmd<'_, O1, A1, G, C1, R1>
{
    const CMD_COUNT: usize = C1::CMD_COUNT + 1;

    fn run<'b>(
        &'b self,
        name: &'b str,
        raw_args: &'b RawArgs<'_>,
        global_opts: G,
    ) -> ActionResult<'b> {
        if self.name == name {
            self.run(raw_args, global_opts)
        } else {
            ActionResult::Suggestion(Suggestion::find(name, &[self.name]))
        }
    }

    fn get_help_list<'b>(
        &'b self,
        pool: Pool<'b, CmdHelp<'b>>,
    ) -> Option<&'b [CmdHelp<'b>]> {
        let (mut parent, children) = pool.alloc(1);
        Some(parent.assign(&[self.help(children)]))
    }
}

impl<O1: Opts, A1: Args, G: Opts, C1: Subcmds<G>, R1: crate::ActionReturn>
    Subcmds<G> for (Cmd<'_, O1, A1, G, C1, R1>,)
{
    const CMD_COUNT: usize = C1::CMD_COUNT + 1;

    fn run<'b>(
        &'b self,
        name: &'b str,
        raw_args: &'b RawArgs<'_>,
        global_opts: G,
    ) -> ActionResult<'b> {
        if self.0.name == name {
            self.0.run(raw_args, global_opts)
        } else {
            ActionResult::Suggestion(Suggestion::find(name, &[self.0.name]))
        }
    }

    fn get_help_list<'b>(
        &'b self,
        pool: Pool<'b, CmdHelp<'b>>,
    ) -> Option<&'b [CmdHelp<'b>]> {
        let (mut parent, children) = pool.alloc(1);
        Some(parent.assign(&[self.0.help(children)]))
    }
}

impl<
    'a,
    G: Opts,
    O1: Opts,
    A1: Args,
    C1: Subcmds<G>,
    R1: crate::ActionReturn,
    O2: Opts,
    A2: Args,
    C2: Subcmds<G>,
    R2: crate::ActionReturn,
> Subcmds<G> for (Cmd<'a, O1, A1, G, C1, R1>, Cmd<'a, O2, A2, G, C2, R2>)
{
    const CMD_COUNT: usize = C1::CMD_COUNT + C2::CMD_COUNT + 2;

    fn run<'b>(
        &'b self,
        name: &'b str,
        raw_args: &'b RawArgs<'_>,
        global_opts: G,
    ) -> ActionResult<'b> {
        if self.0.name == name {
            self.0.run(raw_args, global_opts)
        } else if self.1.name == name {
            self.1.run(raw_args, global_opts)
        } else {
            ActionResult::Suggestion(Suggestion::find(
                name,
                &[self.0.name, self.1.name],
            ))
        }
    }

    fn get_help_list<'b>(
        &'b self,
        pool: Pool<'b, CmdHelp<'b>>,
    ) -> Option<&'b [CmdHelp<'b>]> {
        let (mut parent, children) = pool.alloc(2);
        let (subcmd1_pool, subcmd2_pool) = children.alloc(C1::CMD_COUNT);

        Some(
            parent.assign(&[
                self.0.help(subcmd1_pool),
                self.1.help(subcmd2_pool),
            ]),
        )
    }
}

// Default implementation (25 subcmds)
#[cfg(all(
    not(feature = "subcmds-50"),
    not(feature = "subcmds-75"),
    not(feature = "subcmds-100")
))]
impl_subcmds!(number = 25);

// Additive extensions - only the highest enabled feature runs to avoid duplicates
#[cfg(feature = "subcmds-100")]
impl_subcmds!(number = 100);

#[cfg(all(feature = "subcmds-75", not(feature = "subcmds-100")))]
impl_subcmds!(number = 75);

#[cfg(all(
    feature = "subcmds-50",
    not(feature = "subcmds-75"),
    not(feature = "subcmds-100")
))]
impl_subcmds!(number = 50);

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "std")]
    use crate::args::Args;
    use crate::pool::ArrayPool;
    use crate::{Action, ActionResult};
    #[cfg(feature = "std")]
    use crate::{Opts, ParseError};

    #[cfg(feature = "std")]
    use std::string::String;

    #[cfg(feature = "std")]
    // Test types for unit tests
    #[derive(Debug, PartialEq)]
    struct TestOpts {
        verbose: bool,
    }

    #[cfg(feature = "std")]
    impl Opts for TestOpts {
        fn parse<'a, 'b: 'a>(
            opts: &'a crate::RawOpts<'a, 'b>,
        ) -> Result<(Self, usize), ParseError<'b>> {
            let mut verbose = false;
            for opt in opts {
                if opt.0 == "verbose" || opt.0 == "v" {
                    verbose = true;
                }
            }
            Ok((Self { verbose }, opts.len_consumed()))
        }

        fn specs() -> &'static [crate::Opt] {
            &[crate::Opt {
                name: "verbose",
                ty: "bool",
                required: false,
                flag: true,
                description: "Enable verbose output",
                short: Some('v'),
                format: "-v, --verbose",
            }]
        }
    }

    #[cfg(feature = "std")]
    #[derive(Debug, PartialEq)]
    struct TestArgs {
        name: String,
    }

    #[cfg(feature = "std")]
    impl Args for TestArgs {
        fn parse<'a>(args: &'a [&'a str]) -> Result<Self, ParseError<'a>> {
            if args.len() != 1 {
                return Err(ParseError::ArgumentCountMismatch);
            }
            Ok(Self {
                name: args[0].into(),
            })
        }

        fn specs() -> &'static [crate::ArgGroup] {
            &[crate::ArgGroup {
                tys: &["String"],
                format: "<name>",
                variadic: false,
                description: "The name to process",
            }]
        }
    }

    #[test]
    fn test_cmd_creation() {
        let cmd: Cmd<'_, (), (), (), ()> = Cmd {
            name: "test",
            description: "A test command",
            category: Some("testing"),
            action: Action::no_args(|| {}),
        };

        assert_eq!(cmd.name, "test");
        assert_eq!(cmd.description, "A test command");
        assert_eq!(cmd.category, Some("testing"));
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_cmd_help_generation() {
        let cmd: Cmd<'_, TestOpts, TestArgs, (), ()> = Cmd {
            name: "greet",
            description: "Greet someone",
            category: Some("social"),
            action: Action::opts_args(|_opts: TestOpts, _args: TestArgs| {}),
        };

        let mut pool = ArrayPool::<crate::CmdHelp<'_>, 10>::new();
        let help = cmd.help(pool.as_pool());

        assert_eq!(help.name, "greet");
        assert_eq!(help.description, "Greet someone");
        assert_eq!(help.category, Some("social"));
    }

    #[test]
    fn test_empty_subcmds() {
        let empty_subcmds = ();
        assert_eq!(<() as Subcmds<()>>::CMD_COUNT, 0);

        let args: &[&str] = &[];
        let raw_args = crate::RawArgs::new(args);
        let result = empty_subcmds.run("any", &raw_args, ());
        assert!(matches!(result, ActionResult::Success(0)));

        let mut pool = ArrayPool::<crate::CmdHelp<'_>, 1>::new();
        let help_list =
            <() as Subcmds<()>>::get_help_list(&empty_subcmds, pool.as_pool());
        assert!(help_list.is_none());
    }

    #[test]
    fn test_single_cmd_subcmds() {
        let cmd: Cmd<'_, (), (), (), ()> = Cmd {
            name: "hello",
            description: "Say hello",
            category: None,
            action: Action::no_args(|| {}),
        };

        assert_eq!(<Cmd<'_, (), (), (), ()> as Subcmds<()>>::CMD_COUNT, 1);

        // Test successful command match using Subcmds trait
        let args: &[&str] = &[];
        let raw_args = crate::RawArgs::new(args);
        let result = <Cmd<'_, (), (), (), ()> as Subcmds<()>>::run(
            &cmd,
            "hello",
            &raw_args,
            (),
        );
        assert!(matches!(result, ActionResult::Success(0)));

        // Test command mismatch with suggestion
        let result = <Cmd<'_, (), (), (), ()> as Subcmds<()>>::run(
            &cmd,
            "help",
            &raw_args,
            (),
        );
        assert!(matches!(result, ActionResult::Suggestion(_)));

        // Test help list generation
        let mut pool = ArrayPool::<crate::CmdHelp<'_>, 5>::new();
        let help_list = <Cmd<'_, (), (), (), ()> as Subcmds<()>>::get_help_list(
            &cmd,
            pool.as_pool(),
        );
        assert!(help_list.is_some());
        assert_eq!(help_list.unwrap().len(), 1);
        assert_eq!(help_list.unwrap()[0].name, "hello");
    }

    #[test]
    fn test_single_tuple_subcmds() {
        let cmd: Cmd<'_, (), (), (), ()> = Cmd {
            name: "world",
            description: "Say hello world",
            category: None,
            action: Action::no_args(|| {}),
        };

        let subcmds = (cmd,);
        assert_eq!(<(Cmd<'_, (), (), (), ()>,) as Subcmds<()>>::CMD_COUNT, 1);

        // Test successful command match
        let args: &[&str] = &[];
        let raw_args = crate::RawArgs::new(args);
        let result = subcmds.run("world", &raw_args, ());
        assert!(matches!(result, ActionResult::Success(0)));

        // Test command mismatch
        let result = subcmds.run("hello", &raw_args, ());
        assert!(matches!(result, ActionResult::Suggestion(_)));
    }

    #[test]
    fn test_two_cmd_subcmds() {
        let cmd1: Cmd<'_, (), (), (), ()> = Cmd {
            name: "ping",
            description: "Send a ping",
            category: Some("network"),
            action: Action::no_args(|| {}),
        };

        let cmd2: Cmd<'_, (), (), (), ()> = Cmd {
            name: "pong",
            description: "Send a pong",
            category: Some("network"),
            action: Action::no_args(|| {}),
        };

        let subcmds = (cmd1, cmd2);
        assert_eq!(
            <(Cmd<'_, (), (), (), ()>, Cmd<'_, (), (), (), ()>) as Subcmds<
                (),
            >>::CMD_COUNT,
            2
        );

        // Test first command match
        let args: &[&str] = &[];
        let raw_args = crate::RawArgs::new(args);
        let result = subcmds.run("ping", &raw_args, ());
        assert!(matches!(result, ActionResult::Success(0)));

        // Test second command match
        let result = subcmds.run("pong", &raw_args, ());
        assert!(matches!(result, ActionResult::Success(0)));

        // Test command mismatch with suggestion
        let result = subcmds.run("unknown", &raw_args, ());
        assert!(matches!(result, ActionResult::Suggestion(_)));

        // Test help list generation
        let mut pool = ArrayPool::<crate::CmdHelp<'_>, 10>::new();
        let help_list = subcmds.get_help_list(pool.as_pool());
        assert!(help_list.is_some());
        let helps = help_list.unwrap();
        assert_eq!(helps.len(), 2);
        assert_eq!(helps[0].name, "ping");
        assert_eq!(helps[1].name, "pong");
    }

    #[test]
    fn test_nested_subcmds() {
        type InnerCmd = Cmd<'static, (), (), (), ()>;
        type OuterCmd = Cmd<'static, (), (), (), (InnerCmd,)>;

        let inner_cmd: Cmd<'_, (), (), (), ()> = Cmd {
            name: "inner",
            description: "Inner command",
            category: None,
            action: Action::no_args(|| {}),
        };

        let outer_cmd: OuterCmd = Cmd {
            name: "outer",
            description: "Outer command with subcommands",
            category: None,
            action: Action::subcmds((inner_cmd,)),
        };

        // Test that CMD_COUNT includes nested commands
        assert_eq!(<OuterCmd as Subcmds<()>>::CMD_COUNT, 2);

        // Test help generation for nested structure
        let mut pool = ArrayPool::<crate::CmdHelp<'_>, 10>::new();
        let help = outer_cmd.help(pool.as_pool());
        assert_eq!(help.name, "outer");
        assert_eq!(help.description, "Outer command with subcommands");
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_cmd_with_options_and_args() {
        let cmd: Cmd<'_, TestOpts, TestArgs, (), ()> = Cmd {
            name: "complex",
            description: "A complex command",
            category: None,
            action: Action::opts_args(|_opts: TestOpts, _args: TestArgs| {}),
        };

        // Test help generation includes option specs
        let mut pool = ArrayPool::<crate::CmdHelp<'_>, 10>::new();
        let help = cmd.help(pool.as_pool());
        assert_eq!(help.name, "complex");

        // The help should include option specifications
        let opt_specs = help.opts;
        assert!(!opt_specs.is_empty());
    }

    #[test]
    fn test_cmd_count_calculation() {
        // Type definitions at the top
        type NestedCmd =
            Cmd<'static, (), (), (), (Cmd<'static, (), (), (), ()>,)>;

        // Test that CMD_COUNT is calculated correctly for different structures

        // Empty subcmds
        assert_eq!(<() as Subcmds<()>>::CMD_COUNT, 0);

        // Single command
        assert_eq!(<Cmd<'_, (), (), (), ()> as Subcmds<()>>::CMD_COUNT, 1);

        // Single tuple
        assert_eq!(<(Cmd<'_, (), (), (), ()>,) as Subcmds<()>>::CMD_COUNT, 1);

        // Two commands
        assert_eq!(
            <(Cmd<'_, (), (), (), ()>, Cmd<'_, (), (), (), ()>) as Subcmds<
                (),
            >>::CMD_COUNT,
            2
        );

        // Nested: outer command + inner command = 2 total
        assert_eq!(<NestedCmd as Subcmds<()>>::CMD_COUNT, 2);
    }

    #[test]
    fn test_suggestion_generation() {
        let cmd: Cmd<'_, (), (), (), ()> = Cmd {
            name: "test",
            description: "Test command",
            category: None,
            action: Action::no_args(|| {}),
        };

        // Test that suggestions are generated for similar names
        let args: &[&str] = &[];
        let raw_args = crate::RawArgs::new(args);
        let result = <Cmd<'_, (), (), (), ()> as Subcmds<()>>::run(
            &cmd,
            "tset",
            &raw_args,
            (),
        );
        if let ActionResult::Suggestion(suggestion) = result {
            // Should suggest "test" for "tset"
            assert_eq!(suggestion.cmd, "test");
            assert_eq!(suggestion.typo, "tset");
        } else {
            panic!("Expected suggestion result");
        }
    }
}
