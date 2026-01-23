//! Optional argument parsing and specification.
//!
//! This module provides the [`Opts`] trait and related types for defining and parsing
//! optional command-line arguments (flags and options). Options are parameters that
//! can appear anywhere in the command line and are typically prefixed with `-` or `--`.
//!
//! # Core Types
//!
//! ## [`Opts`] - Optional Argument Trait
//!
//! The main trait for defining optional arguments. Typically implemented using
//! the `#[derive(Opts)]` macro for automatic parsing and validation.
//!
//! ## [`Opt`] - Option Specification
//!
//! Describes an individual option with its properties:
//! - Short and long forms (`-v`, `--verbose`)
//! - Value requirements (flag vs. valued option)
//! - Type information and parsing rules
//! - Help text and descriptions
//!
//! ## [`RawOpts<'a, S>`] - Raw Option Container
//!
//! Internal type that holds unparsed command-line options and manages parsing state.
//!
//! ## [`RawOptsIter<'a, S>`] - Option Iterator
//!
//! Iterator over raw command-line options for processing.
//!
//! # Option Types
//!
//! ## Boolean Flags
//! Simple on/off switches:
//! ```
//! # use shovel::*;
//! #[derive(Opts)]
//! struct MyOpts {
//!     /// Enable verbose output
//!     #[short = 'v']
//!     verbose: bool,
//! }
//! ```
//!
//! ## Valued Options
//! Options that accept values:
//! ```
//! # use shovel::*;
//!
//! #[derive(Opts)]
//! struct MyOpts {
//!     /// Output file path
//!     #[short = 'o']
//!     output: Option<String>,
//!
//!     /// Number of threads
//!     #[short = 'j']
//!     jobs: Option<u32>,
//!
//!     /// Tags
//!     tags: Option<Vec<String>>,
//! }
//! ```
//!
//! ## Required Options
//! Options that must be provided:
//! ```
//! # use shovel::*;
//!
//! #[derive(Opts)]
//! struct MyOpts {
//!     /// Configuration file (required)
//!     #[short = 'c']
//!     config: String,
//!
//!     /// Tags (required)
//!     tags: Vec<String>,
//! }
//! ```
//!
//! ## Multiple Values
//! Options that can be specified multiple times:
//! ```
//! # use shovel::*;
//! #[derive(Opts)]
//! struct MyOpts {
//!     /// Include directories
//!     #[short = 'I']
//!     #[format = "<DIR>,..."]
//!     include: String,
//! }
//! ```
//!
//! # Option Syntax
//!
//! ## Short Options
//! Single character options prefixed with `-`:
//! ```bash
//! myapp -v -o output.txt -j 4
//! myapp -vj 4 -o output.txt    # Short options can be combined
//! ```
//!
//! ## Long Options
//! Multi-character options prefixed with `--`:
//! ```bash
//! myapp --verbose --output output.txt --jobs 4
//! myapp --output=output.txt    # Values can use = syntax
//! ```
//!
//! ## Mixed Usage
//! Short and long forms can be used interchangeably:
//! ```bash
//! myapp -v --output output.txt -j 4
//! ```
//!
//! # Attribute Syntax
//!
//! The `#[derive(Opts)]` macro supports several attributes:
//!
//! ## `#[short = 'c']`
//! Defines the short form of the option:
//! ```
//! # use shovel::*;
//! #[derive(Opts)]
//! struct MyOpts {
//!     #[short = 'v']
//!     verbose: bool,
//! }
//! ```
//!
//! ## `#[short]`
//! Uses the first character of the field name:
//! ```
//! # use shovel::*;
//! #[derive(Opts)]
//! struct MyOpts {
//!     #[short]  // Becomes -v
//!     verbose: bool,
//! }
//! ```
//!
//!
//! # Built-in Options
//!
//! Shovel automatically provides several built-in options:
//!
//! ## Help Options
//! - `-h, --help` - Show help information
//! - Available on all commands automatically
//!
//! ## Version Options
//! - `-V, --version` - Show version information
//! - Available on the root application
//!
//! ## Completion Options (when enabled)
//! - `--completion <SHELL>` - Generate completion scripts
//! - Available when the `completion` feature is enabled
//!
//! # Parsing Behavior
//!
//! - **Type Safety**: Options are parsed to their target types with validation
//! - **Default Values**: `Option<T>` fields default to `None`, `bool` fields to `false`
//! - **Error Handling**: Invalid values or unknown options result in parse errors
//! - **Flexible Syntax**: Supports various command-line conventions
//!
//! # Global vs. Local Options
//!
//! ## Local Options
//! Specific to individual commands:
//! ```rust
//! # use shovel::*;
//! #[derive(Opts)]
//! struct BuildOpts {
//!     #[short = 'r']
//!     release: bool,
//! }
//! ```
//!
//! ## Global Options
//!
//! Available to all commands in the application:
//!
//! ```rust
//! use shovel::*;
//!
//! #[derive(Opts)]
//! pub struct GlobalOpts {
//!     /// Enable verbose output
//!     #[short = 'v']
//!     pub verbose: bool,
//!     /// Set log level
//!     #[short = 'l']
//!     pub log_level: Option<String>,
//! }
//!
//!
//! # fn main() {
//! let action: Action<(), (), GlobalOpts, ()> = Action::global_opts(|global_opts: GlobalOpts| {
//!     if global_opts.verbose {
//!         println!("Verbose mode enabled");
//!     }
//!     if let Some(level) = global_opts.log_level {
//!         println!("Setting log level to: {}", level);
//!         // Here you would typically set up your logging framework
//!     }
//! });
//! # }
//! ```
//!
//! # Examples
//!
//! ## Compiler-like Options
//! ```
//! # use shovel::*;
//! #[derive(Opts)]
//! struct CompilerOpts {
//!     /// Optimization level
//!     #[short = 'O']
//!     optimize: Option<u32>,
//!
//!     /// Enable warnings
//!     #[short = 'W']
//!     warnings: bool,
//!
//!     /// Output file
//!     #[short = 'o']
//!     output: Option<String>,
//!
//!     /// Include directories
//!     #[short = 'I']
//!     #[format = "<DIR>,..."]
//!     include: String,
//! }
//!
//! let action: Action<CompilerOpts, (), (), ()> = Action::opts(|opts: CompilerOpts| {
//!     if opts.warnings {
//!         println!("Warnings enabled");
//!     }
//!
//!     if let Some(level) = opts.optimize {
//!         println!("Optimization level: {}", level);
//!     }
//!
//!     for dir in opts.include.split(',') {
//!         println!("Include directory: {}", dir);
//!     }
//! });
//! ```
//!
//! ## Server Configuration Options
//! ```
//! # use shovel::*;
//! #[derive(Opts)]
//! struct ServerOpts {
//!     /// Server port
//!     #[short = 'p']
//!     port: Option<u16>,
//!
//!     /// Bind address
//!     #[short = 'b']
//!     bind: Option<String>,
//!
//!     /// Enable TLS
//!     #[short = 's']
//!     secure: bool,
//!
//!     /// Configuration file
//!     #[short = 'c']
//!     config: Option<String>,
//! }
//! ```
//!
//! See also: [`Args`](crate::Args), [`Action`](crate::Action), [`Context`](crate::Context)

use core::cell::{RefCell, RefMut};
use core::marker::PhantomData;

use crate::ParseError;
#[cfg(feature = "completion")]
use crate::Shell;

/// Represents a collection of optional arguments for a command.
///
/// This trait should be implemented by user-defined types that represent
/// the optional arguments of a command. Each field of the implementing type
/// corresponds to one optional argument.
///
/// - If a field is of type [`core::option::Option`], the argument is optional.
/// - Otherwise, the argument is required.
///
/// # Implementing
///
/// This trait can be derived using the `#[derive(Opts)]` attribute, which is
/// the recommended approach for most use cases. However, manual implementation
/// is possible for more complex scenarios.
///
/// # Example
///
/// ```no_run
/// use shovel::Opts;
///
/// #[derive(Opts)]
/// pub struct TickOpts {
///     /// The delay time (in seconds)
///     #[short = 'd']
///     pub delay: u64, // This field is required
///     /// Number of times
///     #[short = 'n']
///     pub number: Option<u32>, // The field is optional.
/// }
/// ```
///
/// # Manual Implementation Example
///
/// For advanced use cases, you can manually implement the `Opts` trait:
///
/// ```no_run
/// use shovel::{Opt, Opts, ParseError, RawOpts};
///
/// pub struct TickOpts {
///     pub delay: u64,
///     pub number: Option<u32>,
/// }
///
/// impl Opts for TickOpts {
///     fn parse<'a, 'b: 'a>(
///         opts: &'a RawOpts<'a, 'b>,
///     ) -> Result<(Self, usize), ParseError<'b>> {
///         let mut delay = None;
///         let mut number = None;
///
///         // Parse each option
///         for opt in opts {
///             match opt.0 {
///                 "delay" | "d" => {
///                     let value = opt.1.ok_or(ParseError::MissingValueForOption { name: "delay" })?;
///                     delay = Some(value.parse::<u64>().map_err(|_| {
///                         ParseError::InvalidOptionValue {
///                             name: "delay",
///                             value,
///                             expected: "a positive number",
///                         }
///                     })?);
///                 }
///                 "number" | "n" => {
///                     let value = opt.1.ok_or(ParseError::MissingValueForOption { name: "number" })?;
///                     number = Some(value.parse::<u32>().map_err(|_| {
///                         ParseError::InvalidOptionValue {
///                             name: "number",
///                             value,
///                             expected: "a positive integer",
///                         }
///                     })?);
///                 }
///                 unknown => return Err(ParseError::UnknownOption(unknown)),
///             }
///         }
///
///         // Validate required options
///         let delay = delay.ok_or(ParseError::MissingRequiredOption { name: "delay" })?;
///
///         Ok((Self { delay, number }, opts.len_consumed()))
///     }
///
///     fn specs() -> &'static [Opt] {
///         &[
///             Opt {
///                 name: "delay",
///                 ty: "u64",
///                 required: true,
///                 flag: false,
///                 description: "The delay time (in seconds)",
///                 short: Some('d'),
///                 format: "-d, --delay <SECONDS>",
///             },
///             Opt {
///                 name: "number",
///                 ty: "u32",
///                 required: false,
///                 flag: false,
///                 description: "Number of times to repeat",
///                 short: Some('n'),
///                 format: "-n, --number <COUNT>",
///             },
///         ]
///     }
/// }
/// ```
pub trait Opts: Sized + 'static {
    /// Parses the raw optional arguments into this type.
    ///
    /// # Arguments
    ///
    /// * `opts` - The raw optional arguments to parse.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing:
    /// - On success: A tuple with the parsed `Opts` instance and the number of consumed arguments.
    /// - On failure: A `ParseError` describing the parsing failure.
    ///
    /// # Errors
    ///
    /// This function may return a `ParseError` in the following cases:
    /// - If a required option is missing
    /// - If an option value is invalid or cannot be parsed
    /// - If an unknown option is encountered
    fn parse<'a, 'b: 'a>(
        opts: &'a RawOpts<'a, 'b>,
    ) -> Result<(Self, usize), ParseError<'b>>;

    /// Provides specifications for all optional arguments.
    ///
    /// # Returns
    ///
    /// A static slice of `Opt` structs, each describing an optional argument.
    fn specs() -> &'static [Opt];
}

/// Specifies the properties of an individual optional argument.
#[derive(Debug)]
pub struct Opt {
    /// The full name of the optional argument.
    pub name: &'static str,
    /// The expected type of the argument's value.
    pub ty: &'static str,
    /// Indicates whether the argument is required.
    pub required: bool,
    /// Indicates whether the argument is a flag (boolean, no value).
    pub flag: bool,
    /// A brief description of the argument's purpose.
    pub description: &'static str,
    /// The optional short (single-character) version of the argument.
    pub short: Option<char>,
    /// The formatted representation used in help messages.
    pub format: &'static str,
}

/// An iterator over the raw optional arguments.
///
/// This struct is used internally to iterate over the raw arguments
/// during the parsing process.
#[derive(Debug)]
pub struct RawOptsIter<'a, 'b: 'a> {
    args: &'b [&'b str],
    index: RefMut<'a, usize>,
    multi_flags_index: Option<usize>,
    specs: &'static [Opt],
    is_end: RefMut<'a, bool>,
}

/// Represents the raw optional arguments of a command.
///
/// This struct holds the unparsed arguments and provides methods
/// to iterate over them during the parsing process.
#[derive(Debug)]
pub struct RawOpts<'a, 'b: 'a> {
    args: &'b [&'b str],
    index: RefCell<usize>,
    specs: &'static [Opt],
    is_end: RefCell<bool>,
    _lifetime: PhantomData<&'a ()>,
}

pub(crate) struct HelpOpt;

pub(crate) struct VersionOpt;

#[cfg(feature = "completion")]
pub(crate) struct CompletionOpt;

const BUILTIN_OPTIONS: &[&str] = &[
    "help",
    "h",
    "version",
    "V",
    #[cfg(feature = "completion")]
    "completion",
];

impl Opts for () {
    fn parse<'a, 'b: 'a>(
        opts: &'a RawOpts<'a, 'b>,
    ) -> Result<(Self, usize), ParseError<'b>> {
        for opt in opts {
            if !BUILTIN_OPTIONS.contains(&opt.0) {
                Err(ParseError::UnknownOption(opt.0))?;
            }
        }
        Ok(((), opts.len_consumed()))
    }

    fn specs() -> &'static [Opt] {
        &[]
    }
}

pub(crate) static BUILTIN_OPTS: &[Opt] = &[
    Opt {
        name: "help",
        ty: "bool",
        required: false,
        flag: true,
        description: "Show this help",
        short: Some('h'),
        format: "-h, --help",
    },
    Opt {
        name: "version",
        ty: "bool",
        required: false,
        flag: true,
        description: "Print version information and quit",
        short: Some('V'),
        format: "-V, --version",
    },
    #[cfg(feature = "completion")]
    Opt {
        name: "completion",
        ty: "Shell:<bash|zsh|fish>",
        required: false,
        flag: false,
        description: "Generate shell completion script",
        short: None,
        format: "    --completion",
    },
];

impl HelpOpt {
    pub(crate) fn parse(opts: RawOpts<'_, '_>) -> bool {
        let mut help = false;
        for opt in &opts {
            if opt.0 == "help" || opt.0 == "h" {
                help = true;
                break;
            }
        }
        help
    }
}

impl VersionOpt {
    pub(crate) fn parse(opts: RawOpts<'_, '_>) -> bool {
        let mut version = false;
        for opt in &opts {
            if opt.0 == "version" || opt.0 == "V" {
                version = true;
                break;
            }
        }
        version
    }
}

#[cfg(feature = "completion")]
impl CompletionOpt {
    pub(crate) fn parse(
        opts: RawOpts<'_, '_>,
    ) -> Option<Result<Shell, ParseError<'static>>> {
        for opt in &opts {
            if opt.0 == "completion" {
                if let Some(s) = opt.1 {
                    return Some(s.parse().map_err(|_| {
                        ParseError::InvalidOptionValue {
                            name: "completion",
                            value: "Unknown shell",
                            expected: "a valid shell name",
                        }
                    }));
                }
                return Some(Err(ParseError::MissingValueForOption {
                    name: "completion",
                }));
            }
        }
        None
    }
}

impl<'a, 'b: 'a> RawOpts<'a, 'b> {
    pub(crate) fn new(args: &'b [&'b str], specs: &'static [Opt]) -> Self {
        Self {
            args,
            index: RefCell::new(0),
            specs,
            is_end: RefCell::new(false),
            _lifetime: PhantomData,
        }
    }

    /// Returns an iterator over the raw optional arguments.
    pub fn iter(&'a self) -> RawOptsIter<'a, 'b> {
        RawOptsIter::new(self)
    }

    /// Returns the number of raw arguments that has been consumed.
    pub fn len_consumed(&self) -> usize {
        debug_assert!(*self.is_end.borrow());
        *self.index.borrow()
    }
}

impl<'a, 'b: 'a> IntoIterator for &'a RawOpts<'a, 'b> {
    type Item = (&'b str, Option<&'b str>);
    type IntoIter = RawOptsIter<'a, 'b>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, 'b: 'a> RawOptsIter<'a, 'b> {
    fn new(raw_opts: &'a RawOpts<'a, 'b>) -> Self {
        Self {
            args: raw_opts.args,
            index: raw_opts.index.borrow_mut(),
            multi_flags_index: None,
            specs: raw_opts.specs,
            is_end: raw_opts.is_end.borrow_mut(),
        }
    }

    fn find_spec(&self, name: &str) -> Option<&'static Opt> {
        for spec in self.specs {
            if name == spec.name {
                return Some(spec);
            }
            if let Some(s) = spec.short
                && name.len() == s.len_utf8()
                && name.starts_with(s)
            {
                return Some(spec);
            }
        }
        None
    }
}

impl<'a, 'b: 'a> Iterator for RawOptsIter<'a, 'b> {
    type Item = (&'b str, Option<&'b str>);

    fn next(&mut self) -> Option<Self::Item> {
        if *self.is_end || *self.index >= self.args.len() {
            *self.is_end = true;
            return None;
        }

        let arg = self.args[*self.index];
        if !arg.starts_with('-') {
            *self.is_end = true;
            return None;
        }

        // End of options
        if arg == "--" {
            *self.index += 1;
            *self.is_end = true;
            return None;
        }

        if !arg.is_ascii() && arg.starts_with('-') && !arg.starts_with("--") {
            *self.index += 1;
            return Some((arg.strip_prefix('-').unwrap_or(arg), None));
        }

        let name = if let Some(name) = arg.strip_prefix("--") {
            // Handle --name=value syntax
            if let Some((name, value)) = name.split_once('=') {
                *self.index += 1;
                return Some((name, Some(value)));
            }
            name
        } else {
            if arg.len() > 2 {
                // Handle -n=value syntax for short options (only single character)
                if let Some(eq_pos) = arg.find('=')
                    && eq_pos == 2
                {
                    let name = &arg[1..2];
                    let value = &arg[eq_pos + 1..];
                    *self.index += 1;
                    return Some((name, Some(value)));
                }

                let index = self.multi_flags_index.unwrap_or(0);
                if index < arg.len() - 2 {
                    self.multi_flags_index = Some(index + 1);
                    return Some((&arg[index + 1..=index + 1], None));
                }
            }
            self.multi_flags_index = None;
            &arg[arg.len() - 1..]
        };

        *self.index += 1;

        match self.find_spec(name) {
            None => {
                if !BUILTIN_OPTIONS.contains(&name) {
                    *self.is_end = true;
                }
                Some((name, None))
            }
            Some(spec) => {
                if *self.index >= self.args.len() {
                    return Some((name, None));
                }

                let value = self.args[*self.index];
                if spec.flag || value.starts_with('-') {
                    return Some((name, None));
                }

                *self.index += 1;
                Some((name, Some(value)))
            }
        }
    }
}

/// A simple and fast config option parser.
///
/// # Example
///
/// ```edition2021
/// use shovel::*;
///
/// # fn main() {
/// App {
///     name: "config",
///     description: "An example of ConfigOpts",
///     version: "0.1.0",
///     authors: ["Author Name <author@example.com>"],
///     copyright: "(c) 2026 Company Name",
///     action: Action::opts(|opts: ConfigOpts| {
///         match opts.config {
///             Some(p) => println!("The path of config is '{}'", p.display()),
///             None => println!("No config provided")
///         }
///     }),
/// }
/// .run()
/// .expect("Failed to run application");
/// # }
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
#[derive(Debug)]
pub struct ConfigOpts {
    /// The path of config.
    pub config: Option<std::path::PathBuf>,
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl Opts for ConfigOpts {
    fn parse<'a, 'b: 'a>(
        opts: &'a RawOpts<'a, 'b>,
    ) -> Result<(Self, usize), ParseError<'b>> {
        let mut config = None;

        for opt in opts {
            if opt.0 == "config" || opt.0 == "c" {
                match opt.1 {
                    Some(s) => {
                        config =
                            Some(s.parse::<std::path::PathBuf>().map_err(
                                |_| ParseError::InvalidOptionValue {
                                    name: "config",
                                    value: s,
                                    expected: "a valid file path",
                                },
                            )?);
                    }
                    None => {
                        Err(ParseError::MissingValueForOption {
                            name: "config",
                        })?;
                    }
                }
            } else {
                Err(ParseError::UnknownOption(opt.0))?;
            }
        }

        Ok((Self { config }, opts.len_consumed()))
    }

    fn specs() -> &'static [Opt] {
        &[Opt {
            name: "config",
            description: "The path of config",
            ty: "Path",
            required: false,
            flag: false,
            short: Some('c'),
            format: "-c, --config",
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestOpts {
        a: bool,
        b: bool,
        c: Option<u32>,
    }

    impl Opts for TestOpts {
        fn specs() -> &'static [Opt] {
            &[
                Opt {
                    name: "a",
                    ty: "bool",
                    required: false,
                    flag: true,
                    description: "",
                    short: Some('a'),
                    format: "-a",
                },
                Opt {
                    name: "b",
                    ty: "bool",
                    required: false,
                    flag: true,
                    description: "",
                    short: Some('b'),
                    format: "-b",
                },
                Opt {
                    name: "c",
                    ty: "Option<u32>",
                    required: false,
                    flag: false,
                    description: "",
                    short: Some('c'),
                    format: "-c <u32>",
                },
            ]
        }

        fn parse<'a, 'b: 'a>(
            opts: &'a RawOpts<'a, 'b>,
        ) -> Result<(Self, usize), ParseError<'b>> {
            let mut a = false;
            let mut b = false;
            let mut c = None;

            for (name, value) in opts {
                match name {
                    "a" => match value {
                        Some(_) => Err(ParseError::UnexpectedFlagValue {
                            name: "a",
                            value: value.unwrap(),
                        })?,
                        None => a = true,
                    },
                    "b" => match value {
                        Some(_) => Err(ParseError::UnexpectedFlagValue {
                            name: "b",
                            value: value.unwrap(),
                        })?,
                        None => b = true,
                    },
                    "c" => {
                        let v =
                            value.ok_or(ParseError::MissingValueForOption {
                                name: "c",
                            })?;
                        c = Some(v.parse().map_err(|_| {
                            ParseError::InvalidOptionValue {
                                name: "c",
                                value: v,
                                expected: "u32",
                            }
                        })?);
                    }
                    _ => (), // Ignore unknown options in test
                }
            }

            Ok((TestOpts { a, b, c }, opts.len_consumed()))
        }
    }

    #[test]
    fn test_clustered_bool_flags() {
        let args = &["-ab"];
        let raw_opts = RawOpts::new(args, TestOpts::specs());
        let (opts, len) = TestOpts::parse(&raw_opts).unwrap();
        assert_eq!(
            opts,
            TestOpts {
                a: true,
                b: true,
                c: None
            }
        );
        assert_eq!(len, 1);
    }

    #[test]
    fn test_clustered_short_options() {
        let args = &["-ab", "-c", "123"];
        let raw_opts = RawOpts::new(args, TestOpts::specs());
        let (opts, len) = TestOpts::parse(&raw_opts).unwrap();
        assert_eq!(
            opts,
            TestOpts {
                a: true,
                b: true,
                c: Some(123)
            }
        );
        assert_eq!(len, 3);
    }

    #[test]
    fn test_clustered_short_options_with_value_at_end() {
        let args = &["-ac", "456"];
        let raw_opts = RawOpts::new(args, TestOpts::specs());
        let (opts, len) = TestOpts::parse(&raw_opts).unwrap();
        assert_eq!(
            opts,
            TestOpts {
                a: true,
                b: false,
                c: Some(456)
            }
        );
        assert_eq!(len, 2);
    }

    #[test]
    fn test_single_flag() {
        let args = &["-a"];
        let raw_opts = RawOpts::new(args, TestOpts::specs());
        let (opts, len) = TestOpts::parse(&raw_opts).unwrap();
        assert_eq!(
            opts,
            TestOpts {
                a: true,
                b: false,
                c: None
            }
        );
        assert_eq!(len, 1);
    }

    #[test]
    fn test_clustered_flags_and_value_option() {
        let args = &["-abc", "789"];
        let raw_opts = RawOpts::new(args, TestOpts::specs());
        let (opts, len) = TestOpts::parse(&raw_opts).unwrap();
        assert_eq!(
            opts,
            TestOpts {
                a: true,
                b: true,
                c: Some(789)
            }
        );
        assert_eq!(len, 2);
    }

    #[test]
    fn test_long_option_with_equals() {
        let args = &["--c=456"];
        let raw_opts = RawOpts::new(args, TestOpts::specs());
        let (opts, len) = TestOpts::parse(&raw_opts).unwrap();
        assert_eq!(
            opts,
            TestOpts {
                a: false,
                b: false,
                c: Some(456)
            }
        );
        assert_eq!(len, 1);
    }

    #[test]
    fn test_short_option_with_equals() {
        let args = &["-c=789"];
        let raw_opts = RawOpts::new(args, TestOpts::specs());
        let (opts, len) = TestOpts::parse(&raw_opts).unwrap();
        assert_eq!(
            opts,
            TestOpts {
                a: false,
                b: false,
                c: Some(789)
            }
        );
        assert_eq!(len, 1);
    }

    #[test]
    fn test_version_flag_uses_uppercase_v() {
        let args = &["-V"];
        let raw_opts = RawOpts::new(args, &[]);
        let ((), len) = <() as Opts>::parse(&raw_opts).unwrap();
        assert_eq!(len, 1);

        // Test that -v is not recognized as version (it would be unknown)
        let args = &["-v"];
        let raw_opts = RawOpts::new(args, TestOpts::specs());
        let (opts, len) = TestOpts::parse(&raw_opts).unwrap();
        assert_eq!(
            opts,
            TestOpts {
                a: false,
                b: false,
                c: None
            }
        );
        assert_eq!(len, 1);
    }
}
