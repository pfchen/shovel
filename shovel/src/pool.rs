//! Memory pool implementations for efficient allocation in CLI parsing.
//!
//! This module provides memory pool abstractions that enable efficient memory management
//! for command-line parsing operations. The pools support both stack-based allocation
//! (for no-std environments) and heap-based allocation (for std environments).
//!
//! # Core Types
//!
//! ## [`Pool`] - Generic Pool Interface
//!
//! Unified interface for memory operations used internally and exposed to custom
//! [`Subcmds`][crate::Subcmds] implementations. Users implementing custom [`Subcmds`][crate::Subcmds] will use
//! [`Pool`] in `get_help_list()`:
//!
//! ```rust
//! # use shovel::*;
//! struct MyCustomSubcmds<'a, G: Opts> {
//!     cmd1: Cmd<'a, (), (), G, ()>,
//!     cmd2: Cmd<'a, (), (), G, ()>,
//! }
//!
//! impl<'a, G: Opts> Subcmds<G> for MyCustomSubcmds<'a, G> {
//!     const CMD_COUNT: usize = 2;
//!
//!     fn run<'b>(
//!         &'b self,
//!         name: &'b str,
//!         raw_args: &'b RawArgs<'b>,
//!         global_opts: G,
//!     ) -> ActionResult<'b> {
//!         if self.cmd1.name == name {
//!             self.cmd1.run(raw_args, global_opts)
//!         } else if self.cmd2.name == name {
//!             self.cmd2.run(raw_args, global_opts)
//!         } else {
//!             ActionResult::Suggestion(Suggestion::find(
//!                 name,
//!                 &[self.cmd1.name, self.cmd2.name],
//!             ))
//!         }
//!     }
//!
//!     fn get_help_list<'b>(
//!         &'b self,
//!         pool: Pool<'b, CmdHelp<'b>>,
//!     ) -> Option<&'b [CmdHelp<'b>]> {
//!         let (mut parent, children) = pool.alloc(2);
//!         let (subcmd1_pool, subcmd2_pool) = children.alloc(0); // No nested subcommands
//!
//!         Some(
//!             parent.assign(&[
//!                 self.cmd1.help(subcmd1_pool),
//!                 self.cmd2.help(subcmd2_pool),
//!             ]),
//!         )
//!     }
//! }
//! ```
//!
//! ## `ArrayPool` - Zero-Allocation Stack Pool
//!
//! **Performance**: Zero heap allocation, optimal cache locality
//! ```rust
//! // ArrayPool is used internally by run_heapless()
//! use shovel::*;
//!
//! let app = App {
//!     name: "myapp",
//!     description: "A simple app",
//!     version: "0.1.0",
//!     authors: ["Author Name <author@example.com>"],
//!     copyright: "(c) 2026 Company Name",
//!     action: Action::no_args(|| println!("Hello!")),
//! };
//!
//! // This uses ArrayPool internally for zero allocation
//! // app.run_heapless::<1, _>(&mut writer, &args);
//! ```
//!
//! **Characteristics**:
//! - **Zero heap allocation** - All memory on stack
//! - **Compile-time sizing** - Size known at compile time
//! - **No-std compatible** - Works in embedded environments
//! - **Cache-friendly** - Better memory locality
//! - **Fixed size** - Cannot grow beyond initial capacity
//! - **Stack usage** - Uses stack space
//!
//! ## `VecPool` - Small-Allocation Heap Pool
//!
//! **Performance**: Small heap allocation, runtime flexibility
//! ```rust
//! // VecPool is used internally by run_with_args() and run()
//! use shovel::*;
//!
//! let app = App {
//!     name: "myapp",
//!     description: "A simple app",
//!     version: "0.1.0",
//!     authors: ["Author Name <author@example.com>"],
//!     copyright: "(c) 2026 Company Name",
//!     action: Action::no_args(|| println!("Hello!")),
//! };
//!
//! // This uses VecPool internally for small heap allocation
//! // app.run_with_args(&mut writer, &args);
//! ```
//!
//! **Characteristics**:
//! - **Small heap allocation** - ~capacity * sizeof(T) bytes
//! - **Runtime sizing** - Size determined at runtime
//! - **Simple API** - No const generics required
//! - **Flexible** - Can be resized if needed
//! - **Std only** - Requires heap allocation
//! - **Allocation overhead** - malloc/free costs
//!
//! # Usage Patterns
//!
//! ## `ArrayPool` Usage (Recommended)
//! - Performance-critical applications
//! - No-std/embedded environments
//! - When command count is known at compile time
//! - Memory allocation should be minimized
//!
//! ## `VecPool` Usage
//! - API ergonomics are more important than performance
//! - Testing with custom I/O capture
//! - Integration scenarios where simplicity matters
//! - Small allocation overhead is acceptable
//!
//! # Performance Characteristics
//!
//! For a CLI app with 5 commands:
//!
//! | Pool Type | Stack Usage | Heap Usage | Performance |
//! |-----------|-------------|------------|-------------|
//! | `ArrayPool<T, 5>` | ~320 bytes | 0 bytes | **Optimal** |
//! | `VecPool<T>::new(5)` | ~64 bytes | ~320 bytes | Good |
//!
//! Both pool types provide the same [`Pool`] interface through the `as_pool()` method,
//! ensuring consistent behavior regardless of the underlying allocation strategy.
//! The pools use `MaybeUninit<T>` for safe uninitialized memory handling and support
//! any type `T` that implements the `Copy` trait.
//!
//! See also: [`App`](crate::App), [`Subcmds`](crate::Subcmds), [`CmdHelp`](crate::CmdHelp)

use core::mem::MaybeUninit;

pub(crate) struct ArrayPool<T, const N: usize> {
    data: [MaybeUninit<T>; N],
}

#[cfg(feature = "std")]
pub(crate) struct VecPool<T> {
    data: std::vec::Vec<T>,
}

/// A linear and simple memory pool for efficient allocation of contiguous elements.
///
/// This pool manages a slice of uninitialized memory that can be used to allocate
/// and assign elements of type `T`. It provides methods for allocating chunks of
/// memory and assigning values to them.
///
/// # Type Parameters
/// - `'a`: The lifetime of the underlying memory slice.
/// - `T`: The type of elements stored in the pool. Must implement `Copy`.
#[derive(Debug)]
pub struct Pool<'a, T> {
    slice: &'a mut [MaybeUninit<T>],
}

impl<T: Copy, const N: usize> ArrayPool<T, N> {
    const ELEM: MaybeUninit<T> = MaybeUninit::uninit();
    const INIT: [MaybeUninit<T>; N] = [Self::ELEM; N];

    pub(crate) const fn new() -> Self {
        Self { data: Self::INIT }
    }

    pub(crate) fn as_pool(&mut self) -> Pool<'_, T> {
        Pool::new(&mut self.data)
    }
}

#[cfg(feature = "std")]
impl<T: Copy> VecPool<T> {
    pub(crate) fn new(size: usize) -> Self {
        Self {
            data: std::vec::Vec::with_capacity(size),
        }
    }

    pub(crate) fn as_pool(&mut self) -> Pool<'_, T> {
        Pool::new(self.data.spare_capacity_mut())
    }
}

impl<'a, T: Copy> Pool<'a, T> {
    pub(crate) fn new(slice: &'a mut [MaybeUninit<T>]) -> Self {
        Self { slice }
    }

    /// Allocates a chunk of memory for `size` elements of type `T`.
    ///
    /// This method splits the current pool into two parts:
    /// - The first part contains the allocated chunk of size `size`.
    /// - The second part contains the remaining unallocated memory.
    ///
    /// # Arguments
    /// * `size` - The number of elements to allocate.
    ///
    /// # Returns
    /// A tuple containing two `Pool`s:
    /// - The first `Pool` represents the allocated chunk.
    /// - The second `Pool` represents the remaining unallocated memory.
    ///
    /// # Panics
    /// Panics if `size` is greater than the available pool capacity.
    #[must_use]
    pub fn alloc(self, size: usize) -> (Self, Self) {
        let (left, right) = self.slice.split_at_mut(size);
        (Self::new(left), Self::new(right))
    }

    /// Assigns values from a slice to the pool and returns a reference to the assigned memory.
    ///
    /// This method copies elements from the input slice to the pool's memory.
    /// If the input slice is longer than the available memory, it will be truncated.
    ///
    /// # Arguments
    /// * `data` - A slice containing the values to be assigned.
    ///
    /// # Returns
    /// A reference to the assigned memory in the pool.
    ///
    /// # Safety
    /// This method uses unsafe code to perform unchecked writes and create a slice.
    /// The caller must ensure that the pool has enough capacity for the input data.
    pub fn assign(&mut self, data: &[T]) -> &'a [T] {
        // Silently truncate the data if it is too long.
        let data = if data.len() > self.slice.len() {
            &data[..self.slice.len()]
        } else {
            data
        };

        debug_assert!(data.len() <= self.slice.len());

        for (i, value) in data.iter().enumerate() {
            // SAFETY: The debug_assert at the beginning of the function ensures
            // that the access is within bounds.
            unsafe {
                self.slice.get_unchecked_mut(i).write(*value);
            }
        }

        unsafe {
            core::slice::from_raw_parts(
                self.slice.as_ptr().cast::<T>(),
                data.len(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CmdHelp;

    // Test the actual usage patterns from app.rs, cmd.rs, and help.rs

    #[test]
    fn test_help_system_array_pool_usage() {
        // Simulate app.rs usage: ArrayPool<CmdHelp<'_>, H>::new()
        let mut helps = ArrayPool::<CmdHelp<'_>, 3>::new();
        let pool = helps.as_pool();

        // This is how it's used in app.rs
        assert_eq!(pool.slice.len(), 3);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_help_system_vec_pool_usage() {
        // Simulate app.rs usage: VecPool::new(Self::CMD_COUNT)
        let mut helps = VecPool::<CmdHelp<'_>>::new(5);
        let pool = helps.as_pool();

        // This is how it's used in app.rs
        assert_eq!(pool.slice.len(), 5);
    }

    #[test]
    fn test_help_allocation_pattern() {
        // Simulate cmd.rs usage: get_help_list for single command
        let mut data: [MaybeUninit<CmdHelp<'_>>; 5] =
            [MaybeUninit::uninit(); 5];
        let pool = Pool::new(&mut data);

        let (mut parent, children) = pool.alloc(1);
        assert_eq!(parent.slice.len(), 1);
        assert_eq!(children.slice.len(), 4);

        // Simulate creating a help entry
        let help = CmdHelp::default();
        let result = parent.assign(&[help]);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_dual_command_allocation_pattern() {
        // Simulate cmd.rs usage: get_help_list for tuple of commands
        let mut data: [MaybeUninit<CmdHelp<'_>>; 10] =
            [MaybeUninit::uninit(); 10];
        let pool = Pool::new(&mut data);

        let (mut parent, children) = pool.alloc(2);
        let (subcmd1_pool, subcmd2_pool) = children.alloc(3); // Assume CMD_COUNT = 3

        assert_eq!(parent.slice.len(), 2);
        assert_eq!(subcmd1_pool.slice.len(), 3);
        assert_eq!(subcmd2_pool.slice.len(), 5);

        // Simulate assigning help entries
        let help1 = CmdHelp::default();
        let help2 = CmdHelp::default();
        let result = parent.assign(&[help1, help2]);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_no_std_stack_allocation_pattern() {
        // Simulate app.rs run_heapless usage
        const CMD_COUNT: usize = 3;

        let mut helps = ArrayPool::<CmdHelp<'_>, CMD_COUNT>::new();

        // Test that pool is properly sized for command count
        assert_eq!(helps.as_pool().slice.len(), CMD_COUNT);
    }

    #[test]
    fn test_macro_generated_help_allocation() {
        // Simulate the macro-generated code from subcmds.rs
        // This tests the complex allocation patterns used by the derive macro
        let mut data: [MaybeUninit<CmdHelp<'_>>; 5] =
            [MaybeUninit::uninit(); 5];
        let pool = Pool::new(&mut data);

        // Simulate: let (mut left, right) = pool.alloc(number);
        let (mut left, right) = pool.alloc(3);

        // Simulate: let (subcmd0_pool, right) = right.alloc(C0::CMD_COUNT);
        let (subcmd0_pool, right) = right.alloc(1);
        let (subcmd1_pool, subcmd2_pool) = right.alloc(1);

        assert_eq!(left.slice.len(), 3);
        assert_eq!(subcmd0_pool.slice.len(), 1);
        assert_eq!(subcmd1_pool.slice.len(), 1);
        assert_eq!(subcmd2_pool.slice.len(), 0);

        // Simulate: Some(left.assign(&[helps]))
        let helps =
            [CmdHelp::default(), CmdHelp::default(), CmdHelp::default()];
        let result = left.assign(&helps);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_realistic_command_count_scenarios() {
        // Test with realistic command counts that would be used in practice

        // Small CLI app (3 commands)
        let mut small_helps = ArrayPool::<CmdHelp<'_>, 3>::new();
        assert_eq!(small_helps.as_pool().slice.len(), 3);

        // Medium CLI app (10 commands)
        let mut medium_helps = ArrayPool::<CmdHelp<'_>, 10>::new();
        assert_eq!(medium_helps.as_pool().slice.len(), 10);

        // Large CLI app (50 commands) - would use large_subcmds feature
        let mut large_helps = ArrayPool::<CmdHelp<'_>, 50>::new();
        assert_eq!(large_helps.as_pool().slice.len(), 50);
    }

    // Tests for public API usage by external users implementing Subcmds

    #[test]
    fn test_custom_subcmds_implementation_single() {
        // Test how external users would implement get_help_list for a single command
        let mut data: [MaybeUninit<CmdHelp<'_>>; 3] =
            [MaybeUninit::uninit(); 3];
        let pool = Pool::new(&mut data);

        // User implementation pattern: allocate space for 1 command
        let (mut parent, _children) = pool.alloc(1);

        // Create help for the command (user would call cmd.help(children))
        let cmd_help = CmdHelp::default();
        let result = parent.assign(&[cmd_help]);

        // This is what get_help_list would return
        assert_eq!(result.len(), 1);
        // In real implementation: Some(result)
    }

    #[test]
    fn test_custom_subcmds_implementation_multiple() {
        // Test how external users would implement get_help_list for multiple commands
        let mut data: [MaybeUninit<CmdHelp<'_>>; 10] =
            [MaybeUninit::uninit(); 10];
        let pool = Pool::new(&mut data);

        // User implementation pattern: allocate space for 3 commands
        let (mut parent, children) = pool.alloc(3);

        // Split children pool for each subcommand (if they have nested commands)
        let (cmd1_pool, remaining) = children.alloc(2);
        let (cmd2_pool, cmd3_pool) = remaining.alloc(2);

        // User would create help for each command
        let help1 = CmdHelp::default(); // cmd1.help(cmd1_pool)
        let help2 = CmdHelp::default(); // cmd2.help(cmd2_pool)
        let help3 = CmdHelp::default(); // cmd3.help(cmd3_pool)

        let result = parent.assign(&[help1, help2, help3]);

        assert_eq!(result.len(), 3);
        assert_eq!(cmd1_pool.slice.len(), 2);
        assert_eq!(cmd2_pool.slice.len(), 2);
        assert_eq!(cmd3_pool.slice.len(), 3);
    }

    #[test]
    fn test_custom_subcmds_empty_implementation() {
        // Test how external users would implement get_help_list for no commands
        let mut data: [MaybeUninit<CmdHelp<'_>>; 1] =
            [MaybeUninit::uninit(); 1];
        let pool = Pool::new(&mut data);

        // User implementation for empty subcmds would just ignore the pool
        // and return None, but let's test the pool still works
        let (mut empty_parent, _) = pool.alloc(0);
        let empty_slice: &[CmdHelp<'_>] = &[];
        let result = empty_parent.assign(empty_slice);

        assert_eq!(result.len(), 0);
        // In real implementation: None
    }

    #[test]
    fn test_pool_public_api_bounds_checking() {
        // Test that public API users get safe behavior with oversized allocations
        let mut data: [MaybeUninit<CmdHelp<'_>>; 2] =
            [MaybeUninit::uninit(); 2];
        let mut pool = Pool::new(&mut data);

        // User tries to assign more help entries than pool capacity
        let oversized_helps = [
            CmdHelp::default(),
            CmdHelp::default(),
            CmdHelp::default(),
            CmdHelp::default(),
        ];

        // Should silently truncate to pool capacity (documented behavior)
        let result = pool.assign(&oversized_helps);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_pool_public_api_zero_allocation() {
        // Test edge case where user allocates zero space
        let mut data: [MaybeUninit<CmdHelp<'_>>; 5] =
            [MaybeUninit::uninit(); 5];
        let pool = Pool::new(&mut data);

        let (zero_pool, remaining) = pool.alloc(0);
        assert_eq!(zero_pool.slice.len(), 0);
        assert_eq!(remaining.slice.len(), 5);

        // User can still use the remaining pool normally
        let (mut normal_pool, _) = remaining.alloc(2);
        let helps = [CmdHelp::default(), CmdHelp::default()];
        let result = normal_pool.assign(&helps);
        assert_eq!(result.len(), 2);
    }
}
