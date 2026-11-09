// This example demonstrates how to implement a custom `Writer` for the `shovel` application.
// A custom writer allows you to control how `shovel` outputs information, such as version details
// or error messages. This can be useful for integrating `shovel` into larger systems or for
// custom logging.
//
// To run this example:
// 1. Navigate to the project root directory in your terminal.
// 2. Execute the command: `cargo run --example custom-writer -- --version`
//
// Expected output (version information will vary):
// custom-output-writer 0.1.0
//
// You can also run it without arguments:
// `cargo run --example custom-writer`
// Expected output:
// A custom output writer example

use shovel::*;

pub struct MyWriter(std::io::Stdout);
pub struct MyLogger {
    writer: MyWriter,
}

impl MyLogger {
    fn new() -> Self {
        Self {
            writer: MyWriter(std::io::stdout()),
        }
    }
}

impl std::fmt::Write for MyWriter {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        std::io::Write::write_all(&mut self.0, s.as_bytes())
            .expect("Failed to write to stdout");
        Ok(())
    }
}

impl Writer for MyLogger {
    const ALIGN_WIDTH: usize = 32;

    fn writer(&mut self) -> &mut dyn std::fmt::Write {
        &mut self.writer
    }

    fn write_error(&mut self, error: ParseError) -> std::fmt::Result {
        writeln!(self.writer(), "Parse Error: {error}")
    }

    fn write_version(&mut self, version: Version) -> std::fmt::Result {
        writeln!(self.writer(), "{} {}", version.name, version.version)
    }
}

fn main() {
    let code = App {
        name: "custom-writer",
        description: "A custom writer example",
        version: "0.1.0",
        authors: ["Author Name <author@example.com>"],
        copyright: "(c) 2026 Company Name",
        action: Action::no_args(|| {
            println!("A custom writer example");

            0
        }),
    }
    .run_with_args(&mut MyLogger::new(), &shovel::env::args().to_strs())
    .expect("Failed to run the application");
    std::process::exit(code);
}
