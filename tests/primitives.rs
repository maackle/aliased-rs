use tracing_test::traced_test;

#[test]
fn test_arrays() {
    use aliased::*;

    // Not such a great idea, but this can be done!
    <[u8; 16]>::alias_prefix("u8-16");
    <[u16; 8]>::alias_prefix("u16-8");
    <[u8; 32]>::alias_prefix("u8-32");

    let a = [1u8; 16].alias_numbered();
    let b = [1u16; 8].alias_numbered();
    let c = [1u8; 32].alias_numbered();

    assert_eq!(
        format!("{:?}", (a, b, c).aliased()),
        "(⟪u8-16|#000⟫, ⟪u16-8|#000⟫, ⟪u8-32|#000⟫)"
    );

    a.alias_named("a");
    b.alias_named("b");
    c.alias_named("c");

    assert_eq!(
        format!("{:?}", (a, b, c).aliased()),
        "(⟪u8-16|a⟫, ⟪u16-8|b⟫, ⟪u8-32|c⟫)"
    );
}

// Two distinct types that share a Debug representation cannot be aliased
// independently — the substitution operates on rendered text, which has lost
// all type information, so `debug_names` can only hold one `Repr` per string.
// We can't fix that, but we should at least warn about the collision.
#[traced_test]
#[test]
fn test_tuples_collision_warns() {
    use aliased::*;

    <(u8, u8)>::alias_prefix("8x2");
    <(u16, u16)>::alias_prefix("16x2");

    // Both Debug-format to the literal "(1, 2)" despite being different types.
    let _a = (1u8, 2u8).alias_numbered();
    let _b = (1u16, 2u16).alias_numbered();

    assert!(logs_contain("collision"));
}

// Re-aliasing the *same* value (numbered -> named) is intentional, not a
// collision, and must stay quiet.
#[traced_test]
#[test]
fn test_renumber_does_not_warn() {
    use aliased::*;

    let a = [7u8; 16].alias_numbered();
    a.alias_named("a");

    assert!(!logs_contain("collision"));
}
