use std::any::TypeId;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use crate::pretty::{pretty_regex, pretty_replace};

#[derive(Debug, Clone)]
pub(crate) enum Alias {
    Name(String),
    Number(usize),
}

impl fmt::Display for Alias {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Alias::Name(name) => write!(f, "{}", name),
            Alias::Number(num) => write!(f, "#{:03}", num),
        }
    }
}

#[derive(Clone)]
pub(crate) struct Repr {
    alias: Alias,
    prefix: Option<String>,
    brackets: (&'static str, &'static str),
}

impl fmt::Display for Repr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
/// `Arc<Mutex<_>>`.
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

pub(crate) fn set_prefix(ctx: &AliasContext, type_id: TypeId, prefix: &str) {
    let mut lock = ctx.lock();
    if lock.prefixes.values().any(|v| v == prefix) {
        #[cfg(feature = "tracing")]
        tracing::warn!("There is already a type with prefix `{prefix}`");
    }
    if let Some(_existing) = lock.prefixes.insert(type_id, prefix.to_string()) {
        #[cfg(feature = "tracing")]
        if _existing != prefix {
            tracing::warn!("Prefix collision: overwriting `{_existing}` with `{prefix}`");
        }
    }
}

pub(crate) fn register_numbered(
    ctx: &AliasContext,
    type_id: TypeId,
    debug_key: String,
    pretty_key: String,
) {
    let mut lock = ctx.lock();
    let counter = lock.numbers.entry(type_id).or_default();
    let number = counter.len();
    match counter.entry(debug_key.clone()) {
        std::collections::btree_map::Entry::Occupied(_e) => {
            #[cfg(feature = "tracing")]
            tracing::warn!(
                "Cannot alias_numbered more than once: existing alias is `{}`",
                _e.get(),
            );
            return;
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
    let regex = pretty_regex(&pretty_key);
    lock.debug_names.insert(debug_key, repr.clone());
    lock.pretty_names.insert(pretty_key, (repr, regex));
}

pub(crate) fn register_named(
    ctx: &AliasContext,
    type_id: TypeId,
    debug_key: String,
    pretty_key: String,
    name: &str,
) {
    let mut lock = ctx.lock();
    let prefix = lock.prefixes.get(&type_id).cloned();
    let repr = Repr {
        alias: Alias::Name(name.to_string()),
        prefix,
        brackets: lock.brackets,
    };
    let regex = pretty_regex(&pretty_key);

    if let Some(_existing) = lock.debug_names.insert(debug_key, repr.clone()) {
        #[cfg(feature = "tracing")]
        tracing::warn!("alias name collision (debug): {} vs {}", _existing, repr);
    }
    if let Some((_existing, _)) = lock.pretty_names.insert(pretty_key, (repr.clone(), regex)) {
        #[cfg(feature = "tracing")]
        tracing::warn!("alias name collision (pretty): {} vs {}", _existing, repr);
    }
}

pub(crate) fn fmt_aliased<T: ?Sized + fmt::Debug>(
    val: &T,
    ctx: &AliasContext,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    let lock = ctx.lock();
    if f.alternate() {
        let mut rep = format!("{:#?}", val);
        let mut entries: Vec<_> = lock.pretty_names.iter().collect();
        // Longest keys first so a shorter key can't clobber one that contains
        // it; lex tiebreak for deterministic output.
        entries.sort_by(|(a, _), (b, _)| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
        for (_key, (repr, regex)) in entries {
            rep = pretty_replace(regex, &rep, &repr.to_string());
        }
        write!(f, "{}", rep)
    } else {
        let mut rep = format!("{:?}", val);
        let mut entries: Vec<_> = lock.debug_names.iter().collect();
        entries.sort_by(|(a, _), (b, _)| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
        for (key, repr) in entries {
            rep = rep.replace(key, &repr.to_string());
        }
        write!(f, "{}", rep)
    }
}
