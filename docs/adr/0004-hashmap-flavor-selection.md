# 0004 — Hashmap flavor: `map_new()` (cstr) over `map_new_str()` (Str)

**Status**: Accepted
**Date**: 2026-05-08

## Context

`lib/hashmap.cyr` exposes two hashmap constructors with **incompatible** key conventions:

| Constructor | Key type | Hash function | Equality |
|-------------|----------|---------------|----------|
| `map_new()` | cstr (null-terminated `char*`) | `hash_str` (FNV-1a over null-terminated bytes) | `streq` |
| `map_new_str()` | Str (fat pointer `{data, len}`) | `hash_str_v` (FNV-1a over `(str_data, str_len)`) | `str_eq` |

The two are **not interchangeable**. `_map_hash` and `_map_key_eq` dispatch on a `KeyType` discriminant stored in the map header at `+24`. Feeding a cstr to a `map_new_str()` map calls `hash_str_v` which calls `str_data(key)`/`str_len(key)` — `load64(cstr)` and `load64(cstr + 8)` on raw chars, returning garbage. Result: silent segfault or mass key-mismatch.

aegis has two maps:

1. **Daemon quarantine** — keyed by `agent_id` cstr. Lookups happen on every `aegis_report_event` whose threat triggers auto-quarantine.
2. **Per-event metadata** — keyed by metadata-key cstr (e.g., `"ddl_operation"`, `"violation_reason"`).

Both flow cstrs at the boundary per [`0002-cstr-api-boundary`](0002-cstr-api-boundary.md).

## Decision

**Always `map_new()` for any aegis-owned map.** Both quarantine and metadata maps lazy-init with `map_new()` on first write. The decision is mechanical:

```cyr
fn _aegis_ensure_quarantine_map(d) {
    if (aegis_quarantine(d) == 0) {
        store64(d + 16, map_new());     # cstr-keyed
    }
    return aegis_quarantine(d);
}
```

`map_new_str()` is **not used** in the aegis source tree. If a future map needs Str keys (e.g., a map keyed by event_id where the Str is already in hand), the choice gets re-litigated then.

## Consequences

- **Positive** — call sites pass agent_id / metadata-key cstrs directly with no wrapping ceremony. Internal lookup goes through the FNV-1a-over-bytes path that matches what callers think they're keying on.
- **Negative** — keys are byte-identity-compared (`streq`). For agent IDs that's fine (same agent → same cstr or same string content). For dynamically-built keys (e.g., concatenating "agent:" + id), callers must keep the backing cstr alive for the map's lifetime. Documented in [`SECURITY.md`](../../SECURITY.md) under the in-process metadata-map row.
- **Neutral** — encodes `map_new()` (not `map_new_str()`) as the project default. New contributors see the precedent in src/lib.cyr.

## Alternatives considered

- **`map_new_str()`** — rejected: requires every callsite to `str_from(key_cstr)` before lookup, defeating the cstr-at-boundary convention from [`0002`](0002-cstr-api-boundary.md). Also incurs the Str's 16-byte heap allocation per lookup.
- **`map_u64_*`** — not applicable (keys aren't integers).
- **Avoiding hashmaps entirely** with a vec-of-pairs + linear scan — rejected: quarantine map can grow into the hundreds of entries on a misbehaving system; linear scan is O(n) per `aegis_is_quarantined` call.

## Notes

This was the bug that took down the quarantine slice tests in 0.5.0 — a `map_new_str()` typo silently segfaulted the test runner. The decision is documented here so that future contributors don't re-derive the choice from hash fundamentals.
