# 0005 — Fixed-cap ring buffer for the events log

**Status**: Accepted
**Date**: 2026-05-08

## Context

`AegisSecurityDaemon.events` is a bounded log: `config.max_events` (default 10 000) caps the population, oldest-drops on overflow. Rust-old expressed this as `events: Vec<SecurityEvent>` plus a `prune_events()` that drained the front when over cap.

The cyrius port carried that shape forward through 0.5.0 → 0.8.0:

```cyr
fn _aegis_prune_events(d) {
    var events = aegis_events(d);
    var len = vec_len(events);
    var max = aegis_config_max_events(aegis_config(d));
    if (len <= max) { return 0; }
    var start = len - max;
    var keep = vec_new();
    var i = start;
    while (i < len) { vec_push(keep, vec_get(events, i)); i = i + 1; }
    store64(d + 8, keep);
    return 0;
}
```

This is O(n) per push **once at cap** — every subsequent push allocates a fresh vec and copies `max - 1` elements forward. Bench at 50 k iterations: `aegis_report_event` ≈ 220 µs avg. The vec-rebuild dominates everything else (event alloc, threat-count increment, log emit).

Cyrius `lib/vec.cyr` doesn't ship a `vec_drain_front` or O(1) front-drop primitive. `vec_remove(v, 0)` exists but is O(n) per call (shifts all elements left), which would push the per-overflow cost to O(n²) — strictly worse.

## Decision

Replace the `vec*` events log with a **fixed-capacity ring buffer**:

```
struct ring {
    slots @ 0   ptr to N × 8-byte slot array
    cap   @ 8   max population (clamped to >= 1)
    head  @ 16  index of oldest entry
    count @ 24  current population (0..cap)
}
```

- `aegis_ring_new(cap)` allocates a 32-byte header plus `cap × 8` bytes for slots.
- `aegis_ring_push(rb, val)` writes at `(head + count) % cap`; if at cap, overwrites slot `head` and advances `head` forward — same observable behaviour as the prior prune-and-rebuild.
- `aegis_ring_get(rb, i)` reads at logical index `i` (0 = oldest, `count - 1` = newest).
- Cap is **captured at `aegis_new` time** from `config.max_events`. Subsequent calls to `aegis_config_set_max_events` on the same config don't resize an existing daemon's ring.

## Consequences

- **Positive** — `aegis_report_event` drops from ~220 µs to **4 µs avg at 50 k iter** (≈ 55× speedup). Push is O(1) regardless of cap. The events log no longer dominates the hot path. Bench reproducible — see [`tests/aegis.bcyr`](../../tests/aegis.bcyr) and `bench-history.csv`.
- **Negative** — the ring allocates the full cap upfront: at the default `max_events = 10 000`, that's 80 KB pinned for the lifetime of the daemon. The prior vec scheme grew on demand. For embedded use cases lower `max_events` before `aegis_new`.
- **Negative** — cap is fixed once construction completes. Resizing requires constructing a new daemon. Documented in [`docs/development/state.md`](../development/state.md) and [`docs/architecture/001-cyrius-port-gaps.md`](../architecture/001-cyrius-port-gaps.md). In practice `max_events` is a deployment-time tunable, not a runtime knob.
- **Neutral** — observable behaviour preserved: same iteration order (oldest first), same drop-on-overflow semantics, same `aegis_total_events` answers. All 65 pre-0.8.1 tests pass unchanged.

## Alternatives considered

- **Keep the vec, accept O(n) per push at cap** — rejected: 220 µs per `aegis_report_event` is unacceptable on a daemon expected to handle thousands of events per second.
- **Vec with `vec_drain_front` helper** — rejected: would require modifying `lib/vec.cyr` (out of scope; the cyrius stdlib is upstream) or shipping our own vec implementation. Even with O(1) drain, every push at cap still incurs a head-pointer-style operation; might as well commit to the ring shape.
- **Ring buffer with growable cap** — rejected: would require re-allocating the slot array on `set_max_events`, plus iteration logic to handle the head rotation across the resize. Adds complexity for a feature consumers don't need (max_events is set at deploy, not adjusted live).
- **`vec_remove(v, 0)` per overflow** — rejected: O(n) per call, O(n²) total once at cap. Strictly worse than the rebuild.
- **External "events file" sink** (write events to a log file, drop in-memory) — rejected: out of scope for this perf fix; would change the consumer contract for `aegis_recent_events` and friends.
