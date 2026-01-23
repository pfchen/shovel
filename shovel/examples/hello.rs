// This example demonstrates the simplest possible Shovel application.
// It shows the basic structure without any options or arguments.
//
// To run this example:
// 1. Navigate to the project root directory in your terminal.
// 2. Execute: `cargo run --example hello`
//
// Expected output:
// Hello, world!

use shovel::*;

fn main() {
    App {
        name: "hello",
        description: "A simple greeting application",
        version: "0.1.0",
        authors: ["Author Name <author@example.com>"],
        copyright: "(c) 2026 Company Name",
        action: Action::no_args(|| {
            println!("Hello, world!");
        }),
    }
    .run()
    .expect("Failed to run the application");
}
