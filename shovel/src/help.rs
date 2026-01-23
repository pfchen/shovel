//! Help system and documentation generation.
//!
//! This module provides the help system for Shovel applications, including automatic
//! help text generation, version information display, and command documentation.
//! The help system is designed to provide comprehensive, well-formatted assistance
//! to users of CLI applications.
//!
//! # Core Types
//!
//! ## [`Help`] - Help Information Container
//!
//! The main type that aggregates all help information for display:
//! - App descriptions and usage patterns
//! - Option and argument specifications
//! - Subcommand listings with categories
//! - Examples and additional documentation
//!
//! ## [`Version`] - Version Information
//!
//! Contains application version metadata:
//! - Application name and version string
//! - Author information
//! - Copyright notices
//!
//! ## [`CmdHelp`] - Command-Specific Help
//!
//! Detailed help information for individual commands:
//! - Command name and description
//! - Available options and their descriptions
//! - Positional argument specifications
//! - Subcommand listings
//!
//! ## [`CmdPath`] - Command Path Tracking
//!
//! Tracks the hierarchical path to a command for context-aware help:
//! - Parent command chain
//! - Current command position
//! - Full command path for usage strings
//!
//! # Help Generation Features
//!
//! ## Automatic Help Text
//! - **Usage Lines**: Generated from command structure
//! - **Option Lists**: Formatted with descriptions and defaults
//! - **Argument Lists**: Positional argument documentation
//! - **Subcommand Lists**: Available subcommands with descriptions
//! - **Examples**: Usage examples when provided
//!
//! ## Formatting Features
//! - **Alignment**: Consistent column alignment for readability
//! - **Categorization**: Subcommands grouped by category
//! - **Wrapping**: Long descriptions wrapped appropriately
//! - **Highlighting**: Important information emphasized
//!
//! ## Context Awareness
//! - **Command-Specific**: Help tailored to the current command
//! - **Hierarchical**: Shows command path and context
//! - **Inheritance**: Global options shown in subcommand help
//! - **Filtering**: Only relevant information displayed
//!
//! # Help Triggers
//!
//! Help can be triggered in several ways:
//!
//! ## Built-in Help Options
//! ```bash
//! myapp --help           # Show application help
//! myapp -h               # Short form help
//! myapp command --help   # Show command-specific help
//! ```
//!
//! ## Subcommand Help
//! ```bash
//! myapp                  # Shows help when no subcommand provided
//! myapp command          # Shows subcommand help if no action
//! ```
//!
//! ## Error-Triggered Help
//! ```bash
//! myapp invalid-command  # Shows help with error message
//! myapp --invalid-option # Shows help with error context
//! ```
//!
//! # Version Information
//!
//! ## Built-in Version Options
//! ```bash
//! myapp --version        # Show version information
//! myapp -V               # Short form version
//! ```
//!
//! ## Version Display Format
//! ```text
//! myapp 0.1.0
//! Authors: Developer <dev@example.com>, Team <team@example.com>
//! Copyright: (c) 2026 My Company
//! ```
//!
//! # Customization
//!
//! ## Custom Help Writers
//! Implement [`Writer`](crate::Writer) trait for custom help formatting:
//! ```
//! # use shovel::*;
//! # use core::fmt;
//! struct CustomWriter {
//!     output: String,
//! }
//!
//! impl Writer for CustomWriter {
//!     fn writer(&mut self) -> &mut dyn fmt::Write {
//!         &mut self.output
//!     }
//!
//!     // Override help formatting methods for customization
//! }
//! ```
//!
//! ## Help Content Customization
//! ```
//! # use shovel::*;
//! let app = App {
//!     name: "myapp",
//!     description: "A comprehensive CLI tool with detailed help",
//!     version: "0.1.0",
//!     authors: ["Author Name <author@example.com>"],
//!     copyright: "(c) 2026 Company Name",
//!     action: Action::subcmds((
//!         Cmd {
//!             name: "build",
//!             description: "Build the project with specified options",
//!             category: Some("development"),  // Categorize commands
//!             action: Action::no_args(|| {}),
//!         },
//!     )),
//! };
//! ```
//!
//! # Help Output Examples
//!
//! ## Application Help
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
//!   Database:
//!     migrate           Run database migrations
//!     seed              Seed database with test data
//!
//!   Deployment:
//!     deploy            Deploy to production
//!     rollback          Rollback deployment
//!
//!   Development:
//!     build             Build the project
//!     test              Run tests
//!     lint              Run code linting
//! ```
//!
//! ## Command-Specific Help
//! ```text
//! Some demo utils
//!
//! Usage: full_feature_demo [GLOBAL OPTIONS] utils [OPTIONS] COMMAND
//!
//! Global Options:
//!   -h, --help        Show this help
//!   -v, --version     Print version information and quit
//!       --completion  [Shell:<bash|zsh|fish>] Generate shell completion script
//!   -d, --debug       Debug mode
//!
//! Options:
//!   -h, --help        Show this help
//!
//! Commands:
//!   calculator:
//!     sum               Calculate the sum
//!
//!   utils:
//!     time              Show the current time
//!     tick              Print tick
//!     echo              Echo a message
//! ```
//!
//! # Performance Characteristics
//!
//! - **Lazy Generation**: Help text generated only when requested
//! - **Zero Allocation**: Help generation uses stack-based formatting
//! - **Efficient Formatting**: Minimal string operations and copying
//! - **Compile-Time**: Help structure resolved at compile time
//!
//! # Integration
//!
//! The help system integrates seamlessly with other Shovel components:
//! - **Options**: Automatically includes option descriptions
//! - **Arguments**: Shows argument types and requirements
//! - **Commands**: Lists available subcommands with descriptions
//! - **Errors**: Provides contextual help when errors occur
//!
//! See also: [`Writer`](crate::Writer), [`App`](crate::App), [`Cmd`](crate::Cmd), [`Opts`](crate::Opts), [`Args`](crate::Args)

use core::fmt;

use crate::ArgGroup;
use crate::Opt;

/// Represents the version information of an application.
#[derive(Debug, Clone, Copy)]
pub struct Version<'a> {
    /// The name of the application.
    pub name: &'a str,
    /// The version string of the application.
    pub version: &'a str,
    /// A list of all authors who contributed to the application.
    pub authors: &'a [&'a str],
    /// The copyright notice for the application.
    pub copyright: &'a str,
}

/// Represents help information for a command or subcommand.
///
/// This struct contains essential details about a command, including its name,
/// description, options, and subcommands. It's used to generate help text and
/// navigate the command hierarchy.
///
/// See also: [`Help`], [`Subcmds`]
///
/// [`Help`]: crate::Help
/// [`Subcmds`]: crate::Subcmds
#[derive(Debug, Default, Clone, Copy)]
pub struct CmdHelp<'a> {
    /// The name of the command.
    pub name: &'a str,
    /// A brief description of the command's purpose.
    pub description: &'a str,
    /// An optional category for grouping related commands.
    pub category: Option<&'a str>,
    /// Optional arguments specific to this command.
    pub opts: &'a [Opt],
    /// Built-in optional arguments available for all commands.
    pub builtin_opts: &'static [Opt],
    /// A list of positional argument groups for this command.
    pub groups: &'a [ArgGroup],
    /// A list of help items for subcommands, if any.
    pub subhelps: Option<&'a [CmdHelp<'a>]>,
}

/// Represents the complete help information for a `shovel` [`Cmd`].
///
/// This struct contains all the details needed to display help for a command.
///
/// [`Cmd`]: crate::Cmd
#[derive(Debug, Clone, Copy)]
pub struct Help<'a> {
    /// The full path of the command name, including parent commands.
    pub path: CmdPath<'a>,
    /// The name of this specific command.
    pub name: &'a str,
    /// A detailed description of the command's functionality.
    pub description: &'a str,
    /// Optional global arguments that apply to this command and its subcommands.
    pub global_opts: Option<&'a [Opt]>,
    /// Built-in global optional arguments available for all commands.
    pub builtin_global_opts: Option<&'a [Opt]>,
    /// Optional arguments specific to this command.
    pub opts: &'a [Opt],
    /// Built-in optional arguments available for this command.
    pub builtin_opts: &'static [Opt],
    /// A list of positional argument groups for this command.
    pub groups: &'a [ArgGroup],
    /// A list of help items for subcommands, if any.
    pub subhelps: Option<&'a [CmdHelp<'a>]>,
}

pub(crate) struct ActionHelp<'a> {
    pub(crate) opts: &'a [Opt],
    pub(crate) groups: &'a [ArgGroup],
    pub(crate) subhelps: Option<&'a [CmdHelp<'a>]>,
}

/// The full path of a [`Cmd`] name.
///
/// [`Cmd`]: crate::Cmd
#[derive(Debug, Clone, Copy)]
pub struct CmdPath<'a> {
    /// The app name.
    app: &'a str,
    /// The subpath without the app name.
    subpath: &'a [&'a str],
}

/// Display wrapper for a command subpath.
#[derive(Debug, Clone, Copy)]
pub struct SubPath<'a>(&'a [&'a str]);

impl<'a> CmdPath<'a> {
    /// Creates a new command path from app name and subpath.
    #[must_use]
    pub fn new(app: &'a str, subpath: &'a [&'a str]) -> Self {
        Self { app, subpath }
    }

    /// Gets the app name.
    #[must_use]
    pub fn app(&self) -> &'a str {
        self.app
    }

    /// Gets the subpath without the app name.
    #[must_use]
    pub fn subpath(&self) -> SubPath<'a> {
        SubPath(self.subpath)
    }

    /// Gets the command name.
    #[must_use]
    pub fn name(&self) -> &'a str {
        if self.subpath.is_empty() {
            self.app
        } else {
            self.subpath[self.subpath.len() - 1]
        }
    }

    /// Returns the length of the command path.
    #[must_use]
    pub fn len(&self) -> usize {
        1 + self.subpath.len()
    }

    /// Returns `true` if this cmd path has no elements.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        false // app always present
    }

    /// Gets the raw path components as (app, subpath).
    #[must_use]
    pub fn raw(&self) -> (&'a str, &'a [&'a str]) {
        (self.app, self.subpath)
    }
}

impl<'a> SubPath<'a> {
    /// Gets the raw subpath slice.
    #[must_use]
    pub fn raw(&self) -> &'a [&'a str] {
        self.0
    }
}

impl fmt::Display for SubPath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, name) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{name}")?;
        }
        Ok(())
    }
}

impl fmt::Display for CmdPath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.app)?;
        for name in self.subpath {
            write!(f, " {name}")?;
        }
        Ok(())
    }
}

impl<'a> CmdHelp<'a> {
    pub(crate) fn new(
        name: &'a str,
        description: &'a str,
        category: Option<&'a str>,
        action_help: ActionHelp<'a>,
        builtin_opts: &'static [Opt],
    ) -> Self {
        CmdHelp {
            name,
            description,
            category,
            opts: action_help.opts,
            builtin_opts,
            groups: action_help.groups,
            subhelps: action_help.subhelps,
        }
    }

    pub(crate) fn into_help(
        self,
        app: &'a str,
        subpath: &'a [&'a str],
        global_opts: Option<&'a [Opt]>,
        builtin_global_opts: Option<&'a [Opt]>,
    ) -> Help<'a> {
        let path = CmdPath::new(app, subpath);
        let (global_opts, builtin_global_opts) = if path.app() == self.name {
            (None, None)
        } else {
            (global_opts, builtin_global_opts)
        };
        Help {
            path,
            name: self.name,
            description: self.description,
            global_opts,
            builtin_global_opts,
            opts: self.opts,
            builtin_opts: self.builtin_opts,
            groups: self.groups,
            subhelps: self.subhelps,
        }
    }

    pub(crate) fn find(self, path: &[&str], name: &str) -> Option<CmdHelp<'a>> {
        // If we have a path segment, walk it
        if let Some((head, tail)) = path.split_first() {
            if let Some(subhelps) = self.subhelps {
                for subhelp in subhelps {
                    if subhelp.name == *head {
                        return if tail.is_empty() {
                            // Reached the target node
                            (name == subhelp.name).then_some(*subhelp)
                        } else {
                            // Keep descending
                            subhelp.find(tail, name)
                        };
                    }
                }
            }
            return None; // path specified but no matching subcommand
        }

        // No path provided: fall back to matching the current node
        if name == self.name { Some(self) } else { None }
    }
}

impl Help<'_> {
    /// Checks whether is a app help.
    #[must_use]
    pub fn is_app(&self) -> bool {
        self.path.app() == self.name
    }

    /// Checks whether this command help has subcommand.
    #[must_use]
    pub fn has_subcmds(&self) -> bool {
        self.subhelps.is_some()
    }

    /// Calculates the depth of the commands in the hierarchy.
    #[must_use]
    pub fn depth(&self) -> usize {
        let mut max = 1;
        if let Some(subhelps) = self.subhelps {
            for subhelp in subhelps {
                let d = cmd_depth(subhelp, 2);
                if d > max {
                    max = d;
                }
            }
        }
        max
    }
}

fn cmd_depth(help: &CmdHelp, depth: usize) -> usize {
    let mut max = depth;
    if let Some(subhelps) = help.subhelps {
        for subhelp in subhelps {
            let d = cmd_depth(subhelp, depth + 1);
            if d > max {
                max = d;
            }
        }
    }
    max
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::ArrayPool;
    #[cfg(feature = "std")]
    use std::format;

    #[test]
    fn test_version_creation() {
        let version = Version {
            name: "testapp",
            version: "0.1.0",
            authors: &["Test Author <test@example.com>"],
            copyright: "(c) 2026 Company Name",
        };

        assert_eq!(version.name, "testapp");
        assert_eq!(version.version, "0.1.0");
        assert_eq!(version.authors, &["Test Author <test@example.com>"]);
        assert_eq!(version.copyright, "(c) 2026 Company Name");
    }

    #[test]
    fn test_cmd_path_creation_and_methods() {
        let subpath: &[&str] = &["subcmd", "nested"];
        let cmd_path = CmdPath::new("myapp", subpath);

        assert_eq!(cmd_path.app(), "myapp");
        assert_eq!(cmd_path.name(), "nested");
        assert_eq!(cmd_path.len(), 3);
        assert!(!cmd_path.is_empty());
        assert_eq!(cmd_path.raw(), ("myapp", subpath));
    }

    #[test]
    fn test_cmd_path_single_element() {
        let single_path = CmdPath::new("app", &[]);
        assert_eq!(single_path.app(), "app");
        assert_eq!(single_path.name(), "app");
        assert_eq!(single_path.len(), 1);
        assert!(!single_path.is_empty());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_cmd_path_display() {
        let path = CmdPath::new("myapp", &["subcmd", "nested"]);
        let display_str = format!("{path}");
        assert_eq!(display_str, "myapp subcmd nested");

        let single_path = CmdPath::new("app", &[]);
        let single_display = format!("{single_path}");
        assert_eq!(single_display, "app");
    }

    #[test]
    fn test_cmd_help_creation() {
        let action_help = ActionHelp {
            opts: &[],
            groups: &[],
            subhelps: None,
        };

        let cmd_help = CmdHelp::new(
            "testcmd",
            "A test command",
            Some("testing"),
            action_help,
            &[],
        );

        assert_eq!(cmd_help.name, "testcmd");
        assert_eq!(cmd_help.description, "A test command");
        assert_eq!(cmd_help.category, Some("testing"));
        assert_eq!(cmd_help.opts.len(), 0);
        assert_eq!(cmd_help.builtin_opts.len(), 0);
        assert_eq!(cmd_help.groups.len(), 0);
        assert!(cmd_help.subhelps.is_none());
    }

    #[test]
    fn test_cmd_help_into_help_app_level() {
        let action_help = ActionHelp {
            opts: &[],
            groups: &[],
            subhelps: None,
        };

        let cmd_help =
            CmdHelp::new("myapp", "My application", None, action_help, &[]);

        let help = cmd_help.into_help("myapp", &[], None, None);

        assert_eq!(help.name, "myapp");
        assert_eq!(help.description, "My application");
        assert_eq!(help.path.app(), "myapp");
        assert_eq!(help.path.name(), "myapp");
        assert!(help.is_app());
        assert!(!help.has_subcmds());
        // For app-level help, global opts should be None
        assert!(help.global_opts.is_none());
        assert!(help.builtin_global_opts.is_none());
    }

    #[test]
    fn test_cmd_help_into_help_subcommand_level() {
        let action_help = ActionHelp {
            opts: &[],
            groups: &[],
            subhelps: None,
        };

        let cmd_help = CmdHelp::new(
            "subcmd",
            "A subcommand",
            Some("category"),
            action_help,
            &[],
        );

        let help = cmd_help.into_help("myapp", &["subcmd"], None, None);

        assert_eq!(help.name, "subcmd");
        assert_eq!(help.description, "A subcommand");
        assert_eq!(help.path.app(), "myapp");
        assert_eq!(help.path.name(), "subcmd");
        assert!(!help.is_app());
        assert!(!help.has_subcmds());
        // For subcommand help, global opts should be preserved
        assert!(help.global_opts.is_none());
        assert!(help.builtin_global_opts.is_none());
    }

    #[test]
    fn test_cmd_help_with_subcommands() {
        let mut pool = ArrayPool::<CmdHelp<'_>, 10>::new();
        let pool_ref = pool.as_pool();

        // Create subcommands
        let (mut subhelps_slice, _) = pool_ref.alloc(2);
        let sub1 = CmdHelp {
            name: "sub1",
            description: "First subcommand",
            category: None,
            opts: &[],
            builtin_opts: &[],
            groups: &[],
            subhelps: None,
        };
        let sub2 = CmdHelp {
            name: "sub2",
            description: "Second subcommand",
            category: Some("group"),
            opts: &[],
            builtin_opts: &[],
            groups: &[],
            subhelps: None,
        };
        let subhelps = subhelps_slice.assign(&[sub1, sub2]);

        let action_help = ActionHelp {
            opts: &[],
            groups: &[],
            subhelps: Some(subhelps),
        };

        let cmd_help =
            CmdHelp::new("parent", "Parent command", None, action_help, &[]);

        assert!(cmd_help.subhelps.is_some());
        let subhelps = cmd_help.subhelps.unwrap();
        assert_eq!(subhelps.len(), 2);
        assert_eq!(subhelps[0].name, "sub1");
        assert_eq!(subhelps[1].name, "sub2");
        assert_eq!(subhelps[1].category, Some("group"));

        // Test into_help with subcommands
        let help = cmd_help.into_help("myapp", &["parent"], None, None);
        assert!(help.has_subcmds());
    }

    #[test]
    fn test_cmd_help_find() {
        let mut help_pool = ArrayPool::<CmdHelp<'_>, 10>::new();
        let help_pool_ref = help_pool.as_pool();

        // Create nested structure for testing find
        let (mut nested_slice, remaining_help_pool) = help_pool_ref.alloc(1);
        let nested_cmd = CmdHelp {
            name: "target",
            description: "Target command",
            category: None,
            opts: &[],
            builtin_opts: &[],
            groups: &[],
            subhelps: None,
        };
        let nested_subhelps = nested_slice.assign(&[nested_cmd]);

        let (mut subhelps_slice, _) = remaining_help_pool.alloc(2);
        let sub1 = CmdHelp {
            name: "sub1",
            description: "First subcommand",
            category: None,
            opts: &[],
            builtin_opts: &[],
            groups: &[],
            subhelps: Some(nested_subhelps),
        };
        let sub2 = CmdHelp {
            name: "sub2",
            description: "Second subcommand",
            category: None,
            opts: &[],
            builtin_opts: &[],
            groups: &[],
            subhelps: None,
        };
        let subhelps = subhelps_slice.assign(&[sub1, sub2]);

        let parent = CmdHelp {
            name: "parent",
            description: "Parent command",
            category: None,
            opts: &[],
            builtin_opts: &[],
            groups: &[],
            subhelps: Some(subhelps),
        };

        // Test finding nested command using full path
        let result = parent.find(&["sub1", "target"], "target");
        assert!(result.is_some());
        let found_help = result.unwrap();
        assert_eq!(found_help.name, "target");

        // Test finding direct subcommand
        let result = parent.find(&["sub2"], "sub2");
        assert!(result.is_some());
        let found_help = result.unwrap();
        assert_eq!(found_help.name, "sub2");

        // Test finding parent itself (empty path means root)
        let result = parent.find(&[], "parent");
        assert!(result.is_some());
        let found_help = result.unwrap();
        assert_eq!(found_help.name, "parent");

        // Test finding non-existent command
        let result = parent.find(&["nonexistent"], "nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_help_lookup_when_option_follows_command() {
        let hello_help_entry = CmdHelp {
            name: "hello",
            description: "hello command",
            category: None,
            opts: &[],
            builtin_opts: &[],
            groups: &[],
            subhelps: None,
        };

        let app_help_entry = CmdHelp {
            name: "app",
            description: "",
            category: None,
            opts: &[],
            builtin_opts: &[],
            groups: &[],
            subhelps: Some(&[hello_help_entry]),
        };

        // Simulate args: app hello -h
        let args: &[&str] = &["hello", "-h"];
        let raw_args = crate::RawArgs::new(args);

        // Consume the command name as run() would
        assert_eq!(raw_args.cmd(), Some("hello"));
        let pos = raw_args.current();
        assert_eq!(pos, 1);

        // Build the path slice (should not include "-h")
        let path = raw_args.cmd_path(0, pos);
        assert_eq!(path, &["hello"]);

        let result = app_help_entry.find(path, "hello");
        assert!(result.is_some());
        let help = result.unwrap();
        assert_eq!(help.name, "hello");
    }

    #[test]
    fn test_help_find_disambiguates_duplicate_leaf_commands_by_path() {
        let profiles_list_help_entry = CmdHelp {
            name: "list",
            description: "profiles list",
            category: None,
            opts: &[],
            builtin_opts: &[],
            groups: &[],
            subhelps: None,
        };

        let products_list_help_entry = CmdHelp {
            name: "list",
            description: "products list",
            category: None,
            opts: &[],
            builtin_opts: &[],
            groups: &[],
            subhelps: None,
        };

        let profile_help_entry = CmdHelp {
            name: "profile",
            description: "",
            category: None,
            opts: &[],
            builtin_opts: &[],
            groups: &[],
            subhelps: Some(&[profiles_list_help_entry]),
        };

        let products_help_entry = CmdHelp {
            name: "products",
            description: "",
            category: None,
            opts: &[],
            builtin_opts: &[],
            groups: &[],
            subhelps: Some(&[products_list_help_entry]),
        };

        let tools_help_entry = CmdHelp {
            name: "tools",
            description: "",
            category: None,
            opts: &[],
            builtin_opts: &[],
            groups: &[],
            subhelps: Some(&[profile_help_entry, products_help_entry]),
        };

        let app_help_entry = CmdHelp {
            name: "app",
            description: "",
            category: None,
            opts: &[],
            builtin_opts: &[],
            groups: &[],
            subhelps: Some(&[tools_help_entry]),
        };

        let path_profile = ["tools", "profile", "list"];
        let help = app_help_entry.find(&path_profile, "list").unwrap();
        assert_eq!(help.description, "profiles list");

        let path_products = ["tools", "products", "list"];
        let help = app_help_entry.find(&path_products, "list").unwrap();
        assert_eq!(help.description, "products list");
    }

    #[test]
    fn test_help_depth_calculation() {
        let mut pool = ArrayPool::<CmdHelp<'_>, 10>::new();
        let pool_ref = pool.as_pool();

        // Create a deep nested structure
        let (mut level3_slice, remaining_pool) = pool_ref.alloc(1);
        let level3_cmd = CmdHelp {
            name: "level3",
            description: "Level 3 command",
            category: None,
            opts: &[],
            builtin_opts: &[],
            groups: &[],
            subhelps: None,
        };
        let level3_subhelps = level3_slice.assign(&[level3_cmd]);

        let (mut level2_slice, remaining_pool2) = remaining_pool.alloc(1);
        let level2_cmd = CmdHelp {
            name: "level2",
            description: "Level 2 command",
            category: None,
            opts: &[],
            builtin_opts: &[],
            groups: &[],
            subhelps: Some(level3_subhelps),
        };
        let level2_subhelps = level2_slice.assign(&[level2_cmd]);

        let (mut level1_slice, _) = remaining_pool2.alloc(1);
        let level1_cmd = CmdHelp {
            name: "level1",
            description: "Level 1 command",
            category: None,
            opts: &[],
            builtin_opts: &[],
            groups: &[],
            subhelps: Some(level2_subhelps),
        };
        let level1_subhelps = level1_slice.assign(&[level1_cmd]);

        let action_help = ActionHelp {
            opts: &[],
            groups: &[],
            subhelps: Some(level1_subhelps),
        };

        let root_help =
            CmdHelp::new("root", "Root command", None, action_help, &[]);

        let help = root_help.into_help("myapp", &[], None, None);

        // Should calculate depth as 4: root -> level1 -> level2 -> level3
        assert_eq!(help.depth(), 4);
    }

    #[test]
    fn test_help_depth_single_level() {
        let action_help = ActionHelp {
            opts: &[],
            groups: &[],
            subhelps: None,
        };

        let cmd_help =
            CmdHelp::new("simple", "Simple command", None, action_help, &[]);

        let help = cmd_help.into_help("myapp", &[], None, None);

        // Should have depth 1 (just the command itself)
        assert_eq!(help.depth(), 1);
    }

    #[test]
    fn test_action_help_creation() {
        let action_help = ActionHelp {
            opts: &[],
            groups: &[],
            subhelps: None,
        };

        assert_eq!(action_help.opts.len(), 0);
        assert_eq!(action_help.groups.len(), 0);
        assert!(action_help.subhelps.is_none());
    }

    #[test]
    fn test_cmd_help_default() {
        let default_help = CmdHelp::default();

        assert_eq!(default_help.name, "");
        assert_eq!(default_help.description, "");
        assert!(default_help.category.is_none());
        assert!(default_help.opts.is_empty());
        assert!(default_help.builtin_opts.is_empty());
        assert!(default_help.groups.is_empty());
        assert!(default_help.subhelps.is_none());
    }

    #[test]
    fn test_cmd_help_clone_copy() {
        let original = CmdHelp {
            name: "test",
            description: "Test command",
            category: Some("testing"),
            opts: &[],
            builtin_opts: &[],
            groups: &[],
            subhelps: None,
        };

        let cloned = original;
        let copied = original;

        assert_eq!(cloned.name, "test");
        assert_eq!(copied.name, "test");
        assert_eq!(cloned.description, original.description);
        assert_eq!(copied.category, original.category);
    }
}
