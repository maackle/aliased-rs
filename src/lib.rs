use std::collections::BTreeMap;
use std::sync::LazyLock;
use std::sync::Mutex;

mod pretty;

use crate::pretty::pretty_replace;

/// (debug string, is it pretty)
pub type Name = String;

static DEBUG_NAMES: LazyLock<Mutex<BTreeMap<String, Name>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));
static PRETTY_NAMES: LazyLock<Mutex<BTreeMap<String, Name>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

pub trait Aliasing: std::fmt::Debug {
    fn aliased(&self) -> Aliased<'_, Self> {
        Aliased(self)
    }

    fn named(&self, name: &str) -> &Self {
        DEBUG_NAMES
            .lock()
            .unwrap()
            .insert(format!("{self:?}"), name.to_string());
        PRETTY_NAMES
            .lock()
            .unwrap()
            .insert(format!("{self:#?}"), name.to_string());
        self
    }
}

impl<T> Aliasing for T where T: std::fmt::Debug {}

pub struct Aliased<'a, T: ?Sized>(pub &'a T);

impl<T: ?Sized + std::fmt::Debug> std::fmt::Debug for Aliased<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            let lock = PRETTY_NAMES.lock().unwrap();
            let mut rep = format!("{:#?}", self.0);
            for (key, name) in lock.iter().rev() {
                rep = pretty_replace(&rep, key, name);
            }
            write!(f, "{}", rep)
        } else {
            let lock = DEBUG_NAMES.lock().unwrap();
            let mut rep = format!("{:?}", self.0);
            for (key, name) in lock.iter().rev() {
                rep = rep.replace(key, name);
            }
            write!(f, "{}", rep)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use derive_more::derive::{Deref, Display, From};

    fn to_hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    #[derive(Clone, Copy, Debug, Display, From, Deref)]
    #[display("0x{}", to_hex(_0))]
    pub struct UpstreamKey([u8; 32]);

    #[derive(Clone, Copy, Debug, Display, From, Deref)]
    #[display("0x{}", to_hex(_0))]
    pub struct UpstreamSignature([u8; 64]);

    #[derive(Clone, Copy, Debug, Display)]
    #[display("upstream(sig={sig}, key={key})")]
    pub struct UpstreamStruct {
        sig: UpstreamSignature,
        key: UpstreamKey,
    }

    #[derive(Clone, Copy, Debug, Display)]
    #[display("u: {u}")]
    pub struct MyStruct {
        u: UpstreamStruct,
    }

    #[test]
    fn test_any_next() {
        let key = UpstreamKey([1; 32]);
        let sig = UpstreamSignature([2; 64]);
        let upstream = UpstreamStruct { key, sig };
        let my_struct = MyStruct { u: upstream };

        key.named("key");
        sig.named("sig");

        println!("{my_struct}");
        println!("{:?}", my_struct.aliased());
        println!("{:#?}", my_struct.aliased());
    }
}
