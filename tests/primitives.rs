use tracing_test::traced_test;

#[test]
fn test_arrays() {
    use aliased::contextual::*;
    let ctx = aliased::AliasContext::new();

    // Not such a great idea, but this can be done!
    <[u8; 16]>::alias_prefix(&ctx, "u8-16");
    <[u16; 8]>::alias_prefix(&ctx, "u16-8");
    <[u8; 32]>::alias_prefix(&ctx, "u8-32");

    let a = [1u8; 16].alias_numbered(&ctx);
    let b = [1u16; 8].alias_numbered(&ctx);
    let c = [1u8; 32].alias_numbered(&ctx);

    assert_eq!(
        format!("{:?}", (a, b, c).aliased(&ctx)),
        "(⟪u8-16|#000⟫, ⟪u16-8|#000⟫, ⟪u8-32|#000⟫)"
    );

    a.alias_named(&ctx, "a");
    b.alias_named(&ctx, "b");
    c.alias_named(&ctx, "c");

    assert_eq!(
        format!("{:?}", (a, b, c).aliased(&ctx)),
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
    use aliased::contextual::*;
    let ctx = aliased::AliasContext::new();

    <(u8, u8)>::alias_prefix(&ctx, "8x2");
    <(u16, u16)>::alias_prefix(&ctx, "16x2");

    // Both Debug-format to the literal "(1, 2)" despite being different types.
    let _a = (1u8, 2u8).alias_numbered(&ctx);
    let _b = (1u16, 2u16).alias_numbered(&ctx);

    assert!(logs_contain("collision"));
}

// Re-aliasing the *same* value (numbered -> named) is intentional, not a
// collision, and must stay quiet.
#[traced_test]
#[test]
fn test_renumber_does_not_warn() {
    use aliased::contextual::*;
    let ctx = aliased::AliasContext::new();

    let a = [7u8; 16].alias_numbered(&ctx);
    a.alias_named(&ctx, "a");

    assert!(!logs_contain("collision"));
}
