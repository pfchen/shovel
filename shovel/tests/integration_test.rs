//! Integration tests for Shovel CLI examples.
//!
//! These tests compile and run the example binaries to verify
//! end-to-end CLI behavior including help, arguments, options, and error handling.

use std::process::Command;

/// Helper to run an example with arguments and capture output.
fn run_example(example: &str, args: &[&str]) -> (bool, String, String) {
    run_example_with_env(example, args, &[])
}

/// Helper to run an example with arguments and environment values.
fn run_example_with_env(
    example: &str,
    args: &[&str],
    env: &[(&str, &str)],
) -> (bool, String, String) {
    let output = Command::new("cargo")
        .args(["run", "--example", example, "--"])
        .args(args)
        .envs(env.iter().copied())
        .output()
        .expect("Failed to execute cargo run");

    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// Helper to run an example and expect success.
fn expect_success(example: &str, args: &[&str]) -> (String, String) {
    let (success, stdout, stderr) = run_example(example, args);
    assert!(success, "Expected success but failed.\nStderr: {}", stderr);
    (stdout, stderr)
}

mod hello_example {
    use super::*;

    #[test]
    fn test_basic_run() {
        let (stdout, _) = expect_success("hello", &[]);
        assert!(stdout.contains("Hello, world!"));
    }

    #[test]
    fn test_help_flag() {
        let (stdout, _) = expect_success("hello", &["--help"]);
        assert!(stdout.contains("Usage:"));
        assert!(stdout.contains("hello"));
        assert!(stdout.contains("A simple greeting application"));
    }

    #[test]
    fn test_help_short_flag() {
        let (stdout, _) = expect_success("hello", &["-h"]);
        assert!(stdout.contains("Usage:"));
        assert!(stdout.contains("hello"));
    }

    #[test]
    fn test_version_flag() {
        let (stdout, _) = expect_success("hello", &["--version"]);
        assert!(stdout.contains("0.1.0"));
        assert!(stdout.contains("Author Name"));
    }

    #[test]
    fn test_version_short_flag() {
        let (stdout, _) = expect_success("hello", &["-V"]);
        assert!(stdout.contains("0.1.0"));
    }

    #[test]
    fn test_completion_flag() {
        let (stdout, _) = expect_success("hello", &["--completion", "bash"]);
        assert!(stdout.contains("hello"));
    }
}

mod echo_example {
    use super::*;

    #[test]
    fn test_no_args() {
        let (stdout, _) = expect_success("echo", &[]);
        // No output expected when no args
        assert!(stdout.is_empty() || stdout.trim().is_empty());
    }

    #[test]
    fn test_single_arg() {
        let (stdout, _) = expect_success("echo", &["hello"]);
        assert_eq!(stdout.trim(), "hello");
    }

    #[test]
    fn test_multiple_args() {
        let (stdout, _) = expect_success("echo", &["hello", "world"]);
        assert_eq!(stdout.trim(), "hello world");
    }

    #[test]
    fn test_special_chars() {
        let (stdout, _) =
            expect_success("echo", &["hello", "world!", "test123"]);
        assert_eq!(stdout.trim(), "hello world! test123");
    }

    #[test]
    fn test_help_shows_echo() {
        let (stdout, _) = expect_success("echo", &["--help"]);
        assert!(stdout.contains("echo"));
    }
}

mod builder_example {
    use super::*;

    #[test]
    fn test_help() {
        let (stdout, _) = expect_success("builder", &["--help"]);
        assert!(stdout.contains("builder"));
        assert!(stdout.contains("Usage:"));
    }

    #[test]
    fn test_add_task() {
        let (stdout, _) = expect_success("builder", &["add", "Write tests"]);
        assert!(stdout.contains("Write tests"));
        assert!(stdout.contains("priority:"));
        assert!(stdout.contains("tag:"));
    }

    #[test]
    fn test_add_task_with_options() {
        let (stdout, _) =
            expect_success("builder", &["add", "-p", "high", "Fix bug"]);
        assert!(stdout.contains("Fix bug"));
        assert!(stdout.contains("priority: high"));
    }

    #[test]
    fn test_list_tasks() {
        let (stdout, _) = expect_success("builder", &["list"]);
        assert!(stdout.contains("Listing"));
        assert!(stdout.contains("tasks"));
    }

    #[test]
    fn test_list_with_all_flag() {
        let (stdout, _) = expect_success("builder", &["list", "--all"]);
        assert!(stdout.contains("all"));
    }

    #[test]
    fn test_list_with_tag_filter() {
        let (stdout, _) = expect_success("builder", &["list", "--tag", "ci"]);
        assert!(stdout.contains("ci"));
    }

    #[test]
    fn test_complete_task() {
        let (stdout, _) = expect_success("builder", &["complete", "42"]);
        assert!(stdout.contains("Completed task #42"));
    }

    #[test]
    fn test_config_set() {
        let (stdout, _) =
            expect_success("builder", &["config", "set", "editor", "vim"]);
        assert!(stdout.contains("editor"));
        assert!(stdout.contains("vim"));
    }

    #[test]
    fn test_config_set_global() {
        let (stdout, _) = expect_success(
            "builder",
            &["config", "set", "--global", "key", "value"],
        );
        assert!(stdout.contains("global"));
    }

    #[test]
    fn test_config_get() {
        let (stdout, _) =
            expect_success("builder", &["config", "get", "editor"]);
        assert!(stdout.contains("Reading editor"));
    }

    #[test]
    fn test_add_help() {
        let (stdout, _) = expect_success("builder", &["add", "--help"]);
        assert!(stdout.contains("Adds a new task"));
        assert!(stdout.contains("--priority"));
        assert!(stdout.contains("--tag"));
    }

    #[test]
    fn test_unknown_command() {
        let (_, _, stderr) = run_example("builder", &["unknown"]);
        assert!(
            stderr.contains("no such command") || stderr.contains("unknown")
        );
    }

    #[test]
    fn test_missing_required_arg() {
        let (_, stdout, stderr) = run_example("builder", &["add"]);
        let output = format!("{}{}", stdout, stderr);
        assert!(
            output.contains("Incorrect") || output.contains("arguments"),
            "Expected error message but got: stdout='{}' stderr='{}'",
            stdout,
            stderr
        );
    }
}

mod webserver_example {
    use super::*;

    #[test]
    fn test_help() {
        let (stdout, _) = expect_success("webserver", &["--help"]);
        assert!(stdout.contains("webserver"));
        assert!(stdout.contains("serve"));
        assert!(stdout.contains("db"));
        assert!(stdout.contains("rpc"));
        assert!(stdout.contains("logs"));
        assert!(stdout.contains("config"));
    }

    #[test]
    fn test_serve_default() {
        let (stdout, _) = expect_success("webserver", &["serve"]);
        assert!(stdout.contains("127.0.0.1"));
        assert!(stdout.contains("8080"));
        assert!(stdout.contains("Worker threads: 4"));
    }

    #[test]
    fn test_serve_with_options() {
        let (stdout, _) = expect_success(
            "webserver",
            &["serve", "-p", "3000", "-H", "0.0.0.0", "-w", "8"],
        );
        assert!(stdout.contains("0.0.0.0"));
        assert!(stdout.contains("3000"));
        assert!(stdout.contains("Worker threads: 8"));
    }

    #[test]
    #[ignore]
    fn test_serve_with_tls() {
        let (stdout, _) = expect_success(
            "webserver",
            &["serve", "--tls-cert", "server.crt", "-k", "server.key"],
        );
        assert!(stdout.contains("TLS enabled"));
    }

    #[test]
    fn test_serve_help() {
        let (stdout, _) = expect_success("webserver", &["serve", "--help"]);
        assert!(stdout.contains("Start the web server"));
        assert!(stdout.contains("--port"));
        assert!(stdout.contains("--host"));
    }

    #[test]
    fn test_db_migrate_up() {
        let (stdout, _) = expect_success("webserver", &["db", "migrate", "up"]);
        assert!(stdout.contains("migrations forward"));
    }

    #[test]
    fn test_db_migrate_down() {
        let (stdout, _) =
            expect_success("webserver", &["db", "migrate", "down"]);
        assert!(stdout.contains("Rolling back"));
    }

    #[test]
    fn test_db_migrate_with_options() {
        let (stdout, _) = expect_success(
            "webserver",
            &["db", "migrate", "--dry-run", "--steps", "3", "up"],
        );
        assert!(stdout.contains("DRY RUN"));
        assert!(stdout.contains("Steps: 3"));
    }

    #[test]
    fn test_db_seed() {
        let (stdout, _) =
            expect_success("webserver", &["db", "seed", "data.toml"]);
        assert!(stdout.contains("Seeding database"));
        assert!(stdout.contains("data.toml"));
    }

    #[test]
    fn test_db_seed_force() {
        let (stdout, _) = expect_success(
            "webserver",
            &["db", "seed", "--force", "data.toml"],
        );
        assert!(stdout.contains("Overwriting"));
    }

    #[test]
    fn test_db_backup() {
        let (stdout, _) =
            expect_success("webserver", &["db", "backup", "/tmp/backup.sql"]);
        assert!(stdout.contains("Creating backup"));
        assert!(stdout.contains("backup.sql"));
    }

    #[test]
    fn test_db_backup_compress() {
        let (stdout, _) = expect_success(
            "webserver",
            &["db", "backup", "--compress", "/tmp/backup.sql"],
        );
        assert!(stdout.contains("gzip"));
    }

    #[test]
    fn test_db_help() {
        let (stdout, _) = expect_success("webserver", &["db", "--help"]);
        assert!(stdout.contains("Database management"));
        assert!(stdout.contains("migrate"));
        assert!(stdout.contains("seed"));
        assert!(stdout.contains("backup"));
    }

    #[test]
    fn test_db_migrate_help() {
        let (stdout, _) =
            expect_success("webserver", &["db", "migrate", "--help"]);
        assert!(stdout.contains("Run database migrations"));
        assert!(stdout.contains("Usage: webserver [GLOBAL OPTIONS] db migrate [OPTIONS] [<DIRECTION>]"));
        assert!(stdout.contains("--steps"));
        assert!(stdout.contains("--dry-run"));
    }

    #[test]
    fn test_rpc_server_grpc() {
        let (stdout, _) =
            expect_success("webserver", &["rpc", "server", "grpc"]);
        assert!(stdout.contains("gRPC server"));
    }

    #[test]
    fn test_rpc_server_with_port() {
        let (stdout, _) = expect_success(
            "webserver",
            &["rpc", "server", "-p", "50051", "grpc"],
        );
        assert!(stdout.contains("50051"));
    }

    #[test]
    fn test_rpc_client() {
        let (stdout, _) = expect_success(
            "webserver",
            &["rpc", "client", "localhost:8080", "get_user", "{}"],
        );
        assert!(stdout.contains("get_user"));
        assert!(stdout.contains("localhost:8080"));
    }

    #[test]
    fn test_rpc_client_with_timeout() {
        let (stdout, _) = expect_success(
            "webserver",
            &["rpc", "client", "-t", "30", "localhost:8080", "test", "{}"],
        );
        assert!(stdout.contains("Timeout: 30s"));
    }

    #[test]
    fn test_rpc_codegen() {
        let (stdout, _) = expect_success(
            "webserver",
            &["rpc", "codegen", "schema.proto", "./output", "rust"],
        );
        assert!(stdout.contains("Rust"));
        assert!(stdout.contains("schema.proto"));
        assert!(stdout.contains("output"));
    }

    #[test]
    fn test_rpc_codegen_with_docs() {
        let (stdout, _) = expect_success(
            "webserver",
            &["rpc", "codegen", "--docs", "schema.proto", "./output", "go"],
        );
        assert!(stdout.contains("Including documentation"));
    }

    #[test]
    fn test_rpc_help() {
        let (stdout, _) = expect_success("webserver", &["rpc", "--help"]);
        assert!(stdout.contains("RPC service management"));
        assert!(stdout.contains("server"));
        assert!(stdout.contains("client"));
        assert!(stdout.contains("codegen"));
    }

    #[test]
    fn test_logs_default() {
        let (stdout, _) = expect_success("webserver", &["logs"]);
        assert!(stdout.contains("Log viewer"));
    }

    #[test]
    fn test_logs_with_options() {
        let (stdout, _) = expect_success(
            "webserver",
            &["logs", "-l", "debug", "-f", "-n", "100", "-i", "error"],
        );
        assert!(stdout.contains("debug"));
        assert!(stdout.contains("Following"));
        assert!(stdout.contains("100"));
        assert!(stdout.contains("error"));
    }

    #[test]
    fn test_config_validate() {
        let (stdout, _) =
            expect_success("webserver", &["config", "validate", "config.toml"]);
        assert!(stdout.contains("Validating"));
        assert!(stdout.contains("config.toml"));
    }

    #[test]
    fn test_config_validate_strict() {
        let (stdout, _) = expect_success(
            "webserver",
            &["config", "validate", "--strict", "config.toml"],
        );
        assert!(stdout.contains("Strict mode"));
    }

    #[test]
    fn test_config_generate() {
        let (stdout, _) =
            expect_success("webserver", &["config", "generate", "production"]);
        assert!(stdout.contains("production"));
    }

    #[test]
    fn test_config_generate_with_output() {
        let (stdout, _) = expect_success(
            "webserver",
            &["config", "generate", "-o", "prod.toml", "staging"],
        );
        assert!(stdout.contains("prod.toml"));
    }

    #[test]
    fn test_config_reload() {
        let (stdout, _) =
            expect_success("webserver", &["config", "reload", "SIGHUP"]);
        assert!(stdout.contains("SIGHUP"));
    }

    #[test]
    fn test_config_help() {
        let (stdout, _) = expect_success("webserver", &["config", "--help"]);
        assert!(stdout.contains("Configuration management"));
        assert!(stdout.contains("validate"));
        assert!(stdout.contains("generate"));
        assert!(stdout.contains("reload"));
    }

    #[test]
    fn test_unknown_command() {
        let (_, _, stderr) = run_example("webserver", &["unknown"]);
        assert!(
            stderr.contains("no such command") || stderr.contains("unknown")
        );
    }

    #[test]
    fn test_missing_required_arg() {
        let (_, stdout, stderr) =
            run_example("webserver", &["config", "validate"]);
        let output = format!("{}{}", stdout, stderr);
        assert!(
            output.contains("Incorrect") || output.contains("arguments"),
            "Expected error message but got: stdout='{}' stderr='{}'",
            stdout,
            stderr
        );
    }

    #[test]
    fn test_unknown_subcommand() {
        let (_, _, stderr) = run_example("webserver", &["db", "unknown"]);
        assert!(
            stderr.contains("no such command") || stderr.contains("unknown")
        );
    }

    #[test]
    fn test_version_flag() {
        let (stdout, _) = expect_success("webserver", &["--version"]);
        assert!(stdout.contains("0.1.0"));
    }
}

mod error_handling {
    use super::*;

    #[test]
    fn test_unknown_global_option() {
        let (success, _, stderr) = run_example("hello", &["--unknown"]);
        assert!(!success, "Invalid input must return a failing exit status");
        assert!(
            stderr.contains("Unknown option") || stderr.contains("unknown")
        );
    }

    #[test]
    fn test_invalid_option_format() {
        let (_, _, stderr) = run_example("echo", &["--not-a-valid-option"]);
        assert!(
            stderr.contains("Unknown option")
                || stderr.contains("not-a-valid-option")
        );
    }
}

mod env_fallback_example {
    use super::*;

    #[test]
    fn run_reads_process_environment() {
        let (success, stdout, stderr) = run_example_with_env(
            "env-fallback",
            &[],
            &[("SHOVEL_ENV_FALLBACK", "from-process-env")],
        );
        assert!(success, "Expected success but failed.\nStderr: {stderr}");
        assert_eq!(stdout.trim(), "from-process-env");
    }

    #[test]
    fn cli_values_override_process_environment() {
        let (success, stdout, stderr) = run_example_with_env(
            "env-fallback",
            &["--value", "from-cli"],
            &[("SHOVEL_ENV_FALLBACK", "from-process-env")],
        );
        assert!(success, "Expected success but failed.\nStderr: {stderr}");
        assert_eq!(stdout.trim(), "from-cli");
    }
}
