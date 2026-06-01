//! Edge cases for the substitution engine: overlapping keys, empty
//! registries, many aliases of one type, and pretty-mode dispatch. These
//! guard the single-pass matcher (Aho-Corasick for `{:?}`, one combined
//! alternation regex for `{:#?}`) against regressions in longest-first
//! precedence and alias dispatch.

use aliased::AliasContext;
use aliased::contextual::*;

/// A type whose `Debug` output is an arbitrary string we fully control, so we
/// can construct keys where one is a literal substring of another. Ignores
/// the alternate flag, so `{:?}` and `{:#?}` produce the same single line.
struct Raw(&'static str);

impl std::fmt::Debug for Raw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

#[test]
fn empty_registry_passes_through_unchanged() {
    let ctx = AliasContext::new();
    let v = vec![1, 2, 3];
    assert_eq!(format!("{:?}", v.aliased(&ctx)), "[1, 2, 3]");
    assert_eq!(
        format!("{:#?}", v.aliased(&ctx)),
        "[\n    1,\n    2,\n    3,\n]"
    );
}

#[test]
fn longest_key_wins_when_one_is_a_substring_debug() {
    let ctx = AliasContext::new();
    ctx.set_brackets(("<", ">"));

    Raw("abcdef").alias_named(&ctx, "LONG");
    Raw("abc").alias_named(&ctx, "SHORT");

    // The longer registered key must win where both could match.
    assert_eq!(format!("{:?}", Raw("abcdef").aliased(&ctx)), "<LONG>");
    // The shorter key still applies on its own.
    assert_eq!(format!("{:?}", Raw("abc").aliased(&ctx)), "<SHORT>");
}

#[test]
fn longest_key_wins_when_one_is_a_substring_pretty() {
    let ctx = AliasContext::new();
    ctx.set_brackets(("<", ">"));

    Raw("abcdef").alias_named(&ctx, "LONG");
    Raw("abc").alias_named(&ctx, "SHORT");

    assert_eq!(format!("{:#?}", Raw("abcdef").aliased(&ctx)), "<LONG>");
    assert_eq!(format!("{:#?}", Raw("abc").aliased(&ctx)), "<SHORT>");
}

#[test]
fn many_aliases_each_dispatch_to_their_own_name() {
    let ctx = AliasContext::new();

    let raws: Vec<Raw> = (0..20)
        .map(|i| Raw(Box::leak(format!("val-{i:02}").into_boxed_str())))
        .collect();
    for (i, r) in raws.iter().enumerate() {
        r.alias_named(&ctx, Box::leak(format!("n{i}").into_boxed_str()));
    }

    // Each distinct key resolves to its own distinct alias, even though all
    // share the `val-` prefix (the would-be "common pattern").
    assert_eq!(format!("{:?}", Raw("val-07").aliased(&ctx)), "⟪n7⟫");
    assert_eq!(format!("{:?}", Raw("val-19").aliased(&ctx)), "⟪n19⟫");

    let out = format!("{:?}", raws.aliased(&ctx));
    assert_eq!(out.matches('⟪').count(), 20);
    assert!(out.contains("⟪n0⟫"));
    assert!(out.contains("⟪n19⟫"));
}

#[test]
fn registering_after_a_print_rebuilds_the_matcher() {
    let ctx = AliasContext::new();

    Raw("aaa").alias_named(&ctx, "a");
    assert_eq!(format!("{:?}", Raw("aaa").aliased(&ctx)), "⟪a⟫");

    // A second registration after the matcher has already been built/cached
    // must invalidate the cache so the new alias takes effect.
    Raw("bbb").alias_named(&ctx, "b");
    assert_eq!(
        format!("{:?}", (Raw("aaa"), Raw("bbb")).aliased(&ctx)),
        "(⟪a⟫, ⟪b⟫)"
    );
}
