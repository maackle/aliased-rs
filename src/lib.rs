use std::any::TypeId;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::OnceLock;

mod pretty;

use crate::pretty::pretty_replace;

#[derive(Debug, Clone)]
pub enum Alias {
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

pub type Prefix = String;
pub type AliasString = String;

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

pub struct AliasData {
    /// The brackets used to indicate aliased values, set by [`set_brackets`].
    brackets: (&'static str, &'static str),

    /// The mapping of Debug values to their last numbering, grouped by type.
    numbers: BTreeMap<TypeId, BTreeMap<String, usize>>,

    /// The mapping of Debug values to their aliases.
    debug_names: BTreeMap<String, Repr>,

    /// The mapping of pretty-printed Debug values to their aliases.
    pretty_names: BTreeMap<String, Repr>,

    prefixes: BTreeMap<TypeId, Prefix>,
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

#[derive(Clone)]
pub struct AliasContext(Arc<Mutex<AliasData>>);

impl std::ops::Deref for AliasContext {
    type Target = Mutex<AliasData>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AliasContext {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(AliasData::default())))
    }

    pub fn set_brackets(&mut self, brackets: (&'static str, &'static str)) {
        self.0.lock().unwrap().brackets = brackets;
    }
}

pub trait Aliasing: std::fmt::Debug + 'static {
    fn aliased<'v, 'c>(&'v self, ctx: &'c AliasContext) -> Aliased<'v, 'c, Self> {
        Aliased { val: self, ctx }
    }

    /// Sets the prefix for aliased values of this type
    fn alias_prefix(ctx: &AliasContext, prefix: &str) {
        let type_id = std::any::TypeId::of::<Self>();
        let mut lock = ctx.lock().unwrap();

        if lock.prefixes.values().find(|v| *v == prefix).is_some() {
            #[cfg(feature = "tracing")]
            tracing::warn!("There is already a type with prefix `{prefix}`",);
        }

        if let Some(existing) = lock.prefixes.insert(type_id, prefix.to_string()) {
            #[cfg(feature = "tracing")]
            tracing::warn!("Cannot set prefix more than once: existing prefix is `{existing}`",);
        }
    }

    fn alias_numbered(&self, ctx: &AliasContext) -> &Self {
        let type_id = std::any::TypeId::of::<Self>();
        let mut lock = ctx.lock().unwrap();
        let counter = lock.numbers.entry(type_id).or_default();
        let number = counter.len();
        let entry = counter.entry(format!("{self:?}"));
        match entry {
            std::collections::btree_map::Entry::Occupied(_e) => {
                #[cfg(feature = "tracing")]
                tracing::warn!(
                    "Cannot alias_numbered more than once: existing alias is `{}`",
                    _e.get(),
                );
                return self;
            }
            std::collections::btree_map::Entry::Vacant(e) => e.insert(number),
        };

        let prefix = lock.prefixes.get(&std::any::TypeId::of::<Self>()).cloned();
        let repr = Repr {
            alias: Alias::Number(number),
            prefix,
            brackets: lock.brackets,
        };

        lock.debug_names.insert(format!("{self:?}"), repr.clone());
        lock.pretty_names.insert(format!("{self:#?}"), repr.clone());
        self
    }

    fn alias_named(&self, ctx: &AliasContext, name: &str) -> &Self {
        let mut lock = ctx.lock().unwrap();
        let prefix = lock.prefixes.get(&std::any::TypeId::of::<Self>()).cloned();
        let repr = Repr {
            alias: Alias::Name(name.to_string()),
            prefix,
            brackets: lock.brackets,
        };

        if let Some(existing) = lock.debug_names.insert(format!("{self:?}"), repr.clone()) {
            #[cfg(feature = "tracing")]
            tracing::warn!("alias name collision (debug): {} vs {}", existing, repr,);
        }

        if let Some(existing) = lock.pretty_names.insert(format!("{self:#?}"), repr.clone()) {
            #[cfg(feature = "tracing")]
            tracing::warn!("alias name collision (pretty): {} vs {}", existing, repr,);
        }

        self
    }
}

impl<T> Aliasing for T where T: std::fmt::Debug + 'static {}

pub struct Aliased<'v, 'c, T: ?Sized> {
    val: &'v T,
    ctx: &'c AliasContext,
}

impl<T: ?Sized + std::fmt::Debug> std::fmt::Debug for Aliased<'_, '_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rep;
        let lock = self.ctx.lock().unwrap();
        if f.alternate() {
            rep = format!("{:#?}", self.val);
            for (key, repr) in lock.pretty_names.iter().rev() {
                rep = pretty_replace(&rep, key, &repr.to_string());
            }
            write!(f, "{}", rep)
        } else {
            rep = format!("{:?}", self.val);
            for (key, repr) in lock.debug_names.iter().rev() {
                rep = rep.replace(key, &repr.to_string());
            }
            write!(f, "{}", rep)
        }
    }
}
