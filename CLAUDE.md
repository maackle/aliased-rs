# aliased

A small Rust library that rewrites `Debug` output to replace long opaque
values (keys, hashes, IDs) with short registered aliases. Aimed at making
logs and test failures readable.

## Crate layout

- `src/lib.rs` — feature gate (`compile_error!` if neither `global` nor
  `contextual`), re-exports, crate-level docs.
- `src/shared.rs` — `AliasContext`, internal `AliasData` / `Repr` / `Alias`,
  non-generic helpers (`set_prefix`, `register_named`, `register_numbered`,
  `fmt_aliased`). Always compiled.
- `src/global.rs` — feature `global`. `Aliasing` trait + `Aliased<'v, T>`
  wrapper backed by a `LazyLock<AliasContext>` static. Re-exported at the
  crate root.
- `src/contextual.rs` — feature `contextual`. `Aliasing` trait +
  `Aliased<'v, 'c, T>` wrapper that takes an explicit `&AliasContext`.
  Lives at `aliased::contextual::*`.
- `src/pretty.rs` — regex-based substitution for pretty (`{:#?}`) output.
- `tests/nested.rs` — integration tests, use the `contextual` flavor.

The generic trait methods in `global` / `contextual` exist only to compute
`format!("{self:?}")` and `format!("{self:#?}")` (which require `Self:
Debug`) and forward to the non-generic helpers in `shared`. This keeps both
flavors thin and ensures one source of truth for the registration and
formatting logic.

## Cargo features

- `global` (default) — process-wide static context.
- `contextual` — explicit `&AliasContext` per call.
- `tracing` (default) — emits `tracing::warn!` for misuse / collisions.

Both flavors share the public `AliasContext` type (always exported). Both
flavor traits are blanket-impl'd for `T: Debug + 'static`, so glob-importing
both into the same module will cause method-name ambiguity — users should
import only the flavor they intend to use.

## Tests / feature unification

`Cargo.toml` includes a self dev-dep:

```toml
[dev-dependencies]
aliased = { path = ".", features = ["contextual"] }
```

This activates the `contextual` feature during test/doctest/example builds,
so `cargo test` works without `--features contextual`.

## Key behaviors / gotchas

- `brackets` are snapshotted into each `Repr` at registration time, so
  changing brackets later does not retroactively update existing aliases.
- `alias_numbered` keys off `format!("{:?}", self)`. The second call on the
  same debug string is a no-op (with a `tracing::warn!`).
- Substitution uses single-pass matchers cached in `AliasData`, rebuilt
  lazily on the first print after any registration (the `DebugMatcher` /
  `PrettyMatcher` `Stale`/`Empty`/`Built` enums). Any change that affects
  output (`register_*`, `set_prefix`, `set_brackets`) calls `invalidate()`.
- Plain (`{:?}`) path: one `aho-corasick` automaton built with
  `MatchKind::LeftmostLongest`, so the longest registered key wins at a
  position (no manual longest-first sort needed). `ac.replace_all` does the
  whole substitution in one pass.
- Pretty (`{:#?}`) path: all per-key patterns are combined into a single
  alternation `Regex`, each alternative in a capture group so the matched
  alias can be dispatched (group `i+1` ↔ pattern `i`). Alternatives are
  ordered longest-first so the longest candidate wins regardless of the
  engine's alternation precedence. This is also a correctness improvement
  over the old per-key sequential passes, which could re-match and clobber
  an earlier replacement's output.
- Longest-first / leftmost-longest reduces the chance that a shorter
  registered value clobbers a longer one that contains it. Not foolproof
  against arbitrary overlap.

## Commands

- `just test` → `cargo nextest run`
- `cargo test` also works; the self dev-dep enables `contextual`
