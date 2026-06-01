//! Aliasing API parameterized by an explicit [`AliasContext`].
//!
//! Use this flavor when you want isolated contexts — for example, one per
//! test, so registrations from one test cannot leak into another.

use std::any::TypeId;
use std::fmt;

use crate::shared::{self, AliasContext};

#[cfg(feature = "global")]
pub use crate::global::global_ctx;

/// Extension trait providing aliasing methods that operate on an explicit
/// [`AliasContext`]. Implemented for every `T: Debug + 'static`.
pub trait Aliasing: fmt::Debug + 'static {
    /// Wrap `self` so that formatting it applies the aliases registered in
    /// `ctx`.
    fn aliased<'v, 'c>(&'v self, ctx: &'c AliasContext) -> Aliased<'v, 'c, Self> {
        Aliased { val: self, ctx }
    }

    /// Set a short prefix shown alongside every alias of this type
    /// (e.g. `⟪K|alice⟫`). Should be called at most once per type.
    fn alias_prefix(ctx: &AliasContext, prefix: &str) {
        shared::set_prefix(ctx, TypeId::of::<Self>(), prefix);
    }

    /// Assign `self` an auto-incrementing numeric alias scoped to its type.
    /// Returns `self` for chaining.
    fn alias_numbered(&self, ctx: &AliasContext) -> &Self {
        shared::register_numbered(
            ctx,
            TypeId::of::<Self>(),
            format!("{self:?}"),
            format!("{self:#?}"),
        );
        self
    }

    /// Assign `self` an explicit named alias. Returns `self` for chaining.
    fn alias_named(&self, ctx: &AliasContext, name: &str) -> &Self {
        shared::register_named(
            ctx,
            TypeId::of::<Self>(),
            format!("{self:?}"),
            format!("{self:#?}"),
            name,
        );
        self
    }
}

impl<T> Aliasing for T where T: fmt::Debug + 'static {}

/// `Debug` wrapper produced by [`Aliasing::aliased`]. Formatting it runs the
/// inner value's `Debug` (or `{:#?}`) output and substitutes registered
/// aliases drawn from the context it was created with.
#[derive(Clone)]
pub struct Aliased<'v, 'c, T: ?Sized> {
    val: &'v T,
    ctx: &'c AliasContext,
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Aliased<'_, '_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        shared::fmt_aliased(self.val, self.ctx, f)
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
