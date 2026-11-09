//! Builder pattern for constructing applications and commands.
//!
//! This module provides fluent builder APIs for creating [`App`] and [`Cmd`] instances.
//! The builders offer a more ergonomic alternative to direct struct construction,
//! especially useful when building complex applications with many configuration options.
//!
//! # Core Types
//!
//! ## [`AppBuilder`] - Application Builder
//!
//! Fluent API for constructing [`App`] instances with method chaining:
//! ```rust
//! # #[cfg(feature = "std")] {
//! # use shovel::*;
//! let app = AppBuilder::new("myapp")
//!     .description("A sample application")
//!     .version("0.1.0")
//!     .authors(["Developer <dev@example.com>"])
//!     .copyright("(c) 2026 Company")
//!     .action(Action::no_args(|| println!("Hello!")))
//!     .build();
//! # }
//! ```
//!
//! ## [`CmdBuilder`] - Command Builder
//!
//! Fluent API for constructing [`Cmd`] instances:
//! ```rust
//! # #[cfg(feature = "std")] {
//! # use shovel::*;
//! let cmd: Cmd<'_, (), (), (), ()> = CmdBuilder::new("deploy")
//!     .description("Deploy the application")
//!     .category("deployment")
//!     .action(Action::no_args(|| println!("Deploying...")))
//!     .build();
//! # }
//! ```
//!
//! # Builder Pattern Benefits
//!
//! - **Fluent Interface**: Method chaining for readable configuration
//! - **Type Safety**: Compile-time validation of required fields
//! - **Consistent**: Uniform API across different types
//!
//! # Usage Patterns
//!
//! ## Simple Application
//! ```rust
//! # #[cfg(feature = "std")] {
//! # use shovel::*;
//! # fn main() {
//! AppBuilder::new("greet")
//!     .description("Greeting application")
//!     .version("0.1.0")
//!     .authors(["Author Name"])
//!     .copyright("Copyright 2026")
//!     .action(Action::args(|name: String| {
//!         println!("Hello, {}!", name);
//!     }))
//!     .build()
//!     .run()
//!     .expect("Failed to run application");
//! # }
//! # }
//! ```
//!
//! ## Application with Subcommands
//! ```rust
//! # #[cfg(feature = "std")] {
//! # use shovel::*;
//! # fn main() {
//! AppBuilder::new("tool")
//!     .description("Multi-purpose tool")
//!     .version("2.0.0")
//!     .authors(["Author Name <author@example.com>"])
//!     .copyright("Copyright 2026")
//!     .action(Action::subcmds((
//!         CmdBuilder::new("build")
//!             .description("Build the project")
//!             .category("development")
//!             .action(Action::no_args(|| println!("Building...")))
//!             .build(),
//!         CmdBuilder::new("test")
//!             .description("Run tests")
//!             .category("development")
//!             .action(Action::no_args(|| println!("Testing...")))
//!             .build(),
//!     )))
//!     .build()
//!     .run()
//!     .expect("Failed to run application");
//! # }
//! # }
//! ```
//!
//! ## Nested Command Hierarchies
//! ```rust
//! # #[cfg(feature = "std")] {
//! # use shovel::*;
//! let database_cmd: Cmd<'_, (), (), (), _> = CmdBuilder::new("database")
//!     .description("Database operations")
//!     .action(Action::subcmds((
//!         CmdBuilder::new("migrate")
//!             .description("Run database migrations")
//!             .action(Action::no_args(|| println!("Migrating...")))
//!             .build(),
//!         CmdBuilder::new("seed")
//!             .description("Seed database with test data")
//!             .action(Action::no_args(|| println!("Seeding...")))
//!             .build(),
//!     )))
//!     .build();
//! # }
//! ```
//!
//! # Feature Requirements
//!
//! This module is only available when the `std` feature is enabled, as it relies on
//! heap allocation for building complex command structures.
//!
//! ```toml
//! [dependencies]
//! shovel = { version = "*", features = ["std"] }
//! ```
//!
//! # Performance Considerations
//!
//! - **Build Time**: Builders use heap allocation during construction
//! - **Runtime**: No performance overhead after `build()` is called
//! - **Memory**: Temporary allocations are freed after building
//! - **Optimization**: Final structures are identical to direct construction
//!
//! See also: [`App`], [`Cmd`], [`Action`]

use crate::{Action, App, Args, Cmd, Opts, Subcmds};

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
/// A builder for creating `App` instances.
///
/// This struct provides a fluent interface for constructing `App` instances,
/// allowing users to set various properties of the application in a more
/// convenient and readable manner.
///
/// # Usage
///
/// To create an `App` instance using `AppBuilder`, you can chain method calls
/// to set the desired properties, and then call the `build` method to create
/// the `App` instance.
///
/// # Example
///
/// ```
/// use shovel::*;
///
/// let app = AppBuilder::new("myapp")
///     .description("A simple application")
///     .version("0.1.0")
///     .authors(["Author Name <author@example.com>"])
///     .copyright("(c) 2026 Company Name")
///     .action(Action::no_args(|| println!("Hello, world!")))
///     .build();
///
/// assert_eq!(app.name, "myapp");
/// ```
///
/// # Usage Guidelines
///
/// The builder pattern is particularly useful when you need to conditionally set fields:
/// ```
/// # use shovel::*;
/// let mut builder = AppBuilder::new("myapp")
///     .description("A configurable application")
///     .version("0.1.0");
///
/// #[cfg(feature = "debug")]
/// let builder = builder.authors(["Debug Author"]);
/// #[cfg(not(feature = "debug"))]
/// let builder = builder.authors(["Release Author"]);
///
/// let app = builder
///     .copyright("(c) 2026 Company")
///     .action(Action::no_args(|| {}))
///     .build();
/// ```
///
/// See also: [`App`], [`Action`]
///
/// # Fields
///
/// - `name`: The name of the application, which will be displayed in help messages.
/// - `description`: A brief description of what the application does.
/// - `version`: The version number of the application, useful for displaying version info.
/// - `authors`: A list of authors of the application, which can be displayed in help messages.
/// - `copyright`: The copyright notice for the application, typically displayed in the footer.
/// - `action`: The main action of the application, defining what happens when the app is run.
#[derive(Debug)]
pub struct AppBuilder<
    'a,
    const N: usize,
    O: Opts,
    A: Args,
    C: Subcmds<O>,
    R: crate::ActionReturn = (),
> {
    /// The name of the application.
    name: Option<&'a str>,
    /// A brief description of the application.
    description: Option<&'a str>,
    /// The version number of the application.
    version: Option<&'a str>,
    /// A list of authors of the application.
    authors: Option<[&'a str; N]>,
    /// The copyright notice for the application.
    copyright: Option<&'a str>,
    /// The action of the application.
    action: Option<Action<O, A, O, C, R>>,
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
/// A builder for creating instances of the [`Cmd`] struct.
///
/// This builder allows for a flexible and fluent way to construct a `Cmd`
/// instance by setting its fields step by step.
///
/// # Example
///
/// ```
/// use shovel::*;
///
/// let cmd: Cmd<'_, (), (), (), ()> = CmdBuilder::new("hello")
///     .description("Print hello message")
///     .category("utils")
///     .action(Action::no_args(|| println!("Hello, world!")))
///     .build();
///
/// assert_eq!(cmd.name, "hello");
/// assert_eq!(cmd.description, "Print hello message");
/// assert_eq!(cmd.category, Some("utils"));
/// ```
#[derive(Debug)]
pub struct CmdBuilder<
    'a,
    O: Opts,
    A: Args,
    G: Opts,
    C: Subcmds<G>,
    R: crate::ActionReturn = (),
> {
    name: Option<&'a str>,
    description: Option<&'a str>,
    category: Option<&'a str>,
    action: Option<Action<O, A, G, C, R>>,
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl<
    'a,
    const N: usize,
    O: Opts,
    A: Args,
    C: Subcmds<O>,
    R: crate::ActionReturn,
> AppBuilder<'a, N, O, A, C, R>
{
    /// Creates a new instance of `AppBuilder`.
    ///
    /// This method initializes an `AppBuilder` with the specified name,
    /// while all other fields are set to `None` by default.
    ///
    /// # Parameters
    /// - `name`: The name of the application.
    ///
    /// # Returns
    ///
    /// Returns a new `AppBuilder` instance with the specified name.
    #[must_use]
    pub fn new(name: &'a str) -> Self {
        Self {
            name: Some(name),
            description: None,
            version: None,
            authors: None,
            copyright: None,
            action: None,
        }
    }

    /// Sets the name of the application.
    ///
    /// This method allows you to specify the name of the application,
    /// which will be displayed in help messages.
    ///
    /// # Parameters
    ///
    /// - `name`: A string slice representing the name of the application.
    ///
    /// # Returns
    ///
    /// Returns the updated `AppBuilder` instance for method chaining.
    #[must_use]
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets the description of the application.
    ///
    /// This method allows you to specify a brief description of the application,
    /// which will be displayed in help messages.
    ///
    /// # Parameters
    ///
    /// - `description`: A string slice providing a brief description of the application.
    ///
    /// # Returns
    ///
    /// Returns the updated `AppBuilder` instance for method chaining.
    #[must_use]
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets the version of the application.
    ///
    /// This method allows you to specify the version number of the application,
    /// which can be displayed in help messages and version information.
    ///
    /// # Parameters
    ///
    /// - `version`: A string slice representing the version number of the application.
    ///
    /// # Returns
    ///
    /// Returns the updated `AppBuilder` instance for method chaining.
    #[must_use]
    pub fn version(mut self, version: &'a str) -> Self {
        self.version = Some(version);
        self
    }

    /// Sets the authors of the application.
    ///
    /// This method allows you to specify the authors of the application,
    /// which can be displayed in help messages.
    ///
    /// # Parameters
    ///
    /// - `authors`: An array of string slices representing the authors of the application.
    ///
    /// # Returns
    ///
    /// Returns the updated `AppBuilder` instance for method chaining.
    #[must_use]
    pub fn authors(mut self, authors: [&'a str; N]) -> Self {
        self.authors = Some(authors);
        self
    }

    /// Sets the copyright notice for the application.
    ///
    /// This method allows you to specify the copyright notice,
    /// which is typically displayed in the footer of help messages.
    ///
    /// # Parameters
    ///
    /// - `copyright`: A string slice representing the copyright notice.
    ///
    /// # Returns
    ///
    /// Returns the updated `AppBuilder` instance for method chaining.
    #[must_use]
    pub fn copyright(mut self, copyright: &'a str) -> Self {
        self.copyright = Some(copyright);
        self
    }

    /// Sets the action of the application.
    ///
    /// This method allows you to define the main action that the application will perform
    /// when it is run. The action is specified using the `Action` type.
    ///
    /// # Parameters
    ///
    /// - `action`: An `Action` instance defining what happens when the app is executed.
    ///
    /// # Returns
    ///
    /// Returns the updated `AppBuilder` instance for method chaining.
    #[must_use]
    pub fn action(mut self, action: Action<O, A, O, C, R>) -> Self {
        self.action = Some(action);
        self
    }

    /// Builds the `App` instance from the builder.
    ///
    /// This method finalizes the configuration of the `AppBuilder` and creates a new
    /// `App` instance with the specified properties.
    ///
    /// # Panics
    ///
    /// This method will panic if required fields (name, description, version, authors,
    /// copyright, action) are not set.
    ///
    /// # Returns
    ///
    /// Returns a new `App` instance with the configured properties.
    #[must_use]
    pub fn build(self) -> App<'a, N, O, A, C, R> {
        App {
            name: self.name.expect("Name must be set"),
            description: self.description.expect("Description must be set"),
            version: self.version.expect("Version must be set"),
            authors: self.authors.expect("Authors must be set"),
            copyright: self.copyright.expect("Copyright must be set"),
            action: self.action.expect("Action must be set"),
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl<'a, O: Opts, A: Args, G: Opts, C: Subcmds<G>, R: crate::ActionReturn>
    CmdBuilder<'a, O, A, G, C, R>
{
    /// Creates a new `CmdBuilder` with the specified name.
    ///
    /// This method initializes the builder with the provided name and default values
    /// for other fields:
    /// - `name`: Set to the provided value.
    /// - `description`: `None`.
    /// - `category`: `None`.
    /// - `action`: `None`.
    ///
    /// # Parameters
    /// - `name`: The name of the command.
    ///
    /// # Returns
    /// A new instance of `CmdBuilder` with the specified name.
    #[must_use]
    pub fn new(name: &'a str) -> Self {
        CmdBuilder {
            name: Some(name),
            description: None,
            category: None,
            action: None,
        }
    }

    /// Sets the name of the command.
    ///
    /// # Parameters
    /// * `name` - The name to set for the command.
    ///
    /// # Returns
    /// The builder instance, allowing for method chaining.
    #[must_use]
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets the description of the command.
    ///
    /// # Parameters
    /// * `description` - The description to set for the command.
    ///
    /// # Returns
    /// The builder instance, allowing for method chaining.
    #[must_use]
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets the category of the command.
    ///
    /// # Parameters
    /// * `category` - The category to which the command belongs.
    ///
    /// # Returns
    /// The builder instance, allowing for method chaining.
    #[must_use]
    pub fn category(mut self, category: &'a str) -> Self {
        self.category = Some(category);
        self
    }

    /// Sets the action of the command.
    ///
    /// # Parameters
    /// - `action`: The action to be executed when the command is run.
    ///
    /// # Returns
    /// The builder instance, allowing for method chaining.
    #[must_use]
    pub fn action(mut self, action: Action<O, A, G, C, R>) -> Self {
        self.action = Some(action);
        self
    }

    /// Builds and returns a `Cmd` instance.
    ///
    /// This method finalizes the configuration of the `CmdBuilder` and creates a new
    /// `Cmd` instance with the specified properties.
    ///
    /// # Panics
    ///
    /// This method will panic if required fields (name, description, action) are not set.
    ///
    /// # Returns
    ///
    /// A fully constructed `Cmd` instance.
    #[must_use]
    pub fn build(self) -> Cmd<'a, O, A, G, C, R> {
        Cmd {
            name: self.name.expect("Name must be set"),
            description: self.description.expect("Description must be set"),
            category: self.category,
            action: self.action.expect("Action must be set"),
        }
    }
}
