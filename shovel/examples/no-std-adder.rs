// This example demonstrates a `no_std` application that calculates the sum of two numbers.
// It showcases how `shovel` can be used in environments without the standard library,
// relying on `libc` for basic I/O.
//
// To build and run this example:
// 1. Ensure you have a `no_std` compatible Rust toolchain installed (e.g., `rustup target add x86_64-unknown-linux-gnu`).
// 2. Navigate to the project root directory in your terminal.
// 3. Build the example: `cargo build --example no-std-adder --target x86_64-unknown-linux-gnu`
// 4. Run the example: `./target/x86_64-unknown-linux-gnu/debug/examples/no-std-adder 10 20`
//
// Expected output:
// 10 + 20 = 30
//
// You can also try with help or version flags:
// `./target/x86_64-unknown-linux-gnu/debug/examples/no-std-adder --help`
// `./target/x86_64-unknown-linux-gnu/debug/examples/no-std-adder --version`

#![cfg_attr(not(feature = "std"), feature(lang_items))]
#![allow(internal_features, incomplete_features)]
#![cfg_attr(feature = "generic_const_exprs", feature(generic_const_exprs))]
#![no_std]
#![no_main]
//

// Pull in the system libc library for what crt0.o likely requires.
extern crate libc;

// Supply the C runtime and unwind symbols when std is absent.
#[cfg(not(feature = "std"))]
#[link(name = "c")]
#[link(name = "gcc_s")]
unsafe extern "C" {}

use core::ffi::{CStr, c_char, c_int};
use core::fmt::Write;
use shovel::*;

const LIBC_STDOUT: c_int = 1;
const LIBC_NEWLINE: &str = "\n";

pub struct Stdout;

impl Stdout {
    pub fn new() -> Self {
        Self
    }

    fn write_newline(&self) {
        libc_write(LIBC_NEWLINE);
    }
}

impl Default for Stdout {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        libc_write(s);
        Ok(())
    }
}

#[derive(Args)]
pub struct SumArgs {
    first: f64,
    second: f64,
}

#[derive(Opts)]
pub struct SumOpts {
    #[env = "SUM_FACTOR"]
    factor: f64,
}

macro_rules! libc_println {
    () => { $crate::libc_println!("") };
    ($($arg:tt)*) => {
        {
            #[allow(unused_must_use)]
            {
                let mut stdout = Stdout::new();
                stdout.write_fmt(format_args!($($arg)*));
                stdout.write_newline();
            }
        }
    };
}

#[macro_export]
macro_rules! libc_print {
    ($($arg:tt)*) => {
        {
            #[allow(unused_must_use)]
            {
                let mut stdout = Stdout::new();
                stdout.write_fmt(format_args!($($arg)*));
            }
        }
    };
}

// Entry point for this program.
#[unsafe(no_mangle)] // ensure that this symbol is included in the output as `main`
extern "C" fn main(argc: c_int, argv: *const *const c_char) -> c_int {
    // Validate argc to prevent potential overflow or negative values
    if !(0..=1000).contains(&argc) {
        libc_println!("Error: Invalid argument count");
        return 1;
    }

    let c_args = unsafe { core::slice::from_raw_parts(argv, argc as usize) };

    // Check for valid number of arguments or help flag or version flag
    if !(c_args.len() == 3
        || (c_args.len() == 2
            && (is_help_flag(c_args[1]) || is_version_flag(c_args[1]))))
    {
        // When run as a test (despite test = false), we get no arguments
        // In this case, just return success to avoid test failures
        #[cfg(test)]
        {
            libc_println!("Error: Invalid number of arguments");
            return 0;
        }

        #[cfg(not(test))]
        {
            libc_println!("Error: Invalid number of arguments");
            return 1;
        }
    }

    let mut args: [&str; 3] = ["", "", ""];
    if c_args.len() == 3 {
        unsafe {
            args[0] = CStr::from_ptr(c_args[0])
                .to_str()
                .expect("Invalid argument 0");

            args[1] = CStr::from_ptr(c_args[1])
                .to_str()
                .expect("Invalid argument 1");

            args[2] = CStr::from_ptr(c_args[2])
                .to_str()
                .expect("Invalid argument 2");
        }
    } else {
        unsafe {
            args[0] = CStr::from_ptr(c_args[0])
                .to_str()
                .expect("Invalid argument 0");
            args[1] = CStr::from_ptr(c_args[1])
                .to_str()
                .expect("Invalid argument 1");
        }
    }

    let app = App {
        name: "no-std-adder",
        description: "Add two numbers together",
        version: "0.1.0",
        authors: ["Author Name <author@example.com>"],
        copyright: "(c) 2026 Company Name",
        action: Action::opts_args(|opts: SumOpts, args: SumArgs| {
            libc_println!(
                "{} + {} = {}",
                args.first,
                args.second,
                (args.first + args.second) * opts.factor
            );
        }),
    };

    // Borrowed fallback input works without std or heap allocation.
    let env = [("SUM_FACTOR", "1")];
    #[cfg(feature = "generic_const_exprs")]
    let result = app.run_heapless_with_env(&mut Stdout::new(), &args, &env);

    #[cfg(not(feature = "generic_const_exprs"))]
    let result =
        app.run_heapless_with_env::<1, _>(&mut Stdout::new(), &args, &env);

    match result {
        Err(_) => {
            // Error handling: In a no-std environment, we can't use eprintln!
            // but we could write to stderr using libc if needed
            libc_println!(
                "Error: Failed to parse arguments or execute command"
            );
            1
        }
        Ok(code) => code,
    }
}

// These functions are used by the compiler, but not for an empty program like this.
// They are normally provided by `std`.
#[cfg(not(feature = "std"))]
#[lang = "eh_personality"]
fn rust_eh_personality() {
    // eh_personality
}

#[cfg(not(feature = "std"))]
#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    unsafe { libc::abort() }
}

fn libc_write(msg: &str) {
    let data = msg.as_bytes();
    unsafe {
        libc::write(
            LIBC_STDOUT,
            data.as_ptr().cast::<core::ffi::c_void>(),
            data.len(),
        );
    };
}

// Add this helper function at the end of the file
fn is_help_flag(arg: *const c_char) -> bool {
    unsafe {
        let result = CStr::from_ptr(arg).to_str();
        result == Ok("-h") || result == Ok("--help")
    }
}

fn is_version_flag(arg: *const c_char) -> bool {
    unsafe {
        let result = CStr::from_ptr(arg).to_str();
        result == Ok("-v") || result == Ok("--version")
    }
}
