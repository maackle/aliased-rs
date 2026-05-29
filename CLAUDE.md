# aliased

A small Rust library that rewrites `Debug` output to replace long opaque values
(keys, hashes, IDs) with short registered aliases. Aimed at making logs and
test failures readable.

## Architecture

- `AliasContext` — `Arc<Mutex<AliasData>>` registry. Cheap to clone, all
  registration goes through the mutex.
- `AliasData` — holds:
  - `brackets`: the open/close strings wrapping every alias (default `⟪`, `⟫`)
  - `prefixes`: per-`TypeId` short prefix (e.g. `K` for `PublicKey`)
  - `numbers`: per-`TypeId` counter used by `alias_numbered`
  - `debug_names`: `format!("{:?}", v)` → `Repr`
  - `pretty_names`: `format!("{:#?}", v)` → `Repr`
- `Aliasing` — blanket-impl'd for any `T: Debug + 'static`. Provides
  `alias_named`, `alias_numbered`, `alias_prefix`, and `aliased(&ctx)`.
- `Aliased<'v, 'c, T>` — wrapper whose `Debug` impl formats the inner value,
  then substitutes registered fragments. Alternate (`{:#?}`) goes through
  `pretty::pretty_replace`, which builds a per-line regex tolerant of
  `{:#?}` indentation.

Two parallel maps (`debug_names` / `pretty_names`) exist because `{:?}` and
`{:#?}` produce different strings; both are populated on each registration.

## Key behaviors / gotchas

- `brackets` are snapshotted into each `Repr` at registration time, so
  changing brackets later does not retroactively update existing aliases.
- `alias_numbered` keys off `format!("{:?}", self)`. If the same debug string
  is registered twice, the second call is a no-op (with a `tracing::warn!`).
- Substitution is plain `str::replace` (for `{:?}`) or full regex rebuild per
  key (for `{:#?}`). Cost scales with number of aliases × output length.
- BTreeMap iteration order is alphabetical; the code iterates `.rev()` to get
  reverse alphabetical. This is not the same as "longest first," so if one
  registered debug string is a substring of another, replacement order can be
  wrong. Avoid registering values whose debug forms overlap.

## Layout

- `src/lib.rs` — public API
- `src/pretty.rs` — regex-based substitution for pretty (`{:#?}`) output
- `tests/nested.rs` — integration tests covering named + numbered aliasing
  through nested structs

## Commands

- `just test` → `cargo nextest run`
- `cargo test` also works
