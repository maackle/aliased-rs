use aliased::*;

use derive_more::derive::{Deref, Display, From};

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[derive(Clone, Copy, Debug, Display, From, Deref)]
#[display("0x{}", to_hex(_0))]
pub struct PublicKey([u8; 32]);

#[derive(Clone, Copy, Debug, Display, From, Deref)]
#[display("0x{}", to_hex(_0))]
pub struct SecretKey([u8; 32]);

#[derive(Clone, Debug)]
pub struct UpstreamStruct {
    secret: SecretKey,
    keys: Vec<PublicKey>,
}

#[derive(Clone, Debug)]
pub struct MyStruct {
    u: UpstreamStruct,
}

fn fixture() -> MyStruct {
    let keys = (0..3).map(|i| PublicKey([i; 32])).collect::<Vec<_>>();
    let sig = SecretKey([0; 32]);
    let upstream = UpstreamStruct { keys, secret: sig };
    MyStruct { u: upstream }
}

#[test]
fn test_nested_names() {
    reset();
    let s = fixture();

    set_aliased_brackets(("༼", "༽"));

    PublicKey::aliased_with_prefix("K");
    SecretKey::aliased_with_prefix("S");

    s.u.keys[0].alias_named("key-a");
    s.u.keys[1].alias_named("key-b");
    s.u.keys[2].alias_named("key-c");
    s.u.secret.alias_named("secret");

    let d = format!("{:?}", s.aliased());
    let p = format!("{:#?}", s.aliased());

    assert_eq!(
        d,
        "MyStruct { u: UpstreamStruct { secret: ༼S|secret༽, keys: [༼K|key-a༽, ༼K|key-b༽, ༼K|key-c༽] } }"
    );
    assert_eq!(
        p,
        "
MyStruct {
    u: UpstreamStruct {
        secret: ༼S|secret༽,
        keys: [
            ༼K|key-a༽,
            ༼K|key-b༽,
            ༼K|key-c༽,
        ],
    },
}
"
        .trim()
    );
}

#[test]
fn test_changing_aliases() {
    reset();

    PublicKey::aliased_with_prefix("K");
    SecretKey::aliased_with_prefix("S");

    let keys = (0..3)
        .map(|i| *PublicKey([i; 32]).alias_numbered())
        .collect::<Vec<_>>();
    let secrets = (0..3)
        .map(|i| *SecretKey([i; 32]).alias_numbered())
        .collect::<Vec<_>>();

    let k = format!("{:?}", keys);
    let s = format!("{:?}", secrets);

    println!("{k:?}");
}
