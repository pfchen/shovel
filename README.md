# Shovel

[![Crates.io](https://img.shields.io/crates/v/shovel.svg)](https://crates.io/crates/shovel)
[![Docs](https://docs.rs/shovel/badge.svg)](https://docs.rs/shovel/latest/shovel/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Shovel is a declarative Rust CLI library with typed options and arguments,
nested commands, generated help and shell completion, and `no_std` support.
Its heapless execution API avoids internal allocation.

[Quick start](#quick-start) · [Options and arguments](#options-and-arguments) ·
[Commands](#commands-and-global-options) · [Exit codes](#exit-codes) ·
[Environment](#environment-fallback) · [Execution](#execution-and-no_std) ·
[Features](#completion-and-cargo-features) · [Examples](#examples-and-reference)

## Quick start

```toml
[dependencies]
shovel = "0.1.1"
```

Define options and positional arguments, then attach a handler:

```rust
use shovel::*;

#[derive(Opts)]
struct GreetOpts {
    /// Print an uppercase greeting
    #[short = 'u']
    uppercase: bool,
    /// Number of greetings
    #[short = 'n']
    #[env = "GREET_COUNT"]
    count: Option<u16>,
}

#[derive(Args)]
struct GreetArgs {
    /// Person to greet
    name: String,
}

fn main() -> Result<(), std::fmt::Error> {
    let app = App {
        name: "greet",
        description: "Greet someone",
        version: "0.1.0",
        authors: ["Your Name"],
        copyright: "",
        action: Action::opts_args(|opts: GreetOpts, args: GreetArgs| {
            let greeting = format!("Hello, {}!", args.name);
            for _ in 0..opts.count.unwrap_or(1) {
                if opts.uppercase {
                    println!("{}", greeting.to_uppercase());
                } else {
                    println!("{greeting}");
                }
            }
        }),
    };
    std::process::exit(app.run()?);
}
```

`greet -u -n 2 Ada` prints two uppercase greetings. Built-in `--help`/`-h`
and `--version`/`-V` print application metadata. `run()` reads process
environment variables automatically; other execution methods take environment
values explicitly, as shown [below](#environment-fallback).

## Options and arguments

Derive `Opts` on a named struct. Field names become kebab-case long options;
doc comments become help descriptions.

| Field type | Behavior |
|---|---|
| `bool` | Flag, initially `false` |
| `Option<T>` | Optional typed value |
| `T` | Required typed value |
| `Vec<T>` / `Option<Vec<T>>` | Required / optional list |

Lists split on commas, trim whitespace, and discard empty elements. Repeated
CLI occurrences append: `--tags "a, b" --tags c` yields `a`, `b`, `c`.
An explicitly empty list counts as supplied.

| Attribute | Applies to | Purpose |
|---|---|---|
| `#[short]` / `#[short = 'p']` | `Opts` field | Infer the first letter / choose a short name |
| `#[long = "bind-host"]` | `Opts` field | Override the long name |
| `#[env = "APP_PORT"]` | `Opts` field | Name an environment fallback |
| `#[format = "<PORT>"]` | `Opts` field, `Args` struct or enum variant | Customize the help format |

Valued options accept `--port 8080`, `--port=8080`, and `-p=8080`.
Short flags can be clustered. A `--` separator ends option parsing.

Derive `Args` on a struct for positional fields, or an enum for alternative
argument shapes. A trailing `Vec<T>` accepts variadic arguments. Built-in
implementations also support individual values and tuples. Values use typed
parsing; invalid values and missing required input produce diagnostics.
See the [`Opts`](https://docs.rs/shovel/latest/shovel/trait.Opts.html) and
[`Args`](https://docs.rs/shovel/latest/shovel/trait.Args.html) references for
supported types and manual implementations.

## Commands and global options

Commands form a tree: `Action::subcmds` accepts a command or tuple of commands,
and a command may contain further subcommands. Root options are inherited as
global options. For example, `tool --verbose admin status` reaches this handler:

```rust
use shovel::*;

#[derive(Opts)]
struct GlobalOpts {
    /// Enable verbose output
    #[short = 'v']
    verbose: bool,
}

fn main() -> Result<(), std::fmt::Error> {
    let app = App {
        name: "tool",
        description: "Administration tool",
        version: "0.1.0",
        authors: [],
        copyright: "",
        action: AppAction::subcmds((Cmd {
            name: "admin",
            description: "Administration commands",
            category: Some("management"),
            action: Action::subcmds((Cmd {
                name: "status",
                description: "Show status",
                category: None,
                action: Action::global_opts(|global: GlobalOpts| {
                    println!("Ready; verbose={}", global.verbose);
                    0
                }),
            },)),
        },)),
    };
    std::process::exit(app.run()?);
}
```

Use `category` to group commands in help. Unknown commands receive suggestions.
`Action::new` combines a default handler with subcommands.

Handlers can take options, positional arguments, global options, or combinations
through `Action::opts`, `args`, `global_opts`, `opts_args`,
`opts_with_global_opts`, `args_with_global_opts`, and `all`. Use
`Action::handler` for a `Context` containing all three, or `no_args` for none.
Functions and noncapturing closures are supported.

With `std`, `AppBuilder` and `CmdBuilder` offer fluent construction of the same
tree. See the [builder example](shovel/examples/builder.rs).

## Exit codes

Handlers may return `()` for success or an `i32` exit code. The same constructors
accept both, and commands can mix them. Every execution method returns
`Result<i32, core::fmt::Error>`:

| Outcome | Result |
|---|---|
| Handler returns `()` | `Ok(EXIT_CODE_SUCCESS)` (0) |
| Handler returns an `i32` | `Ok(code)` |
| Explicit help, version, or completion | `Ok(EXIT_CODE_SUCCESS)` (0) |
| Invalid CLI input or environment fallback | `Ok(EXIT_CODE_USAGE)` (2) after writing the diagnostic |
| Output fails | `Err(fmt::Error)` |

The crate exports the codes as `EXIT_CODE_SUCCESS` and `EXIT_CODE_USAGE`.
Shovel returns the status; the application decides whether to call
`std::process::exit`, as in the examples above. Handler return types are inferred.

When explicitly annotating `Action`, `App`, `Cmd`, or their builders, the final
return-type parameter defaults to `()`, preserving unit-handler annotations.
Specify a final `i32` for an explicitly annotated integer handler:

```rust
use shovel::Action;

let unit: Action<(), (), (), ()> = Action::no_args(|| {});
let code: Action<(), (), (), (), i32> = Action::no_args(|| 17);
```

For closures that only panic, call `unimplemented!()`, or exit the process,
annotate their return type as `|| -> () { ... }` (or `i32`). Otherwise Rust
may infer the never type (`!`), which is not a supported handler return type.

Execution methods now return `Result<i32, core::fmt::Error>`; callers explicitly
expecting `Result<(), core::fmt::Error>` must handle or discard the returned code.

## Environment fallback

Only `Opts` fields annotated with `#[env = "NAME"]` use environment fallback.
`run()` collects process arguments and environment variables automatically, so
the `GREET_COUNT` fallback in the quick-start example works out of the box.

The explicit-input methods (`run_with_args`, `run_with_args_and_env`,
`run_heapless`, and `run_heapless_with_env`) do not read process environment
variables. For controlled input, pass borrowed pairs such as
`&[("GREET_COUNT", "2")]` to an environment-aware method. This also works
without `std`.

`shovel::env::vars_os()` and `env::args()` expose the process inputs explicitly
for custom execution paths:

```rust
let vars = shovel::env::vars_os();
std::process::exit(app.run_with_env(&vars.to_strs())?);
```

The `std` helper collects owned UTF-8 names and values and ignores non-Unicode
entries. If a declared variable is present but its value is non-Unicode, it is
treated as absent: a required field reports `MissingRequiredOption`, while an
optional field keeps its default or `None`. `env::args()` similarly collects
process arguments; its `to_strs()` method provides input for explicit-argument
execution. Note that `env::args()` relies on `std::env::args()`, which panics if
a process argument is not valid Unicode.

CLI values override fallback, including explicitly empty lists; overridden
environment values are not parsed. Required fields are checked after fallback.
Booleans accept `true` or `false`; lists use the same comma parsing as CLI input.
Unknown environment names are ignored, and the first matching pair wins for
duplicate names. Positional arguments have no environment fallback. Manual
`Opts` implementations can override `parse_with_env`; the default parses CLI
input only.

## Execution and `no_std`

| Method | Input and output | Requires `std` |
|---|---|---|
| `run()` | Process arguments, process environment, stdout | Yes |
| `run_with_env(env)` | Process arguments, stdout, supplied environment | Yes |
| `run_with_args(writer, args)` | Supplied arguments and writer | Yes |
| `run_with_args_and_env(writer, args, env)` | Supplied arguments, writer, environment | Yes |
| `run_heapless::<H, _>(writer, args)` | Supplied arguments and writer, stack help storage | No |
| `run_heapless_with_env::<H, _>(writer, args, env)` | Same, with supplied environment | No |

Explicit argument slices include the executable name at index zero.
`run` collects arguments and process environment on the heap; `run_with_args`
uses heap help storage.
Heapless methods avoid internal allocation, though selected field types and
handlers can still allocate. `H` is the total number of commands, including the
root. With nightly `generic_const_exprs`, omit the generic arguments entirely.

For testing or custom output, `String` implements `Writer` through
`core::fmt::Write`; `IoWriteAdapter` adapts a `std::io::Write` sink. Implement
`Writer` to customize help, version, completion, and diagnostic rendering; see
the [custom writer example](shovel/examples/custom-writer.rs).

Disable default features for `no_std`:

```toml
[dependencies]
shovel = { version = "0.1.1", default-features = false }
```

Supply a writer and borrowed input; `Stdout`, the environment collection helpers,
and builders require `std`. This library function needs no allocator:

```rust
#![no_std]
#![cfg_attr(feature = "generic_const_exprs", feature(generic_const_exprs))]
#![allow(incomplete_features)]

use shovel::*;

pub fn run(writer: &mut impl Writer) -> Result<i32, core::fmt::Error> {
    let app = App {
        name: "embedded",
        description: "Minimal CLI",
        version: "0.1.0",
        authors: [],
        copyright: "",
        action: Action::no_args(|| {}),
    };
    #[cfg(not(feature = "generic_const_exprs"))]
    let result = app.run_heapless_with_env::<1, _>(writer, &["embedded"], &[]);
    #[cfg(feature = "generic_const_exprs")]
    let result = app.run_heapless_with_env(writer, &["embedded"], &[]);
    result
}
```

A runnable example with platform I/O and panic handling is provided in
[no-std-adder.rs](shovel/examples/no-std-adder.rs).

## Completion and Cargo features

Help and version are built in. With completion enabled, generate scripts for
Bash, Zsh, or Fish and install them using your shell's completion setup:

```sh
greet --completion bash > greet.bash
greet --completion zsh > _greet
greet --completion fish > greet.fish
```

Scripts cover commands, nested commands, and option names.

| Feature | Effect |
|---|---|
| `std` | Process I/O, collection helpers, builders, and heap-backed execution |
| `completion` | `--completion` and Bash, Zsh, Fish generation |
| `generic_const_exprs` | Infer heapless command capacity; requires nightly Rust |
| `subcmds-50`, `subcmds-75`, `subcmds-100` | Raise the per-level subcommand limit from 25 |

`std` and `completion` are enabled by default and can be selected independently.
Subcommand-limit features are additive; the highest enabled limit wins. Nested
command trees let you organize larger interfaces into smaller groups.

## Examples and reference

| Example | Demonstrates |
|---|---|
| [hello](shovel/examples/hello.rs) | Minimal application |
| [echo](shovel/examples/echo.rs) | Variadic positional arguments |
| [builder](shovel/examples/builder.rs) | Builders, commands, categories, options |
| [webserver](shovel/examples/webserver.rs) | Larger command tree and custom value types |
| [custom-writer](shovel/examples/custom-writer.rs) | Output adapters and custom diagnostics |
| [no-std-adder](shovel/examples/no-std-adder.rs) | Heapless execution and environment fallback without `std` |

Run a standard example with `cargo run --example hello -- --help`.
The [API reference](https://docs.rs/shovel/latest/shovel/) contains the full
trait and constructor documentation.

## Contributing and license

Issues and pull requests are welcome. Shovel is distributed under the
[MIT license](LICENSE).
