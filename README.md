# aliased

Replace noisy `Debug` output with short, human-friendly aliases.

When you `format!("{:?}", thing)` and `thing` contains long opaque values
(public keys, hashes, IDs), the output becomes unreadable. `aliased` lets you
register aliases for specific values up front, then post-processes the `Debug`
(or `{:#?}`) output to substitute those values with the aliases.

```rust
use aliased::*;

#[derive(Debug)]
struct Key([u8; 32]);

let ctx = AliasContext::new();
Key::alias_prefix(&ctx, "K");

let a = Key([1; 32]);
let b = Key([2; 32]);
a.alias_named(&ctx, "alice");
b.alias_named(&ctx, "bob");

let pair = (a, b);
assert_eq!(
    format!("{:?}", pair.aliased(&ctx)),
    "(⟪K|alice⟫, ⟪K|bob⟫)",
);
```

## How it works

`alias_named` / `alias_numbered` store a mapping from the value's
`format!("{:?}", v)` string to a chosen alias. When you print
`value.aliased(&ctx)`, the crate formats `value` with `Debug`, then runs
string substitution to replace each registered debug fragment with its alias.

Pretty-printed output (`{:#?}`) is handled by a regex that tolerates the
indentation `{:#?}` introduces, so nested values still get aliased.

## Features

- `tracing` (default): emits `tracing::warn!` for misuse (duplicate prefixes,
  re-aliasing, collisions). Disable to drop the `tracing` dependency.

## When to use this

This is a debugging / logging aid. It is not a general-purpose `Debug`
replacement, and substitution is O(n × m) over the formatted string for every
print. Use it where readable logs matter more than print throughput.

## License

MIT OR Apache-2.0
