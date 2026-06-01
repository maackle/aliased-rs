//! Aliasing API backed by a process-wide static [`AliasContext`].
//!
//! No context to thread through call sites — the trait methods read and
//! write a single static registry obtained via [`global_ctx`]. Convenient
//! for application-level logging; use the [`crate::contextual`] flavor when
//! you need isolated contexts (e.g. per-test).

use std::any::TypeId;
use std::fmt;
use std::sync::LazyLock;

use crate::shared::{self, AliasContext};

static GLOBAL: LazyLock<AliasContext> = LazyLock::new(AliasContext::new);

/// The process-wide [`AliasContext`] used by the global API.
pub fn global_ctx() -> &'static AliasContext {
    &GLOBAL
}

/// Extension trait providing aliasing methods backed by the process-wide
/// [`global_ctx`]. Implemented for every `T: Debug + 'static`.
pub trait Aliasing: fmt::Debug + 'static {
    /// Wrap `self` so that formatting it applies the globally registered
    /// aliases.
    fn aliased(&self) -> Aliased<'_, Self> {
        Aliased { val: self }
    }

    /// Set a short prefix shown alongside every alias of this type
    /// (e.g. `⟪K|alice⟫`). Should be called at most once per type.
    fn alias_prefix(prefix: &str) {
        shared::set_prefix(global_ctx(), TypeId::of::<Self>(), prefix);
    }

    /// Assign `self` an auto-incrementing numeric alias scoped to its type.
    /// Returns `self` for chaining.
    fn alias_numbered(&self) -> &Self {
        shared::register_numbered(
            global_ctx(),
            TypeId::of::<Self>(),
            format!("{self:?}"),
            format!("{self:#?}"),
        );
        self
    }

    /// Assign `self` an explicit named alias. Returns `self` for chaining.
    fn alias_named(&self, name: &str) -> &Self {
        shared::register_named(
            global_ctx(),
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
/// inner value's `Debug` (or `{:#?}`) output and substitutes aliases drawn
/// from the global context.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Aliased<'v, T: ?Sized> {
    val: &'v T,
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Aliased<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        shared::fmt_aliased(self.val, global_ctx(), f)
    }
}
