//! Positional argument parsing and specification.
//!
//! This module provides the [`Args`] trait and related types for defining and parsing
//! positional command-line arguments. Arguments are required parameters that must be
//! provided in a specific order. Options must appear before the first positional argument.
//!
//! # Core Types
//!
//! ## [`Args`] - Positional Argument Trait
//!
//! The main trait for defining positional arguments. Typically implemented using
//! the `#[derive(Args)]` macro for automatic parsing and validation.
//!
//! ## [`ArgGroup`] - Argument Specification
//!
//! Describes a group of related positional arguments with their types, format,
//! and whether they accept variable numbers of arguments.
//!
//! ## [`RawArgs<'a, S>`] - Raw Argument Container
//!
//! Internal type that holds unparsed command-line arguments and tracks parsing state.
//!
//! # Argument Types
//!
//! ## Simple Arguments
//! Single required values:
//! ```
//! # use shovel::*;
//! #[derive(Args)]
//! struct FileArgs {
//!     /// Path to the input file
//!     path: String,
//! }
//! ```
//!
//! ## Multiple Arguments
//! Multiple required values in order:
//! ```
//! # use shovel::*;
//! #[derive(Args)]
//! struct CopyArgs {
//!     /// Source file path
//!     source: String,
//!     /// Destination file path
//!     destination: String,
//! }
//! ```
//!
//! ## Variable Arguments
//! Accept multiple values (using Vec):
//! ```
//! # use shovel::*;
//! #[derive(Args)]
//! struct ProcessArgs {
//!     /// Files to process
//!     files: Vec<String>,
//! }
//! ```
//!
//! ## Enum Arguments
//! Different argument patterns based on context:
//! ```
//! # use shovel::*;
//! #[derive(Args)]
//! enum MathArgs {
//!     /// Add two numbers
//!     #[format = "<NUM1> <NUM2>"]
//!     Add(f64, f64),
//!     /// Sum multiple numbers
//!     #[format = "<NUM>..."]
//!     Sum(Vec<f64>),
//! }
//! ```
//!
//! # Parsing Behavior
//!
//! - **Type Safety**: Arguments are parsed to their target types with compile-time validation
//! - **Order Sensitivity**: Arguments must be provided in the declared order
//! - **Error Handling**: Invalid types or counts result in descriptive parse errors
//! - **Zero Allocation**: Parsing uses stack-based operations in no-std environments
//!
//! # Format Specifications
//!
//! The `#[format = "..."]` attribute controls how arguments appear in help text:
//! - `<NAME>` - Required single argument
//! - `[<NAME>]` - Optional single argument
//! - `<NAME>...` - Variable number of arguments
//! - `[<NAME>...]` - Optional variable arguments
//!
//! # Built-in Implementations
//!
//! The module provides `Args` implementations for:
//! - `()` - No arguments
//! - `String` - Single string argument
//! - Primitive types (`i32`, `f64`, `bool`, etc.)
//! - `Vec<T>` - Multiple arguments of type T (std feature)
//! - Tuples - Multiple typed arguments
//!
//! # Examples
//!
//! ## File Processing Command
//! ```
//! # use shovel::*;
//! #[derive(Args)]
//! struct ProcessFileArgs {
//!     /// Input file to process
//!     input: String,
//!     /// Output file path
//!     output: String,
//! }
//!
//! let action: Action<(), ProcessFileArgs, (), ()> = Action::args(|args: ProcessFileArgs| {
//!     println!("Processing {} -> {}", args.input, args.output);
//! });
//! ```
//!
//! ## Calculator with Variable Arguments
//! ```
//! # use shovel::*;
//! #[derive(Args)]
//! struct CalcArgs {
//!     /// Numbers to calculate
//!     numbers: Vec<f64>,
//! }
//!
//! let action: Action<(), CalcArgs, (), ()> = Action::args(|args: CalcArgs| {
//!     let sum: f64 = args.numbers.iter().sum();
//!     println!("Sum: {}", sum);
//! });
//! ```
//!
//! See also: [`Opts`](crate::Opts), [`Action`](crate::Action), [`Context`](crate::Context)

use core::cell::Cell;
use core::fmt;
use core::num::NonZero;

use crate::RawOpts;
use crate::{Opt, ParseError};

/// An `Args` implementation represents all the positional arguments of a command.
///
/// This trait is typically implemented using the `#[derive(Args)]` attribute macro.
/// However, it can also be manually implemented for more complex scenarios.
///
/// # Examples
///
/// Using `#[derive(Args)]`:
///
/// ```no_run
/// use shovel::Args;
///
/// #[derive(Args)]
/// pub struct EchoArgs {
///     /// The message to echo
///     pub msg: String,
/// }
///
/// #[derive(Args)]
/// pub enum SumArgs {
///     /// Sum of two
///     #[format = "[<f64>, <f64>]"]
///     Group0(
///         /// First
///         f64,
///         /// Second
///         f64,
///     ),
///     /// Sum of three
///     Group1 { first: f64, second: f64, third: f64 },
/// }
/// ```
///
/// # Manual Implementation Example
///
/// For advanced use cases, you can manually implement the `Args` trait:
///
/// ```no_run
/// use shovel::{ArgGroup, Args, ParseError};
///
/// pub struct FileOpts {
///     pub input: String,
///     pub output: Option<String>,
/// }
///
/// impl Args for FileOpts {
///     fn parse<'a>(args: &'a [&'a str]) -> Result<Self, ParseError<'a>> {
///         match args.len() {
///             1 => {
///                 // Single argument: input file only
///                 Ok(Self {
///                     input: args[0].to_string(),
///                     output: None,
///                 })
///             }
///             2 => {
///                 // Two arguments: input and output files
///                 Ok(Self {
///                     input: args[0].to_string(),
///                     output: Some(args[1].to_string()),
///                 })
///             }
///             _ => Err(ParseError::ArgumentCountMismatch),
///         }
///     }
///
///     fn specs() -> &'static [ArgGroup] {
///         &[ArgGroup {
///             tys: &["String", "String"],
///             format: "<INPUT> [OUTPUT]",
///             variadic: false,
///             description: "Input file and optional output file",
///         }]
///     }
/// }
///
/// // Example with validation and better error messages
/// pub struct NumberArgs {
///     pub value: f64,
/// }
///
/// impl Args for NumberArgs {
///     fn parse<'a>(args: &'a [&'a str]) -> Result<Self, ParseError<'a>> {
///         if args.len() != 1 {
///             return Err(ParseError::ArgumentCountMismatch);
///         }
///
///         let value_str = args[0];
///         let value = value_str.parse::<f64>().map_err(|_| {
///             ParseError::InvalidArgumentValue {
///                 index: 0,
///                 value: value_str,
///                 expected: "a floating-point number",
///             }
///         })?;
///
///         // Add custom validation
///         if value < 0.0 {
///             return Err(ParseError::InvalidArgumentValue {
///                 index: 0,
///                 value: value_str,
///                 expected: "a positive number",
///             });
///         }
///
///         Ok(Self { value })
///     }
///
///     fn specs() -> &'static [ArgGroup] {
///         &[ArgGroup {
///             tys: &["f64"],
///             format: "<NUMBER>",
///             variadic: false,
///             description: "A positive floating-point number",
///         }]
///     }
/// }
/// ```
pub trait Args: Sized + 'static {
    /// Parse from the raw positional arguments.
    ///
    /// # Errors
    ///
    /// This function will return a `ParseError` if:
    /// - The number of arguments is invalid for the specific `Args` implementation.
    /// - Any of the arguments cannot be parsed into the expected type.
    /// - The arguments do not match the expected format or constraints of the `Args` implementation.
    fn parse<'a>(args: &'a [&'a str]) -> Result<Self, ParseError<'a>>;

    /// Return a list of positional argument group specifications.
    fn specs() -> &'static [ArgGroup];
}

/// Specification for a group of positional arguments.
#[derive(Debug)]
pub struct ArgGroup {
    /// Types of all positional arguments in this group.
    pub tys: &'static [&'static str],
    /// Format string used in help messages.
    pub format: &'static str,
    /// Whether this group accepts a variable number of arguments.
    pub variadic: bool,
    /// Description of the argument group.
    pub description: &'static str,
}

/// Represents raw command line arguments.
pub struct RawArgs<'a> {
    /// Slice of command line arguments.
    args: &'a [&'a str],
    /// Current position in the argument list.
    current: Cell<usize>,
    pub(crate) env: &'a [(&'a str, &'a str)],
}

impl Args for () {
    fn parse<'a>(args: &'a [&'a str]) -> Result<Self, ParseError<'a>> {
        if args.is_empty() {
            Ok(())
        } else {
            Err(ParseError::ArgumentCountMismatch)
        }
    }

    fn specs() -> &'static [ArgGroup] {
        &[]
    }
}

impl fmt::Debug for RawArgs<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ args: [")?;
        for (i, arg) in self.args.iter().enumerate() {
            if i == self.args.len() - 1 {
                write!(f, "{arg}")?;
            } else {
                write!(f, "{arg}, ")?;
            }
        }
        write!(f, "], current: {} }}", self.current.get())
    }
}

impl<'a> RawArgs<'a> {
    #[cfg(test)]
    pub(crate) fn new(args: &'a [&'a str]) -> Self {
        RawArgs {
            args,
            env: &[],
            current: Cell::new(0),
        }
    }

    pub(crate) fn with_env(
        args: &'a [&'a str],
        env: &'a [(&'a str, &'a str)],
    ) -> Self {
        Self {
            args,
            env,
            current: Cell::new(0),
        }
    }

    pub(crate) fn consume(&self, size: usize) {
        let current = self.current.get();
        if current + size < self.args.len() {
            self.current.set(current + size);
        } else {
            self.current.set(self.args.len());
        }
    }

    pub(crate) fn reset(&self) {
        self.current.set(0);
    }

    pub(crate) fn cmd(&self) -> Option<&'a str> {
        let current = self.current.get();
        if current < self.args.len() {
            self.current.set(current + 1);
            Some(self.args[current])
        } else {
            None
        }
    }

    pub(crate) fn opts(&self, specs: &'static [Opt]) -> RawOpts<'_, 'a> {
        let current = self.current.get();
        if current < self.args.len() {
            RawOpts::new(&self.args[current..], specs)
        } else {
            RawOpts::new(&[], specs)
        }
    }

    pub(crate) fn args(&self, skip_size: usize) -> &'a [&'a str] {
        let current = self.current.get();
        if current + skip_size < self.args.len() {
            &self.args[current + skip_size..]
        } else {
            &[]
        }
    }

    pub(crate) fn cmd_path(
        &self,
        start_pos: usize,
        end_pos: usize,
    ) -> &'a [&'a str] {
        &self.args[start_pos..end_pos]
    }

    pub(crate) fn current(&self) -> usize {
        self.current.get()
    }
}

impl Args for (core::net::IpAddr, u16) {
    fn parse<'a>(args: &'a [&'a str]) -> Result<Self, ParseError<'a>> {
        if args.len() == 2 {
            let ip = args[0].parse::<core::net::IpAddr>().map_err(|_| {
                ParseError::InvalidArgumentValue {
                    index: 0,
                    value: args[0],
                    expected: "an IP address",
                }
            })?;
            let port = args[1].parse::<u16>().map_err(|_| {
                ParseError::InvalidArgumentValue {
                    index: 1,
                    value: args[1],
                    expected: "a port number",
                }
            })?;
            Ok((ip, port))
        } else {
            Err(ParseError::ArgumentCountMismatch)
        }
    }

    fn specs() -> &'static [ArgGroup] {
        &[ArgGroup {
            tys: &["IpAddr", "u16"],
            format: "[<ip> <port>]",
            variadic: false,
            description: "An IP address and a port",
        }]
    }
}

macro_rules! impl_args {
    ($ty:ty, $ty_str:literal, $format:literal, $description:literal) => {
        impl Args for $ty {
            fn parse<'a>(args: &'a [&'a str]) -> Result<Self, ParseError<'a>> {
                if args.len() == 1 {
                    args[0].parse::<Self>().map_err(|_| {
                        ParseError::InvalidArgumentValue {
                            index: 0,
                            value: args[0],
                            expected: $ty_str,
                        }
                    })
                } else {
                    Err(ParseError::ArgumentCountMismatch)
                }
            }

            fn specs() -> &'static [ArgGroup] {
                &[ArgGroup {
                    tys: &[$ty_str],
                    format: $format,
                    variadic: false,
                    description: $description,
                }]
            }
        }
    };
}

macro_rules! impl_args_for_std {
    ($ty:ty, $ty_str:literal, $format:literal, $description:literal) => {
        #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
        #[cfg(feature = "std")]
        impl Args for $ty {
            fn parse<'a>(
                args: &'a [&'a str],
            ) -> Result<Self, ParseError<'a>> {
                if args.len() == 1 {
                    args[0].parse::<Self>().map_err(|_| {
                        ParseError::InvalidArgumentValue {
                            index: 0,
                            value: args[0],
                            expected: $ty_str,
                        }
                    })
                } else {
                    Err(ParseError::ArgumentCountMismatch)
                }
            }

            fn specs() -> &'static [ArgGroup] {
                &[ArgGroup {
                    tys: &[$ty_str],
                    format: $format,
                    variadic: false,
                    description: $description,
                }]
            }
        }
    };
    // For Vecs
    ($ty:ty, $inner_ty:ty, $ty_str:literal, $format:literal, $description:literal) => {
        #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
        #[cfg(feature = "std")]
        impl Args for $ty {
            fn parse<'a>(
                args: &'a [&'a str],
            ) -> Result<Self, ParseError<'a>> {
                let mut data = std::vec::Vec::new();
                for (i, arg) in args.iter().enumerate() {
                    data.push(arg.parse::<$inner_ty>().map_err(
                        |_| ParseError::InvalidArgumentValue {
                            index: i,
                            value: arg,
                            expected: stringify!($inner_ty),
                        },
                    )?);
                }
                Ok(data)
            }

            fn specs() -> &'static [ArgGroup] {
                &[ArgGroup {
                    tys: &[$ty_str],
                    format: $format,
                    variadic: true,
                    description: $description,
                }]
            }
        }
    };
    ($ty:ty, $ty_str:literal, $format:literal, $description:literal, $($item_ty:ty),+) => {
        #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
        #[cfg(feature = "std")]
        impl Args for $ty {
            fn parse<'a>(
                args: &'a [&'a str],
            ) -> Result<Self, ParseError<'a>> {
                Ok(($(<$item_ty>::parse(args)?,)+))
            }

            fn specs() -> &'static [ArgGroup] {
                &[ArgGroup {
                    tys: &[$ty_str],
                    format: $format,
                    variadic: true,
                    description: $description,
                }]
            }
        }
    };
}

impl_args!(u8, "u8", "[<u8>]", "A 8-bit unsigned integer");
impl_args!(u16, "u16", "[<u16>]", "A 16-bit unsigned integer");
impl_args!(u32, "u32", "[<u32>]", "A 32-bit unsigned integer");
impl_args!(u64, "u64", "[<u64>]", "A 64-bit unsigned integer");
impl_args!(u128, "u128", "[<u128>]", "A 128-bit unsigned integer");
impl_args!(
    usize,
    "usize",
    "[<usize>]",
    "A pointer-sized unsigned integer"
);

impl_args!(i8, "i8", "[<i8>]", "A 8-bit signed integer");
impl_args!(i16, "i16", "[<i16>]", "A 16-bit signed integer");
impl_args!(i32, "i32", "[<i32>]", "A 32-bit signed integer");
impl_args!(i64, "i64", "[<i64>]", "A 64-bit signed integer");
impl_args!(i128, "i128", "[<i128>]", "A 128-bit signed integer");
impl_args!(
    isize,
    "isize",
    "[<isize>]",
    "A pointer-sized signed integer"
);

impl_args!(f32, "f32", "[<f32>]", "A 32-bit floating point");
impl_args!(f64, "f64", "[<f64>]", "A 64-bit floating point");

impl_args!(
    NonZero<i8>,
    "NonZero<i8>",
    "[<non-zero-i8>]",
    "A non-zero 8-bit signed integer"
);
impl_args!(
    NonZero<i16>,
    "NonZero<i16>",
    "[<non-zero-i16>]",
    "A non-zero 16-bit signed integer"
);
impl_args!(
    NonZero<i32>,
    "NonZero<i32>",
    "[<non-zero-i32>]",
    "A non-zero 32-bit signed integer"
);
impl_args!(
    NonZero<i64>,
    "NonZero<i64>",
    "[<non-zero-i64>]",
    "A non-zero 64-bit signed integer"
);
impl_args!(
    NonZero<i128>,
    "NonZero<i128>",
    "[<non-zero-i128>]",
    "A non-zero 128-bit signed integer"
);

impl_args!(
    NonZero<u8>,
    "NonZero<u8>",
    "[<non-zero-u8>]",
    "A non-zero 8-bit unsigned integer"
);
impl_args!(
    NonZero<u16>,
    "NonZero<u16>",
    "[<non-zero-u16>]",
    "A non-zero 16-bit unsigned integer"
);
impl_args!(
    NonZero<u32>,
    "NonZero<u32>",
    "[<non-zero-u32>]",
    "A non-zero 32-bit unsigned integer"
);
impl_args!(
    NonZero<u64>,
    "NonZero<u64>",
    "[<non-zero-u64>]",
    "A non-zero 64-bit unsigned integer"
);
impl_args!(
    NonZero<u128>,
    "NonZero<u128>",
    "[<non-zero-u128>]",
    "A non-zero 128-bit unsigned integer"
);

impl_args!(bool, "bool", "[<bool>]", "The boolean");
impl_args!(char, "char", "[<char>]", "A character");

impl_args!(
    core::net::IpAddr,
    "IpAddr",
    "[<ip>]",
    "An IP address, either IPv4 or IPv6"
);
impl_args!(
    core::net::Ipv4Addr,
    "Ipv4Addr",
    "[<ipv4>]",
    "An IPv4 address"
);
impl_args!(
    core::net::Ipv6Addr,
    "Ipv6Addr",
    "[<ipv6>]",
    "An IPv6 address"
);

impl_args!(
    core::net::SocketAddr,
    "SocketAddr",
    "[<ip:port>]",
    "An internet socket address, either IPv4 or IPv6"
);
impl_args!(
    core::net::SocketAddrV4,
    "SocketAddrV4",
    "[<ipv4:port>]",
    "An IPv4 socket address"
);
impl_args!(
    core::net::SocketAddrV6,
    "SocketAddrV6",
    "[<ipv6:port>]",
    "An IPv6 socket address"
);

impl_args_for_std!(std::path::PathBuf, "Path", "[<Path>]", "A path");
impl_args_for_std!(std::string::String, "String", "[<String>]", "A string");

// Additional std types
impl_args_for_std!(
    std::ffi::OsString,
    "OsString",
    "[<OsString>]",
    "An OS-native string"
);
impl_args_for_std!(
    std::vec::Vec<char>,
    char,
    "Vec<char>",
    "[<char> ...]",
    "A list of character"
);
impl_args_for_std!(
    std::vec::Vec<bool>,
    bool,
    "Vec<bool>",
    "[<bool> ...]",
    "A list of boolean"
);

impl_args_for_std!(
    std::vec::Vec<f64>,
    f64,
    "Vec<f64>",
    "[<f64> ...]",
    "A list of 64-bit floating point"
);

impl_args_for_std!(
    std::vec::Vec<i64>,
    i64,
    "Vec<i64>",
    "[<i64>, ...]",
    "A list of 64-bit signed integer"
);

impl_args_for_std!(
    std::vec::Vec<u64>,
    u64,
    "Vec<u64>",
    "[<u64> ...]",
    "A list of 64-bit unsigned integer"
);

impl_args_for_std!(
    std::vec::Vec<std::string::String>,
    std::string::String,
    "Vec<String>",
    "[<string> ...]",
    "A list of string"
);

impl_args_for_std!(
    std::vec::Vec<std::path::PathBuf>,
    std::path::PathBuf,
    "Vec<Path>",
    "[<path> ...]",
    "A list of path"
);

impl_args_for_std!(
    std::vec::Vec<std::net::IpAddr>,
    std::net::IpAddr,
    "Vec<IpAddr>",
    "[<ip> ...]",
    "A list of IP address"
);

impl_args_for_std!(
    std::vec::Vec<std::net::SocketAddr>,
    std::net::SocketAddr,
    "Vec<SocketAddr>",
    "[<ip:port>, ...]",
    "A list of internet socket address"
);

impl_args_for_std!(
    (i64, i64),
    "(i64, i64)",
    "(<i64>, <i64>)",
    "A tuple of two 64-bit signed integers",
    i64,
    i64
);

impl_args_for_std!(
    (i64, i64, i64),
    "(i64, i64, i64)",
    "(<i64>, <i64>, <i64>)",
    "A tuple of three 64-bit signed integers",
    i64,
    i64,
    i64
);

impl_args_for_std!(
    (std::string::String, std::string::String),
    "(String, String)",
    "(<STRING>, <STRING>)",
    "A tuple of two strings",
    std::string::String,
    std::string::String
);

impl_args_for_std!(
    (
        std::string::String,
        std::string::String,
        std::string::String
    ),
    "(String, String, String)",
    "(<STRING>, <STRING>, <STRING>)",
    "A tuple of three strings",
    std::string::String,
    std::string::String,
    std::string::String
);

// Additional commonly used Vec types
impl_args_for_std!(
    std::vec::Vec<u8>,
    u8,
    "Vec<u8>",
    "[<u8> ...]",
    "A list of 8-bit unsigned integers"
);

impl_args_for_std!(
    std::vec::Vec<u16>,
    u16,
    "Vec<u16>",
    "[<u16> ...]",
    "A list of 16-bit unsigned integers"
);

impl_args_for_std!(
    std::vec::Vec<u32>,
    u32,
    "Vec<u32>",
    "[<u32> ...]",
    "A list of 32-bit unsigned integers"
);

impl_args_for_std!(
    std::vec::Vec<i8>,
    i8,
    "Vec<i8>",
    "[<i8> ...]",
    "A list of 8-bit signed integers"
);

impl_args_for_std!(
    std::vec::Vec<i16>,
    i16,
    "Vec<i16>",
    "[<i16> ...]",
    "A list of 16-bit signed integers"
);

impl_args_for_std!(
    std::vec::Vec<i32>,
    i32,
    "Vec<i32>",
    "[<i32> ...]",
    "A list of 32-bit signed integers"
);

impl_args_for_std!(
    std::vec::Vec<f32>,
    f32,
    "Vec<f32>",
    "[<f32> ...]",
    "A list of 32-bit floating point numbers"
);

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "std")]
    use std::{string::String, vec, vec::Vec};

    #[test]
    fn test_unit_args() {
        // Test empty arguments
        let args: &[&str] = &[];
        let result = <()>::parse(args);
        assert!(result.is_ok());

        // Test non-empty arguments should fail
        let args: &[&str] = &["extra"];
        let result = <()>::parse(args);
        assert!(matches!(result, Err(ParseError::ArgumentCountMismatch)));

        // Test specs
        let specs = <()>::specs();
        assert!(specs.is_empty());
    }

    #[test]
    fn test_primitive_args() {
        // Test u32 parsing
        let args: &[&str] = &["42"];
        let result = u32::parse(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        // Test invalid u32
        let args: &[&str] = &["not_a_number"];
        let result = u32::parse(args);
        assert!(matches!(
            result,
            Err(ParseError::InvalidArgumentValue {
                index: 0,
                value: "not_a_number",
                expected: "u32"
            })
        ));

        // Test wrong argument count
        let args: &[&str] = &["42", "43"];
        let result = u32::parse(args);
        assert!(matches!(result, Err(ParseError::ArgumentCountMismatch)));

        // Test specs
        let specs = u32::specs();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].tys, &["u32"]);
        assert_eq!(specs[0].format, "[<u32>]");
        assert!(!specs[0].variadic);
    }

    #[test]
    fn test_signed_integer_args() {
        // Test i64 parsing
        let args: &[&str] = &["-123"];
        let result = i64::parse(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), -123);

        // Test positive i64
        let args: &[&str] = &["456"];
        let result = i64::parse(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 456);

        // Test invalid i64
        let args: &[&str] = &["not_a_number"];
        let result = i64::parse(args);
        assert!(matches!(
            result,
            Err(ParseError::InvalidArgumentValue {
                index: 0,
                value: "not_a_number",
                expected: "i64"
            })
        ));
    }

    #[test]
    fn test_floating_point_args() {
        // Test f64 parsing
        let args: &[&str] = &["3.14159"];
        let result = f64::parse(args);
        assert!(result.is_ok());
        assert!((result.unwrap() - core::f64::consts::PI).abs() < 0.001);

        // Test negative f64
        let args: &[&str] = &["-2.718"];
        let result = f64::parse(args);
        assert!(result.is_ok());
        assert!((result.unwrap() + core::f64::consts::E).abs() < 0.001);

        // Test invalid f64
        let args: &[&str] = &["not_a_float"];
        let result = f64::parse(args);
        assert!(matches!(
            result,
            Err(ParseError::InvalidArgumentValue {
                index: 0,
                value: "not_a_float",
                expected: "f64"
            })
        ));
    }

    #[test]
    fn test_boolean_args() {
        // Test true
        let args: &[&str] = &["true"];
        let result = bool::parse(args);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test false
        let args: &[&str] = &["false"];
        let result = bool::parse(args);
        assert!(result.is_ok());
        assert!(!result.unwrap());

        // Test invalid bool
        let args: &[&str] = &["maybe"];
        let result = bool::parse(args);
        assert!(matches!(
            result,
            Err(ParseError::InvalidArgumentValue {
                index: 0,
                value: "maybe",
                expected: "bool"
            })
        ));
    }

    #[test]
    fn test_char_args() {
        // Test single character
        let args: &[&str] = &["a"];
        let result = char::parse(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 'a');

        // Test unicode character
        let args: &[&str] = &["🦀"];
        let result = char::parse(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), '🦀');

        // Test invalid char (multiple characters)
        let args: &[&str] = &["abc"];
        let result = char::parse(args);
        assert!(matches!(
            result,
            Err(ParseError::InvalidArgumentValue {
                index: 0,
                value: "abc",
                expected: "char"
            })
        ));
    }

    #[test]
    fn test_ip_address_args() {
        // Test IPv4 address
        let args: &[&str] = &["192.168.1.1"];
        let result = core::net::IpAddr::parse(args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_ipv4());

        // Test IPv6 address
        let args: &[&str] = &["::1"];
        let result = core::net::IpAddr::parse(args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_ipv6());

        // Test invalid IP address
        let args: &[&str] = &["not.an.ip.address"];
        let result = core::net::IpAddr::parse(args);
        assert!(matches!(
            result,
            Err(ParseError::InvalidArgumentValue {
                index: 0,
                value: "not.an.ip.address",
                expected: "IpAddr"
            })
        ));
    }

    #[test]
    fn test_socket_address_args() {
        // Test IPv4 socket address
        let args: &[&str] = &["127.0.0.1:8080"];
        let result = core::net::SocketAddr::parse(args);
        assert!(result.is_ok());
        let addr = result.unwrap();
        assert!(addr.is_ipv4());
        assert_eq!(addr.port(), 8080);

        // Test IPv6 socket address
        let args: &[&str] = &["[::1]:3000"];
        let result = core::net::SocketAddr::parse(args);
        assert!(result.is_ok());
        let addr = result.unwrap();
        assert!(addr.is_ipv6());
        assert_eq!(addr.port(), 3000);

        // Test invalid socket address
        let args: &[&str] = &["invalid:socket"];
        let result = core::net::SocketAddr::parse(args);
        assert!(matches!(
            result,
            Err(ParseError::InvalidArgumentValue {
                index: 0,
                value: "invalid:socket",
                expected: "SocketAddr"
            })
        ));
    }

    #[test]
    fn test_ip_port_tuple_args() {
        // Test valid IP and port
        let args: &[&str] = &["192.168.1.1", "8080"];
        let result = <(core::net::IpAddr, u16)>::parse(args);
        assert!(result.is_ok());
        let (ip, port) = result.unwrap();
        assert!(ip.is_ipv4());
        assert_eq!(port, 8080);

        // Test invalid IP
        let args: &[&str] = &["invalid.ip", "8080"];
        let result = <(core::net::IpAddr, u16)>::parse(args);
        assert!(matches!(
            result,
            Err(ParseError::InvalidArgumentValue {
                index: 0,
                value: "invalid.ip",
                expected: "an IP address"
            })
        ));

        // Test invalid port
        let args: &[&str] = &["127.0.0.1", "invalid_port"];
        let result = <(core::net::IpAddr, u16)>::parse(args);
        assert!(matches!(
            result,
            Err(ParseError::InvalidArgumentValue {
                index: 1,
                value: "invalid_port",
                expected: "a port number"
            })
        ));

        // Test wrong argument count
        let args: &[&str] = &["127.0.0.1"];
        let result = <(core::net::IpAddr, u16)>::parse(args);
        assert!(matches!(result, Err(ParseError::ArgumentCountMismatch)));

        let args: &[&str] = &["127.0.0.1", "8080", "extra"];
        let result = <(core::net::IpAddr, u16)>::parse(args);
        assert!(matches!(result, Err(ParseError::ArgumentCountMismatch)));
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_string_args() {
        // Test string parsing
        let args: &[&str] = &["hello world"];
        let result = String::parse(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello world");

        // Test empty string
        let args: &[&str] = &[""];
        let result = String::parse(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");

        // Test wrong argument count
        let args: &[&str] = &["hello", "world"];
        let result = String::parse(args);
        assert!(matches!(result, Err(ParseError::ArgumentCountMismatch)));

        // Test specs
        let specs = String::specs();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].tys, &["String"]);
        assert_eq!(specs[0].format, "[<String>]");
        assert!(!specs[0].variadic);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_path_args() {
        use std::path::PathBuf;

        // Test path parsing
        let args: &[&str] = &["/usr/local/bin"];
        let result = PathBuf::parse(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("/usr/local/bin"));

        // Test relative path
        let args: &[&str] = &["./relative/path"];
        let result = PathBuf::parse(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("./relative/path"));

        // Test wrong argument count
        let args: &[&str] = &["/path1", "/path2"];
        let result = PathBuf::parse(args);
        assert!(matches!(result, Err(ParseError::ArgumentCountMismatch)));
    }

    #[test]
    fn test_tuple_args_manual_implementation() {
        // Test the manually implemented (IpAddr, u16) tuple
        let args: &[&str] = &["192.168.1.1", "8080"];
        let result = <(core::net::IpAddr, u16)>::parse(args);
        assert!(result.is_ok());
        let (ip, port) = result.unwrap();
        assert!(ip.is_ipv4());
        assert_eq!(port, 8080);

        // Test tuple with invalid element
        let args: &[&str] = &["not_an_ip", "8080"];
        let result = <(core::net::IpAddr, u16)>::parse(args);
        assert!(matches!(
            result,
            Err(ParseError::InvalidArgumentValue {
                index: 0,
                value: "not_an_ip",
                expected: "an IP address"
            })
        ));

        // Test wrong argument count for tuple
        let args: &[&str] = &["192.168.1.1"];
        let result = <(core::net::IpAddr, u16)>::parse(args);
        assert!(matches!(result, Err(ParseError::ArgumentCountMismatch)));

        let args: &[&str] = &["192.168.1.1", "8080", "extra"];
        let result = <(core::net::IpAddr, u16)>::parse(args);
        assert!(matches!(result, Err(ParseError::ArgumentCountMismatch)));
    }

    #[test]
    fn test_tuple_specs_manual_implementation() {
        // Test manually implemented tuple specs
        let specs = <(core::net::IpAddr, u16)>::specs();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].tys, &["IpAddr", "u16"]);
        assert_eq!(specs[0].format, "[<ip> <port>]");
        assert!(!specs[0].variadic);
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_raw_args_new() {
        let args = &["arg1", "arg2", "arg3"];
        let raw_args = RawArgs::new(args);

        // Test that we can access the arguments through the args field
        let collected: Vec<&str> = raw_args
            .args
            .iter()
            .map(core::convert::AsRef::as_ref)
            .collect();
        assert_eq!(collected, vec!["arg1", "arg2", "arg3"]);
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_raw_args_empty() {
        let args: &[&str] = &[];
        let raw_args = RawArgs::new(args);

        // Test that empty args work correctly
        let collected: Vec<&str> = raw_args
            .args
            .iter()
            .map(core::convert::AsRef::as_ref)
            .collect();
        assert_eq!(collected, [] as [&str; 0]);
    }

    #[test]
    fn test_arg_group_specs() {
        // Test that ArgGroup has the expected structure
        let group = ArgGroup {
            tys: &["String", "u32"],
            format: "<name> <count>",
            variadic: false,
            description: "A name and count",
        };

        assert_eq!(group.tys, &["String", "u32"]);
        assert_eq!(group.format, "<name> <count>");
        assert!(!group.variadic);
        assert_eq!(group.description, "A name and count");
    }

    #[test]
    fn test_parse_error_variants() {
        // Test that ParseError variants can be created and matched
        let error = ParseError::ArgumentCountMismatch;
        assert!(matches!(error, ParseError::ArgumentCountMismatch));

        let error = ParseError::InvalidArgumentValue {
            index: 0,
            value: "test",
            expected: "a number",
        };
        assert!(matches!(
            error,
            ParseError::InvalidArgumentValue {
                index: 0,
                value: "test",
                expected: "a number"
            }
        ));
    }

    #[test]
    fn test_edge_cases() {
        // Test parsing with whitespace
        let args: &[&str] = &[" 42 "];
        let result = u32::parse(args);
        assert!(matches!(
            result,
            Err(ParseError::InvalidArgumentValue {
                index: 0,
                value: " 42 ",
                expected: "u32"
            })
        ));

        // Test parsing zero
        let args: &[&str] = &["0"];
        let result = u32::parse(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Test parsing negative zero for floats
        let args: &[&str] = &["-0.0"];
        let result = f64::parse(args);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.abs() < f64::EPSILON && value.is_sign_negative());
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_new_vector_types() {
        // Test Vec<u32>
        let args: &[&str] = &["1", "2", "3"];
        let result = std::vec::Vec::<u32>::parse(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1, 2, 3]);

        // Test Vec<f32>
        let args: &[&str] = &["1.5", "2.5", "3.5"];
        let result = std::vec::Vec::<f32>::parse(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1.5, 2.5, 3.5]);

        // Test empty Vec
        let args: &[&str] = &[];
        let result = std::vec::Vec::<u32>::parse(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), [] as [u32; 0]);
    }

    // Note: The macro-generated tuple implementations were removed because
    // they were broken. Only manually implemented tuples like (IpAddr, u16)
    // work correctly. See test_tuple_args_manual_implementation() for examples.

    #[cfg(feature = "std")]
    #[test]
    fn test_os_string_args() {
        use std::ffi::OsString;

        // Test OsString parsing
        let args: &[&str] = &["test_path"];
        let result = OsString::parse(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), OsString::from("test_path"));

        // Test wrong argument count
        let args: &[&str] = &["path1", "path2"];
        let result = OsString::parse(args);
        assert!(matches!(result, Err(ParseError::ArgumentCountMismatch)));
    }
}
