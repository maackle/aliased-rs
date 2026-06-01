//! No-op fallback compiled when neither `global` nor `contextual` is enabled.
//!
//! Mirrors both public surfaces — the crate-root global-shaped API and the
//! [`contextual`] submodule — but every method is a pass-through. [`Aliased`]
//! formats its inner value with plain `Debug` (no substitution, no registry),
//! and the substitution dependencies (`aho-corasick`, `regex`) are not
//! compiled. Lets a production build switch aliasing off with
//! `default-features = false` without removing any call sites.

use std::fmt;

/// No-op stand-in for the real registry. Holds nothing; every method is a
/// pass-through so `contextual` call sites keep compiling.
#[derive(Clone, Default)]
pub struct AliasContext;

impl AliasContext {
    /// Create a context. No-op: there is nothing to register into.
    pub fn new() -> Self {
        Self
    }

    /// No-op: brackets are only used by the (absent) substitution path.
    pub fn set_brackets(&self, _brackets: (&'static str, &'static str)) {}
}

/// No-op mirror of the global-flavor [`crate::Aliasing`] trait. Blanket-impl'd
/// for every `T: Debug + 'static`; all methods are pass-throughs.
pub trait Aliasing: fmt::Debug + 'static {
    /// Wrap `self`; formatting it forwards to plain `Debug`.
    fn aliased(&self) -> Aliased<'_, Self> {
        Aliased { val: self }
    }

    /// No-op.
    fn alias_prefix(_prefix: &str) {}

    /// No-op; returns `self` for chaining.
    fn alias_numbered(&self) -> &Self {
        self
    }

    /// No-op; returns `self` for chaining.
    fn alias_named(&self, _name: &str) -> &Self {
        self
    }
}

impl<T> Aliasing for T where T: fmt::Debug + 'static {}

/// No-op `Debug` wrapper. Formatting forwards to the inner value's `Debug`,
/// preserving the `{:#?}` alternate flag — only alias substitution is gone.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Aliased<'v, T: ?Sized> {
    val: &'v T,
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Aliased<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.val, f)
    }
}

/// No-op mirror of [`crate::contextual`]. Same signatures (the `&AliasContext`
/// argument is accepted and ignored), so contextual call sites keep compiling.
pub mod contextual {
    use std::fmt;

    pub use super::AliasContext;

    /// No-op mirror of the contextual-flavor `Aliasing` trait.
    pub trait Aliasing: fmt::Debug + 'static {
        /// Wrap `self`; formatting it forwards to plain `Debug`.
        fn aliased<'v, 'c>(&'v self, ctx: &'c AliasContext) -> Aliased<'v, 'c, Self> {
            Aliased { val: self, ctx }
        }

        /// No-op.
        fn alias_prefix(_ctx: &AliasContext, _prefix: &str) {}

        /// No-op; returns `self` for chaining.
        fn alias_numbered(&self, _ctx: &AliasContext) -> &Self {
            self
        }

        /// No-op; returns `self` for chaining.
        fn alias_named(&self, _ctx: &AliasContext, _name: &str) -> &Self {
            self
        }
    }

    impl<T> Aliasing for T where T: fmt::Debug + 'static {}

    /// No-op `Debug` wrapper. Carries the same three type parameters as the
    /// real contextual wrapper for API fidelity; the held context is ignored.
    #[derive(Clone)]
    pub struct Aliased<'v, 'c, T: ?Sized> {
        val: &'v T,
        ctx: &'c AliasContext,
    }

    impl<T: ?Sized + fmt::Debug> fmt::Debug for Aliased<'_, '_, T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let _ = self.ctx;
            fmt::Debug::fmt(self.val, f)
        }
    }

    impl<'v, 'c, T: ?Sized + Eq> Eq for Aliased<'v, 'c, T> {}

    impl<'v, 'c, T: ?Sized + Ord> Ord for Aliased<'v, 'c, T> {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.val.cmp(other.val)
        }
    }

    impl<'v, 'c, T: ?Sized + PartialOrd> PartialOrd for Aliased<'v, 'c, T> {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.val.partial_cmp(other.val)
        }
    }

    impl<'v, 'c, T: ?Sized + PartialEq> PartialEq for Aliased<'v, 'c, T> {
        fn eq(&self, other: &Self) -> bool {
            self.val.eq(other.val)
        }
    }

    impl<'v, 'c, T: ?Sized + std::hash::Hash> std::hash::Hash for Aliased<'v, 'c, T> {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.val.hash(state);
        }
    }
}
