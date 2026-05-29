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
- `pretty_names` stores a precompiled `Regex` next to the `Repr` so the
  per-key regex is compiled once at registration, not on every print.
- Substitution sorts keys longest-first to reduce the chance that a shorter
  registered value clobbers a longer one that contains it. Not foolproof
  against arbitrary overlap.
- BTreeMap iteration order is alphabetical; `Aliased::fmt` materializes the
  entries into a `Vec` to re-sort by `len` desc, then lex asc.

## Commands

- `just test` → `cargo nextest run`
- `cargo test` also works; the self dev-dep enables `contextual`
