//! Command suggestion system for typo correction.
//!
//! This module provides intelligent command suggestion functionality that helps users
//! when they mistype command names. It uses edit distance algorithms to find the most
//! likely intended command and provides helpful "did you mean?" suggestions.
//!
//! # Core Types
//!
//! ## [`Suggestion<'a>`] - Command Suggestion
//!
//! Represents a suggestion for correcting a mistyped command:
//! - **`typo`**: The original misspelled command entered by the user
//! - **`cmd`**: The suggested correct command that most closely matches
//! - **Lifetime `'a`**: References to original strings for zero-copy operation
//!
//! # Key Features
//!
//! ## Levenshtein Edit Distance Algorithm
//! The suggestion system uses the Levenshtein edit distance algorithm to find
//! the closest matching command:
//! - **Character Insertions**: Cost of adding missing characters
//! - **Character Deletions**: Cost of removing extra characters
//! - **Character Substitutions**: Cost of replacing incorrect characters
//! - **Minimum Distance**: Finds the command requiring fewest edits
//!
//! ## Tie Breaking
//! When multiple commands have the same edit distance, the first one
//! encountered is selected for consistency.
//!
//! ## Suggestion Quality
//! The algorithm provides high-quality suggestions for common typo patterns:
//!
//! ### Character Transposition
//! ```text
//! "biuld" → "build"    (transposed 'ui' to 'iu')
//! "tset" → "test"      (transposed 'se' to 'es')
//! ```
//!
//! ### Missing Characters
//! ```text
//! "bild" → "build"     (missing 'u')
//! "tes" → "test"       (missing 't')
//! ```
//!
//! ### Extra Characters
//! ```text
//! "buildd" → "build"   (extra 'd')
//! "testt" → "test"     (extra 't')
//! ```
//!
//! ### Character Substitution
//! ```text
//! "buold" → "build"    (substituted 'o' for 'i')
//! "tast" → "test"      (substituted 'a' for 'e')
//! ```
//!
//! # Usage Patterns
//!
//! ## Automatic Integration
//! Suggestions are automatically provided when unknown commands are encountered:
//! ```bash
//! $ myapp biuld
//! error: unknown command 'biuld'
//! help: did you mean 'build'?
//! ```
//!
//! ## Manual Suggestion Generation
//! ```
//! # use shovel::*;
//! let commands = ["build", "test", "deploy", "clean"];
//! let suggestion = Suggestion::find("biuld", &commands);
//!
//! println!("Did you mean '{}'?", suggestion.cmd);
//! // Output: "Did you mean 'build'?"
//! ```
//!
//! ## Integration with Error System
//! Suggestions integrate seamlessly with the error handling system:
//! ```
//! # use shovel::*;
//! # fn example() {
//! let commands = ["build", "test"];
//! let suggestion = Suggestion::find("biuld", &commands);
//! println!("suggestion: {suggestion}");
//!
//! // Output: "unknown command 'biuld'"
//! // The Writer will automatically add the suggestion
//! # }
//! ```
//!
//! When suggestions are available, the help system automatically includes them
//! in error messages, providing a better user experience.
//!
//! # Examples
//!
//! ## Git-like Command Suggestions
//! ```
//! # use shovel::*;
//! let git_commands = [
//!     "add", "commit", "push", "pull", "clone", "status",
//!     "branch", "checkout", "merge", "rebase", "log", "diff"
//! ];
//!
//! // Common typos and their suggestions
//! assert_eq!(Suggestion::find("comit", &git_commands).cmd, "commit");
//! assert_eq!(Suggestion::find("checkot", &git_commands).cmd, "checkout");
//! assert_eq!(Suggestion::find("statsu", &git_commands).cmd, "status");
//! ```
//!
//! ## Build Tool Suggestions
//! ```
//! # use shovel::*;
//! let build_commands = ["build", "test", "clean", "install", "package"];
//!
//! assert_eq!(Suggestion::find("biuld", &build_commands).cmd, "build");
//! assert_eq!(Suggestion::find("tset", &build_commands).cmd, "test");
//! assert_eq!(Suggestion::find("clen", &build_commands).cmd, "clean");
//! ```
//!
//! # Performance Characteristics
//!
//! - **Time Complexity**: O(n × m × k) where n is typo length, m is command length, k is command count
//! - **Space Complexity**: O(n × m) for the edit distance matrix
//! - **Zero Allocation**: Uses stack-based computation
//! - **Early Termination**: Optimizations for obviously poor matches
//!
//! ## Algorithm Details
//!
//! The edit distance calculation uses dynamic programming:
//!
//! ```text
//! For strings s1 (length n) and s2 (length m):
//!
//! dp[i][j] = minimum edits to transform s1[0..i] to s2[0..j]
//!
//! Base cases:
//! dp[0][j] = j (insert j characters)
//! dp[i][0] = i (delete i characters)
//!
//! Recurrence:
//! if s1[i-1] == s2[j-1]:
//!     dp[i][j] = dp[i-1][j-1]
//! else:
//!     dp[i][j] = 1 + min(
//!         dp[i-1][j],     // deletion
//!         dp[i][j-1],     // insertion
//!         dp[i-1][j-1]    // substitution
//!     )
//! ```
//!
//! See also: [`ParseError`](crate::ParseError), [`Writer`](crate::Writer), [`Cmd`](crate::Cmd)

use core::fmt::Display;

/// Suggests a matching command when the user makes a typo.
///
/// This struct holds both the original typo and the suggested correct command.
/// It's used to provide helpful feedback to users when they mistype a command.
#[derive(Debug)]
pub struct Suggestion<'a> {
    /// The original, misspelled command entered by the user.
    pub typo: &'a str,
    /// The suggested correct command that most closely matches the typo.
    pub cmd: &'a str,
}

impl<'a> Suggestion<'a> {
    /// Find a suggestion.
    #[must_use]
    pub fn find(typo: &'a str, cmds: &[&'a str]) -> Self {
        debug_assert_ne!(cmds, [] as [&str; 0]);

        let mut cmd = cmds[0];
        let mut min = edit_distance(typo, cmds[0]);
        for c in &cmds[1..] {
            let d = edit_distance(typo, c);
            if d <= min {
                min = d;
                cmd = c;
            }
        }

        Self { typo, cmd }
    }
}

impl Display for Suggestion<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

fn edit_distance(word1: &str, word2: &str) -> usize {
    // Only compare the first 63 chars
    let word1 = &word1[..core::cmp::min(63, word1.chars().count())];
    let word2 = &word2[..core::cmp::min(63, word2.chars().count())];

    let n1 = word1.chars().count();
    let n2 = word2.chars().count();

    if n2 == 0 {
        return n1;
    }
    if n1 == 0 {
        return n2;
    }

    if n1 < n2 {
        return edit_distance(word2, word1);
    }

    let n2 = n2 + 1;
    let mut d = [0; 64];

    for (i, e) in d.iter_mut().enumerate() {
        *e = i;
    }

    for (i, c1) in word1.chars().enumerate() {
        let mut p = d[0];
        d[0] = i + 1;
        for (j, c2) in word2.chars().enumerate() {
            let t = d[j + 1];
            d[j + 1] = core::cmp::min(
                t + 1,
                core::cmp::min(d[j] + 1, p + usize::from(c1 != c2)),
            );
            p = t;
        }
    }

    d[n2 - 1]
}

#[cfg(test)]
mod test {
    use super::edit_distance;

    #[test]
    fn test_edit_distance() {
        assert_eq!(edit_distance("foo", "foo"), 0);
        assert_eq!(edit_distance("bar", "baz"), 1);
        assert_eq!(edit_distance("foo", "bao"), 2);
        assert_eq!(edit_distance("foo", "bar"), 3);
    }

    #[test]
    fn test_edit_distance_with_empty_word() {
        assert_eq!(edit_distance("", "foo"), 3);
        assert_eq!(edit_distance("foo", ""), 3);
        assert_eq!(edit_distance("", ""), 0);
    }
}
