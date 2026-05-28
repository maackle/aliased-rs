use aliased::*;

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

    set_aliased_brackets(("༼", "༽"));

    UpstreamKey::aliased_with_prefix("K");
    UpstreamSignature::aliased_with_prefix("S");

    key.alias_named("key");
    sig.alias_named("sig");

    println!("{my_struct}");
    let d = format!("{:?}", my_struct.aliased());
    let p = format!("{:#?}", my_struct.aliased());

    assert_eq!(
        d,
        "MyStruct { u: UpstreamStruct { sig: ༼S|sig༽, key: ༼K|key༽ } }"
    );
    assert_eq!(
        p,
        "
MyStruct {
    u: UpstreamStruct {
        sig: ༼S|sig༽,
        key: ༼K|key༽,
    },
}"
        .trim()
    );
}
