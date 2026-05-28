use std::any::TypeId;
use std::collections::BTreeMap;
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

#[derive(Debug, Clone)]
struct Repr {
    alias: Alias,
    prefix: Option<String>,
}

pub type Prefix = String;
pub type AliasString = String;

impl std::fmt::Display for Repr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (open, close) = brackets();
        if let Some(prefix) = &self.prefix {
            write!(f, "{open}{prefix}|{}{close}", self.alias)
        } else {
            write!(f, "{open}{}{close}", self.alias)
        }
    }
}

/// The brackets used to indicate aliased values, set by [`set_brackets`].
static BRACKETS: OnceLock<(&'static str, &'static str)> = OnceLock::new();

/// The mapping of Debug values to their last numbering, grouped by type.
static NUMBERS: LazyLock<Mutex<BTreeMap<TypeId, BTreeMap<String, usize>>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

/// The mapping of Debug values to their aliases.
static DEBUG_NAMES: LazyLock<Mutex<BTreeMap<String, AliasString>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

/// The mapping of pretty-printed Debug values to their aliases.
static PRETTY_NAMES: LazyLock<Mutex<BTreeMap<String, AliasString>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

static PREFIXES: LazyLock<Mutex<BTreeMap<TypeId, Prefix>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

pub fn dump() {
    let numbers = NUMBERS.lock().unwrap();
    let debug_names = DEBUG_NAMES.lock().unwrap();
    let pretty_names = PRETTY_NAMES.lock().unwrap();
    let prefixes = PREFIXES.lock().unwrap();
    println!("NUMBERS: {numbers:#?}");
    println!("DEBUG_NAMES: {debug_names:#?}");
    // println!("PRETTY_NAMES: {pretty_names:#?}");
    println!("PREFIXES: {prefixes:#?}");
}

/// Change the brackets used to indicate aliased values.
/// This can only be called once, and **panics** if called more than once.
pub fn set_aliased_brackets(brackets: (&'static str, &'static str)) {
    BRACKETS
        .set(brackets)
        .expect("Cannot set `aliased` brackets more than once");
}

/// For testing, forget all aliasing data
pub fn reset() {
    NUMBERS.lock().unwrap().clear();
    DEBUG_NAMES.lock().unwrap().clear();
    PRETTY_NAMES.lock().unwrap().clear();
    PREFIXES.lock().unwrap().clear();
}

fn brackets() -> (&'static str, &'static str) {
    *BRACKETS.get_or_init(|| ("⟪", "⟫"))
}

pub trait Aliasing: std::fmt::Debug + 'static {
    fn aliased(&self) -> Aliased<'_, Self> {
        Aliased(self)
    }

    /// Sets the prefix for aliased values of this type
    fn aliased_with_prefix(prefix: &str) {
        let type_id = std::any::TypeId::of::<Self>();
        let mut lock = PREFIXES.lock().unwrap();

        if lock.values().find(|v| *v == prefix).is_some() {
            #[cfg(feature = "tracing")]
            tracing::warn!("There is already a type with prefix `{prefix}`",);
        }

        if let Some(existing) = lock.insert(type_id, prefix.to_string()) {
            #[cfg(feature = "tracing")]
            tracing::warn!("Cannot set prefix more than once: existing prefix is `{existing}`",);
        }
    }

    fn alias_numbered(&self) -> &Self {
        let type_id = std::any::TypeId::of::<Self>();
        let mut lock = NUMBERS.lock().unwrap();
        let counter = lock.entry(type_id).or_default();
        let entry = counter.entry(format!("{self:?}")).or_insert(0);
        let number = *entry;
        *entry += 1;

        let prefix = PREFIXES
            .lock()
            .unwrap()
            .get(&std::any::TypeId::of::<Self>())
            .cloned();
        let repr = Repr {
            alias: Alias::Number(number),
            prefix,
        };

        DEBUG_NAMES
            .lock()
            .unwrap()
            .insert(format!("{self:?}"), repr.to_string());
        PRETTY_NAMES
            .lock()
            .unwrap()
            .insert(format!("{self:#?}"), repr.to_string());
        self
    }

    fn alias_named(&self, name: &str) -> &Self {
        let prefix = PREFIXES
            .lock()
            .unwrap()
            .get(&std::any::TypeId::of::<Self>())
            .cloned();
        let repr = Repr {
            alias: Alias::Name(name.to_string()),
            prefix,
        };

        DEBUG_NAMES
            .lock()
            .unwrap()
            .insert(format!("{self:?}"), repr.to_string());
        PRETTY_NAMES
            .lock()
            .unwrap()
            .insert(format!("{self:#?}"), repr.to_string());
        self
    }
}

impl<T> Aliasing for T where T: std::fmt::Debug + 'static {}

pub struct Aliased<'a, T: ?Sized>(pub &'a T);

impl<T: ?Sized + std::fmt::Debug> std::fmt::Debug for Aliased<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rep;
        if f.alternate() {
            let lock = PRETTY_NAMES.lock().unwrap();
            rep = format!("{:#?}", self.0);
            for (key, repr) in lock.iter().rev() {
                rep = pretty_replace(&rep, key, &repr.to_string());
            }
            write!(f, "{}", rep)
        } else {
            let lock = DEBUG_NAMES.lock().unwrap();
            rep = format!("{:?}", self.0);
            for (key, repr) in lock.iter().rev() {
                rep = rep.replace(key, &repr.to_string());
            }
            write!(f, "{}", rep)
        }
    }
}
