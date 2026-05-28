use std::any::TypeId;
use std::collections::BTreeMap;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::OnceLock;

mod pretty;

use crate::pretty::pretty_replace;

pub type Name = String;

#[derive(Debug, Clone)]
struct Repr {
    name: Name,
    prefix: Option<String>,
}

pub type Prefix = String;
pub type AliasString = String;

impl std::fmt::Display for Repr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (open, close) = brackets();
        if let Some(prefix) = &self.prefix {
            write!(f, "{open}{prefix}|{}{close}", self.name)
        } else {
            write!(f, "{open}{}{close}", self.name)
        }
    }
}

/// The brackets used to indicate aliased values, set by [`set_brackets`].
static BRACKETS: OnceLock<(&'static str, &'static str)> = OnceLock::new();

/// The mapping of Debug values to their aliases.
static DEBUG_ALIASES: LazyLock<Mutex<BTreeMap<String, AliasString>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

/// The mapping of pretty-printed Debug values to their aliases.
static PRETTY_ALIASES: LazyLock<Mutex<BTreeMap<String, AliasString>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

static PREFIXES: LazyLock<Mutex<BTreeMap<TypeId, Prefix>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

/// Change the brackets used to indicate aliased values.
/// This can only be called once, and **panics** if called more than once.
pub fn set_aliased_brackets(brackets: (&'static str, &'static str)) {
    BRACKETS
        .set(brackets)
        .expect("Cannot set `aliased` brackets more than once");
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
            panic!("There is already a type with prefix `{prefix}`",);
        }

        if let Some(existing) = lock.insert(type_id, prefix.to_string()) {
            panic!("Cannot set prefix more than once: existing prefix is `{existing}`",);
        }
    }

    fn alias_named(&self, name: &str) -> &Self {
        let prefix = PREFIXES
            .lock()
            .unwrap()
            .get(&std::any::TypeId::of::<Self>())
            .cloned();
        let repr = Repr {
            name: name.to_string(),
            prefix,
        };

        DEBUG_ALIASES
            .lock()
            .unwrap()
            .insert(format!("{self:?}"), repr.to_string());
        PRETTY_ALIASES
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
        if f.alternate() {
            let lock = PRETTY_ALIASES.lock().unwrap();
            let mut debug = format!("{:#?}", self.0);
            for (key, repr) in lock.iter().rev() {
                debug = pretty_replace(&debug, key, &repr.to_string());
            }
            write!(f, "{}", debug)
        } else {
            let lock = DEBUG_ALIASES.lock().unwrap();
            let mut rep = format!("{:?}", self.0);
            for (key, repr) in lock.iter().rev() {
                rep = rep.replace(key, &repr.to_string());
            }
            write!(f, "{}", rep)
        }
    }
}
