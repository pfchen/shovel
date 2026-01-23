//! Error types and parsing error handling.
//!
//! This module provides comprehensive error types for command-line argument parsing failures.
//! The error system is designed to provide detailed, user-friendly error messages that help
//! users understand and correct their command-line input mistakes.
//!
//! # Core Type
//!
//! ## [`ParseError<'a>`] - Parsing Error Enumeration
//!
//! The main error type that represents all possible parsing failures:
//! - **Lifetime `'a`**: References to the original command-line input for zero-copy error reporting
//! - **Detailed Variants**: Specific error types for different failure modes
//! - **User-Friendly**: Error messages designed for end-user consumption
//!
//! # Error Categories
//!
//! ## Option Errors
//! Errors related to command-line options (flags and named parameters):
//!
//! ### `UnknownOption(&'a str)`
//! An unrecognized option was provided:
//! ```text
//! error: unknown option '--invalid'
//! ```
//!
//! ### `UnexpectedFlagValue { name, value }`
//! A boolean flag was incorrectly given a value:
//! ```text
//! error: flag '--verbose' does not accept a value, but 'true' was provided
//! ```
//!
//! ### `InvalidOptionValue { name, value, expected }`
//! An option value could not be parsed to the expected type:
//! ```text
//! error: invalid value 'abc' for option '--count': expected a number
//! ```
//!
//! ### `MissingValueForOption { name }`
//! A required option value was not provided:
//! ```text
//! error: option '--output' requires a value
//! ```
//!
//! ## Argument Errors
//! Errors related to positional arguments:
//!
//! ### `ArgumentCountMismatch`
//! Wrong number of positional arguments provided:
//! ```text
//! error: expected 2 arguments, but 3 were provided
//! ```
//!
//! ### `InvalidArgumentValue { index, value, expected }`
//! An argument value could not be parsed to the expected type:
//! ```text
//! error: invalid argument 'not-a-number': expected a floating-point number
//! ```
//!
//! ## Command Errors
//! Errors related to command and subcommand resolution:
//!
//!
//! # Error Display
//!
//! All errors implement `Display` and provide human-readable error messages:
//! ```
//! # use shovel::*;
//! # fn example() {
//! let error = ParseError::UnknownOption("--invalid");
//! println!("{}", error); // "unknown option '--invalid'"
//! # }
//! ```
//!
//! # Error Handling Patterns
//!
//! ## Basic Error Handling
//! ```
//! # #[cfg(feature = "std")] {
//! # use shovel::*;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let app = App {
//!     name: "myapp",
//!     description: "Example app",
//!     version: "0.1.0",
//!     authors: ["Author Name <author@example.com>"],
//!     copyright: "(c) 2026 Company Name",
//!     action: Action::no_args(|| println!("Success!")),
//! };
//!
//! // This will handle errors automatically and print them to stderr
//! app.run()?;
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! ## Custom Error Handling
//! ```
//! # #[cfg(feature = "std")] {
//! # use shovel::*;
//! # fn main() {
//! # let app = App { name: "app", description: "desc", version: "1.0", authors: ["author"], copyright: "copy", action: Action::no_args(|| {}) };
//! let mut output = std::io::Cursor::new(Vec::new());
//! let mut writer = IoWriteAdapter::new(&mut output);
//!
//! match app.run_with_args(&mut writer, &["app", "--invalid"]) {
//!     Ok(()) => println!("Command executed successfully"),
//!     Err(e) => {
//!         eprintln!("Error: {}", e);
//!         // Custom error handling logic here
//!         // For example, you could:
//!         // - Log the error to a file
//!         // - Send error metrics to a monitoring service
//!         // - Show a user-friendly error dialog
//!         // - Exit with a specific error code
//!         std::process::exit(1);
//!     }
//! }
//! # }
//! # }
//! ```
//!
//! # Zero-Copy Error Reporting
//!
//! The error system uses string references (`&'a str`) to avoid allocating
//! new strings for error messages, making it suitable for no-std environments:
//!
//! ```
//! # use shovel::*;
//! # fn example() {
//! // Error references the original input without copying
//! let input = "--unknown-flag";
//! let error = ParseError::UnknownOption(input);
//! // `error` contains a reference to `input`, not a copy
//! # }
//! ```
//!
//! # Error Context
//!
//! Errors maintain context about where they occurred in the parsing process:
//! - **Option Context**: Which option caused the error
//! - **Value Context**: What value was being parsed
//! - **Type Context**: What type was expected
//! - **Command Context**: Which command was being processed
//!
//! # Integration with Standard Error Handling
//!
//! `ParseError` implements the standard `Error` trait, making it compatible
//! with Rust's error handling ecosystem:
//!
//! ```
//! # use shovel::*;
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let error = ParseError::UnknownOption("--invalid");
//!
//! // Can be used with ? operator
//! // return Err(error.into());
//!
//! // Can be used with error handling crates
//! // anyhow::bail!(error);
//! # Ok(())
//! # }
//! ```
//!
//! # Performance Characteristics
//!
//! - **Zero Allocation**: Error creation doesn't allocate memory
//! - **Lazy Formatting**: Error messages are formatted only when displayed
//! - **Reference-Based**: Uses string references instead of owned strings
//! - **Compile-Time**: Error types are resolved at compile time
//!
//! See also: [`App`](crate::App), [`Opts`](crate::Opts), [`Args`](crate::Args), [`Writer`](crate::Writer)

use core::error::Error;
use core::fmt;

/// An enum representing various argument parsing errors.
///
/// This enum encapsulates different types of errors that can occur during
/// command-line argument parsing. Each variant provides specific information
/// about the nature of the error, allowing for detailed error reporting and
/// diagnostics.
#[derive(Debug)]
pub enum ParseError<'a> {
    /// Encountered an unknown optional argument.
    ///
    /// This error occurs when the parser encounters an option that is not
    /// defined in the argument specification.
    UnknownOption(&'a str),

    /// A flag argument was incorrectly provided with a value.
    ///
    /// This error occurs when a boolean flag argument (which should not
    /// have a value) is given a value.
    UnexpectedFlagValue {
        /// The name of the flag argument.
        name: &'static str,
        /// The incorrectly provided value.
        value: &'a str,
    },

    /// An invalid value was provided for an optional argument.
    ///
    /// This error occurs when the value provided for an option does not
    /// meet the expected format or constraints.
    InvalidOptionValue {
        /// The name of the optional argument.
        name: &'static str,
        /// The invalid value that was provided.
        value: &'a str,
        /// A description of what was expected.
        expected: &'static str,
    },

    /// An optional argument requiring a value was not provided with one.
    ///
    /// This error occurs when an option that expects a value is used
    /// without providing that value.
    MissingValueForOption {
        /// The name of the optional argument.
        name: &'static str,
    },

    /// A required optional argument was not provided.
    ///
    /// This error occurs when an option marked as required is not present
    /// in the command-line arguments.
    MissingRequiredOption {
        /// The name of the required optional argument.
        name: &'static str,
    },

    /// An invalid value was provided for a positional argument.
    ///
    /// This error occurs when a positional argument's value does not meet
    /// the expected format or constraints.
    InvalidArgumentValue {
        /// The index of the positional argument (0-based).
        index: usize,
        /// The invalid value that was provided.
        value: &'a str,
        /// A description of what was expected.
        expected: &'static str,
    },

    /// An incorrect number of positional arguments was provided.
    ///
    /// This error occurs when the number of positional arguments does not
    /// match the expected count (either too few or too many).
    ArgumentCountMismatch,
}

impl fmt::Display for ParseError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownOption(opt) => write!(f, "Unknown option '{opt}'"),
            Self::UnexpectedFlagValue { name, value } => {
                write!(f, "Flag '{name}' doesn't accept value '{value}'")
            }
            Self::InvalidOptionValue {
                name,
                value,
                expected,
            } => {
                write!(
                    f,
                    "Invalid value '{value}' for option '{name}': expected {expected}"
                )
            }
            Self::MissingValueForOption { name } => {
                write!(f, "Missing value for option '{name}'")
            }
            Self::MissingRequiredOption { name } => {
                write!(f, "Required option '{name}' is missing")
            }
            Self::InvalidArgumentValue {
                index,
                value,
                expected,
            } => {
                write!(
                    f,
                    "Invalid value '{value}' for argument {}: expected {expected}",
                    index + 1
                )
            }
            Self::ArgumentCountMismatch => {
                write!(f, "Incorrect number of arguments")
            }
        }
    }
}

impl Error for ParseError<'_> {}
