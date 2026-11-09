//! Standard environment helpers.
//!
//! This module provides ergonomic helpers for working with command-line arguments
//! and process environment variables. [`args()`] converts the result of
//! `std::env::args()` into a format suitable for use by Shovel's `run_*` methods,
//! while [`vars_os()`] collects the Unicode environment entries into [`Vars`].

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

/// Owned UTF-8 environment names and values.
#[derive(Debug, Clone)]
pub struct Vars {
    entries: std::vec::Vec<(std::string::String, std::string::String)>,
}

impl Vars {
    /// Borrow the entries for environment-aware execution methods.
    #[must_use]
    pub fn to_strs(&self) -> std::vec::Vec<(&str, &str)> {
        self.entries
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
            .collect()
    }
}

/// Collect process environment variables, keeping only Unicode names and values.
///
/// Non-Unicode entries are ignored. In particular, a declared variable that is
/// present but non-Unicode is treated as if it were absent.
#[must_use]
pub fn vars_os() -> Vars {
    collect_vars(std::env::vars_os())
}

fn collect_vars(
    entries: impl IntoIterator<Item = (std::ffi::OsString, std::ffi::OsString)>,
) -> Vars {
    let entries = entries
        .into_iter()
        .filter_map(|(name, value)| {
            Some((name.into_string().ok()?, value.into_string().ok()?))
        })
        .collect();
    Vars { entries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_borrowable_pairs() {
        let vars = collect_vars([("PORT".into(), "8080".into())]);
        assert_eq!(vars.to_strs(), [("PORT", "8080")]);
    }

    #[cfg(unix)]
    #[test]
    fn ignores_non_unicode_names_and_values() {
        use std::os::unix::ffi::OsStringExt;
        let invalid = std::ffi::OsString::from_vec(std::vec![0xff]);
        let vars = collect_vars([
            (invalid.clone(), "value".into()),
            ("GOOD".into(), "ok".into()),
            ("BAD_VALUE".into(), invalid),
        ]);
        assert_eq!(vars.to_strs(), [("GOOD", "ok")]);
    }
}
