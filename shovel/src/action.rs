//! Command action definitions and execution handlers.
//!
//! This module provides the core action system for Shovel commands. Actions define what happens
//! when a command is executed and can take several forms: handler functions, subcommands, or both.
//! The action system is designed for type safety, zero allocation, and flexible command composition.
//!
//! # Core Types
//!
//! ## [`Action<O, A, G, C>`] - Type-Safe Command Actions
//!
//! The main action type that defines command behavior:
//! - **`O: Opts`** - Command-specific options (e.g., `--verbose`, `--output`)
//! - **`A: Args`** - Positional arguments (e.g., file names, URLs)
//! - **`G: Opts`** - Global options inherited from parent commands
//! - **`C: Subcmds<G>`** - Subcommands that can be invoked
//!
//! ## [`AppAction<G>`] - Application-Level Actions
//!
//! Special action type for root-level application commands, typically used with subcommands.
//!
//! # Action Types
//!
//! Actions can be one of three types:
//!
//! ## 1. Handler Functions
//! Execute code when the command is invoked:
//!
//! ```rust,no_run
//! # use shovel::*;
//! # fn main() {
//! # let _action: Action<(), (), (), ()> =
//! Action::no_args(|| println!("Hello, world!"));
//! # }
//! ```
//!
//! ## 2. Subcommands
//! Delegate to nested commands:
//! ```rust,no_run
//! # use shovel::*;
//! # fn main() {
//! # let action: Action<(), (), (), _> =
//! Action::subcmds((
//!     Cmd { name: "start", description: "Start service", category: None, action: Action::no_args(|| {}) },
//!     Cmd { name: "stop", description: "Stop service", category: None, action: Action::no_args(|| {}) },
//! ));
//! # }
//! ```
//!
//! ## 3. Both Handler and Subcommands
//! Handler runs when no subcommand is specified:
//! ```rust,no_run
//! # use shovel::*;
//! # fn main() {
//! # let action: Action<(), (), (), _> =
//! Action::new(
//!     |_cx| println!("Default action"),
//!     (
//!         Cmd { name: "sub", description: "Subcommand", category: None, action: Action::no_args(|| {}) },
//!     )
//! );
//! # }
//! ```
//!
//! # Handler Variants
//!
//! The action system supports various handler signatures for different argument combinations:
//!
//! - `Action::no_args(|| {})` - No arguments
//! - `Action::opts(|opts: MyOpts| {})` - Options only
//! - `Action::args(|args: MyArgs| {})` - Arguments only
//! - `Action::opts_args(|opts: MyOpts, args: MyArgs| {})` - Options and arguments
//! - `Action::handler(|ctx: Context<MyOpts, MyArgs, GlobalOpts>| {})` - Full context
//!
//! # Performance Characteristics
//!
//! - **Zero allocation**: All action dispatch is stack-based
//! - **Compile-time optimization**: Handler selection resolved at compile time
//! - **Type safety**: Invalid argument combinations caught at compile time
//! - **Minimal overhead**: Direct function calls with no dynamic dispatch
//!
//! # Examples
//!
//! ## Simple Command with Options
//! ```rust,no_run
//! # use shovel::*;
//! #
//! # #[derive(Opts)] struct VerboseOpts { verbose: bool }
//! # fn main() {
//! # let action: Action<VerboseOpts, (), (), ()> =
//! Action::opts(|opts: VerboseOpts| {
//!     if opts.verbose {
//!         println!("Verbose mode enabled");
//!     }
//! });
//! # }
//! ```
//!
//! ## Command with Arguments and Global Options
//! ```rust,no_run
//! # #[cfg(feature = "std")] {
//! # use shovel::*;
//! # #[derive(Args)] struct FileArgs { path: String }
//! # #[derive(Opts)] struct GlobalOpts { debug: bool }
//! # fn main() {
//! # let action: Action<(), FileArgs, GlobalOpts, ()> =
//! Action::args_with_global_opts(|args: FileArgs, global: GlobalOpts| {
//!     if global.debug {
//!         println!("Processing file: {}", args.path);
//!     }
//! });
//! # }
//! # }
//! ```
//!
//! See also: [`Cmd`](crate::Cmd), [`App`](crate::App), [`Opts`], [`Args`], [`Context`]

use core::marker::PhantomData;

use crate::Context;
use crate::Opts;
use crate::ParseError;
use crate::Pool;
use crate::Subcmds;
use crate::Suggestion;
use crate::{ActionHelp, CmdHelp};
use crate::{Args, RawArgs};

/// Represents the action of a `shovel` [`Cmd`](crate::Cmd).
///
/// An `Action` defines what happens when a command is executed. It can be one of three types:
/// 1. **Handler function** - Executes code when the command is invoked
/// 2. **Set of subcommands** - Delegates to nested commands ([`Subcmds`])
/// 3. **Both** - A handler function with subcommands (handler runs when no subcommand is specified)
///
/// # Generic Parameters
/// - `O`: Optional arguments (derived from [`Opts`])
/// - `A`: Positional arguments (derived from [`Args`])
/// - `G`: Global optional arguments (derived from [`Opts`])
/// - `C`: Subcommands (implementing [`Subcmds<G>`])
///
/// # Usage Guidelines
///
/// ## Simple Handler (No Arguments)
/// ```rust,no_run
/// # use shovel::*;
/// # fn main() {
/// # let action: Action<(), (), (), ()> =
/// Action::no_args(|| println!("Hello, world!"));
/// # }
/// ```
///
/// ## Handler with Options
/// ```rust,no_run
/// # use shovel::*;
/// # #[derive(Opts)] struct MyOpts { verbose: bool }
/// # fn main() {
/// # let action: Action<MyOpts, (), (), ()> =
/// Action::opts(|opts: MyOpts| {
///     if opts.verbose { println!("Verbose mode enabled"); }
/// });
/// # }
/// ```
///
/// ## Handler with Arguments
/// ```
/// # use shovel::*;
/// # #[derive(Args)] struct MyArgs { name: String }
/// # fn main() {
/// # let action: Action<(), MyArgs, (), ()> =
/// Action::args(|args: MyArgs| {
///     println!("Hello, {}!", args.name);
/// });
/// # }
/// ```
///
/// ## Subcommands Only
/// ```
/// # use shovel::*;
/// # fn main() {
/// # let action: Action<(), (), (), _> =
/// Action::subcmds((
///     Cmd { name: "sub1", description: "First subcommand", category: None, action: Action::no_args(|| {}) },
///     Cmd { name: "sub2", description: "Second subcommand", category: None, action: Action::no_args(|| {}) },
/// ));
/// # }
/// ```
///
/// See also: [`App`](crate::App), [`Cmd`](crate::Cmd), [`Subcmds`], [`Opts`], [`Args`], [`Context`]
#[derive(Debug)]
pub struct Action<O: Opts, A: Args, G: Opts, C: Subcmds<G>>(
    ActionInner<O, A, G, C>,
);

#[derive(Debug)]
enum ActionInner<O: Opts, A: Args, G: Opts, C: Subcmds<G>> {
    Handler(Handler<O, A, G>),
    Subcmds(C),
    Both {
        handler: Handler<O, A, G>,
        subcmds: C,
    },
}

/// A special action builder that can only be used with [`App`](crate::App).
///
/// This struct is used to create actions specifically for the root-level application command.
/// It provides methods to define subcommands for the main application. Unlike regular [`Action`],
/// `AppAction` is designed for the top-level application structure.
///
/// # Generic Parameters
/// - `G`: The type of global optional arguments, which must implement the [`Opts`] trait.
///
/// # Usage
///
/// `AppAction` is typically used when your application consists primarily of subcommands:
/// ```
/// # use shovel::*;
/// App {
///     name: "myapp",
///     description: "My CLI application",
///     version: "0.1.0",
///     authors: ["Author Name <author@example.com>"],
///     copyright: "(c) 2026 Company Name",
///     action: AppAction::<()>::subcmds((
///         Cmd { name: "cmd1", description: "First command", category: None, action: Action::no_args(|| {}) },
///         Cmd { name: "cmd2", description: "Second command", category: None, action: Action::no_args(|| {}) },
///     )),
/// };
/// ```
///
/// See also: [`App`](crate::App), [`Action`], [`Opts`]
#[derive(Debug)]
pub struct AppAction<G: Opts>(PhantomData<G>);

/// The result of executing an action.
#[derive(Debug)]
pub enum ActionResult<'a> {
    /// Indicates that help should be displayed for the specified command.
    ShowHelp {
        /// The name of the command for which help is requested.
        cmd: &'a str,
        /// The position of the command in the raw arguments.
        pos: usize,
    },
    /// Provides a suggestion to the user.
    Suggestion(Suggestion<'a>),
    /// Represents a parsing error.
    Error(ParseError<'a>),
    /// Indicates successful execution of the action.
    Success,
}

/// The action handler of a shovel Cmd.
///
/// The generic parameters O, A and G represent optional arguments, positional arguments and
/// global optional arguments respectively.
#[derive(Debug)]
enum Handler<O: Opts, A: Args, G: Opts> {
    NoArgs(fn()),
    Opts(fn(O)),
    Args(fn(A)),
    GlobalOpts(fn(G)),
    OptsArgs(fn(O, A)),
    OptsWithGlobal(fn(O, G)),
    ArgsWithGlobal(fn(A, G)),
    All(fn(O, A, G)),
    Context(fn(Context<O, A, G>)),
}

impl<O: Opts, A: Args, G: Opts> Handler<O, A, G> {
    pub(crate) fn call(&self, cx: Context<O, A, G>) {
        match self {
            Self::NoArgs(f) => f(),
            Self::Opts(f) => {
                let Context { opts, .. } = cx;
                f(opts);
            }
            Self::Args(f) => {
                let Context { args, .. } = cx;
                f(args);
            }
            Self::OptsArgs(f) => {
                let Context { opts, args, .. } = cx;
                f(opts, args);
            }
            Self::GlobalOpts(f) => {
                let Context { global_opts, .. } = cx;
                f(global_opts);
            }
            Self::OptsWithGlobal(f) => {
                let Context {
                    opts, global_opts, ..
                } = cx;
                f(opts, global_opts);
            }
            Self::ArgsWithGlobal(f) => {
                let Context {
                    args, global_opts, ..
                } = cx;
                f(args, global_opts);
            }
            Self::All(f) => {
                let Context {
                    opts,
                    args,
                    global_opts,
                } = cx;
                f(opts, args, global_opts);
            }
            Self::Context(f) => f(cx),
        }
    }
}

impl<O: Opts, A: Args, G: Opts, C: Subcmds<G>> Action<O, A, G, C> {
    /// Creates an `Action` with both a handler and a set of subcommands.
    ///
    /// This method allows you to create an `Action` that can handle both direct invocation
    /// and subcommands. The handler function will be called when no subcommand is specified,
    /// while the subcommands will be used when a matching subcommand is provided.
    ///
    /// # Arguments
    /// * `handler` - A function that takes a `Context<O, A, G>` as its argument and handles the action when no subcommand is specified.
    /// * `subcmds` - A set of subcommands implementing the `Subcmds<G>` trait.
    ///
    /// # Panics
    /// This function will panic at compile-time if `C::CMD_COUNT` is 0, ensuring that
    /// there is at least one subcommand defined.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use shovel::*;
    ///
    /// #[derive(Opts)]
    /// struct MyOpts {
    ///     // ... option fields ...
    /// }
    ///
    /// #[derive(Args)]
    /// struct MyArgs {
    ///     // ... argument fields ...
    /// }
    ///
    /// fn my_handler(cx: Context<MyOpts, MyArgs, ()>) {
    ///     // ... handler logic ...
    /// }
    ///
    /// let action = Action::new(my_handler, (
    ///     Cmd {
    ///         name: "subcmd1",
    ///         description: "Subcommand 1",
    ///         category: None,
    ///         action: Action::no_args(|| println!("Subcommand 1 executed")),
    ///     },
    ///     // ... more subcommands ...
    /// ));
    /// ```
    pub fn new(handler: fn(cx: Context<O, A, G>), subcmds: C) -> Self {
        #[cfg(not(feature = "generic_const_exprs"))]
        const {
            assert!(C::CMD_COUNT > 0);
        };
        #[cfg(feature = "generic_const_exprs")]
        debug_assert!(C::CMD_COUNT > 0);
        Action(ActionInner::Both {
            handler: Handler::Context(handler),
            subcmds,
        })
    }

    pub(crate) fn run<'b>(
        &'b self,
        name: &'b str,
        raw_args: &'b RawArgs<'b>,
        global_opts: G,
        is_app_action: bool,
    ) -> ActionResult<'b> {
        match &self.0 {
            ActionInner::Subcmds(subcmds) => match raw_args.cmd() {
                Some(cmd) => {
                    if cmd.starts_with('-') || cmd.starts_with("--") {
                        ActionResult::Error(ParseError::UnknownOption(cmd))
                    } else {
                        subcmds.run(cmd, raw_args, global_opts)
                    }
                }
                None => ActionResult::ShowHelp {
                    cmd: name,
                    pos: raw_args.current(),
                },
            },
            ActionInner::Handler(handler) => {
                // Run with the app handler
                if is_app_action {
                    raw_args.reset();
                }
                let cx: Context<O, A, G> =
                    match Context::parse(raw_args, global_opts) {
                        Err(e) => {
                            return ActionResult::Error(e);
                        }
                        Ok(cx) => cx,
                    };
                handler.call(cx);
                ActionResult::Success
            }
            ActionInner::Both { handler, subcmds } => {
                if let Some(cmd) = raw_args.cmd() {
                    if cmd.starts_with('-') || cmd.starts_with("--") {
                        ActionResult::Error(ParseError::UnknownOption(cmd))
                    } else {
                        subcmds.run(cmd, raw_args, global_opts)
                    }
                } else {
                    let cx: Context<O, A, G> =
                        match Context::parse(raw_args, global_opts) {
                            Err(e) => {
                                return ActionResult::Error(e);
                            }
                            Ok(cx) => cx,
                        };
                    handler.call(cx);
                    ActionResult::Success
                }
            }
        }
    }

    pub(crate) fn help<'a>(
        &'a self,
        pool: Pool<'a, CmdHelp<'a>>,
    ) -> ActionHelp<'a> {
        match &self.0 {
            ActionInner::Handler(_) => {
                let opts = O::specs();
                let groups = A::specs();
                ActionHelp {
                    opts,
                    groups,
                    subhelps: None,
                }
            }
            ActionInner::Subcmds(subcmds)
            | ActionInner::Both { subcmds, .. } => {
                let opts = O::specs();
                let groups = A::specs();
                let subhelps = subcmds.get_help_list(pool);
                ActionHelp {
                    opts,
                    groups,
                    subhelps,
                }
            }
        }
    }
}

impl<G: Opts> AppAction<G> {
    /// Creates an app `Action` with a set of subcommands.
    ///
    /// This method is specifically designed for use with the `App` struct to define
    /// the root-level command structure of your CLI application.
    ///
    /// # Arguments
    ///
    /// * `subcmds` - A tuple of `Cmd` structs representing the subcommands.
    ///
    /// # Returns
    ///
    /// Returns an `Action<G, (), G, C>` where:
    /// - `G` is the global options type
    /// - `C` is the type of the subcommands tuple
    ///
    /// # Panics
    ///
    /// This function will panic at compile-time if no subcommands are provided.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use shovel::*;
    ///
    /// # fn main() {
    /// App {
    ///     name: "print",
    ///     description: "Print something",
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
    /// };
    /// # }
    /// ```
    pub fn subcmds<C: Subcmds<G>>(subcmds: C) -> Action<G, (), G, C> {
        #[cfg(not(feature = "generic_const_exprs"))]
        const {
            assert!(C::CMD_COUNT > 0);
        };
        #[cfg(feature = "generic_const_exprs")]
        debug_assert!(C::CMD_COUNT > 0);
        Action(ActionInner::Subcmds(subcmds))
    }
}

impl<G: Opts, C: Subcmds<G>> Action<(), (), G, C> {
    /// Creates an `Action` with a set of subcommands.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use shovel::*;
    ///
    /// # fn main() {
    /// # let action: Action<(), (), (), _> =
    /// Action::subcmds((
    ///     Cmd {
    ///         name: "ping",
    ///         description: "Print a ping",
    ///         category: None,
    ///         action: Action::no_args(|| {
    ///             println!("ping");
    ///         }),
    ///     },
    ///     Cmd {
    ///         name: "pong",
    ///         description: "Print a pong",
    ///         category: None,
    ///         action: Action::no_args(|| {
    ///             println!("pong");
    ///         }),
    ///     },
    /// ));
    /// # }
    /// ```
    pub fn subcmds(subcmds: C) -> Self {
        #[cfg(not(feature = "generic_const_exprs"))]
        const {
            assert!(C::CMD_COUNT > 0);
        };
        #[cfg(feature = "generic_const_exprs")]
        debug_assert!(C::CMD_COUNT > 0);
        Self(ActionInner::Subcmds(subcmds))
    }
}

impl<O: Opts, A: Args, G: Opts> Action<O, A, G, ()> {
    fn with_handler(handler: Handler<O, A, G>) -> Self {
        Self(ActionInner::Handler(handler))
    }

    /// Creates an `Action` with a handler that receives a `Context`.
    ///
    /// This method allows you to create an action that has access to all arguments
    /// (optional, positional, and global) through a single `Context` object.
    ///
    /// # Examples
    ///
    /// ```
    /// use shovel::*;
    ///
    /// #[derive(Args)]
    /// #[format = "[<name>]"]
    /// pub struct HelloArgs {
    ///     /// Name used to say hello
    ///     pub name: String,
    /// }
    ///
    /// #[derive(Opts)]
    /// pub struct HelloOpts {
    ///     /// Number of times to greet
    ///     #[short = 'n']
    ///     pub number: Option<u32>,
    /// }
    ///
    /// #[derive(Opts)]
    /// pub struct GlobalOpts {
    ///     /// Enable debug output
    ///     #[short = 'd']
    ///     pub debug: bool,
    /// }
    ///
    /// Action::handler(|cx: Context<HelloOpts, HelloArgs, GlobalOpts>| {
    ///     if cx.global_opts.debug {
    ///         println!("Debug: Greeting {} times", cx.opts.number.unwrap_or(1));
    ///     }
    ///     for _ in 0..cx.opts.number.unwrap_or(1) {
    ///         println!("Hello, {}!", cx.args.name);
    ///     }
    /// });
    /// ```
    pub fn handler(f: fn(cx: Context<O, A, G>)) -> Self {
        Self::with_handler(Handler::Context(f))
    }

    /// Creates an `Action` with a handler that receives all arguments separately.
    ///
    /// This method allows you to create an action that receives optional arguments,
    /// positional arguments, and global optional arguments as separate parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use shovel::*;
    ///
    /// #[derive(Args)]
    /// #[format = "[<name>]"]
    /// pub struct HelloArgs {
    ///     /// Name used to say hello
    ///     pub name: String,
    /// }
    ///
    /// #[derive(Opts)]
    /// pub struct HelloOpts {
    ///     /// Number of times to greet
    ///     #[short = 'n']
    ///     pub number: Option<u32>,
    /// }
    ///
    /// #[derive(Opts)]
    /// pub struct GlobalOpts {
    ///     /// Enable debug output
    ///     #[short = 'd']
    ///     pub debug: bool,
    /// }
    ///
    /// Action::all(|opts: HelloOpts, args: HelloArgs, global_opts: GlobalOpts| {
    ///     if global_opts.debug {
    ///         println!("Debug: Greeting {} times", opts.number.unwrap_or(1));
    ///     }
    ///     for _ in 0..opts.number.unwrap_or(1) {
    ///         println!("Hello, {}!", args.name);
    ///     }
    /// });
    /// ```
    pub fn all(f: fn(opts: O, args: A, global_opts: G)) -> Self {
        Self::with_handler(Handler::All(f))
    }
}

impl<G: Opts> Action<(), (), G, ()> {
    /// Creates an `Action` with a handler that takes no arguments.
    ///
    /// This method is useful for creating simple actions that don't require any input parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use shovel::*;
    /// # fn main() {
    /// # let action: Action<(), (), (), ()> =
    /// Action::no_args(|| {
    ///     println!("Hello, world!");
    /// });
    /// # }
    /// ```
    pub fn no_args(f: fn()) -> Self {
        Self::with_handler(Handler::NoArgs(f))
    }
}

impl<A: Args, G: Opts> Action<(), A, G, ()> {
    /// Creates an `Action` with a handler that only processes positional arguments.
    ///
    /// This method is useful when you want to create an action that focuses solely on
    /// positional arguments, ignoring any optional or global arguments.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that takes a single argument of type `A`, where `A` implements the `Args` trait.
    ///
    /// # Returns
    ///
    /// Returns a new `Action` instance that will execute the provided function when invoked.
    ///
    /// # Examples
    ///
    /// ```
    /// use shovel::*;
    ///
    /// #[derive(Args)]
    /// pub struct GreetArgs {
    ///     /// Name to greet
    ///     name: String,
    ///     /// Number of times to greet
    ///     count: u32,
    /// }
    ///
    /// fn greet(args: GreetArgs) {
    ///     for _ in 0.. args.count {
    ///         println!("Hello, {}!", args.name);
    ///     }
    /// }
    ///
    /// # fn main() {
    /// # let action: Action<(), GreetArgs, (), ()> =
    /// Action::args(greet);
    /// # }
    /// ```
    ///
    /// In this example, `GreetArgs` defines two positional arguments: a required `name` and an optional `count`.
    /// The `greet` function uses these arguments to print a greeting message. The `Action::args` method
    /// is used to create an action that will invoke the `greet` function with the parsed arguments.
    pub fn args(f: fn(args: A)) -> Self {
        Self::with_handler(Handler::Args(f))
    }
}

impl<O: Opts, G: Opts> Action<O, (), G, ()> {
    /// Creates an `Action` with a handler that processes only optional arguments.
    ///
    /// This method is useful when you want to create an action that focuses solely on
    /// optional arguments, ignoring any positional or global arguments.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that takes a single argument of type `O`, where `O` implements the `Opts` trait.
    ///
    /// # Returns
    ///
    /// Returns a new `Action` instance that will execute the provided function when invoked.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use shovel::*;
    ///
    /// #[derive(Opts)]
    /// pub struct TickOpts {
    ///     /// The delay time (in seconds)
    ///     #[short = 'd']
    ///     pub delay: u64,
    ///     /// Number of times to tick
    ///     #[short = 'n']
    ///     pub number: Option<u32>,
    /// }
    ///
    /// fn tick(opts: TickOpts) {
    ///     for _ in 0..opts.number.unwrap_or(3) {
    ///         println!("tick");
    ///         std::thread::sleep(std::time::Duration::from_secs(opts.delay));
    ///     }
    /// }
    ///
    /// # fn main() {
    /// # let action: Action<TickOpts, (), (), ()> =
    /// Action::opts(tick);
    /// # }
    /// ```
    ///
    /// In this example, `TickOpts` defines two optional arguments: `delay` and `number`.
    /// The `tick` function uses these options to control the timing and repetition of the "tick" output.
    /// The `Action::opts` method is used to create an action that will invoke the `tick` function
    /// with the parsed optional arguments.
    pub fn opts(f: fn(opts: O)) -> Self {
        Self::with_handler(Handler::Opts(f))
    }
}

impl<O: Opts, A: Args, G: Opts> Action<O, A, G, ()> {
    /// Creates an `Action` with a handler that processes both optional and positional arguments.
    ///
    /// This method is useful when you want to create an action that needs access to both
    /// optional flags/parameters and positional arguments, but doesn't require global options.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that takes two arguments:
    ///   - `opts: O` where `O` implements the `Opts` trait
    ///   - `args: A` where `A` implements the `Args` trait
    ///
    /// # Returns
    ///
    /// Returns a new `Action` instance that will execute the provided function when invoked.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use shovel::*;
    ///
    /// #[derive(Args)]
    /// pub struct EchoArgs {
    ///     /// Message to echo
    ///     pub msg: String,
    /// }
    ///
    /// #[derive(Opts)]
    /// pub struct EchoOpts {
    ///     /// Do not output the trailing newline
    ///     #[short = 'n']
    ///     pub no_newline: bool,
    ///     /// Number of times to repeat the message
    ///     #[short = 'r']
    ///     pub repeat: Option<u32>,
    /// }
    ///
    /// # fn main() {
    /// # let action: Action<EchoOpts, EchoArgs, (), ()> =
    /// Action::opts_args(
    ///     |opts: EchoOpts, args: EchoArgs| {
    ///         let repeat = opts.repeat.unwrap_or(1);
    ///         for _ in 0..repeat {
    ///             if opts.no_newline {
    ///                 print!("{}", args.msg);
    ///             } else {
    ///                 println!("{}", args.msg);
    ///             }
    ///         }
    ///     }
    /// );
    /// # }
    /// ```
    ///
    /// In this example, `EchoOpts` defines optional flags for controlling output,
    /// while `EchoArgs` contains the positional argument for the message to echo.
    /// The action created will use both the optional and positional arguments to
    /// determine how to print the message.
    pub fn opts_args(f: fn(opts: O, args: A)) -> Self {
        Self::with_handler(Handler::OptsArgs(f))
    }
}

impl<A: Args, G: Opts> Action<(), A, G, ()> {
    /// Creates an `Action` with a handler that processes both positional arguments and global optional arguments.
    ///
    /// This method is useful when you need to handle positional arguments while also having access to global options
    /// that are shared across multiple commands.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that takes two parameters:
    ///   - `args: A` - The parsed positional arguments
    ///   - `global_opts: G` - The parsed global options
    ///
    /// # Returns
    ///
    /// Returns a new `Action` instance that will execute the provided function when invoked.
    ///
    /// # Examples
    ///
    /// ```
    /// use shovel::*;
    ///
    /// #[derive(Args)]
    /// pub struct SumArgs {
    ///     first: f64,
    ///     second: f64,
    /// }
    ///
    /// #[derive(Opts)]
    /// pub struct GlobalOpts {
    ///     /// Enable verbose output
    ///     #[short = 'v']
    ///     pub verbose: bool,
    /// }
    ///
    /// # fn main() {
    /// # let action: Action<(), SumArgs, GlobalOpts, ()> =
    /// Action::args_with_global_opts(
    ///     |args: SumArgs, global_opts: GlobalOpts| {
    ///         let result = args.first + args.second;
    ///         if global_opts.verbose {
    ///             println!("Calculating sum: {} + {} = {}", args.first, args.second, result);
    ///         } else {
    ///             println!("Sum: {}", result);
    ///         }
    ///     }
    /// );
    /// # }
    /// ```
    ///
    /// In this example, `SumArgs` defines the positional arguments for two numbers to sum,
    /// while `GlobalOpts` provides a verbose flag that affects the output format.
    /// The action created will use both the positional arguments and the global options
    /// to determine how to calculate and display the sum.
    pub fn args_with_global_opts(f: fn(args: A, global_opts: G)) -> Self {
        Self::with_handler(Handler::ArgsWithGlobal(f))
    }
}

impl<O: Opts, G: Opts> Action<O, (), G, ()> {
    /// Creates an `Action` with a handler that processes both command-specific optional arguments and global optional arguments.
    ///
    /// This method is useful when you need to handle command-specific options while also having access to global options
    /// that are shared across multiple commands.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that takes two parameters:
    ///   - `opts: O` - The parsed command-specific options
    ///   - `global_opts: G` - The parsed global options
    ///
    /// # Returns
    ///
    /// Returns a new `Action` instance that will execute the provided function when invoked.
    ///
    /// # Examples
    ///
    /// ```
    /// use shovel::*;
    ///
    /// #[derive(Opts)]
    /// pub struct TickOpts {
    ///     /// The delay time (in seconds)
    ///     #[short = 'd']
    ///     pub delay: u64,
    ///     /// Number of times to tick
    ///     #[short = 'n']
    ///     pub number: Option<u32>,
    /// }
    ///
    /// #[derive(Opts)]
    /// pub struct GlobalOpts {
    ///     /// Enable verbose output
    ///     #[short = 'v']
    ///     pub verbose: bool,
    /// }
    ///
    /// # fn main() {
    /// # let action: Action<TickOpts, (), GlobalOpts, ()> =
    /// Action::opts_with_global_opts(
    ///     |opts: TickOpts, global_opts: GlobalOpts| {
    ///         let count = opts.number.unwrap_or(3);
    ///         if global_opts.verbose {
    ///             println!("Ticking {} times with {} second delay", count, opts.delay);
    ///         }
    ///         for i in 1..=count {
    ///             if global_opts.verbose {
    ///                 println!("Tick {} of {}", i, count);
    ///             } else {
    ///                 println!("tick");
    ///             }
    ///             std::thread::sleep(std::time::Duration::from_secs(opts.delay));
    ///         }
    ///     }
    /// );
    /// # }
    /// ```
    ///
    /// In this example, `TickOpts` defines command-specific options for controlling the tick behavior,
    /// while `GlobalOpts` provides a verbose flag that affects the output format across potentially multiple commands.
    /// The action created will use both the command-specific options and the global options to determine
    /// how to perform and display the ticking operation.
    pub fn opts_with_global_opts(f: fn(opts: O, global_opts: G)) -> Self {
        Self::with_handler(Handler::OptsWithGlobal(f))
    }
}

impl<G: Opts> Action<(), (), G, ()> {
    /// Creates an `Action` with a handler that only processes global optional arguments.
    ///
    /// This method is useful when you want to create an action that focuses solely on
    /// global arguments, which are shared across multiple commands in your CLI application.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that takes a single argument of type `G`, where `G` implements the `Opts` trait. This function will be called with the parsed global options.
    ///
    /// # Returns
    ///
    /// Returns a new `Action` instance that will execute the provided function when invoked.
    ///
    /// # Examples
    ///
    /// ```
    /// use shovel::*;
    ///
    /// #[derive(Opts)]
    /// pub struct GlobalOpts {
    ///     /// Enable verbose output
    ///     #[short = 'v']
    ///     pub verbose: bool,
    ///     /// Set log level
    ///     #[short = 'l']
    ///     pub log_level: Option<String>,
    /// }
    ///
    /// # fn main() {
    /// # let action: Action<(), (), GlobalOpts, ()> =
    /// Action::global_opts(|global_opts: GlobalOpts| {
    ///     if global_opts.verbose {
    ///         println!("Verbose mode enabled");
    ///     }
    ///     if let Some(level) = global_opts.log_level {
    ///         println!("Setting log level to: {}", level);
    ///         // Here you would typically set up your logging framework
    ///     }
    /// });
    /// # }
    /// ```
    ///
    /// In this example, `GlobalOpts` defines two global options: a `verbose` flag and a `log_level` option.
    /// The `setup_logging` function uses these global options to configure the application's logging behavior.
    /// The `Action::global_opts` method is used to create an action that will invoke the `setup_logging` function
    /// with the parsed global arguments, allowing for consistent logging setup across different commands.
    pub fn global_opts(f: fn(global_opts: G)) -> Self {
        Self::with_handler(Handler::GlobalOpts(f))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::ArrayPool;
    use crate::{Cmd, ParseError};
    #[cfg(feature = "std")]
    use std::string::String;

    // Test types for unit tests
    #[derive(Debug, PartialEq)]
    struct TestOpts {
        verbose: bool,
        count: Option<u32>,
    }

    impl Opts for TestOpts {
        fn parse<'a, 'b: 'a>(
            opts: &'a crate::RawOpts<'a, 'b>,
        ) -> Result<(Self, usize), ParseError<'b>> {
            let mut verbose = false;
            let mut count = None;

            for opt in opts {
                match opt.0 {
                    "verbose" | "v" => {
                        verbose = true;
                    }
                    "count" | "c" => {
                        if let Some(value) = opt.1 {
                            count = Some(value.parse().map_err(|_| {
                                ParseError::InvalidOptionValue {
                                    name: "count",
                                    value,
                                    expected: "a number",
                                }
                            })?);
                        } else {
                            return Err(ParseError::MissingValueForOption {
                                name: "count",
                            });
                        }
                    }
                    _ => {}
                }
            }

            Ok((Self { verbose, count }, opts.len_consumed()))
        }

        fn specs() -> &'static [crate::Opt] {
            &[
                crate::Opt {
                    name: "verbose",
                    ty: "bool",
                    required: false,
                    flag: true,
                    description: "Enable verbose output",
                    short: Some('v'),
                    format: "-v, --verbose",
                },
                crate::Opt {
                    name: "count",
                    ty: "u32",
                    required: false,
                    flag: false,
                    description: "Set count value",
                    short: Some('c'),
                    format: "-c, --count <COUNT>",
                },
            ]
        }
    }

    #[cfg(feature = "std")]
    #[derive(Debug, PartialEq)]
    struct TestArgs {
        name: String,
        value: Option<String>,
    }

    #[cfg(feature = "std")]
    impl Args for TestArgs {
        fn parse<'a>(args: &'a [&'a str]) -> Result<Self, ParseError<'a>> {
            match args.len() {
                1 => Ok(Self {
                    name: args[0].into(),
                    value: None,
                }),
                2 => Ok(Self {
                    name: args[0].into(),
                    value: Some(args[1].into()),
                }),
                _ => Err(ParseError::ArgumentCountMismatch),
            }
        }

        fn specs() -> &'static [crate::ArgGroup] {
            &[crate::ArgGroup {
                tys: &["String", "String"],
                format: "<name> [value]",
                variadic: false,
                description: "Name and optional value",
            }]
        }
    }

    #[derive(Debug, PartialEq)]
    struct GlobalOpts {
        debug: bool,
    }

    impl Opts for GlobalOpts {
        fn parse<'a, 'b: 'a>(
            opts: &'a crate::RawOpts<'a, 'b>,
        ) -> Result<(Self, usize), ParseError<'b>> {
            let mut debug = false;

            for opt in opts {
                if opt.0 == "debug" || opt.0 == "d" {
                    debug = true;
                }
            }

            Ok((Self { debug }, opts.len_consumed()))
        }

        fn specs() -> &'static [crate::Opt] {
            &[crate::Opt {
                name: "debug",
                ty: "bool",
                required: false,
                flag: true,
                description: "Enable debug output",
                short: Some('d'),
                format: "-d, --debug",
            }]
        }
    }

    #[test]
    fn test_action_no_args() {
        fn test_handler() {}

        // Test with a simple function pointer
        let action: Action<(), (), (), ()> = Action::no_args(test_handler);

        // Create test context
        let args: &[&str] = &[];
        let raw_args = crate::RawArgs::new(args);
        let global_opts = ();

        // Execute action
        let result = action.run("test", &raw_args, global_opts, false);

        // Verify execution
        assert!(matches!(result, ActionResult::Success));
    }

    #[test]
    fn test_action_opts() {
        fn test_handler(_opts: TestOpts) {}

        let action: Action<TestOpts, (), (), ()> = Action::opts(test_handler);

        // Create test context with options
        let args: &[&str] = &["--verbose", "--count", "5"];
        let raw_args = crate::RawArgs::new(args);
        let global_opts = ();

        // Execute action
        let result = action.run("test", &raw_args, global_opts, false);

        // Verify execution
        assert!(matches!(result, ActionResult::Success));
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_action_args() {
        fn test_handler(_args: TestArgs) {}

        let action: Action<(), TestArgs, (), ()> = Action::args(test_handler);

        // Create test context with arguments
        let args: &[&str] = &["hello", "world"];
        let raw_args = crate::RawArgs::new(args);
        let global_opts = ();

        // Execute action
        let result = action.run("test", &raw_args, global_opts, false);

        // Verify execution
        assert!(matches!(result, ActionResult::Success));
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_action_opts_args() {
        fn test_handler(_opts: TestOpts, _args: TestArgs) {}

        let action: Action<TestOpts, TestArgs, (), ()> =
            Action::opts_args(test_handler);

        // Create test context with both options and arguments
        let args: &[&str] = &["--verbose", "test_name"];
        let raw_args = crate::RawArgs::new(args);
        let global_opts = ();

        // Execute action
        let result = action.run("test", &raw_args, global_opts, false);

        // Verify execution
        assert!(matches!(result, ActionResult::Success));
    }

    #[test]
    fn test_action_global_opts() {
        fn test_handler(_global_opts: GlobalOpts) {}

        let action: Action<(), (), GlobalOpts, ()> =
            Action::global_opts(test_handler);

        // Create test context with global options
        let args: &[&str] = &[];
        let raw_args = crate::RawArgs::new(args);
        let global_opts = GlobalOpts { debug: true };

        // Execute action
        let result = action.run("test", &raw_args, global_opts, false);

        // Verify execution
        assert!(matches!(result, ActionResult::Success));
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_action_all() {
        fn test_handler(
            _opts: TestOpts,
            _args: TestArgs,
            _global_opts: GlobalOpts,
        ) {
        }

        let action: Action<TestOpts, TestArgs, GlobalOpts, ()> =
            Action::all(test_handler);

        // Create test context with all types of arguments
        let args: &[&str] = &["--verbose", "test_name"];
        let raw_args = crate::RawArgs::new(args);
        let global_opts = GlobalOpts { debug: true };

        // Execute action
        let result = action.run("test", &raw_args, global_opts, false);

        // Verify execution
        assert!(matches!(result, ActionResult::Success));
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_action_handler() {
        fn test_handler(_cx: Context<TestOpts, TestArgs, GlobalOpts>) {}

        let action: Action<TestOpts, TestArgs, GlobalOpts, ()> =
            Action::handler(test_handler);

        // Create test context
        let args: &[&str] = &["--verbose", "test_name"];
        let raw_args = crate::RawArgs::new(args);
        let global_opts = GlobalOpts { debug: true };

        // Execute action
        let result = action.run("test", &raw_args, global_opts, false);

        // Verify execution
        assert!(matches!(result, ActionResult::Success));
    }

    #[test]
    fn test_action_subcmds() {
        fn sub1_handler() {}
        fn sub2_handler() {}

        let cmd1: Cmd<'_, (), (), (), ()> = Cmd {
            name: "sub1",
            description: "First subcommand",
            category: None,
            action: Action::no_args(sub1_handler),
        };

        let cmd2: Cmd<'_, (), (), (), ()> = Cmd {
            name: "sub2",
            description: "Second subcommand",
            category: None,
            action: Action::no_args(sub2_handler),
        };

        let action: Action<(), (), (), _> = Action::subcmds((cmd1, cmd2));

        // Test successful subcommand execution
        let args: &[&str] = &["sub1"];
        let raw_args = crate::RawArgs::new(args);
        let result = action.run("test", &raw_args, (), false);
        assert!(matches!(result, ActionResult::Success));

        // Test unknown subcommand
        let args: &[&str] = &["unknown"];
        let raw_args = crate::RawArgs::new(args);
        let result = action.run("test", &raw_args, (), false);
        assert!(matches!(result, ActionResult::Suggestion(_)));

        // Test no subcommand provided - should show help
        let args: &[&str] = &[];
        let raw_args = crate::RawArgs::new(args);
        let result = action.run("test", &raw_args, (), false);
        assert!(matches!(
            result,
            ActionResult::ShowHelp {
                cmd: "test",
                pos: 0
            }
        ));
    }

    #[test]
    fn test_action_both_handler_and_subcmds() {
        fn test_handler(_cx: Context<(), (), ()>) {}
        fn sub_handler() {}

        let cmd: Cmd<'_, (), (), (), ()> = Cmd {
            name: "sub",
            description: "Subcommand",
            category: None,
            action: Action::no_args(sub_handler),
        };

        let action = Action::new(test_handler, (cmd,));

        // Test handler execution when no subcommand provided
        let args: &[&str] = &[];
        let raw_args = crate::RawArgs::new(args);
        let result = action.run("test", &raw_args, (), false);
        assert!(matches!(result, ActionResult::Success));

        // Test subcommand execution
        let args: &[&str] = &["sub"];
        let raw_args = crate::RawArgs::new(args);
        let result = action.run("test", &raw_args, (), false);
        assert!(matches!(result, ActionResult::Success));
    }

    #[test]
    fn test_app_action_subcmds() {
        fn test_cmd_handler() {}

        let cmd: Cmd<'_, (), (), (), ()> = Cmd {
            name: "test_cmd",
            description: "Test command",
            category: None,
            action: Action::no_args(test_cmd_handler),
        };

        let action: Action<(), (), (), _> = AppAction::<()>::subcmds((cmd,));

        // Test subcommand execution
        let args: &[&str] = &["test_cmd"];
        let raw_args = crate::RawArgs::new(args);
        let result = action.run("app", &raw_args, (), true);
        assert!(matches!(result, ActionResult::Success));

        // Test no subcommand - should show help
        let args: &[&str] = &[];
        let raw_args = crate::RawArgs::new(args);
        let result = action.run("app", &raw_args, (), true);
        assert!(matches!(
            result,
            ActionResult::ShowHelp { cmd: "app", pos: 0 }
        ));
    }

    #[test]
    fn test_action_error_handling() {
        fn test_handler(_opts: TestOpts) {}

        let action: Action<TestOpts, (), (), ()> = Action::opts(test_handler);

        // Test invalid option value
        let args: &[&str] = &["--count", "invalid"];
        let raw_args = crate::RawArgs::new(args);
        let result = action.run("test", &raw_args, (), false);
        assert!(matches!(
            result,
            ActionResult::Error(ParseError::InvalidOptionValue { .. })
        ));

        // Test missing option value
        let args: &[&str] = &["--count"];
        let raw_args = crate::RawArgs::new(args);
        let result = action.run("test", &raw_args, (), false);
        assert!(matches!(
            result,
            ActionResult::Error(ParseError::MissingValueForOption { .. })
        ));
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_action_help_generation() {
        fn test_handler(_opts: TestOpts, _args: TestArgs) {}

        let action: Action<TestOpts, TestArgs, (), ()> =
            Action::opts_args(test_handler);

        let mut pool = ArrayPool::<crate::CmdHelp<'_>, 10>::new();
        let help = action.help(pool.as_pool());

        // Verify help contains option specs
        assert!(!help.opts.is_empty());
        assert_eq!(help.opts.len(), 2); // verbose and count options

        // Verify help contains argument specs
        assert!(!help.groups.is_empty());
        assert_eq!(help.groups.len(), 1); // name and value args

        // No subcommands for this action
        assert!(help.subhelps.is_none());
    }

    #[test]
    fn test_action_help_with_subcmds() {
        fn subcmd_handler() {}

        let cmd: Cmd<'_, (), (), (), ()> = Cmd {
            name: "subcmd",
            description: "A subcommand",
            category: None,
            action: Action::no_args(subcmd_handler),
        };

        let action: Action<(), (), (), _> = Action::subcmds((cmd,));

        let mut pool = ArrayPool::<crate::CmdHelp<'_>, 10>::new();
        let help = action.help(pool.as_pool());

        // Should have subcommand help
        assert!(help.subhelps.is_some());
        let subhelps = help.subhelps.unwrap();
        assert_eq!(subhelps.len(), 1);
        assert_eq!(subhelps[0].name, "subcmd");
    }

    #[test]
    fn test_action_option_flag_detection() {
        fn subcmd_handler() {}

        let cmd: Cmd<'_, (), (), (), ()> = Cmd {
            name: "subcmd",
            description: "A subcommand",
            category: None,
            action: Action::no_args(subcmd_handler),
        };

        let action: Action<(), (), (), _> = Action::subcmds((cmd,));

        // Test that options starting with - are treated as unknown options
        let args: &[&str] = &["--unknown-flag"];
        let raw_args = crate::RawArgs::new(args);
        let result = action.run("test", &raw_args, (), false);
        assert!(matches!(
            result,
            ActionResult::Error(ParseError::UnknownOption("--unknown-flag"))
        ));

        // Test that options starting with - are treated as unknown options
        let args: &[&str] = &["-x"];
        let raw_args = crate::RawArgs::new(args);
        let result = action.run("test", &raw_args, (), false);
        assert!(matches!(
            result,
            ActionResult::Error(ParseError::UnknownOption("-x"))
        ));
    }

    #[test]
    fn test_action_result_variants() {
        // Test that ActionResult variants can be created and matched
        let result = ActionResult::Success;
        assert!(matches!(result, ActionResult::Success));

        let result = ActionResult::ShowHelp {
            cmd: "test",
            pos: 0,
        };
        assert!(matches!(
            result,
            ActionResult::ShowHelp {
                cmd: "test",
                pos: 0
            }
        ));

        let result = ActionResult::Error(ParseError::ArgumentCountMismatch);
        assert!(matches!(
            result,
            ActionResult::Error(ParseError::ArgumentCountMismatch)
        ));
    }

    #[test]
    fn test_action_result_showhelp_with_positions() {
        // Test that ShowHelp correctly captures the command position
        let args: &[&str] = &["--verbose", "arg1", "arg2"];
        let raw_args = crate::RawArgs::new(args);

        // Simulate what happens when a command shows help after parsing some args
        raw_args.consume(1); // Consume "--verbose"
        let pos = raw_args.current();

        let result = ActionResult::ShowHelp {
            cmd: "mycommand",
            pos,
        };

        assert!(matches!(
            result,
            ActionResult::ShowHelp {
                cmd: "mycommand",
                pos: 1
            }
        ));

        // Test with different positions
        raw_args.consume(2); // Consume "arg1", "arg2"
        let pos = raw_args.current();

        let result = ActionResult::ShowHelp {
            cmd: "mycommand",
            pos,
        };

        assert!(matches!(
            result,
            ActionResult::ShowHelp {
                cmd: "mycommand",
                pos: 3
            }
        ));
    }
}
