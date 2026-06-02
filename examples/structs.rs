use std::collections::BTreeMap;

#[allow(unused)]
fn main() {
    use aliased::*;

    #[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
    struct ContentHash([u8; 32]);

    #[derive(Clone, Debug)]
    struct ContentBlob(Vec<u8>);

    #[derive(Clone, Debug)]
    struct Uuid(String);

    ContentHash::alias_prefix("H");
    ContentBlob::alias_prefix("Blob");
    Uuid::alias_prefix("UUID");

    let hash_a = ContentHash([1; 32]);
    let hash_b = ContentHash([2; 32]);
    let hash_c = ContentHash([3; 32]);

    let blob_a = ContentBlob(vec![4; 256]);
    let blob_b = ContentBlob(vec![5; 256]);
    let blob_c = ContentBlob(vec![6; 256]);

    let uuid1 = "123e4567-e89b-12d3-a456-426614174000".to_string();
    let uuid2 = "123e4567-e89b-12d3-a456-426614174001".to_string();
    let d = Uuid(uuid1.clone());
    let e = Uuid(uuid2.clone());

    #[derive(Debug)]
    struct ComplexShape {
        uuids: Vec<Uuid>,
        content: BTreeMap<ContentHash, ContentBlob>,
        description: String,
    }

    let shape = ComplexShape {
        uuids: vec![d.clone(), e.clone()],
        content: BTreeMap::from([
            (hash_a.clone(), blob_a.clone()),
            (hash_b.clone(), blob_b.clone()),
            (hash_c.clone(), blob_c.clone()),
        ]),
        description: format!(
            "Here's some text that gets untouched and even contains one of the aliased UUIDs ({uuid1})"
        ),
    };

    println!("Without aliases, this is thousands of lines long!");
    println!("```");
    dbg!(shape.aliased());
    println!("```");

    println!("");
    println!("");
    println!("You probably can't even find the beginning of the output above.");

    hash_a.alias_named("a");
    hash_b.alias_named("b");
    hash_c.alias_named("c");
    blob_a.alias_named("a");
    blob_b.alias_named("b");
    blob_c.alias_named("c");
    d.alias_numbered();
    e.alias_numbered();

    println!("However, with aliases, this is a very readable 12 lines:");
    println!("");
    println!("```");
    dbg!(shape.aliased());
    println!("```");

    pretty_assertions::assert_eq!(format!("{:#?}", shape.aliased()), 
r#"
ComplexShape {
    uuids: [
        ⟪UUID|#000⟫,
        ⟪UUID|#001⟫,
    ],
    content: {
        ⟪H|a⟫: ⟪Blob|a⟫,
        ⟪H|b⟫: ⟪Blob|b⟫,
        ⟪H|c⟫: ⟪Blob|c⟫,
    },
    description: "Here's some text that gets untouched and even contains one of the aliased UUIDs (123e4567-e89b-12d3-a456-426614174000)",
}
"#.trim()
);
}
