//! Standard environment helpers.
//!
//! This module provides ergonomic helpers for working with command-line arguments.
//! The primary function `args()` converts the result of `std::env::args()` into
//! a format suitable for use by Shovel's `run_*` methods.

/// Owned environment arguments with a convenient conversion to `&str`.
///
/// This struct stores command-line arguments as owned `String`s and provides
/// a method to convert them to string slices for use in argument parsing.
#[derive(Debug, Clone)]
pub struct Args {
    argv: std::vec::Vec<std::string::String>,
}

impl Args {
    /// Returns a borrowed view of the owned args.
    ///
    /// Converts the owned `String`s to `&str` slices, which is the primary purpose
    /// of this module - to transform the result of `std::env::args()` into
    /// `&[&str]` format for use by Shovel's `run_*` methods.
    #[must_use]
    pub fn to_strs(&self) -> std::vec::Vec<&str> {
        self.argv.iter().map(std::string::String::as_str).collect()
    }
}

/// Collects `std::env::args()` into an owned buffer.
///
/// This function converts the result of `std::env::args()` into an owned buffer
/// that can be transformed into `&[&str]` format via the `to_strs()` method.
/// This is the primary mechanism used by Shovel's `run_*` methods to process
/// command-line arguments.
///
/// # Example
///
/// ```no_run
/// # #[cfg(feature = "std")] {
/// use shovel::env;
///
/// let args = env::args();  // Collects std::env::args() into an owned Args struct
/// let arg_strings: Vec<&str> = args.to_strs();  // Convert to string slice references for run_with_args
/// # }
/// ```
#[must_use]
pub fn args() -> Args {
    Args {
        argv: std::env::args().collect(),
    }
}
