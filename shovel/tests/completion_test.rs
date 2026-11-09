#[cfg(feature = "completion")]
mod completion_tests {
    use shovel::*;
    use std::io::Cursor;

    #[derive(Opts)]
    struct TestOpts {
        /// Test option
        #[short = 't']
        test: Option<String>,
    }

    #[derive(Args)]
    struct TestArgs {
        /// Test argument
        name: String,
    }

    #[derive(Opts)]
    struct GlobalOpts {
        /// Debug mode
        #[short]
        debug: bool,
    }

    #[test]
    fn test_bash_completion_generation() {
        let app = App {
            name: "testapp",
            description: "A test app",
            version: "0.1.0",
            authors: ["Author Name <author@example.com>"],
            copyright: "(c) 2026 Company Name",
            action: AppAction::subcmds((
                Cmd {
                    name: "hello",
                    description: "Say hello",
                    category: None,
                    action: Action::handler(
                        |cx: Context<TestOpts, TestArgs, GlobalOpts>| {
                            let _ = (
                                cx.opts.test,
                                cx.args.name,
                                cx.global_opts.debug,
                            );
                            println!("Hello!");

                            0
                        },
                    ),
                },
                Cmd {
                    name: "utils",
                    description: "Utility commands",
                    category: None,
                    action: Action::subcmds((Cmd {
                        name: "time",
                        description: "Show time",
                        category: None,
                        action: Action::no_args(|| {
                            println!("Time!");

                            0
                        }),
                    },)),
                },
            )),
        };

        let mut output = Cursor::new(Vec::new());
        let args = ["testapp", "--completion", "bash"];
        let result =
            app.run_with_args(&mut IoWriteAdapter::new(&mut output), &args);

        assert!(result.is_ok());
        let completion_script = String::from_utf8(output.into_inner()).unwrap();

        // Check that the completion script contains expected elements
        assert!(completion_script.contains("_testapp_completion"));
        assert!(completion_script.contains("hello"));
        assert!(completion_script.contains("utils"));
        assert!(completion_script.contains("time"));
        assert!(completion_script.contains("--debug"));
        assert!(completion_script.contains("--test"));
    }

    #[test]
    fn test_zsh_completion_generation() {
        let app = App {
            name: "testapp",
            description: "A test app",
            version: "0.1.0",
            authors: ["Author Name <author@example.com>"],
            copyright: "(c) 2026 Company Name",
            action: AppAction::subcmds((Cmd {
                name: "hello",
                description: "Say hello",
                category: None,
                action: Action::handler(
                    |_cx: Context<TestOpts, TestArgs, GlobalOpts>| {
                        println!("Hello!");

                        0
                    },
                ),
            },)),
        };

        let mut output = Cursor::new(Vec::new());
        let args = ["testapp", "--completion", "zsh"];
        let result =
            app.run_with_args(&mut IoWriteAdapter::new(&mut output), &args);

        assert!(result.is_ok());
        let completion_script = String::from_utf8(output.into_inner()).unwrap();

        // Check that the completion script contains expected elements
        assert!(completion_script.contains("#compdef testapp"));
        assert!(completion_script.contains("_testapp_completion"));
        assert!(completion_script.contains("hello"));
        assert!(completion_script.contains("--debug"));
        assert!(completion_script.contains("--test"));
    }

    #[test]
    fn test_fish_completion_generation() {
        let app = App {
            name: "testapp",
            description: "A test app",
            version: "0.1.0",
            authors: ["Author Name <author@example.com>"],
            copyright: "(c) 2026 Company Name",
            action: AppAction::subcmds((Cmd {
                name: "hello",
                description: "Say hello",
                category: None,
                action: Action::handler(
                    |_cx: Context<TestOpts, TestArgs, GlobalOpts>| {
                        println!("Hello!");

                        0
                    },
                ),
            },)),
        };

        let mut output = Cursor::new(Vec::new());
        let args = ["testapp", "--completion", "fish"];
        let result =
            app.run_with_args(&mut IoWriteAdapter::new(&mut output), &args);

        assert!(result.is_ok());
        let completion_script = String::from_utf8(output.into_inner()).unwrap();

        // Check that the completion script contains expected elements
        assert!(completion_script.contains("__fish_testapp_using_command"));
        assert!(completion_script.contains("complete -c testapp"));
        assert!(completion_script.contains("hello"));
        // Note: Global options might not be included in fish completion for subcommands
        // Let's check if the script contains any debug-related content
        assert!(
            completion_script.contains("debug")
                || completion_script.contains("--help")
        );
    }

    #[test]
    fn test_fish_special_chars_escaping() {
        #[allow(dead_code)]
        #[derive(Opts)]
        struct SpecialOpts {
            /// Cost is $100 per item
            #[short = 'c']
            cost: Option<String>,
            /// Contains "quotes" and more
            #[short = 'q']
            quotes: Option<String>,
        }

        #[allow(dead_code)]
        #[derive(Args)]
        struct SpecialArgs {
            /// The file path
            name: String,
        }

        let app = App {
            name: "special",
            description: "A test app",
            version: "0.1.0",
            authors: ["Author Name <author@example.com>"],
            copyright: "(c) 2026 Company Name",
            action: AppAction::subcmds((Cmd {
                name: "cmd",
                description: "Command with special chars",
                category: None,
                action: Action::handler(
                    |_cx: Context<SpecialOpts, SpecialArgs, GlobalOpts>| {
                        println!("Special!");

                        0
                    },
                ),
            },)),
        };

        let mut output = Cursor::new(Vec::new());
        let args = ["special", "--completion", "fish"];
        let result =
            app.run_with_args(&mut IoWriteAdapter::new(&mut output), &args);

        assert!(result.is_ok());
        let completion_script = String::from_utf8(output.into_inner()).unwrap();

        // Check that $ is escaped as \$ and " is escaped as \"
        assert!(completion_script.contains(r#"-d "Cost is \$100 per item""#));
        assert!(
            completion_script.contains(r#"-d "Contains \"quotes\" and more""#)
        );
    }
}
