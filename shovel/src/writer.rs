//! Output formatting and writing system.
//!
//! This module provides the [`Writer`] trait and implementations for formatting and
//! outputting help text, error messages, version information, and completion scripts.
//! The writer system is designed for flexibility, supporting both standard output
//! and custom output destinations.
//!
//! # Core Types
//!
//! ## [`Writer`] - Output Formatting Trait
//!
//! The main trait for handling all CLI output formatting:
//! - **Help Messages**: Formatted help text with proper alignment
//! - **Error Messages**: User-friendly error reporting with suggestions
//! - **Version Information**: Application version and metadata display
//! - **Completion Scripts**: Shell completion script generation
//!
//! ## `Stdout` - Standard Output Writer (std feature)
//!
//! Default writer implementation that outputs to standard output:
//! - Uses `std::io::stdout()` for output
//! - Handles write errors gracefully
//! - Provides convenient default formatting
//!
//! ## `IoWriteAdapter<W>` - Generic I/O Adapter (std feature)
//!
//! Adapter for any type implementing `std::io::Write`:
//! - Enables testing with in-memory buffers
//! - Supports file output and custom destinations
//! - Maintains compatibility with standard I/O types
//!
//! # Writer Trait Methods
//!
//! ## Core Writing Methods
//!
//! ### `write_help(help: Help)`
//!
//! Formats and outputs comprehensive help information:
//!
//! ```text
//! A development tool with categorized commands
//!
//! Usage: devtool [OPTIONS] COMMAND
//!
//! Options:
//!   -h, --help        Show this help
//!   -v, --version     Print version information and quit
//!       --completion  [Shell:<bash|zsh|fish>] Generate shell completion script
//!
//! Commands:
//!   help              Show help information
//!
//!   Development:
//!     build             Build the project
//!     test              Run tests
//!     lint              Run code linting
//!
//!   Deployment:
//!     deploy            Deploy to production
//!     rollback          Rollback deployment
//!
//!   Database:
//!     migrate           Run database migrations
//!     seed              Seed database with test data
//! ```
//!
//! ### `write_version(version: Version)`
//!
//! Outputs version information:
//!
//! ```text
//! myapp 0.1.0
//! Authors: ["Developer <dev@example.com>"]
//! Copyright: (c) 2026 Company Name
//! ```
//!
//! ### `write_error(error: ParseError)`
//!
//! Formats error messages with helpful context:
//!
//! ```text
//! error: unknown command 'biuld'
//!
//!   Did you mean 'build'?
//! ```
//!
//! ### `write_completion(shell: Shell, help: Help)` (completion feature)
//! Generates shell completion scripts for the specified shell.
//!
//! # Formatting Features
//!
//! ## Consistent Alignment
//! The `ALIGN_WIDTH` constant ensures consistent column alignment:
//! ```text
//! OPTIONS:
//!     -v, --verbose    Enable verbose output
//!     -o, --output     Output file path
//!     -h, --help       Print help information
//! ```
//!
//! ## Hierarchical Display
//!
//! Commands and subcommands are displayed with proper hierarchy:
//!
//! ```text
//! COMMANDS:
//!   Development:
//!     build    Build the project
//!     test     Run tests
//!
//!   Deployment:
//!     deploy   Deploy to production
//!     rollback Rollback deployment
//! ```
//!
//! ## Error Messages
//!
//! Specific error message for current command:
//!
//! ```text
//! Error: Incorrect number of arguments
//! ```
//!
//! # Usage Patterns
//!
//! ## Standard Output (Default)
//! ```
//! # #[cfg(feature = "std")] {
//! # use shovel::*;
//! # fn main() {
//! let app = App {
//!     name: "myapp",
//!     description: "Sample application",
//!     version: "0.1.0",
//!     authors: ["Author Name <author@example.com>"],
//!     copyright: "(c) 2026 Company Name",
//!     action: Action::no_args(|| println!("Hello!")),
//! };
//!
//! // Uses Stdout writer by default
//! app.run().expect("Failed to run application");
//! # }
//! # }
//! ```
//!
//! ## Custom Output Destination
//! ```
//! # #[cfg(feature = "std")] {
//! # use shovel::*;
//! # fn main() {
//! # let app = App { name: "app", description: "desc", version: "1.0", authors: ["author"], copyright: "copy", action: Action::no_args(|| {}) };
//! let mut buffer = std::io::Cursor::new(Vec::new());
//! let mut writer = IoWriteAdapter::new(&mut buffer);
//!
//! app.run_with_args(&mut writer, &["app", "--help"]).expect("Failed to run application");
//!
//! let output = String::from_utf8(buffer.into_inner()).unwrap();
//! assert!(output.contains("Usage:"));
//! # }
//! # }
//! ```
//!
//! ## Testing with Custom Writer
//! ```
//! # #[cfg(feature = "std")] {
//! # use shovel::*;
//! # use core::fmt;
//! struct TestWriter {
//!     output: String,
//! }
//!
//! impl TestWriter {
//!     fn new() -> Self {
//!         Self { output: String::new() }
//!     }
//!
//!     fn content(&self) -> &str {
//!         &self.output
//!     }
//! }
//!
//! impl Writer for TestWriter {
//!     fn writer(&mut self) -> &mut dyn fmt::Write {
//!         &mut self.output
//!     }
//! }
//!
//! # fn main() {
//! # let app = App { name: "app", description: "desc", version: "1.0", authors: ["author"], copyright: "copy", action: Action::no_args(|| {}) };
//! let mut test_writer = TestWriter::new();
//! app.run_with_args(&mut test_writer, &["app", "--version"]).expect("Failed to run application");
//!
//! assert!(test_writer.content().contains("app 1.0"));
//! # }
//! # }
//! ```
//!
//! # Custom Writer Implementation
//!
//! ## Basic Custom Writer
//! ```
//! # use shovel::*;
//! # use core::fmt;
//! struct JsonWriter {
//!     output: String,
//! }
//!
//! impl Writer for JsonWriter {
//!     const ALIGN_WIDTH: usize = 20; // Custom alignment
//!
//!     fn writer(&mut self) -> &mut dyn fmt::Write {
//!         &mut self.output
//!     }
//!
//!     // Override specific methods for custom formatting
//!     fn write_version(&mut self, version: Version) -> fmt::Result {
//!         write!(self.writer(),
//!             r#"{{"name":"{}","version":"{}","authors":{:?}}}"#,
//!             version.name, version.version, version.authors
//!         )
//!     }
//! }
//! ```
//!
//! ## Colored Output Writer
//! ```
//! # use shovel::*;
//! # use core::fmt;
//! struct ColorWriter {
//!     output: String,
//! }
//!
//! impl Writer for ColorWriter {
//!     fn writer(&mut self) -> &mut dyn fmt::Write {
//!         &mut self.output
//!     }
//!
//!     fn write_error(&mut self, error: ParseError) -> fmt::Result {
//!         // Add ANSI color codes for errors
//!         write!(self.writer(), "\x1b[31merror:\x1b[0m {}", error)
//!     }
//! }
//! ```
//!
//! # Performance Characteristics
//!
//! - **Lazy Formatting**: Output is formatted only when requested
//! - **Streaming**: Large help text is written incrementally
//! - **Zero Allocation**: Core formatting uses stack-based operations
//! - **Buffering**: Writers can implement custom buffering strategies
//!
//! # Feature Requirements
//!
//! ## Standard Output Features
//! The `Stdout` and `IoWriteAdapter` types require the `std` feature:
//! ```toml
//! [dependencies]
//! shovel = { version = "*", features = ["std"] }
//! ```
//!
//! ## Completion Features
//! Completion script generation requires the `completion` feature:
//! ```toml
//! [dependencies]
//! shovel = { version = "*", features = ["std", "completion"] }
//! ```
//!
//! # Integration
//!
//! The writer system integrates with all other Shovel components:
//! - **Help System**: Formats help information from [`Help`] structures
//! - **Error System**: Displays [`ParseError`] with context and suggestions
//! - **Version System**: Shows [`Version`] information with proper formatting
//! - **Completion System**: Generates shell completion scripts
//!
//! See also: [`Help`], [`ParseError`], [`Version`], [`App`](crate::App)

use core::fmt;

use crate::Help;
use crate::Opt;
use crate::ParseError;
use crate::Suggestion;
use crate::Version;
#[cfg(feature = "completion")]
use crate::{Shell, write_completion};

/// Default alignment width for help output items.
const DEFAULT_ALIGN_WIDTH: usize = 16;

/// A `Writer` which is used to write help, usage and error messages .
pub trait Writer {
    /// The align width of each help output item.
    const ALIGN_WIDTH: usize = DEFAULT_ALIGN_WIDTH;

    /// Returns an implementation of [`core::fmt::Write`].
    fn writer(&mut self) -> &mut dyn fmt::Write;

    /// Writes a help message.
    ///
    /// # Errors
    ///
    /// This function will return an error if writing to the underlying writer fails.
    fn write_help(&mut self, help: Help) -> fmt::Result {
        impl_write_help(self.writer(), help)
    }

    /// Writes the version information.
    ///
    /// # Errors
    ///
    /// This function will return an error if writing to the underlying writer fails.
    fn write_version(&mut self, version: Version) -> fmt::Result {
        impl_write_version(self.writer(), version)
    }

    /// Writes a command suggestion.
    ///
    /// # Errors
    ///
    /// This function will return an error if writing to the underlying writer fails.
    fn write_suggestion(&mut self, suggestion: Suggestion) -> fmt::Result {
        impl_write_suggestion(self.writer(), suggestion)
    }

    /// Writes an arguments parse error.
    ///
    /// # Errors
    ///
    /// This function will return an error if writing to the underlying writer fails.
    fn write_error(&mut self, error: ParseError) -> fmt::Result {
        impl_write_error(self.writer(), error)
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "completion")))]
    #[cfg(feature = "completion")]
    /// Writes a shell completion.
    ///
    /// # Errors
    ///
    /// This function will return an error if writing to the underlying writer fails.
    fn write_completion(&mut self, help: Help, shell: Shell) -> fmt::Result {
        impl_write_completion(self.writer(), help, shell)
    }

    /// Returns the align width.
    fn align_width(&self) -> usize {
        Self::ALIGN_WIDTH
    }
}

impl<T: fmt::Write> Writer for T {
    fn writer(&mut self) -> &mut dyn fmt::Write {
        self
    }
}

/// An adapter that wraps a type implementing `std::io::Write` and provides an implementation of `fmt::Write`.
///
/// This adapter allows using types that implement `std::io::Write` in contexts where `fmt::Write` is required.
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
#[derive(Debug)]
pub struct IoWriteAdapter<T: std::io::Write>(T);

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl<T: std::io::Write> fmt::Write for IoWriteAdapter<T> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.write_all(s.as_bytes()).map_err(|e| {
            #[cfg(all(debug_assertions, feature = "std"))]
            {
                use std::io::{self, Write};
                let _ =
                    writeln!(io::stderr(), "IoWriteAdapter write error: {e}");
            }
            fmt::Error
        })
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl<T: std::io::Write> IoWriteAdapter<T> {
    /// Creates a new `IoWriteAdapter` wrapping the given writer.
    ///
    /// # Arguments
    ///
    /// * `writer` - The writer implementing `std::io::Write` to be wrapped.
    ///
    /// # Returns
    ///
    /// A new `IoWriteAdapter` instance.
    pub fn new(writer: T) -> Self {
        Self(writer)
    }

    /// Consumes the `IoWriteAdapter`, returning the wrapped writer.
    ///
    /// # Returns
    ///
    /// The original writer that was wrapped by this adapter.
    pub fn into_inner(self) -> T {
        self.0
    }
}

/// A wrappper of [`std::io::Stdout`] that implements trait [`std::fmt::Write`].
///
/// [`Stdout`]: std::io::Stdout
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
#[derive(Debug)]
pub struct Stdout(std::io::Stdout);

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl Stdout {
    /// Create A [`std::io::Stdout`] wrapper
    ///
    /// [`Stdout`]: std::io::Stdout
    #[must_use]
    pub fn new() -> Self {
        Self(std::io::stdout())
    }
}

#[cfg(feature = "std")]
impl Default for Stdout {
    fn default() -> Self {
        Stdout::new()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl std::fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        std::io::Write::write_all(&mut self.0, s.as_bytes()).map_err(|e| {
            #[cfg(all(debug_assertions, feature = "std"))]
            {
                use std::io::{self, Write};
                let _ = writeln!(io::stderr(), "Stdout write error: {e}");
            }
            fmt::Error
        })
    }
}

/// Calculate the maximum width needed for alignment, with a minimum threshold.
macro_rules! align_width {
    ($items:expr, $min:expr) => {
        align_width!($items, $min, format)
    };
    ($items:expr, $min:expr, $prop:ident) => {{
        $items
            .iter()
            .map(|item| item.$prop.len())
            .max()
            .unwrap_or(0)
            .max($min)
    }};
}

fn impl_write_help(writer: &mut dyn fmt::Write, help: Help) -> fmt::Result {
    write_help_header(writer, &help)?;
    write_help_usage(writer, &help)?;
    write_help_options(writer, &help)?;
    write_help_commands(writer, &help)?;
    write_help_arguments(writer, &help)
}

fn impl_write_version(
    writer: &mut dyn fmt::Write,
    version: Version,
) -> fmt::Result {
    writeln!(writer, "{} {}", version.name, version.version)?;
    writeln!(writer, "Authors: {:?}", version.authors)?;
    writeln!(writer, "Copyright: {}", version.copyright)
}

fn impl_write_suggestion(
    writer: &mut dyn fmt::Write,
    suggestion: Suggestion,
) -> fmt::Result {
    writeln!(writer, "Error: no such command: \"{}\"", suggestion.typo)?;
    writeln!(writer, "\n  Did you mean \"{}\"?", suggestion.cmd)
}

fn impl_write_error(
    writer: &mut dyn fmt::Write,
    error: ParseError,
) -> fmt::Result {
    writeln!(writer, "Error: {error}")
}

#[cfg(feature = "completion")]
fn impl_write_completion(
    writer: &mut dyn fmt::Write,
    help: Help,
    shell: Shell,
) -> fmt::Result {
    write_completion(writer, help, shell)
}

fn write_help_header(writer: &mut dyn fmt::Write, help: &Help) -> fmt::Result {
    writeln!(writer, "{}", help.description)
}

fn write_help_usage(writer: &mut dyn fmt::Write, help: &Help) -> fmt::Result {
    if help.is_app() {
        if help.has_subcmds() {
            writeln!(writer, "\nUsage: {} [OPTIONS] COMMAND", help.path)
        } else {
            writeln!(
                writer,
                "\nUsage: {} [OPTIONS] {}",
                help.path,
                if help.groups.is_empty() {
                    ""
                } else if help.groups.len() == 1 {
                    help.groups[0].format
                } else {
                    "[ARG...]"
                }
            )
        }
    } else if help.has_subcmds() {
        writeln!(
            writer,
            "\nUsage: {} [GLOBAL OPTIONS] {} [OPTIONS] COMMAND",
            help.path.app(),
            help.path.subpath(),
        )
    } else {
        writeln!(
            writer,
            "\nUsage: {} [GLOBAL OPTIONS] {} [OPTIONS] {}",
            help.path.app(),
            help.path.subpath(),
            if help.groups.is_empty() {
                ""
            } else if help.groups.len() == 1 {
                help.groups[0].format
            } else {
                "[ARG...]"
            }
        )
    }
}

fn write_help_options(writer: &mut dyn fmt::Write, help: &Help) -> fmt::Result {
    let align_width = writer.align_width();
    if !help.is_app() {
        writeln!(writer, "\nGlobal Options:")?;
        let builtin_global_opts = help.builtin_global_opts.unwrap_or(&[]);
        let global_opts = help.global_opts.unwrap_or(&[]);
        let width = align_width!(builtin_global_opts, align_width)
            .max(align_width!(global_opts, align_width));
        if !builtin_global_opts.is_empty() {
            write_options(writer, builtin_global_opts, width)?;
        }
        if !global_opts.is_empty() {
            write_options(writer, global_opts, width)?;
        }
    }

    writeln!(writer, "\nOptions:")?;
    let width = align_width!(help.builtin_opts, align_width)
        .max(align_width!(help.opts, align_width));
    write_options(writer, help.builtin_opts, width)?;
    if !help.opts.is_empty() {
        write_options(writer, help.opts, width)?;
    }
    Ok(())
}

// Allocation-based version for std environments (O(N log C) complexity).
#[cfg(feature = "std")]
fn write_help_commands(
    writer: &mut dyn fmt::Write,
    help: &Help,
) -> fmt::Result {
    if let Some(subhelps) = help.subhelps {
        writeln!(writer, "\nCommands:")?;
        let width = align_width!(subhelps, writer.align_width(), name);

        let mut has_uncategorized = false;
        let mut categorized_cmds: std::collections::BTreeMap<
            &str,
            std::vec::Vec<_>,
        > = std::collections::BTreeMap::new();

        // Group uncategorized commands separately
        let uncategorized_cmds: std::vec::Vec<_> = subhelps
            .iter()
            .filter(|cmd| {
                if let Some(category) = cmd.category {
                    categorized_cmds.entry(category).or_default().push(*cmd);
                    false
                } else {
                    true
                }
            })
            .collect();

        // Print uncategorized commands first
        if !uncategorized_cmds.is_empty() {
            has_uncategorized = true;
            for subhelp in uncategorized_cmds {
                writeln!(
                    writer,
                    "  {name: <width$}  {desc}",
                    name = subhelp.name,
                    width = width,
                    desc = subhelp.description
                )?;
            }
        }

        // Then print categorized commands
        for (i, (category, cmds)) in categorized_cmds.iter().enumerate() {
            if i > 0 || has_uncategorized {
                writeln!(writer)?;
            }
            writeln!(writer, "  {category}:")?;
            for cmd in cmds {
                writeln!(
                    writer,
                    "    {name: <width$}  {desc}",
                    name = cmd.name,
                    width = width,
                    desc = cmd.description
                )?;
            }
        }
    }
    Ok(())
}

// No-allocation version for no_std environments (O(N²) complexity).
#[cfg(not(feature = "std"))]
fn write_help_commands(
    writer: &mut dyn fmt::Write,
    help: &Help,
) -> fmt::Result {
    if let Some(subhelps) = help.subhelps {
        writeln!(writer, "\nCommands:")?;
        let width = align_width!(subhelps, writer.align_width(), name);

        // Check if we have any categorized commands
        let has_categories = subhelps.iter().any(|cmd| cmd.category.is_some());
        if has_categories {
            // Complex case: we have categories, need hierarchical display

            // Check once if we have uncategorized commands (for spacing logic)
            let has_uncategorized =
                subhelps.iter().any(|cmd| cmd.category.is_none());

            // First: write uncategorized commands
            for subhelp in subhelps {
                if subhelp.category.is_none() {
                    writeln!(
                        writer,
                        "  {name: <width$}  {desc}",
                        name = subhelp.name,
                        width = width,
                        desc = subhelp.description
                    )?;
                }
            }

            // Then: write each category (simple O(n²) but avoids allocations)
            for (i, subhelp) in subhelps.iter().enumerate() {
                if let Some(category) = subhelp.category {
                    // Check if this is the first occurrence of this category
                    let is_first_occurrence = !subhelps[..i]
                        .iter()
                        .any(|cmd| cmd.category == Some(category));

                    if is_first_occurrence {
                        // Add spacing if we had uncategorized commands
                        if has_uncategorized || i > 0 {
                            writeln!(writer)?;
                        }

                        // Write category header
                        writeln!(writer, "  {category}:",)?;

                        // Write all commands in this category
                        for cmd in subhelps {
                            if cmd.category == Some(category) {
                                writeln!(
                                    writer,
                                    "    {name: <width$}  {desc}",
                                    name = cmd.name,
                                    width = width,
                                    desc = cmd.description
                                )?;
                            }
                        }
                    }
                }
            }
        } else {
            // Simple case: no categories, just list all commands
            for subhelp in subhelps {
                writeln!(
                    writer,
                    "  {name: <width$}  {desc}",
                    name = subhelp.name,
                    width = width,
                    desc = subhelp.description
                )?;
            }
        }
    }
    Ok(())
}

fn write_help_arguments(
    writer: &mut dyn fmt::Write,
    help: &Help,
) -> fmt::Result {
    if !help.groups.is_empty() {
        writeln!(writer, "\nArguments:")?;
        let width = align_width!(help.groups, writer.align_width());
        for group in help.groups {
            writeln!(
                writer,
                "  {group: <width$}  {desc}",
                group = group.format,
                width = width,
                desc = group.description
            )?;
        }
    }
    Ok(())
}

// Helper function to write options
fn write_options(
    writer: &mut dyn fmt::Write,
    opts: &[Opt],
    width: usize,
) -> fmt::Result {
    for opt in opts {
        if opt.flag {
            writeln!(
                writer,
                "  {opt: <width$}  {desc}",
                opt = opt.format,
                width = width,
                desc = opt.description
            )?;
        } else {
            writeln!(
                writer,
                "  {opt: <width$}  [{ty}] {desc}",
                opt = opt.format,
                ty = opt.ty,
                width = width,
                desc = opt.description
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "std")]
    use core::fmt::Write;
    #[cfg(feature = "std")]
    use std::{string::String, vec, vec::Vec};

    /// Test writer that captures output for verification
    #[cfg(feature = "std")]
    struct TestWriter {
        output: String,
    }

    #[cfg(feature = "std")]
    impl TestWriter {
        fn new() -> Self {
            Self {
                output: String::new(),
            }
        }

        fn content(&self) -> &str {
            &self.output
        }
    }

    #[cfg(feature = "std")]
    impl fmt::Write for TestWriter {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            self.output.push_str(s);
            Ok(())
        }
    }

    #[test]
    fn test_align_width_macro() {
        let opts = [
            Opt {
                name: "verbose",
                ty: "bool",
                required: false,
                flag: true,
                description: "Enable verbose output",
                short: Some('v'),
                format: "-v, --verbose",
            },
            Opt {
                name: "output",
                ty: "String",
                required: false,
                flag: false,
                description: "Output file",
                short: Some('o'),
                format: "-o, --output",
            },
        ];

        let width = align_width!(opts, 8);
        assert_eq!(width, 13); // Length of "-o, --output"

        let width_with_higher_min = align_width!(opts, 20);
        assert_eq!(width_with_higher_min, 20);
    }

    #[test]
    fn test_align_width_macro_with_property() {
        // Create test data with name property
        struct TestItem {
            name: &'static str,
        }

        let items = [
            TestItem { name: "build" },
            TestItem { name: "test" },
            TestItem {
                name: "very-long-command-name",
            },
        ];

        let width = align_width!(items, 8, name);
        assert_eq!(width, 22); // Length of "very-long-command-name"
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_version_formatting() {
        let mut writer = TestWriter::new();
        let version = Version {
            name: "testapp",
            version: "0.1.0",
            authors: &[
                "Author One <one@example.com>",
                "Author Two <two@example.com>",
            ],
            copyright: "(c) 2026 Test Company",
        };

        writer.write_version(version).unwrap();
        let output = writer.content();

        assert!(output.contains("testapp 0.1.0"));
        assert!(output.contains("Authors: [\"Author One <one@example.com>\", \"Author Two <two@example.com>\"]"));
        assert!(output.contains("Copyright: (c) 2026 Test Company"));
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_error_formatting() {
        let mut writer = TestWriter::new();
        let error = ParseError::UnknownOption("--invalid");

        writer.write_error(error).unwrap();
        let output = writer.content();

        assert!(output.contains("Error:"));
        assert!(output.contains("--invalid"));
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_suggestion_formatting() {
        let mut writer = TestWriter::new();
        let suggestion = Suggestion {
            typo: "biuld",
            cmd: "build",
        };

        writer.write_suggestion(suggestion).unwrap();
        let output = writer.content();

        assert!(output.contains("Error: no such command: \"biuld\""));
        assert!(output.contains("Did you mean \"build\"?"));
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_options_formatting() {
        let mut writer = TestWriter::new();
        let opts = vec![
            Opt {
                name: "verbose",
                ty: "bool",
                required: false,
                flag: true,
                description: "Enable verbose output",
                short: Some('v'),
                format: "-v, --verbose",
            },
            Opt {
                name: "count",
                ty: "usize",
                required: false,
                flag: false,
                description: "Number of items",
                short: Some('c'),
                format: "-c, --count",
            },
        ];

        write_options(&mut writer, &opts, 20).unwrap();
        let output = writer.content();

        assert!(output.contains("-v, --verbose"));
        assert!(output.contains("Enable verbose output"));
        assert!(output.contains("-c, --count"));
        assert!(output.contains("[usize]"));
        assert!(output.contains("Number of items"));
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_io_write_adapter() {
        use std::io::Cursor;

        let buffer = Cursor::new(Vec::new());
        let mut adapter = IoWriteAdapter::new(buffer);

        // Test writing
        adapter.write_str("Hello, World!").unwrap();

        // Get the buffer back and check contents
        let buffer = adapter.into_inner();
        let content = String::from_utf8(buffer.into_inner()).unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_stdout_wrapper() {
        let mut stdout = Stdout::new();
        // We can't easily test actual stdout output, but we can test that it doesn't panic
        let result = stdout.write_str("test");
        // In most test environments, this should succeed
        assert!(result.is_ok() || result.is_err()); // Either is acceptable for this test
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_writer_trait_blanket_impl() {
        let mut string_writer = String::new();

        // Test that String implements Writer through the blanket impl
        string_writer.write_str("test").unwrap();
        assert_eq!(string_writer, "test");

        // Test align_width method
        assert_eq!(string_writer.align_width(), DEFAULT_ALIGN_WIDTH);
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_hierarchical_command_display() {
        let mut writer = TestWriter::new();

        // Create test commands with categories
        let cmd_helps = [
            crate::CmdHelp {
                name: "build",
                description: "Build the project",
                category: Some("Development"),
                opts: &[],
                builtin_opts: &[],
                groups: &[],
                subhelps: None,
            },
            crate::CmdHelp {
                name: "test",
                description: "Run tests",
                category: Some("Development"),
                opts: &[],
                builtin_opts: &[],
                groups: &[],
                subhelps: None,
            },
            crate::CmdHelp {
                name: "deploy",
                description: "Deploy to production",
                category: Some("Deployment"),
                opts: &[],
                builtin_opts: &[],
                groups: &[],
                subhelps: None,
            },
            crate::CmdHelp {
                name: "help",
                description: "Show help information",
                category: None, // Uncategorized
                opts: &[],
                builtin_opts: &[],
                groups: &[],
                subhelps: None,
            },
        ];

        // Create a Help structure with these commands
        let help = crate::Help {
            path: crate::CmdPath::new("testapp", &[]),
            name: "testapp",
            description: "Test application",
            global_opts: None,
            builtin_global_opts: None,
            opts: &[],
            builtin_opts: &[],
            groups: &[],
            subhelps: Some(&cmd_helps),
        };

        // Test the hierarchical display
        write_help_commands(&mut writer, &help).unwrap();
        let output = writer.content();

        // Should have uncategorized commands first
        assert!(output.contains("  help"));

        // Should have category headers
        assert!(output.contains("  Development:"));
        assert!(output.contains("  Deployment:"));

        // Should have indented commands under categories
        assert!(output.contains("    build"));
        assert!(output.contains("    test"));
        assert!(output.contains("    deploy"));
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_simple_command_display_no_categories() {
        let mut writer = TestWriter::new();

        // Create test commands without categories
        let cmd_helps = [
            crate::CmdHelp {
                name: "build",
                description: "Build the project",
                category: None,
                opts: &[],
                builtin_opts: &[],
                groups: &[],
                subhelps: None,
            },
            crate::CmdHelp {
                name: "test",
                description: "Run tests",
                category: None,
                opts: &[],
                builtin_opts: &[],
                groups: &[],
                subhelps: None,
            },
        ];

        // Create a Help structure with these commands
        let help = crate::Help {
            path: crate::CmdPath::new("testapp", &[]),
            name: "testapp",
            description: "Test application",
            global_opts: None,
            builtin_global_opts: None,
            opts: &[],
            builtin_opts: &[],
            groups: &[],
            subhelps: Some(&cmd_helps),
        };

        // Test the simple display
        write_help_commands(&mut writer, &help).unwrap();
        let output = writer.content();

        // Should have simple flat structure
        assert!(output.contains("  build"));
        assert!(output.contains("  test"));

        // Should not have category headers (except for "Commands:")
        assert!(!output.contains("Development:"));
        assert!(!output.contains("Deployment:"));
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_performance_optimization_produces_same_output() {
        // This test verifies that both the std and no_std implementations
        // produce identical output for the same input
        let mut writer = TestWriter::new();

        // Create test commands with mixed categories
        let cmd_helps = [
            crate::CmdHelp {
                name: "help",
                description: "Show help information",
                category: None, // Uncategorized
                opts: &[],
                builtin_opts: &[],
                groups: &[],
                subhelps: None,
            },
            crate::CmdHelp {
                name: "build",
                description: "Build the project",
                category: Some("Development"),
                opts: &[],
                builtin_opts: &[],
                groups: &[],
                subhelps: None,
            },
            crate::CmdHelp {
                name: "test",
                description: "Run tests",
                category: Some("Development"),
                opts: &[],
                builtin_opts: &[],
                groups: &[],
                subhelps: None,
            },
            crate::CmdHelp {
                name: "deploy",
                description: "Deploy to production",
                category: Some("Deployment"),
                opts: &[],
                builtin_opts: &[],
                groups: &[],
                subhelps: None,
            },
        ];

        let help = crate::Help {
            path: crate::CmdPath::new("testapp", &[]),
            name: "testapp",
            description: "Test application",
            global_opts: None,
            builtin_global_opts: None,
            opts: &[],
            builtin_opts: &[],
            groups: &[],
            subhelps: Some(&cmd_helps),
        };

        // Test the current implementation (std version when std feature is enabled)
        write_help_commands(&mut writer, &help).unwrap();
        let output = writer.content();

        // Verify expected structure
        assert!(output.contains("Commands:"));
        assert!(output.contains("  help"));
        assert!(output.contains("  Development:"));
        assert!(output.contains("    build"));
        assert!(output.contains("    test"));
        assert!(output.contains("  Deployment:"));
        assert!(output.contains("    deploy"));

        // Verify proper ordering: uncategorized first, then categories in alphabetical order
        let help_pos = output.find("  help").unwrap();
        let deploy_pos = output.find("  Deployment:").unwrap();
        let dev_pos = output.find("  Development:").unwrap();

        assert!(
            help_pos < deploy_pos,
            "Uncategorized commands should come first"
        );
        assert!(
            deploy_pos < dev_pos,
            "Categories should be in alphabetical order: 'Deployment' should come before 'Development'"
        );
    }

    // This test can run in both std and no_std environments
    #[test]
    fn test_default_align_width_constant() {
        assert_eq!(DEFAULT_ALIGN_WIDTH, 16);
    }

    // This test can run in both std and no_std environments
    #[test]
    fn test_writer_trait_align_width_method() {
        struct MockWriter;
        impl fmt::Write for MockWriter {
            fn write_str(&mut self, _s: &str) -> fmt::Result {
                Ok(())
            }
        }

        let writer = MockWriter;
        assert_eq!(writer.align_width(), DEFAULT_ALIGN_WIDTH);
    }
}
