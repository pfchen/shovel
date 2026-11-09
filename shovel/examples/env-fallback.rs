//! Demonstrates `App::run()` reading process environment variables.

use shovel::*;

#[derive(Opts)]
struct EnvOpts {
    /// Value read from `SHOVEL_ENV_FALLBACK`
    #[env = "SHOVEL_ENV_FALLBACK"]
    value: String,
}

fn main() -> Result<(), core::fmt::Error> {
    let app = App {
        name: "env-fallback",
        description: "Reads an environment fallback",
        version: "0.1.0",
        authors: [],
        copyright: "",
        action: Action::opts(|opts: EnvOpts| println!("{}", opts.value)),
    };
    std::process::exit(app.run()?);
}
