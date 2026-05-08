# 0003 — Integer-array threat counts (not a hashmap)

**Status**: Accepted
**Date**: 2026-05-08

## Context

Rust-old declares `threat_counts: HashMap<ThreatLevel, usize>` on both `AegisSecurityDaemon` and the serialized `AegisStats` snapshot. Cyrius `lib/hashmap.cyr` is string-keyed (cstr or Str); there is no first-class int-keyed map. (There's `map_u64_*` in newer cyrius releases, but its API surface is half-cooked and the constants live behind `_map_u64_*`.)

`ThreatLevel` is a closed set: 5 variants (`Critical / High / Medium / Low / Info`), integer values 0..4, defined by an `enum` declaration. Lookups are dense (every level may have a count) and hot (`_aegis_inc_threat_count` fires inside `aegis_report_event` for every reported event).

## Decision

Store `threat_counts` as an **inline 5-slot i64 array** at offset 32 in the daemon record (and in the `AegisStats` snapshot). Indexed directly by `threat_level`:

```
fn aegis_threat_count(d, level) {
    return load64(d + 32 + level * 8);
}

fn _aegis_inc_threat_count(d, level) {
    var off = d + 32 + level * 8;
    store64(off, load64(off) + 1);
    return 0;
}
```

JSON serialization renders this as a nested object with PascalCase string keys (`{"Critical": 0, "High": 2, ...}`) to match rust serde's `HashMap<ThreatLevel, usize>` rendering — see [`0004-hashmap-flavor-selection`](0004-hashmap-flavor-selection.md) for related map decisions.

## Consequences

- **Positive** — O(1) read/increment with no hash, no allocation, no key lifetime concerns. Iteration is a 5-step loop with known indices. Snapshots into `AegisStats` are a 5-step `store64` block. Total cost: 40 bytes of inline storage in two places, no header overhead.
- **Negative** — the array shape leaks the enum's discriminant range (`0..4`). If a future `ThreatLevel` variant is added, the array size and every iteration loop need to grow in lockstep. Mitigation: every loop is `while (i < 5)`; a future `THREAT_LEVEL_COUNT` constant + grep would catch sites if we add a 6th level.
- **Neutral** — wire format diverges from a generic int-keyed map: keys are stringified PascalCase variants, not integer keys. Matches rust serde's default (which also stringifies enum keys), so this is a feature for cross-stack interop.

## Alternatives considered

- **`map_u64_*` from `lib/hashmap.cyr`** — rejected: surface API isn't well-documented, hash-overhead per increment is meaningful in `report_event`'s hot path, and the `0..4` index space doesn't justify a hash structure.
- **`map_new()` cstr-keyed with stringified keys** — rejected: same hash overhead plus a `str_from`/`streq` per increment. Materially slower for no win.
- **One i64 field per level** (`threat_critical`, `threat_high`, ...) — rejected: indistinguishable in storage from the array but loses the `level` parameter form, requiring 5-way `if/elif` branches at every call site.
