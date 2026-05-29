//! Replace noisy `Debug` output with short, human-friendly aliases.
//!
//! Register aliases for specific values up front, then format with
//! `aliased(...)`: the crate runs the value's `Debug` (or `{:#?}`) output
//! through string substitution and swaps each registered value's debug
//! representation for its alias.
//!
//! # Flavors
//!
//! Two flavors are available, selected by Cargo features:
//!
//! - **`global`** (default): the trait methods at [`Aliasing`] use a
//!   process-wide [`AliasContext`] obtained via [`global_ctx`]. No context
//!   to thread through call sites.
//! - **`contextual`**: [`contextual::Aliasing`] takes an explicit
//!   `&AliasContext`, so callers can keep isolated contexts (e.g. one per
//!   test).
//!
//! Either, both, or neither feature combination is up to the user — but at
//! least one of the two must be enabled, or this crate fails to compile.
//!
//! # Example (global)
//!
//! ```
//! # #[cfg(feature = "global")]
//! # {
//! use aliased::*;
//!
//! #[derive(Debug)]
//! struct Key([u8; 32]);
//!
//! Key::alias_prefix("K");
//!
//! let a = Key([1; 32]);
//! a.alias_named("alice");
//!
//! assert_eq!(format!("{:?}", a.aliased()), "⟪K|alice⟫");
//! # }
//! ```
//!
//! # Example (contextual)
//!
//! ```
//! # #[cfg(feature = "contextual")]
//! # {
//! use aliased::AliasContext;
//! use aliased::contextual::*;
//!
//! #[derive(Debug)]
//! struct Key([u8; 32]);
//!
//! let ctx = AliasContext::new();
//! Key::alias_prefix(&ctx, "K");
//!
//! let a = Key([1; 32]);
//! a.alias_named(&ctx, "alice");
//!
//! assert_eq!(format!("{:?}", a.aliased(&ctx)), "⟪K|alice⟫");
//! # }
//! ```

#[cfg(not(any(feature = "global", feature = "contextual")))]
compile_error!(
    "the `aliased` crate requires at least one of the `global` or `contextual` features to be enabled"
);

mod pretty;
mod shared;

pub use shared::AliasContext;

#[cfg(feature = "contextual")]
pub mod contextual;

#[cfg(feature = "global")]
mod global;
#[cfg(feature = "global")]
pub use global::{Aliased, Aliasing};
