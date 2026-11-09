# Shovel Macros

Derive macros for [Shovel](https://crates.io/crates/shovel), a declarative Rust
CLI library with typed options, positional arguments, and nested commands.

Use these macros through the `shovel` crate:

```toml
[dependencies]
shovel = "0.1.1"
```

```rust
use shovel::{Args, Opts};

#[derive(Opts)]
struct Options {
    #[short = 'p']
    #[env = "APP_PORT"]
    port: Option<u16>,
}

#[derive(Args)]
struct Arguments {
    input: String,
}
```

See the [Shovel guide](https://github.com/pfchen/shovel#readme) for complete
applications and the [macro API reference](https://docs.rs/shovel_macros/) for
attributes and examples. `impl_subcmds!` supports Shovel's internal command tuple
implementations.

Distributed under the [MIT license](LICENSE).
