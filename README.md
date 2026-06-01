# aliased

Replace noisy `Debug` output with short, human-friendly aliases.

When you `format!("{:?}", thing)` and `thing` contains long opaque values
(public keys, hashes, IDs), the output becomes unreadable. `aliased` lets
you register aliases for specific values up front, then post-processes the
`Debug` (or `{:#?}`) output to substitute those values with the aliases.

## Two flavors

Selected via Cargo features. With neither enabled, the whole API still
compiles but becomes a [no-op](#no-op-builds).

### `global` (default)

A process-wide static `AliasContext` is used implicitly — nothing to thread
through call sites.

```rust
use aliased::*;

#[derive(Debug)]
struct Key([u8; 32]);

Key::alias_prefix("K");

let a = Key([1; 32]);
let b = Key([2; 32]);
a.alias_named("alice");
b.alias_named("bob");

assert_eq!(format!("{:?}", (a, b).aliased()), "(⟪K|alice⟫, ⟪K|bob⟫)");
```

### `contextual`

You pass an explicit `&AliasContext` to every call. Useful when you want
isolated registries — for example, one per test.

```rust
use aliased::AliasContext;
use aliased::contextual::*;

// ...same API as global, but each method takes `&ctx`.
let ctx = AliasContext::new();

#[derive(Debug)]
struct Key([u8; 32]);

Key::alias_prefix(&ctx, "K");

let a = Key([1; 32]);
let b = Key([2; 32]);
a.alias_named(&ctx, "alice");
b.alias_named(&ctx, "bob");

assert_eq!(format!("{:?}", (a, b).aliased(&ctx)), "(⟪K|alice⟫, ⟪K|bob⟫)");
```

Enable in `Cargo.toml`:

```toml
[dependencies]
aliased = { version = "0.1", default-features = false, features = ["contextual"] }
# or both:
aliased = { version = "0.1", features = ["contextual"] }
```

## How it works

`alias_named` / `alias_numbered` store a mapping from the value's
`format!("{:?}", v)` string to a chosen alias. When you print
`value.aliased(..)`, the crate formats `value` with `Debug`, then runs
string substitution to replace each registered debug fragment with its
alias.

All registered values are combined into a single matcher so each print
scans the output once rather than once per registered alias: plain `{:?}`
output uses an [Aho–Corasick](https://en.wikipedia.org/wiki/Aho%E2%80%93Corasick_algorithm)
automaton, and pretty `{:#?}` output uses one alternation regex whose
patterns tolerate the indentation `{:#?}` introduces, so nested values
still get aliased. The matcher is rebuilt lazily after a registration and
reused across prints. Matches resolve longest-first to reduce the chance
that a shorter registered value clobbers a longer one that contains it.

## Features

| feature      | default | effect                                               |
| ------------ | ------- | ---------------------------------------------------- |
| `global`     | yes     | exposes `aliased::Aliasing`, `Aliased`, `global_ctx` |
| `contextual` | no      | exposes `aliased::contextual::{Aliasing, Aliased}`   |
| `tracing`    | yes     | emits `tracing::warn!` for misuse / collisions       |

### No-op builds

Building with neither `global` nor `contextual` is **not** an error. The whole
API — both the global-shaped methods and `aliased::contextual::*` — still
compiles, but every call is a no-op: `value.aliased(..)` formats with plain
`Debug` (the `{:#?}` alternate form still pretty-prints) and registration calls
do nothing. The substitution machinery and its `aho-corasick` / `regex`
dependencies are not compiled.

This lets you keep `aliased` call sites in a production app while turning the
feature off:

```toml
aliased = { version = "0.1", default-features = false }
```

## When to use this

This is a debugging / logging aid. Each print makes a single O(m) pass over
the formatted string (after a one-time matcher build amortized across
prints) — fine for logs, not for hot paths.

## License

MIT OR Apache-2.0
