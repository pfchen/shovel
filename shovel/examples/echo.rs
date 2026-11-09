// This example demonstrates a simple echo application that takes multiple arguments
// and prints them concatenated with spaces.
//
// To run this example:
// 1. Navigate to the project root directory in your terminal.
// 2. Execute the command: `cargo run --example echo hello world`
//
// Expected output:
// hello world
use shovel::*;

fn main() {
    let code = App {
        name: "echo",
        description: "A simple echo application",
        version: "0.1.0",
        authors: ["Author Name <author@example.com>"],
        copyright: "(c) 2026 Company Name",
        action: Action::args(|msg: Vec<String>| {
            println!("{}", msg.join(" "));

            0
        }),
    }
    .run()
    .expect("Failed to run the application");
    std::process::exit(code);
}
