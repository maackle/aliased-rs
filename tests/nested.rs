use aliased::AliasContext;
use aliased::contextual::*;

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
    let ctx = AliasContext::new();
    let s = fixture();

    PublicKey::alias_prefix(&ctx, "K");
    SecretKey::alias_prefix(&ctx, "S");

    s.u.keys[0].alias_named(&ctx, "key-a");
    s.u.keys[1].alias_named(&ctx, "key-b");
    s.u.keys[2].alias_named(&ctx, "key-c");
    s.u.secret.alias_named(&ctx, "secret");

    let d = format!("{:?}", s.aliased(&ctx));
    let p = format!("{:#?}", s.aliased(&ctx));

    assert_eq!(
        d,
        "MyStruct { u: UpstreamStruct { secret: ⟪S|secret⟫, keys: [⟪K|key-a⟫, ⟪K|key-b⟫, ⟪K|key-c⟫] } }"
    );
    assert_eq!(
        p,
        "
MyStruct {
    u: UpstreamStruct {
        secret: ⟪S|secret⟫,
        keys: [
            ⟪K|key-a⟫,
            ⟪K|key-b⟫,
            ⟪K|key-c⟫,
        ],
    },
}
"
        .trim()
    );
}

#[test]
fn test_changing_aliases() {
    let ctx = AliasContext::new();

    PublicKey::alias_prefix(&ctx, "K");
    SecretKey::alias_prefix(&ctx, "S");

    let keys = (0..3)
        .map(|i| *PublicKey([i; 32]).alias_numbered(&ctx))
        .collect::<Vec<_>>();
    let secrets = (0..3)
        .map(|i| *SecretKey([i; 32]).alias_numbered(&ctx))
        .collect::<Vec<_>>();

    assert_eq!(
        format!("{:?}", keys.aliased(&ctx)),
        "[⟪K|#000⟫, ⟪K|#001⟫, ⟪K|#002⟫]"
    );
    assert_eq!(
        format!("{:?}", secrets.aliased(&ctx)),
        "[⟪S|#000⟫, ⟪S|#001⟫, ⟪S|#002⟫]"
    );

    for (i, k) in keys.iter().enumerate() {
        k.alias_named(&ctx, &format!("key-{i}"));
    }

    assert_eq!(
        format!("{:?}", keys.aliased(&ctx)),
        "[⟪K|key-0⟫, ⟪K|key-1⟫, ⟪K|key-2⟫]"
    );
    assert_eq!(
        format!("{:?}", secrets.aliased(&ctx)),
        "[⟪S|#000⟫, ⟪S|#001⟫, ⟪S|#002⟫]"
    );
}

#[test]
fn test_references() {
    let ctx = AliasContext::new();

    #[derive(Debug)]
    struct S;

    S::alias_prefix(&ctx, "S");

    let s = S;
    s.alias_named(&ctx, "foo");
    (&s).alias_named(&ctx, "bar");

    assert_eq!(format!("{:?}", s.aliased(&ctx)), "⟪S|bar⟫");
    assert_eq!(format!("{:?}", (&s).aliased(&ctx)), "⟪S|bar⟫");
}
