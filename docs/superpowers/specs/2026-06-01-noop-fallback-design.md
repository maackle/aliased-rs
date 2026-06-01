# No-op fallback when no flavor feature is enabled

## Goal

Building `aliased` with neither `global` nor `contextual` should no longer be a
compile error. Instead, the whole public API compiles as a no-op. This lets a
production app switch aliasing off via `default-features = false` without
deleting any `aliased` call sites.

Two decisions, already settled:

- **Coverage:** both flavors. With no features, both the crate-root
  global-shaped API (`.aliased()`, `.alias_named(name)`, …) and
  `aliased::contextual::*` (`.aliased(&ctx)`, …) compile as no-ops, so a
  consumer keeps building regardless of which flavor they used.
- **Dependencies:** stripped. A no-op build pulls neither `aho-corasick` nor
  `regex`, and compiles only a thin pass-through.

## Changes

### Cargo.toml — deps become optional, pulled in by either flavor

```toml
aho-corasick = { version = "1", optional = true }
regex        = { version = "1", optional = true }
tracing      = { version = "0.1", optional = true }

[features]
default    = ["global", "tracing"]
global     = ["dep:aho-corasick", "dep:regex"]
contextual = ["dep:aho-corasick", "dep:regex"]
tracing    = ["dep:tracing"]
```

### lib.rs — gate the machinery; add the fallback

"Active" mode = `any(feature = "global", feature = "contextual")`. The
substitution machinery (`shared`, `pretty`, the real `AliasContext`) compiles
only in active mode. A new `noop` module compiles only when neither feature is
set. The `compile_error!` is deleted.

```rust
#[cfg(any(feature = "global", feature = "contextual"))] mod shared; // + pretty, pub use AliasContext
#[cfg(feature = "contextual")] pub mod contextual;
#[cfg(feature = "global")]     pub use global::{Aliased, Aliasing};

#[cfg(not(any(feature = "global", feature = "contextual")))] mod noop;
#[cfg(not(any(feature = "global", feature = "contextual")))]
pub use noop::{AliasContext, Aliased, Aliasing, contextual};
```

(`global`-only and `contextual`-only builds are unchanged from today — e.g. a
`global`-only build still has no `aliased::contextual` module.)

### src/noop.rs — new, dependency-free mirror of both surfaces

- Zero-sized `AliasContext` with no-op `new()` / `set_brackets()` / `Default`.
- Root global-shaped `Aliasing` trait + `Aliased<'v, T>`, blanket-impl'd for
  `T: Debug + 'static`. `alias_prefix` does nothing; `alias_numbered` /
  `alias_named` return `&self`; `aliased()` wraps.
- A `contextual` submodule: context-taking `Aliasing` trait + `Aliased<'v,'c,T>`
  with signatures identical to the real flavor (the `&ctx` argument is ignored).
- Both `Aliased` `Debug` impls are just `fmt::Debug::fmt(self.val, f)`. This
  forwards the formatter's alternate flag, so `{:#?}` still pretty-prints —
  only the alias substitution is gone.
- The wrappers mirror the same derives / trait impls the real ones carry
  (`Clone`, `Eq`, `Ord`, `Hash`, …) for API fidelity.

The existing active modules (`shared.rs`, `pretty.rs`, `global.rs`,
`contextual.rs`) are untouched apart from the cfg gates above.

### Docs

Update README (the "compile error" line + features table) and CLAUDE.md
(feature list + behaviors) to describe the no-op fallback.

## Caveat

In no-op mode `contextual::Aliased` keeps the same three type parameters as the
real one, so naming it explicitly still compiles; only its `Debug` behavior
differs.

## Verification

The self dev-dep always forces `contextual` on for `cargo test`, so the no-op
path can't be exercised through `cargo test` — it is verified by build:

- `cargo build --no-default-features` → compiles; `cargo tree` shows no
  `regex` / `aho-corasick`.
- `cargo build` / `--features contextual` / `--features global,contextual` →
  unchanged.
- `cargo nextest run` → existing tests pass.
