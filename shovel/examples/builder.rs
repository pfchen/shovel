// This example demonstrates using `AppBuilder` and `CmdBuilder` to assemble
// a CLI for a tiny task tracker with nested commands and options.
//
// To run this example:
// 1. Navigate to the project root directory in your terminal.
// 2. Execute: `cargo run --example builder -- --help`
//
// Try these commands:
//   cargo run --example builder -- add "Write docs" --priority high
//   cargo run --example builder -- add "Fix tests" --priority low --tag ci
//   cargo run --example builder -- list --all
//   cargo run --example builder -- list --tag ci
//   cargo run --example builder -- complete 42
//   cargo run --example builder -- config set --global editor vim
//   cargo run --example builder -- config get editor
use shovel::*;

#[derive(Opts)]
struct AddTaskOpts {
    /// Task priority (low, medium, high)
    #[short = 'p']
    priority: Option<String>,
    /// Optional tag for grouping tasks
    #[short = 't']
    tag: Option<String>,
}

#[derive(Args)]
struct AddTaskArgs {
    /// Task title
    title: String,
}

#[derive(Opts)]
struct ListOpts {
    /// Show all tasks, including completed ones
    #[short = 'a']
    all: bool,
    /// Filter by tag
    #[short = 't']
    tag: Option<String>,
}

#[derive(Args)]
struct CompleteArgs {
    /// Task id
    id: u32,
}

#[derive(Opts)]
struct ConfigSetOpts {
    /// Store the value globally
    #[short = 'g']
    global: bool,
}

#[derive(Args)]
struct ConfigSetArgs {
    /// Configuration key
    key: String,
    /// Configuration value
    value: String,
}

#[derive(Args)]
struct ConfigGetArgs {
    /// Configuration key
    key: String,
}

fn main() {
    AppBuilder::new("builder")
        .description("A tiny task tracker built with the builder API")
        .version("0.1.0")
        .authors(["Author Name <author@example.com>"])
        .copyright("(c) 2026 Company Name")
        .action(Action::subcmds((
            CmdBuilder::new("add")
                .description("Adds a new task")
                .action(Action::opts_args(
                    |opts: AddTaskOpts, args: AddTaskArgs| {
                        let priority = opts
                            .priority
                            .unwrap_or_else(|| "medium".to_string());
                        let tag =
                            opts.tag.unwrap_or_else(|| "general".to_string());
                        println!(
                            "Added task: \"{}\" (priority: {}, tag: {})",
                            args.title, priority, tag
                        );
                    },
                ))
                .build(),
            CmdBuilder::new("list")
                .description("Lists tasks")
                .action(Action::opts(|opts: ListOpts| {
                    let scope = if opts.all { "all" } else { "open" };
                    match opts.tag {
                        Some(tag) => {
                            println!("Listing {scope} tasks tagged \"{tag}\"")
                        }
                        None => println!("Listing {scope} tasks"),
                    }
                }))
                .build(),
            CmdBuilder::new("complete")
                .description("Marks a task as completed")
                .action(Action::args(|args: CompleteArgs| {
                    println!("Completed task #{}", args.id);
                }))
                .build(),
            CmdBuilder::new("config")
                .description("Manages configuration")
                .action(Action::subcmds((
                    CmdBuilder::new("set")
                        .description("Sets a configuration value")
                        .category("configuration")
                        .action(Action::opts_args(
                            |opts: ConfigSetOpts, args: ConfigSetArgs| {
                                if opts.global {
                                    println!(
                                        "Setting global {} = {}",
                                        args.key, args.value
                                    );
                                } else {
                                    println!(
                                        "Setting {} = {}",
                                        args.key, args.value
                                    );
                                }
                            },
                        ))
                        .build(),
                    CmdBuilder::new("get")
                        .description("Reads a configuration value")
                        .category("configuration")
                        .action(Action::args(|args: ConfigGetArgs| {
                            println!("Reading {}", args.key);
                        }))
                        .build(),
                )))
                .build(),
        )))
        .build()
        .run()
        .expect("Failed to run the application");
}
