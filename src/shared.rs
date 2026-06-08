use std::any::TypeId;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use aho_corasick::{AhoCorasick, MatchKind};

use crate::pretty::{combined_pretty_regex, pretty_pattern, pretty_replace_all};

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
    /// The type that registered this alias. Used to distinguish a genuine
    /// cross-type Debug-string collision from an intentional re-alias of the
    /// same value (e.g. numbered then named). Only read by the collision
    /// warning, so it is dead code when `tracing` is disabled.
    #[cfg_attr(not(feature = "tracing"), allow(dead_code))]
    type_id: TypeId,
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

/// Cached single-pass matcher for plain (`{:?}`) substitution. Derived from
/// `debug_names`; rebuilt lazily on the first print after any registration.
enum DebugMatcher {
    /// Needs (re)building from the current registry.
    Stale,
    /// Nothing registered — formatting is a pass-through.
    Empty,
    /// Built automaton plus replacements parallel to its patterns.
    Built {
        ac: AhoCorasick,
        replacements: Vec<String>,
    },
}

/// Cached single-pass matcher for pretty (`{:#?}`) substitution. Derived from
/// `pretty_names`; rebuilt lazily on the first print after any registration.
enum PrettyMatcher {
    Stale,
    Empty,
    Built {
        regex: regex::Regex,
        replacements: Vec<String>,
    },
}

pub(crate) struct AliasData {
    brackets: (&'static str, &'static str),
    numbers: BTreeMap<TypeId, BTreeMap<String, usize>>,
    debug_names: BTreeMap<String, Repr>,
    pretty_names: BTreeMap<String, Repr>,
    prefixes: BTreeMap<TypeId, String>,
    debug_matcher: DebugMatcher,
    pretty_matcher: PrettyMatcher,
}

impl Default for AliasData {
    fn default() -> Self {
        Self {
            brackets: ("⟪", "⟫"),
            numbers: BTreeMap::default(),
            debug_names: BTreeMap::default(),
            pretty_names: BTreeMap::default(),
            prefixes: BTreeMap::default(),
            debug_matcher: DebugMatcher::Stale,
            pretty_matcher: PrettyMatcher::Stale,
        }
    }
}

impl AliasData {
    /// Mark the cached matchers stale so the next print rebuilds them. Called
    /// after any change that affects substitution output (new alias, prefix,
    /// or brackets).
    fn invalidate(&mut self) {
        self.debug_matcher = DebugMatcher::Stale;
        self.pretty_matcher = PrettyMatcher::Stale;
    }

    /// Register `repr` under both its plain and pretty keys and invalidate the
    /// cached matchers.
    ///
    /// Detecting a collision is free: `BTreeMap::insert` already hands back any
    /// displaced value, so we only inspect what we already have. When the
    /// displaced alias belonged to a *different* type, two distinct types share
    /// a Debug string and cannot be aliased independently — the substitution
    /// runs on rendered text, which has lost all type information, so only the
    /// last writer survives. We warn rather than try to fix it (impossible with
    /// string substitution). A same-type displacement is an intentional
    /// re-alias (e.g. numbered then named) and stays quiet.
    fn insert_repr(&mut self, debug_key: String, pretty_key: String, repr: Repr) {
        let _prev_debug = self.debug_names.insert(debug_key, repr.clone());
        let _prev_pretty = self.pretty_names.insert(pretty_key, repr.clone());
        #[cfg(feature = "tracing")]
        {
            if let Some(existing) = _prev_debug
                && existing.type_id != repr.type_id
            {
                tracing::warn!(
                    "Alias collision: new alias `{repr}` overshadows `{existing}` because they both have the same Debug formatting"
                );
            }
            if let Some(existing) = _prev_pretty
                && existing.type_id != repr.type_id
            {
                tracing::warn!(
                    "Alias collision: new alias `{repr}` overshadows `{existing}` because they both have the same pretty (`{{:#?}}`) formatting"
                );
            }
        }
        self.invalidate();
    }

    /// Apply plain (`{:?}`) aliasing to `rep`, rebuilding the cached matcher if
    /// it went stale since the last print.
    fn aliased_debug(&mut self, rep: &str) -> String {
        if let DebugMatcher::Stale = self.debug_matcher {
            self.debug_matcher = if self.debug_names.is_empty() {
                DebugMatcher::Empty
            } else {
                let mut keys = Vec::with_capacity(self.debug_names.len());
                let mut replacements = Vec::with_capacity(self.debug_names.len());
                for (key, repr) in &self.debug_names {
                    keys.push(key.clone());
                    replacements.push(repr.to_string());
                }
                // LeftmostLongest makes the longest key win where several could
                // match at a position, matching the old longest-first loop.
                let ac = AhoCorasick::builder()
                    .match_kind(MatchKind::LeftmostLongest)
                    .build(&keys)
                    .expect("aho-corasick build from registered keys");
                DebugMatcher::Built { ac, replacements }
            };
        }
        match &self.debug_matcher {
            DebugMatcher::Built { ac, replacements } => ac.replace_all(rep, replacements),
            _ => rep.to_string(),
        }
    }

    /// Apply pretty (`{:#?}`) aliasing to `rep`, rebuilding the cached matcher
    /// if it went stale since the last print.
    fn aliased_pretty(&mut self, rep: &str) -> String {
        if let PrettyMatcher::Stale = self.pretty_matcher {
            self.pretty_matcher = if self.pretty_names.is_empty() {
                PrettyMatcher::Empty
            } else {
                // Longest keys first so a shorter key can't clobber one that
                // contains it; lex tiebreak for deterministic output.
                let mut entries: Vec<_> = self.pretty_names.iter().collect();
                entries.sort_by(|(a, _), (b, _)| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
                let mut patterns = Vec::with_capacity(entries.len());
                let mut replacements = Vec::with_capacity(entries.len());
                for (key, repr) in entries {
                    patterns.push(pretty_pattern(key));
                    replacements.push(repr.to_string());
                }
                let regex = combined_pretty_regex(&patterns);
                PrettyMatcher::Built {
                    regex,
                    replacements,
                }
            };
        }
        match &self.pretty_matcher {
            PrettyMatcher::Built {
                regex,
                replacements,
            } => pretty_replace_all(regex, rep, replacements),
            _ => rep.to_string(),
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
        let mut lock = self.lock();
        lock.brackets = brackets;
        lock.invalidate();
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
    lock.invalidate();
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
            tracing::debug!(
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
        type_id,
    };
    lock.insert_repr(debug_key, pretty_key, repr);
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
        type_id,
    };
    lock.insert_repr(debug_key, pretty_key, repr);
}

pub(crate) fn fmt_aliased<T: ?Sized + fmt::Debug>(
    val: &T,
    ctx: &AliasContext,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    let mut lock = ctx.lock();
    let rendered = if f.alternate() {
        let rep = format!("{:#?}", val);
        lock.aliased_pretty(&rep)
    } else {
        let rep = format!("{:?}", val);
        lock.aliased_debug(&rep)
    };
    write!(f, "{}", rendered)
}
