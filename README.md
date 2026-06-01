# aliased

Replace noisy `Debug` output with short, human-friendly aliases.

When you `format!("{:?}", thing)` and `thing` contains long opaque values
(public keys, hashes, IDs), the output becomes unreadable. `aliased` lets
you register aliases for specific values up front, then post-processes the
`Debug` (or `{:#?}`) output to substitute those values with the aliases.

## Two flavors

Selected via Cargo features. At least one must be enabled.

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

Pretty-printed output (`{:#?}`) is handled by a per-key regex that
tolerates the indentation `{:#?}` introduces, so nested values still get
aliased. Replacements are applied longest-key-first to reduce the chance
that a shorter registered value clobbers a longer one that contains it.

## Features

| feature      | default | effect                                               |
| ------------ | ------- | ---------------------------------------------------- |
| `global`     | yes     | exposes `aliased::Aliasing`, `Aliased`, `global_ctx` |
| `contextual` | no      | exposes `aliased::contextual::{Aliasing, Aliased}`   |
| `tracing`    | yes     | emits `tracing::warn!` for misuse / collisions       |

Building with neither `global` nor `contextual` is a compile error.

## When to use this

This is a debugging / logging aid. Substitution is roughly O(n × m) over
the formatted string for every print — fine for logs, not for hot paths.

## License

MIT OR Apache-2.0
