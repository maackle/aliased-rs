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
//!   process-wide [`AliasContext`] obtained via [`contextual::global_ctx`]. No context
//!   to thread through call sites.
//! - **`contextual`**: [`contextual::Aliasing`] takes an explicit
//!   `&AliasContext`, so callers can keep isolated contexts (e.g. one per
//!   test).
//!
//! Either or both may be enabled while testing.
//! For release builds, it's intended that you use *neither* feature,
//! i.e. use `default-features = false`.
//!
//! When using no features, the whole API still compiles, but as a no-op:
//! `aliased(...)` formats with plain `Debug` and registration calls do nothing.
//! This lets a production build switch aliasing off with `default-features = false`
//! without touching call sites (and without pulling in `aho-corasick` / `regex`)
//! for a minimal footprint.
//!
//! # Example (global)
//!
//! ```rust
//! # #[cfg(feature = "global")]
//! # {
//! use aliased::*;
//!
//! #[derive(Debug)]
//! struct Key([u8; 32]);
//!
//! // All aliases for this type will be prefixed with "K|"
//! Key::alias_prefix("K");
//!
//! let a = Key([1; 32]);
//! let b = Key([2; 32]);
//! let c = Key([3; 32]);
//!
//! // The default Debug output is verbose.
//! assert_eq!(format!("{:?}", a), "Key([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1])");
//!
//! // Without applying any aliases, the `aliased()` output is unchanged.
//! assert_eq!(format!("{:?}", a.aliased()), "Key([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1])");
//!
//! // You can apply a numbered alias:
//!
//! a.alias_numbered();
//! b.alias_numbered();
//! c.alias_numbered();
//! assert_eq!(format!("{:?}", a.aliased()), "⟪K|#000⟫");
//! assert_eq!(format!("{:?}", b.aliased()), "⟪K|#001⟫");
//! assert_eq!(format!("{:?}", c.aliased()), "⟪K|#002⟫");
//!
//! // Or you can apply a named alias (even after already applying a numbered alias)
//!
//! a.alias_named("alice");
//! b.alias_named("bob");
//! c.alias_named("carol");
//! assert_eq!(format!("{:?}", a.aliased()), "⟪K|alice⟫");
//! assert_eq!(format!("{:?}", b.aliased()), "⟪K|bob⟫");
//! assert_eq!(format!("{:?}", c.aliased()), "⟪K|carol⟫");
//!
//! // Works for pretty-printed output too.
//! // Without `aliased()`, this output would be 34 lines long!.
//! assert_eq!(format!("{:#?}", a.aliased()), "⟪K|alice⟫");
//! # }
//! ```
//!
//! # Example (contextual)
//!
//! ```rust
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

// Substitution machinery (and its `aho-corasick` / `regex` deps) is compiled
// only when a flavor is enabled; the `noop` fallback below covers the rest.
#[cfg(any(feature = "global", feature = "contextual"))]
mod pretty;
#[cfg(any(feature = "global", feature = "contextual"))]
mod shared;
#[cfg(any(feature = "global", feature = "contextual"))]
pub use shared::AliasContext;

#[cfg(feature = "contextual")]
pub mod contextual;

#[cfg(feature = "global")]
mod global;
#[cfg(feature = "global")]
pub use global::{Aliased, Aliasing};

// No flavor enabled: expose both surfaces as dependency-free no-ops.
#[cfg(not(any(feature = "global", feature = "contextual")))]
mod noop;
#[cfg(not(any(feature = "global", feature = "contextual")))]
pub use noop::{AliasContext, Aliased, Aliasing, contextual};
