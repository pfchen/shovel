//! Shell completion script generation.
//!
//! This module provides functionality for generating shell completion scripts that enable
//! tab completion for Shovel CLI applications. It supports Bash, Zsh, and Fish shells
//! with intelligent completion of commands, subcommands, options, and arguments.
//!
//! # Core Types
//!
//! ## [`Shell`] - Shell Type Enumeration
//!
//! Represents the target shell for completion script generation:
//! - `Shell::Bash` - Bash shell completion
//! - `Shell::Zsh` - Zsh shell completion
//! - `Shell::Fish` - Fish shell completion
//!
//! # Completion Features
//!
//! ## Command Completion
//! - Complete command and subcommand names
//! - Context-aware suggestions based on command hierarchy
//! - Fuzzy matching for partial command names
//!
//! ## Option Completion
//! - Complete long options (`--option`)
//! - Complete short options (`-o`)
//! - Value completion for options that accept arguments
//! - Flag recognition (options that don't take values)
//!
//! ## Argument Completion
//! - File path completion for file arguments
//! - Custom completion for specific argument types
//! - Multiple argument completion support
//!
//! # Usage
//!
//! ## Automatic Integration
//! When the `completion` feature is enabled, Shovel automatically adds a `--completion`
//! option to your application:
//!
//! ```bash
//! # Generate completion script for Bash
//! myapp --completion bash > ~/.bash_completion.d/myapp
//! source ~/.bash_completion.d/myapp
//!
//! # Generate completion script for Zsh
//! myapp --completion zsh > ~/.zsh/completions/_myapp
//!
//! # Generate completion script for Fish
//! myapp --completion fish > ~/.config/fish/completions/myapp.fish
//! ```
//!
//! ## Manual Script Generation
//! ```rust
//! # use shovel::*;
//! # use std::io::Write;
//! # fn main() {
//! let app = App {
//!     name: "myapp",
//!     description: "My application",
//!     version: "0.1.0",
//!     authors: ["Author Name <author@example.com>"],
//!     copyright: "(c) 2026 Company Name",
//!     action: Action::subcmds((
//!         Cmd { name: "build", description: "Build project", category: None, action: Action::no_args(|| {}) },
//!         Cmd { name: "test", description: "Run tests", category: None, action: Action::no_args(|| {}) },
//!     )),
//! };
//!
//! // Generate completion for Bash
//! let mut output = Vec::new();
//! let mut writer = IoWriteAdapter::new(&mut output);
//! // completion::write_completion(&mut writer, &app, Shell::Bash).expect("Failed to generate Bash completion");
//! # }
//! ```
//!
//! # Shell-Specific Features
//!
//! ## Bash Completion
//! - Uses `complete` builtin for registration
//! - Supports programmable completion with `compgen`
//! - File and directory completion for path arguments
//! - Custom completion functions for complex scenarios
//!
//! ## Zsh Completion
//! - Uses Zsh's completion system (`_arguments`)
//! - Rich completion descriptions and help text
//! - Advanced completion contexts and modifiers
//! - Integration with Zsh's completion menu system
//!
//! ## Fish Completion
//! - Uses Fish's `complete` command
//! - Condition-based completion with `--condition`
//! - Rich descriptions and help integration
//! - File completion with custom filters
//!
//! # Performance
//!
//! - **Generation**: Completion scripts are generated once at install time
//! - **Runtime**: Minimal overhead during shell completion
//! - **Caching**: Shells cache completion results for performance
//! - **Lazy Loading**: Completion scripts are loaded only when needed
//!
//! # Feature Requirements
//!
//! This module is only available when the `completion` feature is enabled:
//!
//! ```toml
//! [dependencies]
//! shovel = { version = "*", features = ["completion"] }
//! ```
//!
//! The `completion` feature DOES NOT require the `std` feature as it uses stack allocation
//! for generating completion scripts.
//!
//! # Examples
//!
//! ## Application with Rich Completion
//! ```rust
//! # use shovel::*;
//! # #[derive(Opts)] struct BuildOpts { release: bool, target: Option<String> }
//! # #[derive(Args)] struct BuildArgs { project: String }
//! let app = App {
//!     name: "cargo-like",
//!     description: "Cargo-like build tool",
//!     version: "0.1.0",
//!     authors: ["Author Name <author@example.com>"],
//!     copyright: "(c) 2026 Company Name",
//!     action: Action::subcmds((
//!         Cmd {
//!             name: "build",
//!             description: "Build the project",
//!             category: Some("development"),
//!             action: Action::opts_args(|opts: BuildOpts, args: BuildArgs| {
//!                 println!("Building {} (release: {})", args.project, opts.release);
//!             }),
//!         },
//!         Cmd {
//!             name: "clean",
//!             description: "Clean build artifacts",
//!             category: Some("development"),
//!             action: Action::no_args(|| println!("Cleaning...")),
//!         },
//!     )),
//! };
//! ```
//!
//! This will generate completion that includes:
//! - Command completion: `build`, `clean`
//! - Option completion: `--release`, `--target`
//! - Help integration: Descriptions appear in completion menus
//! - Context awareness: Options only appear for relevant commands
//!
//! See also: [`App`](crate::App), [`Cmd`](crate::Cmd), [`Writer`](crate::Writer)

use core::fmt;
use core::str::FromStr;

use crate::Help;

/// Represents the type of shell to use for completion generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    /// Bash shell
    Bash,
    /// Zsh shell
    Zsh,
    /// Fish shell
    Fish,
}

impl FromStr for Shell {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bash" => Ok(Shell::Bash),
            "zsh" => Ok(Shell::Zsh),
            "fish" => Ok(Shell::Fish),
            _ => Err("Unknown shell"),
        }
    }
}

impl fmt::Display for Shell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bash => f.write_str("bash"),
            Self::Zsh => f.write_str("zsh"),
            Self::Fish => f.write_str("fish"),
        }
    }
}

/// A lightweight node that lives on the stack.
/// It forms a reverse linked-list (pointing to parent).
struct PathNode<'a> {
    parent: Option<&'a PathNode<'a>>,
    name: &'a str,
}

impl PathNode<'_> {
    // Helper to recursively write the path from Root -> Leaf
    // e.g., "subcmd1 subcmd2"
    fn write_path(
        &self,
        writer: &mut dyn fmt::Write,
        separator: &str,
    ) -> fmt::Result {
        if let Some(parent) = self.parent {
            parent.write_path(writer, separator)?;
            writer.write_str(separator)?;
        }
        writer.write_str(self.name)
    }
}

pub(crate) fn write_completion(
    writer: &mut dyn fmt::Write,
    help: Help,
    shell: Shell,
) -> fmt::Result {
    match shell {
        Shell::Bash => write_bash_completion(writer, &help),
        Shell::Zsh => write_zsh_completion(writer, &help),
        Shell::Fish => write_fish_completion(writer, &help),
    }
}

fn write_zsh_description(
    writer: &mut dyn fmt::Write,
    description: &str,
) -> fmt::Result {
    for c in description.chars() {
        match c {
            '\'' => writer.write_str("'\\''")?,
            ']' => writer.write_str(r"\]")?,
            '\n' | '\r' => writer.write_char(' ')?,
            _ => writer.write_char(c)?,
        }
    }
    Ok(())
}

fn write_fish_description(
    writer: &mut dyn fmt::Write,
    description: &str,
) -> fmt::Result {
    for c in description.chars() {
        match c {
            '\\' => writer.write_str("\\\\")?,
            '"' => writer.write_str("\\\"")?,
            '$' => writer.write_str("\\$")?,
            '\n' | '\r' => writer.write_char(' ')?,
            _ => writer.write_char(c)?,
        }
    }
    Ok(())
}

fn write_option_takes_value_cases(
    writer: &mut dyn fmt::Write,
    help: &Help,
) -> fmt::Result {
    fn write_opt_case(
        writer: &mut dyn fmt::Write,
        name: &str,
        short: Option<char>,
    ) -> fmt::Result {
        writeln!(writer, "        --{name}|--{name}=*) return 0 ;;")?;
        if let Some(c) = short {
            writeln!(writer, "        -{c}|-{c}=*) return 0 ;;")?;
        }
        Ok(())
    }

    if let Some(global_opts) = help.global_opts {
        for opt in global_opts {
            if !opt.flag {
                write_opt_case(writer, opt.name, opt.short)?;
            }
        }
    }
    if let Some(builtin_global_opts) = help.builtin_global_opts {
        for opt in builtin_global_opts {
            if !opt.flag {
                write_opt_case(writer, opt.name, opt.short)?;
            }
        }
    }
    for opt in help.opts {
        if !opt.flag {
            write_opt_case(writer, opt.name, opt.short)?;
        }
    }
    for opt in help.builtin_opts {
        if !opt.flag {
            write_opt_case(writer, opt.name, opt.short)?;
        }
    }

    if let Some(subhelps) = help.subhelps {
        for subhelp in subhelps {
            let sub_help = Help {
                path: help.path,
                name: subhelp.name,
                description: subhelp.description,
                global_opts: help.global_opts,
                builtin_global_opts: help.builtin_global_opts,
                opts: subhelp.opts,
                builtin_opts: subhelp.builtin_opts,
                groups: subhelp.groups,
                subhelps: subhelp.subhelps,
            };
            write_option_takes_value_cases(writer, &sub_help)?;
        }
    }

    Ok(())
}

fn write_fish_option_takes_value_cases(
    writer: &mut dyn fmt::Write,
    help: &Help,
) -> fmt::Result {
    fn write_opt_case(
        writer: &mut dyn fmt::Write,
        name: &str,
        short: Option<char>,
    ) -> fmt::Result {
        writeln!(writer, "        case {name}")?;
        writeln!(writer, "            return 0")?;
        if let Some(c) = short {
            writeln!(writer, "        case {c}")?;
            writeln!(writer, "            return 0")?;
        }
        Ok(())
    }

    if let Some(global_opts) = help.global_opts {
        for opt in global_opts {
            if !opt.flag {
                write_opt_case(writer, opt.name, opt.short)?;
            }
        }
    }
    if let Some(builtin_global_opts) = help.builtin_global_opts {
        for opt in builtin_global_opts {
            if !opt.flag {
                write_opt_case(writer, opt.name, opt.short)?;
            }
        }
    }
    for opt in help.opts {
        if !opt.flag {
            write_opt_case(writer, opt.name, opt.short)?;
        }
    }
    for opt in help.builtin_opts {
        if !opt.flag {
            write_opt_case(writer, opt.name, opt.short)?;
        }
    }

    if let Some(subhelps) = help.subhelps {
        for subhelp in subhelps {
            let sub_help = Help {
                path: help.path,
                name: subhelp.name,
                description: subhelp.description,
                global_opts: help.global_opts,
                builtin_global_opts: help.builtin_global_opts,
                opts: subhelp.opts,
                builtin_opts: subhelp.builtin_opts,
                groups: subhelp.groups,
                subhelps: subhelp.subhelps,
            };
            write_fish_option_takes_value_cases(writer, &sub_help)?;
        }
    }

    Ok(())
}

fn write_bash_option_takes_value(
    writer: &mut dyn fmt::Write,
    help: &Help,
    app_name: &str,
) -> fmt::Result {
    writeln!(writer, "__{app_name}_option_takes_value() {{")?;
    writeln!(writer, "    case \"$1\" in")?;
    write_option_takes_value_cases(writer, help)?;
    writeln!(writer, "    esac")?;
    writeln!(writer, "    return 1")?;
    writeln!(writer, "}}")?;
    writeln!(writer)?;
    Ok(())
}

fn write_bash_subcommands_for_path(
    writer: &mut dyn fmt::Write,
    help: &Help,
    app_name: &str,
) -> fmt::Result {
    writeln!(writer, "__{app_name}_subcommands_for_path() {{")?;
    writeln!(writer, "    local path=\"$1\"")?;
    writeln!(writer, "    case \"$path\" in")?;
    write_bash_subcommands_case(writer, help, None)?;
    writeln!(writer, "        *) echo \"\" ;;")?;
    writeln!(writer, "    esac")?;
    writeln!(writer, "}}")?;
    writeln!(writer)?;
    Ok(())
}

fn write_bash_subcommands_case(
    writer: &mut dyn fmt::Write,
    help: &Help,
    path: Option<&PathNode>,
) -> fmt::Result {
    write!(writer, "        \"")?;
    if let Some(p) = path {
        p.write_path(writer, " ")?;
    }
    write!(writer, "\") echo \"")?;
    if let Some(subhelps) = help.subhelps {
        for subhelp in subhelps {
            write!(writer, "{} ", subhelp.name)?;
        }
    }
    writeln!(writer, "\" ;;")?;

    if let Some(subhelps) = help.subhelps {
        for subhelp in subhelps {
            let next_node = PathNode {
                parent: path,
                name: subhelp.name,
            };
            let sub_help = Help {
                path: help.path,
                name: subhelp.name,
                description: subhelp.description,
                global_opts: help.global_opts,
                builtin_global_opts: help.builtin_global_opts,
                opts: subhelp.opts,
                builtin_opts: subhelp.builtin_opts,
                groups: subhelp.groups,
                subhelps: subhelp.subhelps,
            };
            write_bash_subcommands_case(writer, &sub_help, Some(&next_node))?;
        }
    }

    Ok(())
}

fn write_zsh_option_takes_value(
    writer: &mut dyn fmt::Write,
    help: &Help,
    app_name: &str,
) -> fmt::Result {
    writeln!(writer, "function __{app_name}_option_takes_value() {{")?;
    writeln!(writer, "    case \"$1\" in")?;
    write_option_takes_value_cases(writer, help)?;
    writeln!(writer, "    esac")?;
    writeln!(writer, "    return 1")?;
    writeln!(writer, "}}")?;
    writeln!(writer)?;
    Ok(())
}

fn write_fish_option_takes_value(
    writer: &mut dyn fmt::Write,
    help: &Help,
    app_name: &str,
) -> fmt::Result {
    writeln!(writer, "function __fish_{app_name}_option_takes_value")?;
    writeln!(writer, "    set -l opt $argv[1]")?;
    writeln!(writer, "    if string match -qr '^--' -- $opt")?;
    writeln!(writer, "        if string match -qr '=' -- $opt")?;
    writeln!(writer, "            return 0")?;
    writeln!(writer, "        end")?;
    writeln!(
        writer,
        "        set opt (string replace -r '^--' '' -- $opt)"
    )?;
    writeln!(writer, "    else if string match -qr '^-' -- $opt")?;
    writeln!(writer, "        if string match -qr '=' -- $opt")?;
    writeln!(writer, "            return 0")?;
    writeln!(writer, "        end")?;
    writeln!(writer, "        if test (string length -- $opt) -ne 2")?;
    writeln!(writer, "            return 1")?;
    writeln!(writer, "        end")?;
    writeln!(writer, "        set opt (string sub -s 2 -l 1 -- $opt)")?;
    writeln!(writer, "    else")?;
    writeln!(writer, "        return 1")?;
    writeln!(writer, "    end")?;
    writeln!(writer, "    switch $opt")?;
    write_fish_option_takes_value_cases(writer, help)?;
    writeln!(writer, "    end")?;
    writeln!(writer, "    return 1")?;
    writeln!(writer, "end")?;
    writeln!(writer)?;
    Ok(())
}

fn write_fish_subcommands_for_path(
    writer: &mut dyn fmt::Write,
    help: &Help,
    app_name: &str,
) -> fmt::Result {
    writeln!(writer, "function __fish_{app_name}_subcommands_for_path")?;
    writeln!(writer, "    set -l path (string join ' ' $argv)")?;
    writeln!(writer, "    switch $path")?;
    write_fish_subcommands_case(writer, help, None)?;
    writeln!(writer, "    end")?;
    writeln!(writer, "end")?;
    writeln!(writer)?;
    Ok(())
}

fn write_fish_subcommands_case(
    writer: &mut dyn fmt::Write,
    help: &Help,
    path: Option<&PathNode>,
) -> fmt::Result {
    write!(writer, "        case '")?;
    if let Some(p) = path {
        p.write_path(writer, " ")?;
    }
    writeln!(writer, "'")?;
    write!(writer, "            echo")?;
    if let Some(subhelps) = help.subhelps {
        for subhelp in subhelps {
            write!(writer, " {}", subhelp.name)?;
        }
    }
    writeln!(writer)?;

    if let Some(subhelps) = help.subhelps {
        for subhelp in subhelps {
            let next_node = PathNode {
                parent: path,
                name: subhelp.name,
            };
            let sub_help = Help {
                path: help.path,
                name: subhelp.name,
                description: subhelp.description,
                global_opts: help.global_opts,
                builtin_global_opts: help.builtin_global_opts,
                opts: subhelp.opts,
                builtin_opts: subhelp.builtin_opts,
                groups: subhelp.groups,
                subhelps: subhelp.subhelps,
            };
            write_fish_subcommands_case(writer, &sub_help, Some(&next_node))?;
        }
    }

    Ok(())
}

fn write_bash_completion(
    writer: &mut dyn fmt::Write,
    help: &Help,
) -> fmt::Result {
    let app_name = help.name;

    write_bash_option_takes_value(writer, help, app_name)?;
    write_bash_subcommands_for_path(writer, help, app_name)?;

    // Start of the completion function
    writeln!(writer, "_{app_name}_completion() {{")?;
    writeln!(writer, "    local cur prev words cword")?;
    writeln!(writer, "    local -a cmd_path=()")?;
    writeln!(writer, "    COMPREPLY=()")?;
    writeln!(writer, "    cur=\"${{COMP_WORDS[COMP_CWORD]}}\"")?;
    writeln!(writer, "    prev=\"${{COMP_WORDS[COMP_CWORD-1]}}\"")?;
    writeln!(writer, "    words=(\"${{COMP_WORDS[@]}}\")")?;
    writeln!(writer, "    cword=${{COMP_CWORD}}")?;
    writeln!(writer)?;

    // Helper function to get current command path
    writeln!(writer, "    local path=")?;
    writeln!(writer, "    for ((i=1; i<cword; i++)); do")?;
    writeln!(writer, "        local word=${{words[i]}}")?;
    writeln!(writer, "        if [[ \"$word\" == \"--\" ]]; then")?;
    writeln!(writer, "            break")?;
    writeln!(writer, "        fi")?;
    writeln!(writer, "        if [[ \"$word\" == -* ]]; then")?;
    writeln!(
        writer,
        "            if __{app_name}_option_takes_value \"$word\"; then"
    )?;
    writeln!(writer, "                if [[ \"$word\" != *=* ]]; then")?;
    writeln!(writer, "                    ((i++))")?;
    writeln!(writer, "                fi")?;
    writeln!(writer, "            fi")?;
    writeln!(writer, "            continue")?;
    writeln!(writer, "        fi")?;
    writeln!(writer, "        local subs")?;
    writeln!(
        writer,
        "        subs=$(__{app_name}_subcommands_for_path \"$path\")"
    )?;
    writeln!(
        writer,
        "        if [[ \" $subs \" == *\" $word \"* ]]; then"
    )?;
    writeln!(writer, "            cmd_path+=(\"$word\")")?;
    writeln!(writer, "            if [[ -z \"$path\" ]]; then")?;
    writeln!(writer, "                path=\"$word\"")?;
    writeln!(writer, "            else")?;
    writeln!(writer, "                path=\"$path $word\"")?;
    writeln!(writer, "            fi")?;
    writeln!(writer, "        else")?;
    writeln!(writer, "            break")?;
    writeln!(writer, "        fi")?;
    writeln!(writer, "    done")?;
    writeln!(writer)?;

    // Generate completion logic recursively
    write_bash_completion_recursive(writer, help, None, 0)?;

    writeln!(writer, "    return 0")?;
    writeln!(writer, "}}")?;
    writeln!(writer)?;

    // Register the completion function
    writeln!(writer, "complete -F _{app_name}_completion {app_name}")?;

    Ok(())
}

fn write_bash_checks(
    writer: &mut dyn fmt::Write,
    node: &PathNode,
    index: usize,
) -> fmt::Result {
    // Recurse to root first
    if let Some(parent) = node.parent {
        write_bash_checks(writer, parent, index - 1)?;
        writer.write_str(" && ")?;
    }

    // Write check for this node
    write!(writer, "\"${{cmd_path[{}]}}\" == \"{}\"", index, node.name)?;
    Ok(())
}

fn write_bash_completion_recursive(
    writer: &mut dyn fmt::Write,
    help: &Help,
    path: Option<&PathNode>,
    depth: usize,
) -> fmt::Result {
    // Generate condition for current command path
    if depth == 0 {
        writeln!(writer, "    # Root level completion")?;
        writeln!(writer, "    if [[ ${{#cmd_path[@]}} -eq 0 ]]; then")?;
    } else {
        write!(writer, "    elif [[ ${{#cmd_path[@]}} -eq {depth}")?;
        if let Some(p) = path {
            writer.write_str(" && ")?;
            write_bash_checks(writer, p, depth - 1)?;
        }
        writeln!(writer, " ]]; then")?;
    }

    // Generate options for current level
    write!(writer, "        local opts=\"")?;

    // Add global options if at root level
    if depth == 0 {
        if let Some(global_opts) = help.global_opts {
            for opt in global_opts {
                write!(writer, "--{} ", opt.name)?;
                if let Some(c) = opt.short {
                    write!(writer, "-{c} ")?;
                }
            }
        }
        if let Some(builtin_global_opts) = help.builtin_global_opts {
            for opt in builtin_global_opts {
                write!(writer, "--{} ", opt.name)?;
                if let Some(c) = opt.short {
                    write!(writer, "-{c} ")?;
                }
            }
        }
    }

    // Add command-specific options
    for opt in help.opts {
        write!(writer, "--{} ", opt.name)?;
        if let Some(c) = opt.short {
            write!(writer, "-{c} ")?;
        }
    }
    for opt in help.builtin_opts {
        write!(writer, "--{} ", opt.name)?;
        if let Some(c) = opt.short {
            write!(writer, "-{c} ")?;
        }
    }
    writeln!(writer, "\"")?;

    // Generate subcommands for current level
    if let Some(subhelps) = help.subhelps {
        write!(writer, "        local subcommands=\"")?;
        for subhelp in subhelps {
            write!(writer, "{} ", subhelp.name)?;
        }
        writeln!(writer, "\"")?;
        writeln!(
            writer,
            "        COMPREPLY=( $(compgen -W \"$opts $subcommands\" -- \"$cur\") )"
        )?;

        // Recursively handle subcommands
        for subhelp in subhelps {
            let next_node = PathNode {
                parent: path,
                name: subhelp.name,
            };

            // Create a Help struct for the subcommand
            let sub_help = Help {
                path: help.path, // Keep the same path structure
                name: subhelp.name,
                description: subhelp.description,
                global_opts: help.global_opts,
                builtin_global_opts: help.builtin_global_opts,
                opts: subhelp.opts,
                builtin_opts: subhelp.builtin_opts,
                groups: subhelp.groups,
                subhelps: subhelp.subhelps,
            };

            write_bash_completion_recursive(
                writer,
                &sub_help,
                Some(&next_node),
                depth + 1,
            )?;
        }
    } else {
        writeln!(
            writer,
            "        COMPREPLY=( $(compgen -W \"$opts\" -- \"$cur\") )"
        )?;
    }

    // Only close the if/elif chain at root level
    if depth == 0 {
        writeln!(writer, "    fi")?;
    }
    Ok(())
}

fn write_zsh_completion(
    writer: &mut dyn fmt::Write,
    help: &Help,
) -> fmt::Result {
    let app_name = help.name;

    // Start of the completion function
    writeln!(writer, "#compdef {app_name}")?;
    write_zsh_option_takes_value(writer, help, app_name)?;
    writeln!(writer, "function _{app_name}_completion() {{")?;
    writeln!(writer, "    local context state line")?;
    writeln!(writer, "    typeset -A opt_args")?;
    writeln!(writer)?;

    // Generate the main completion function
    write_zsh_completion_recursive(writer, help, None, 0, app_name)?;

    writeln!(writer, "}}")?;
    writeln!(writer, "_{app_name}_completion \"$@\"")?;

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn write_zsh_completion_recursive(
    writer: &mut dyn fmt::Write,
    help: &Help,
    path: Option<&PathNode>,
    depth: usize,
    app_name: &str,
) -> fmt::Result {
    if depth == 0 {
        // Root level - set up the main _arguments call
        writeln!(writer, "    _arguments \\")?;

        // Add global options
        if let Some(global_opts) = help.global_opts {
            for opt in global_opts {
                if let Some(c) = opt.short {
                    write!(
                        writer,
                        "        '(-{} --{})'{{-{},--{}}}[",
                        c, opt.name, c, opt.name
                    )?;
                } else {
                    write!(writer, "        '--{}[", opt.name)?;
                }
                write_zsh_description(writer, opt.description)?;
                if opt.flag {
                    writeln!(writer, "]' \\")?;
                } else {
                    writeln!(writer, "]: : ' \\")?;
                }
            }
        }
        if let Some(builtin_global_opts) = help.builtin_global_opts {
            for opt in builtin_global_opts {
                if let Some(c) = opt.short {
                    write!(
                        writer,
                        "        '(-{} --{})'{{-{},--{}}}[",
                        c, opt.name, c, opt.name
                    )?;
                } else {
                    write!(writer, "        '--{}[", opt.name)?;
                }
                write_zsh_description(writer, opt.description)?;
                if opt.flag {
                    writeln!(writer, "]' \\")?;
                } else {
                    writeln!(writer, "]: : ' \\")?;
                }
            }
        }

        // Add command-specific options
        for opt in help.opts {
            if let Some(c) = opt.short {
                write!(
                    writer,
                    "        '(-{} --{})'{{-{},--{}}}[",
                    c, opt.name, c, opt.name
                )?;
            } else {
                write!(writer, "        '--{}[", opt.name)?;
            }
            write_zsh_description(writer, opt.description)?;
            if opt.flag {
                writeln!(writer, "]' \\")?;
            } else {
                writeln!(writer, "]: : ' \\")?;
            }
        }
        for opt in help.builtin_opts {
            if let Some(c) = opt.short {
                write!(
                    writer,
                    "        '(-{} --{})'{{-{},--{}}}[",
                    c, opt.name, c, opt.name
                )?;
            } else {
                write!(writer, "        '--{}[", opt.name)?;
            }
            write_zsh_description(writer, opt.description)?;
            if opt.flag {
                writeln!(writer, "]' \\")?;
            } else {
                writeln!(writer, "]: : ' \\")?;
            }
        }

        // Add subcommand completion
        if help.subhelps.is_some() {
            writeln!(writer, "        '1: :->commands' \\")?;
            writeln!(writer, "        '*:: :->args'")?;
        } else {
            writeln!(writer, "        '*:: :->args'")?;
        }

        writeln!(writer)?;
        writeln!(writer, "    case $state in")?;
        writeln!(writer, "        commands)")?;

        if let Some(subhelps) = help.subhelps {
            writeln!(writer, "            local -a commands")?;
            writeln!(writer, "            commands=(")?;
            for subhelp in subhelps {
                write!(writer, "                '{}:", subhelp.name)?;
                write_zsh_description(writer, subhelp.description)?;
                writeln!(writer, "'")?;
            }
            writeln!(writer, "            )")?;
            writeln!(writer, "            _describe 'command' commands")?;
        }

        writeln!(writer, "            ;;")?;
        writeln!(writer, "        args)")?;

        // Handle subcommand arguments
        if let Some(subhelps) = help.subhelps {
            writeln!(writer, "            local subcmd=\"\"")?;
            writeln!(writer, "            local -a args")?;
            writeln!(writer, "            args=($words[2,-1])")?;
            writeln!(writer, "            local i=1")?;
            writeln!(writer, "            while (( i <= $#args )); do")?;
            writeln!(writer, "                local word=\"$args[$i]\"")?;
            writeln!(
                writer,
                "                if [[ \"$word\" == \"--\" ]]; then"
            )?;
            writeln!(writer, "                    break")?;
            writeln!(writer, "                fi")?;
            writeln!(writer, "                if [[ \"$word\" == -* ]]; then")?;
            writeln!(
                writer,
                "                    if __{app_name}_option_takes_value \"$word\"; then"
            )?;
            writeln!(
                writer,
                "                        if [[ \"$word\" != *=* ]]; then"
            )?;
            writeln!(writer, "                            (( i++ ))")?;
            writeln!(writer, "                        fi")?;
            writeln!(writer, "                    fi")?;
            writeln!(writer, "                    (( i++ ))")?;
            writeln!(writer, "                    continue")?;
            writeln!(writer, "                fi")?;
            writeln!(writer, "                subcmd=\"$word\"")?;
            writeln!(writer, "                break")?;
            writeln!(writer, "            done")?;
            writeln!(writer, "            case $subcmd in")?;
            for subhelp in subhelps {
                writeln!(writer, "                {}", subhelp.name)?;

                // Create a Help struct for the subcommand
                let sub_help = Help {
                    path: help.path,
                    name: subhelp.name,
                    description: subhelp.description,
                    global_opts: help.global_opts,
                    builtin_global_opts: help.builtin_global_opts,
                    opts: subhelp.opts,
                    builtin_opts: subhelp.builtin_opts,
                    groups: subhelp.groups,
                    subhelps: subhelp.subhelps,
                };

                let next_node = PathNode {
                    parent: path,
                    name: subhelp.name,
                };
                write_zsh_subcommand_completion(
                    writer,
                    &sub_help,
                    Some(&next_node),
                    1,
                )?;
                writeln!(writer, "                    ;;")?;
            }
            writeln!(writer, "            esac")?;
        }

        writeln!(writer, "            ;;")?;
        writeln!(writer, "    esac")?;
    }

    Ok(())
}

fn write_zsh_subcommand_completion(
    writer: &mut dyn fmt::Write,
    help: &Help,
    _path: Option<&PathNode>,
    _depth: usize,
) -> fmt::Result {
    // Generate options for this subcommand
    writeln!(writer, "                    _arguments \\")?;

    for opt in help.opts {
        if let Some(c) = opt.short {
            write!(
                writer,
                "                        '(-{} --{})'{{-{},--{}}}[",
                c, opt.name, c, opt.name
            )?;
        } else {
            write!(writer, "                        '--{}[", opt.name)?;
        }
        write_zsh_description(writer, opt.description)?;
        if opt.flag {
            writeln!(writer, "]' \\")?;
        } else {
            writeln!(writer, "]: : ' \\")?;
        }
    }
    for opt in help.builtin_opts {
        if let Some(c) = opt.short {
            write!(
                writer,
                "                        '(-{} --{})'{{-{},--{}}}[",
                c, opt.name, c, opt.name
            )?;
        } else {
            write!(writer, "                        '--{}[", opt.name)?;
        }
        write_zsh_description(writer, opt.description)?;
        if opt.flag {
            writeln!(writer, "]' \\")?;
        } else {
            writeln!(writer, "]: : ' \\")?;
        }
    }

    if let Some(subhelps) = help.subhelps {
        writeln!(writer, "                        '1: :->subcommands' \\")?;
        writeln!(writer, "                        '*:: :->subargs'")?;
        writeln!(writer)?;
        writeln!(writer, "                    case $state in")?;
        writeln!(writer, "                        subcommands)")?;
        writeln!(writer, "                            local -a subcommands")?;
        writeln!(writer, "                            subcommands=(")?;
        for subhelp in subhelps {
            write!(
                writer,
                "                                '{}:",
                subhelp.name
            )?;
            write_zsh_description(writer, subhelp.description)?;
            writeln!(writer, "'")?;
        }
        writeln!(writer, "                            )")?;
        writeln!(
            writer,
            "                            _describe 'subcommand' subcommands"
        )?;
        writeln!(writer, "                            ;;")?;
        writeln!(writer, "                    esac")?;
    } else {
        writeln!(writer, "                        '*:: :->files'")?;
        writeln!(writer)?;
        writeln!(writer, "                    case $state in")?;
        writeln!(writer, "                        files)")?;
        writeln!(writer, "                            _files")?;
        writeln!(writer, "                            ;;")?;
        writeln!(writer, "                    esac")?;
    }

    Ok(())
}

fn write_fish_completion(
    writer: &mut dyn fmt::Write,
    help: &Help,
) -> fmt::Result {
    let app_name = help.name;

    write_fish_option_takes_value(writer, help, app_name)?;
    write_fish_subcommands_for_path(writer, help, app_name)?;

    // Helper function to check if we're using a specific command path
    writeln!(writer, "function __fish_{app_name}_using_command")?;
    writeln!(writer, "    set -l cmd_path (__fish_{app_name}_cmd_path)")?;
    writeln!(writer, "    if test (count $cmd_path) -eq 0")?;
    writeln!(writer, "        return 0")?;
    writeln!(writer, "    end")?;
    writeln!(writer, "    return 1")?;
    writeln!(writer, "end")?;
    writeln!(writer)?;

    // Helper function to check if we're using a specific subcommand path
    writeln!(writer, "function __fish_{app_name}_using_subcommand")?;
    writeln!(writer, "    set -l cmd_path (__fish_{app_name}_cmd_path)")?;
    writeln!(writer, "    set -l path $argv")?;
    writeln!(writer, "    if test (count $cmd_path) -ne (count $path)")?;
    writeln!(writer, "        return 1")?;
    writeln!(writer, "    end")?;
    writeln!(writer, "    for i in (seq (count $path))")?;
    writeln!(writer, "        if test $cmd_path[$i] != $path[$i]")?;
    writeln!(writer, "            return 1")?;
    writeln!(writer, "        end")?;
    writeln!(writer, "    end")?;
    writeln!(writer, "    return 0")?;
    writeln!(writer, "end")?;
    writeln!(writer)?;

    // Compute the current subcommand path, ignoring options and option values.
    writeln!(writer, "function __fish_{app_name}_cmd_path")?;
    writeln!(writer, "    set -l cmd (commandline -opc)")?;
    writeln!(writer, "    set -l path")?;
    writeln!(writer, "    set -l i 2")?;
    writeln!(writer, "    while test $i -le (count $cmd)")?;
    writeln!(writer, "        set -l word $cmd[$i]")?;
    writeln!(writer, "        if test $word = \"--\"")?;
    writeln!(writer, "            break")?;
    writeln!(writer, "        end")?;
    writeln!(writer, "        if string match -qr '^-' -- $word")?;
    writeln!(
        writer,
        "            if __fish_{app_name}_option_takes_value $word"
    )?;
    writeln!(writer, "                if string match -qr '=' -- $word")?;
    writeln!(writer, "                    set i (math $i + 1)")?;
    writeln!(writer, "                else")?;
    writeln!(writer, "                    set i (math $i + 2)")?;
    writeln!(writer, "                end")?;
    writeln!(writer, "            else")?;
    writeln!(writer, "                set i (math $i + 1)")?;
    writeln!(writer, "            end")?;
    writeln!(writer, "            continue")?;
    writeln!(writer, "        end")?;
    writeln!(
        writer,
        "        set -l subs (__fish_{app_name}_subcommands_for_path $path)"
    )?;
    writeln!(writer, "        if contains -- $word $subs")?;
    writeln!(writer, "            set path $path $word")?;
    writeln!(writer, "            set i (math $i + 1)")?;
    writeln!(writer, "        else")?;
    writeln!(writer, "            break")?;
    writeln!(writer, "        end")?;
    writeln!(writer, "    end")?;
    writeln!(writer, "    printf '%s\n' $path")?;
    writeln!(writer, "end")?;
    writeln!(writer)?;

    // Generate completions recursively
    write_fish_completion_recursive(writer, help, None, app_name)?;

    Ok(())
}

fn write_fish_completion_recursive(
    writer: &mut dyn fmt::Write,
    help: &Help,
    path: Option<&PathNode>,
    app_name: &str,
) -> fmt::Result {
    if path.is_none() {
        write_fish_root_completion(writer, help, app_name)?;
    } else {
        write_fish_subcommand_completion(writer, help, path, app_name)?;
    }

    // Recursively handle subcommands
    if let Some(subhelps) = help.subhelps {
        for subhelp in subhelps {
            let next_node = PathNode {
                parent: path,
                name: subhelp.name,
            };

            let sub_help = Help {
                path: help.path,
                name: subhelp.name,
                description: subhelp.description,
                global_opts: help.global_opts,
                builtin_global_opts: help.builtin_global_opts,
                opts: subhelp.opts,
                builtin_opts: subhelp.builtin_opts,
                groups: subhelp.groups,
                subhelps: subhelp.subhelps,
            };

            write_fish_completion_recursive(
                writer,
                &sub_help,
                Some(&next_node),
                app_name,
            )?;
        }
    }

    Ok(())
}

fn write_fish_root_completion(
    writer: &mut dyn fmt::Write,
    help: &Help,
    app_name: &str,
) -> fmt::Result {
    // Root level completion
    for opt in help.opts {
        write!(writer, "complete -c {app_name}")?;
        if let Some(c) = opt.short {
            write!(writer, " -s {c}")?;
        }
        write!(
            writer,
            " -l {name} -n '__fish_{app_name}_using_command' -d \"",
            name = opt.name
        )?;
        write_fish_description(writer, opt.description)?;
        writeln!(writer, "\"")?;
    }

    for opt in help.builtin_opts {
        write!(writer, "complete -c {app_name}")?;
        if let Some(c) = opt.short {
            write!(writer, " -s {c}")?;
        }
        write!(
            writer,
            " -l {name} -n '__fish_{app_name}_using_command' -d \"",
            name = opt.name
        )?;
        write_fish_description(writer, opt.description)?;
        writeln!(writer, "\"")?;
    }

    // Generate global options at root level
    if let Some(global_opts) = help.global_opts {
        for opt in global_opts {
            write!(writer, "complete -c {app_name}")?;
            if let Some(c) = opt.short {
                write!(writer, " -s {c}")?;
            }
            write!(
                writer,
                " -l {name} -n '__fish_{app_name}_using_command' -d \"",
                name = opt.name
            )?;
            write_fish_description(writer, opt.description)?;
            writeln!(writer, "\"")?;
        }
    }
    if let Some(builtin_global_opts) = help.builtin_global_opts {
        for opt in builtin_global_opts {
            write!(writer, "complete -c {app_name}")?;
            if let Some(c) = opt.short {
                write!(writer, " -s {c}")?;
            }
            write!(
                writer,
                " -l {name} -n '__fish_{app_name}_using_command' -d \"",
                name = opt.name
            )?;
            write_fish_description(writer, opt.description)?;
            writeln!(writer, "\"")?;
        }
    }

    // Generate subcommands at root level
    if let Some(subhelps) = help.subhelps {
        for subhelp in subhelps {
            write!(
                writer,
                "complete -c {app_name} -f -n '__fish_{app_name}_using_command' -a {name} -d \"",
                name = subhelp.name
            )?;
            write_fish_description(writer, subhelp.description)?;
            writeln!(writer, "\"")?;
        }
    }

    Ok(())
}

fn write_fish_subcommand_completion(
    writer: &mut dyn fmt::Write,
    help: &Help,
    path: Option<&PathNode>,
    app_name: &str,
) -> fmt::Result {
    // Subcommand level completion - generate options for this subcommand
    write!(
        writer,
        "complete -c {app_name} -s h -l help -n '__fish_{app_name}_using_subcommand "
    )?;
    if let Some(p) = path {
        p.write_path(writer, " ")?;
    }
    writeln!(writer, "' -d \"Show help for this command\"")?;

    for opt in help.opts {
        write!(writer, "complete -c {app_name}")?;
        if let Some(c) = opt.short {
            write!(writer, " -s {c}")?;
        }
        write!(
            writer,
            " -l {name} -n '__fish_{app_name}_using_subcommand ",
            name = opt.name
        )?;
        if let Some(p) = path {
            p.write_path(writer, " ")?;
        }
        write!(writer, "' -d \"")?;
        write_fish_description(writer, opt.description)?;
        writeln!(writer, "\"")?;
    }

    for opt in help.builtin_opts {
        write!(writer, "complete -c {app_name}")?;
        if let Some(c) = opt.short {
            write!(writer, " -s {c}")?;
        }
        write!(
            writer,
            " -l {name} -n '__fish_{app_name}_using_subcommand ",
            name = opt.name
        )?;
        if let Some(p) = path {
            p.write_path(writer, " ")?;
        }
        write!(writer, "' -d \"")?;
        write_fish_description(writer, opt.description)?;
        writeln!(writer, "\"")?;
    }

    // Generate subcommands for this level
    if let Some(subhelps) = help.subhelps {
        for subhelp in subhelps {
            write!(
                writer,
                "complete -c {app_name} -f -n '__fish_{app_name}_using_subcommand "
            )?;
            if let Some(p) = path {
                p.write_path(writer, " ")?;
            }
            write!(writer, "' -a {name} -d \"", name = subhelp.name)?;
            write_fish_description(writer, subhelp.description)?;
            writeln!(writer, "\"")?;
        }
    }

    Ok(())
}
