#![cfg_attr(feature = "generic_const_exprs", feature(generic_const_exprs))]
#![allow(incomplete_features)]
#![cfg(feature = "std")]
use shovel::*;

#[derive(Opts)]
struct Settings {
    #[env = "PORT"]
    port: u16,
    #[env = "TAGS"]
    tags: Option<Vec<String>>,
    #[env = "DEBUG"]
    debug: bool,
}

fn app() -> App<'static, 0, Settings, (), (), i32> {
    App {
        name: "test",
        description: "",
        version: "1",
        authors: [],
        copyright: "",
        action: Action::opts(|opts: Settings| {
            assert_eq!(opts.port, 8080);
            assert_eq!(opts.tags, Some(vec!["a".into(), "b".into()]));
            assert!(opts.debug);
            -17
        }),
    }
}

fn run(args: &[&str], env: &[(&str, &str)]) -> (i32, String) {
    let mut output = Vec::new();
    let code = app()
        .run_with_args_and_env(&mut IoWriteAdapter::new(&mut output), args, env)
        .unwrap();
    (code, String::from_utf8(output).unwrap())
}

#[test]
fn fallback_and_cli_precedence() {
    assert_eq!(
        run(
            &["test"],
            &[("PORT", "8080"), ("TAGS", " a,, b, "), ("DEBUG", "true")]
        )
        .0,
        -17
    );
    assert_eq!(
        run(
            &[
                "test", "--port", "8080", "--tags", "a", "--tags", "b",
                "--debug"
            ],
            &[
                ("PORT", "invalid"),
                ("TAGS", "ignored"),
                ("DEBUG", "invalid")
            ]
        )
        .0,
        -17
    );
}

#[test]
fn diagnostics_and_builtin_statuses() {
    let (code, text) = run(&["test"], &[("PORT", "bad")]);
    assert_eq!(code, EXIT_CODE_USAGE);
    assert!(text.contains("PORT"));
    assert_eq!(run(&["test"], &[]).0, EXIT_CODE_USAGE);
    assert_eq!(
        run(&["test", "--port", "bad"], &[("PORT", "8080")]).0,
        EXIT_CODE_USAGE
    );
    let (code, help) = run(&["test", "--help"], &[("PORT", "invalid")]);
    assert_eq!(code, EXIT_CODE_SUCCESS);
    assert!(help.contains("[env: PORT]"));
    assert_eq!(run(&["test", "--version"], &[]).0, EXIT_CODE_SUCCESS);
}

#[derive(Opts)]
struct Lists {
    #[env = "LIST"]
    values: Vec<u16>,
    #[env = "FLAG"]
    flag: bool,
}

#[test]
fn empty_lists_and_false_flags() {
    let app = App {
        name: "test",
        description: "",
        version: "1",
        authors: [],
        copyright: "",
        action: Action::opts(|opts: Lists| {
            assert!(opts.values.is_empty());
            assert!(!opts.flag);
            23
        }),
    };
    let mut output = Vec::new();
    let mut writer = IoWriteAdapter::new(&mut output);
    #[cfg(not(feature = "generic_const_exprs"))]
    let code = app.run_heapless_with_env::<1, _>(
        &mut writer,
        &["test", "--values", ""],
        &[("LIST", "invalid"), ("FLAG", "false")],
    );
    #[cfg(feature = "generic_const_exprs")]
    let code = app.run_heapless_with_env(
        &mut writer,
        &["test", "--values", ""],
        &[("LIST", "invalid"), ("FLAG", "false")],
    );
    assert_eq!(code.unwrap(), 23);
}

#[derive(Opts)]
struct Global {
    #[env = "COUNT"]
    count: u16,
}

#[test]
fn nested_commands_receive_environment_and_propagate_codes() {
    let app = App {
        name: "test",
        description: "",
        version: "1",
        authors: [],
        copyright: "",
        action: AppAction::<Global>::subcmds((Cmd {
            name: "child",
            description: "",
            category: None,
            action: Action::subcmds((Cmd {
                name: "leaf",
                description: "",
                category: None,
                action: Action::opts_with_global_opts(
                    |opts: Lists, global: Global| {
                        assert_eq!(opts.values, vec![1, 2]);
                        assert_eq!(global.count, 3);
                        42
                    },
                ),
            },)),
        },)),
    };
    let mut output = Vec::new();
    assert_eq!(
        app.run_with_args_and_env(
            &mut IoWriteAdapter::new(&mut output),
            &["test", "child", "leaf"],
            &[("COUNT", "3"), ("LIST", "1,2")]
        )
        .unwrap(),
        42
    );
}

#[test]
fn output_failure_remains_an_error() {
    struct Broken;
    impl core::fmt::Write for Broken {
        fn write_str(&mut self, _: &str) -> core::fmt::Result {
            Err(core::fmt::Error)
        }
    }
    assert!(
        app()
            .run_with_args(&mut Broken, &["test", "--help"])
            .is_err()
    );
}

#[test]
fn all_handler_forms_support_unit_and_exit_codes() {
    fn execute<R: ActionReturn>(action: Action<(), (), (), (), R>) -> i32 {
        let app = App {
            name: "test",
            description: "",
            version: "1",
            authors: [],
            copyright: "",
            action,
        };
        let mut output = Vec::new();
        app.run_with_args(&mut IoWriteAdapter::new(&mut output), &["test"])
            .unwrap()
    }
    macro_rules! check {
        ($method:ident, $unit:expr, $code:expr) => {
            assert_eq!(execute(Action::$method($unit)), EXIT_CODE_SUCCESS);
            assert_eq!(execute(Action::$method($code)), -31);
        };
    }
    fn unit_handler() {}
    fn code_handler() -> i32 {
        -31
    }
    check!(no_args, unit_handler, code_handler);
    check!(opts, |_: ()| {}, |_: ()| -31);
    check!(args, |_: ()| {}, |_: ()| -31);
    check!(global_opts, |_: ()| {}, |_: ()| -31);
    check!(opts_args, |_: (), _: ()| {}, |_: (), _: ()| -31);
    check!(opts_with_global_opts, |_: (), _: ()| {}, |_: (), _: ()| -31);
    check!(args_with_global_opts, |_: (), _: ()| {}, |_: (), _: ()| -31);
    check!(all, |_: (), _: (), _: ()| {}, |_: (), _: (), _: ()| -31);
    check!(handler, |_: Context<(), (), ()>| {}, |_: Context<
        (),
        (),
        (),
    >| -31);
}

#[test]
fn explicit_annotations_preserve_unit_handlers_and_support_exit_codes() {
    fn execute<R: ActionReturn>(app: App<'_, 0, (), (), (), R>) -> i32 {
        app.run_with_args(&mut String::new(), &["test"]).unwrap()
    }
    let unit: Action<(), (), (), ()> = Action::no_args(|| {});
    let unit_builder: AppBuilder<'_, 0, (), (), ()> = AppBuilder::new("test")
        .description("")
        .version("1")
        .authors([])
        .copyright("")
        .action(unit);
    let unit_app: App<'_, 0, (), (), ()> = unit_builder.build();
    assert_eq!(execute(unit_app), EXIT_CODE_SUCCESS);

    let code: Action<(), (), (), (), i32> = Action::no_args(|| -23);
    let code_builder: AppBuilder<'_, 0, (), (), (), i32> =
        AppBuilder::new("test")
            .description("")
            .version("1")
            .authors([])
            .copyright("")
            .action(code);
    let code_app: App<'_, 0, (), (), (), i32> = code_builder.build();
    assert_eq!(execute(code_app), -23);

    let unit_builder: CmdBuilder<'_, (), (), (), ()> = CmdBuilder::new("unit")
        .description("")
        .action(Action::no_args(|| {}));
    let unit_cmd: Cmd<'_, (), (), (), ()> = unit_builder.build();
    let code_builder: CmdBuilder<'_, (), (), (), (), i32> =
        CmdBuilder::new("code")
            .description("")
            .action(Action::no_args(|| 19));
    let code_cmd: Cmd<'_, (), (), (), (), i32> = code_builder.build();
    let app = App {
        name: "test",
        description: "",
        version: "1",
        authors: [],
        copyright: "",
        action: Action::subcmds((unit_cmd, code_cmd)),
    };
    assert_eq!(
        app.run_with_args(&mut String::new(), &["test", "code"])
            .unwrap(),
        19
    );
}

#[test]
fn diverging_closures_accept_explicit_return_types() {
    fn accepts<R: ActionReturn>(_: Action<(), (), (), (), R>) {}
    accepts(Action::no_args(|| -> () { panic!("handler failed") }));
    accepts(Action::no_args(|| -> i32 { unimplemented!() }));
    accepts(Action::no_args(|| -> () { std::process::exit(17) }));
}

#[test]
fn mixed_return_codes_survive_nested_dispatch_and_heapless_execution() {
    for (args, expected) in [
        (&["test", "unit-root"][..], EXIT_CODE_SUCCESS),
        (&["test", "unit-root", "code"][..], 42),
        (&["test", "code-root"][..], -7),
        (&["test", "code-root", "unit"][..], EXIT_CODE_SUCCESS),
        (&["test", "third"][..], 3),
    ] {
        for heapless in [false, true] {
            let app = App {
                name: "test",
                description: "",
                version: "1",
                authors: [],
                copyright: "",
                action: Action::subcmds((
                    Cmd {
                        name: "unit-root",
                        description: "",
                        category: None,
                        action: Action::new(
                            |_: Context<(), (), ()>| {},
                            (Cmd {
                                name: "code",
                                description: "",
                                category: None,
                                action: Action::no_args(|| 42),
                            },),
                        ),
                    },
                    Cmd {
                        name: "code-root",
                        description: "",
                        category: None,
                        action: Action::new(
                            |_: Context<(), (), ()>| -7,
                            (Cmd {
                                name: "unit",
                                description: "",
                                category: None,
                                action: Action::no_args(|| {}),
                            },),
                        ),
                    },
                    Cmd {
                        name: "third",
                        description: "",
                        category: None,
                        action: Action::no_args(|| 3),
                    },
                )),
            };
            let mut output = String::new();
            let result = if heapless {
                #[cfg(not(feature = "generic_const_exprs"))]
                let result = app.run_heapless::<6, _>(&mut output, args);
                #[cfg(feature = "generic_const_exprs")]
                let result = app.run_heapless(&mut output, args);
                result
            } else {
                app.run_with_args(&mut output, args)
            };
            assert_eq!(result.unwrap(), expected);
        }
    }
}

#[test]
fn mixed_returns_work_in_builders_and_command_tuples() {
    for (command, expected) in [
        ("unit", EXIT_CODE_SUCCESS),
        ("code", 19),
        ("nested", EXIT_CODE_SUCCESS),
    ] {
        let app = AppBuilder::new("test")
            .description("")
            .version("1")
            .authors([])
            .copyright("")
            .action(Action::subcmds((
                CmdBuilder::new("unit")
                    .description("")
                    .action(Action::no_args(|| {}))
                    .build(),
                CmdBuilder::new("code")
                    .description("")
                    .action(Action::no_args(|| 19))
                    .build(),
                CmdBuilder::new("nested")
                    .description("")
                    .action(Action::new(
                        |_: Context<(), (), ()>| {},
                        (
                            CmdBuilder::new("child")
                                .description("")
                                .action(Action::no_args(|| 7))
                                .build(),
                            CmdBuilder::new("other")
                                .description("")
                                .action(Action::no_args(|| {}))
                                .build(),
                        ),
                    ))
                    .build(),
            )))
            .build();
        let mut output = Vec::new();
        assert_eq!(
            app.run_with_args(
                &mut IoWriteAdapter::new(&mut output),
                &["test", command]
            )
            .unwrap(),
            expected
        );
    }
}

#[test]
fn option_fields_do_not_capture_generated_parser_identifiers() {
    #[derive(Opts, Debug, PartialEq)]
    struct CollisionOpts {
        #[env = "VALUE"]
        value: u16,
        #[env = "PARSED"]
        parsed: Option<u16>,
        #[env = "ENV"]
        env: u16,
        #[env = "OPTS"]
        opts: u16,
        #[env = "OPT"]
        opt: u16,
        #[env = "S"]
        s: Option<u16>,
        #[env = "V"]
        v: Vec<u16>,
        #[env = "PART"]
        part: Vec<u16>,
        #[env = "KEY"]
        key: bool,
        #[env = "EXISTING"]
        existing: Option<Vec<u16>>,
        #[long = "internal-value"]
        #[env = "INTERNAL_VALUE"]
        __shovel_value: u16,
        #[long = "internal-env"]
        #[env = "INTERNAL_ENV"]
        __shovel_env: u16,
        #[long = "internal-opts"]
        #[env = "INTERNAL_OPTS"]
        __shovel_opts: u16,
        #[long = "internal-field"]
        #[env = "INTERNAL_FIELD"]
        __shovel_field_value: u16,
        #[long = "kind"]
        #[env = "KIND"]
        r#type: u16,
    }
    let env = [
        ("VALUE", "1"),
        ("PARSED", "2"),
        ("ENV", "3"),
        ("OPTS", "4"),
        ("OPT", "5"),
        ("S", "6"),
        ("V", "7, 8"),
        ("PART", "9,10"),
        ("KEY", "true"),
        ("EXISTING", "11,12"),
        ("INTERNAL_VALUE", "13"),
        ("INTERNAL_ENV", "14"),
        ("INTERNAL_OPTS", "15"),
        ("INTERNAL_FIELD", "16"),
        ("KIND", "17"),
    ];
    let cli = [
        "test",
        "--value",
        "1",
        "--parsed",
        "2",
        "--env",
        "3",
        "--opts",
        "4",
        "--opt",
        "5",
        "--s",
        "6",
        "--v",
        "7",
        "--v",
        "8",
        "--part",
        "9,10",
        "--key",
        "--existing",
        "11,12",
        "--internal-value",
        "13",
        "--internal-env",
        "14",
        "--internal-opts",
        "15",
        "--internal-field",
        "16",
        "--kind",
        "17",
    ];
    let invalid_env: Vec<_> =
        env.iter().map(|(name, _)| (*name, "invalid")).collect();
    for (args, fallback) in [
        (&["test"][..], &env[..]),
        (&cli[..], &invalid_env[..]),
        (&cli[..], &[][..]),
    ] {
        let app = App {
            name: "test",
            description: "",
            version: "1",
            authors: [],
            copyright: "",
            action: Action::opts(|opts: CollisionOpts| {
                assert_eq!(
                    opts,
                    CollisionOpts {
                        value: 1,
                        parsed: Some(2),
                        env: 3,
                        opts: 4,
                        opt: 5,
                        s: Some(6),
                        v: vec![7, 8],
                        part: vec![9, 10],
                        key: true,
                        existing: Some(vec![11, 12]),
                        __shovel_value: 13,
                        __shovel_env: 14,
                        __shovel_opts: 15,
                        __shovel_field_value: 16,
                        r#type: 17,
                    }
                );
                37
            }),
        };
        let mut output = String::new();
        assert_eq!(
            app.run_with_args_and_env(&mut output, args, fallback)
                .unwrap(),
            37
        );
    }
}

#[cfg(unix)]
#[test]
fn non_unicode_environment_values_behave_as_absent() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    use std::process::Command;

    const CASE: &str = "SHOVEL_NON_UNICODE_TEST_CASE";
    if let Ok(case) = std::env::var(CASE) {
        #[derive(Opts)]
        struct Options {
            #[env = "SHOVEL_REQUIRED"]
            required: u16,
            #[env = "SHOVEL_OPTIONAL"]
            optional: Option<u16>,
            #[env = "SHOVEL_FLAG"]
            flag: bool,
        }
        let app = App {
            name: "test",
            description: "",
            version: "1",
            authors: [],
            copyright: "",
            action: Action::opts(|opts: Options| {
                assert_eq!(opts.required, 7);
                if std::env::var(CASE).unwrap() == "cli" {
                    assert_eq!(opts.optional, Some(8));
                    assert!(opts.flag);
                } else {
                    assert_eq!(opts.optional, None);
                    assert!(!opts.flag);
                }
            }),
        };
        let args: &[&str] = match case.as_str() {
            "required" => &["test"],
            "optional" => &["test", "--required", "7"],
            "cli" => &["test", "--required", "7", "--optional", "8", "--flag"],
            _ => panic!("unknown child test case"),
        };
        let vars = shovel::env::vars_os();
        let mut output = String::new();
        let code = app
            .run_with_args_and_env(&mut output, args, &vars.to_strs())
            .unwrap();
        if case == "required" {
            assert_eq!(code, EXIT_CODE_USAGE);
            assert!(output.contains("Required option 'required' is missing"));
        } else {
            assert_eq!(code, EXIT_CODE_SUCCESS);
        }
        return;
    }

    // Isolate process environment changes from the parallel test runner.
    for case in ["required", "optional", "cli"] {
        let invalid = OsString::from_vec(vec![0xff]);
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "non_unicode_environment_values_behave_as_absent",
            ])
            .env(CASE, case)
            .env("SHOVEL_REQUIRED", &invalid)
            .env("SHOVEL_OPTIONAL", &invalid)
            .env("SHOVEL_FLAG", &invalid)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "case {case} failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
}

#[test]
fn root_default_handler_replays_cli_options_before_environment_fallback() {
    #[derive(Opts)]
    struct RootOpts {
        #[env = "PORT"]
        port: u16,
    }
    for (args, env, expected) in [
        (&["test", "--port", "8080"][..], &[][..], 8080),
        (
            &["test", "--port", "8080"][..],
            &[("PORT", "9000")][..],
            8080,
        ),
        (&["test"][..], &[("PORT", "9000")][..], 9000),
        (
            &["test", "--port", "8080", "child"][..],
            &[("PORT", "9000")][..],
            8081,
        ),
        (&["test", "child"][..], &[("PORT", "9000")][..], 9001),
    ] {
        for heapless in [false, true] {
            let app = App {
                name: "test",
                description: "",
                version: "1",
                authors: [],
                copyright: "",
                action: Action::new(
                    |cx: Context<RootOpts, (), RootOpts>| {
                        assert_eq!(cx.opts.port, cx.global_opts.port);
                        i32::from(cx.opts.port)
                    },
                    (Cmd {
                        name: "child",
                        description: "",
                        category: None,
                        action: Action::global_opts(|global: RootOpts| {
                            i32::from(global.port) + 1
                        }),
                    },),
                ),
            };
            let mut output = String::new();
            let result = if heapless {
                #[cfg(not(feature = "generic_const_exprs"))]
                let result =
                    app.run_heapless_with_env::<2, _>(&mut output, args, env);
                #[cfg(feature = "generic_const_exprs")]
                let result = app.run_heapless_with_env(&mut output, args, env);
                result
            } else {
                app.run_with_args_and_env(&mut output, args, env)
            };
            assert_eq!(
                result.unwrap(),
                expected,
                "args={args:?}, env={env:?}, output={output}"
            );
        }
    }
}
