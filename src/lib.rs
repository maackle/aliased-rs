//! Replace noisy `Debug` output with short, human-friendly aliases.
//!
//! Register aliases for specific values up front via
//! [`Aliasing::alias_named`] or [`Aliasing::alias_numbered`], then print
//! through [`Aliasing::aliased`]: the crate runs the value's `Debug` (or
//! `{:#?}`) output through string substitution and swaps each registered
//! value's debug representation for its alias.
//!
//! # Example
//!
//! ```
//! use aliased::*;
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
//! ```

use std::any::TypeId;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

mod pretty;

use crate::pretty::{pretty_regex, pretty_replace};

#[derive(Debug, Clone)]
enum Alias {
    Name(String),
    Number(usize),
}

impl std::fmt::Display for Alias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Alias::Name(name) => write!(f, "{}", name),
            Alias::Number(num) => write!(f, "#{:03}", num),
        }
    }
}

#[derive(Clone)]
struct Repr {
    alias: Alias,
    prefix: Option<String>,
    brackets: (&'static str, &'static str),
}

impl std::fmt::Display for Repr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (open, close) = self.brackets;
        if let Some(prefix) = &self.prefix {
            write!(f, "{open}{prefix}|{}{close}", self.alias)
        } else {
            write!(f, "{open}{}{close}", self.alias)
        }
    }
}

pub(crate) struct AliasData {
    brackets: (&'static str, &'static str),
    numbers: BTreeMap<TypeId, BTreeMap<String, usize>>,
    debug_names: BTreeMap<String, Repr>,
    pretty_names: BTreeMap<String, (Repr, regex::Regex)>,
    prefixes: BTreeMap<TypeId, String>,
}

impl Default for AliasData {
    fn default() -> Self {
        Self {
            brackets: ("⟪", "⟫"),
            numbers: BTreeMap::default(),
            debug_names: BTreeMap::default(),
            pretty_names: BTreeMap::default(),
            prefixes: BTreeMap::default(),
        }
    }
}

/// A registry of value-to-alias mappings.
///
/// Cheap to clone — all clones share the same underlying registry via
/// `Arc<Mutex<_>>`. Register aliases through the [`Aliasing`] trait, then
/// print values via [`Aliasing::aliased`].
#[derive(Clone)]
pub struct AliasContext(Arc<Mutex<AliasData>>);

impl AliasContext {
    /// Create a new, empty context with the default brackets `⟪…⟫`.
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(AliasData::default())))
    }

    /// Change the brackets used to wrap aliases in formatted output.
    ///
    /// Brackets are snapshotted into each alias at registration time, so this
    /// only affects aliases registered after the call.
    pub fn set_brackets(&self, brackets: (&'static str, &'static str)) {
        self.lock().brackets = brackets;
    }

    pub(crate) fn lock(&self) -> MutexGuard<'_, AliasData> {
        self.0.lock().expect("AliasContext mutex poisoned")
    }
}

impl Default for AliasContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait implemented for every `T: Debug + 'static`, providing the
/// alias registration methods and the [`Aliasing::aliased`] formatter.
pub trait Aliasing: std::fmt::Debug + 'static {
    /// Wrap `self` in an [`Aliased`] so that formatting it applies the
    /// aliases registered in `ctx`.
    fn aliased<'v, 'c>(&'v self, ctx: &'c AliasContext) -> Aliased<'v, 'c, Self> {
        Aliased { val: self, ctx }
    }

    /// Set a short prefix (e.g. `"K"`) shown alongside every alias of this
    /// type, like `⟪K|alice⟫`.
    ///
    /// Should be called at most once per type. Repeat calls are ignored, and
    /// warn if the `tracing` feature is enabled.
    fn alias_prefix(ctx: &AliasContext, prefix: &str) {
        let type_id = TypeId::of::<Self>();
        let mut lock = ctx.lock();

        if lock.prefixes.values().any(|v| v == prefix) {
            #[cfg(feature = "tracing")]
            tracing::warn!("There is already a type with prefix `{prefix}`");
        }

        if let Some(existing) = lock.prefixes.insert(type_id, prefix.to_string()) {
            #[cfg(feature = "tracing")]
            tracing::warn!(
                "Cannot set prefix more than once: existing prefix is `{existing}`"
            );
        }
    }

    /// Assign `self` an auto-incrementing numeric alias scoped to its type
    /// (e.g. `#000`, `#001`, …). Returns `self` for chaining.
    ///
    /// Calling this on an already-numbered value is a no-op, and warns if
    /// the `tracing` feature is enabled.
    fn alias_numbered(&self, ctx: &AliasContext) -> &Self {
        let type_id = TypeId::of::<Self>();
        let mut lock = ctx.lock();
        let counter = lock.numbers.entry(type_id).or_default();
        let number = counter.len();
        let debug_key = format!("{self:?}");
        match counter.entry(debug_key.clone()) {
            std::collections::btree_map::Entry::Occupied(_e) => {
                #[cfg(feature = "tracing")]
                tracing::warn!(
                    "Cannot alias_numbered more than once: existing alias is `{}`",
                    _e.get(),
                );
                return self;
            }
            std::collections::btree_map::Entry::Vacant(e) => {
                e.insert(number);
            }
        }

        let prefix = lock.prefixes.get(&type_id).cloned();
        let repr = Repr {
            alias: Alias::Number(number),
            prefix,
            brackets: lock.brackets,
        };

        let pretty_key = format!("{self:#?}");
        let regex = pretty_regex(&pretty_key);
        lock.debug_names.insert(debug_key, repr.clone());
        lock.pretty_names.insert(pretty_key, (repr, regex));
        self
    }

    /// Assign `self` an explicit named alias. Returns `self` for chaining.
    ///
    /// Re-registering the same value overwrites its prior alias; collisions
    /// warn if the `tracing` feature is enabled.
    fn alias_named(&self, ctx: &AliasContext, name: &str) -> &Self {
        let type_id = TypeId::of::<Self>();
        let mut lock = ctx.lock();
        let prefix = lock.prefixes.get(&type_id).cloned();
        let repr = Repr {
            alias: Alias::Name(name.to_string()),
            prefix,
            brackets: lock.brackets,
        };

        let debug_key = format!("{self:?}");
        let pretty_key = format!("{self:#?}");
        let regex = pretty_regex(&pretty_key);

        if let Some(existing) = lock.debug_names.insert(debug_key, repr.clone()) {
            #[cfg(feature = "tracing")]
            tracing::warn!("alias name collision (debug): {} vs {}", existing, repr);
        }

        if let Some((existing, _)) =
            lock.pretty_names.insert(pretty_key, (repr.clone(), regex))
        {
            #[cfg(feature = "tracing")]
            tracing::warn!("alias name collision (pretty): {} vs {}", existing, repr);
        }

        self
    }
}

impl<T> Aliasing for T where T: std::fmt::Debug + 'static {}

/// `Debug` wrapper produced by [`Aliasing::aliased`]. Formatting it runs the
/// inner value's `Debug` (or `{:#?}`) output and substitutes registered
/// aliases.
pub struct Aliased<'v, 'c, T: ?Sized> {
    val: &'v T,
    ctx: &'c AliasContext,
}

impl<T: ?Sized + std::fmt::Debug> std::fmt::Debug for Aliased<'_, '_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lock = self.ctx.lock();
        let mut rep;
        if f.alternate() {
            rep = format!("{:#?}", self.val);
            let mut entries: Vec<_> = lock.pretty_names.iter().collect();
            // Longest keys first so a shorter key can't clobber one that
            // contains it; lexicographic tiebreak for deterministic output.
            entries.sort_by(|(a, _), (b, _)| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
            for (_key, (repr, regex)) in entries {
                rep = pretty_replace(regex, &rep, &repr.to_string());
            }
            write!(f, "{}", rep)
        } else {
            rep = format!("{:?}", self.val);
            let mut entries: Vec<_> = lock.debug_names.iter().collect();
            entries.sort_by(|(a, _), (b, _)| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
            for (key, repr) in entries {
                rep = rep.replace(key, &repr.to_string());
            }
            write!(f, "{}", rep)
        }
    }
}
