# 0001 — Sentinel values for absent state

**Status**: Accepted
**Date**: 2026-05-08

## Context

Cyrius has no native `Option<T>`. Several aegis fields are conceptually optional, with rust-old precedent:

| Field | Rust type | Where |
|-------|-----------|-------|
| `auto_release_at` | `Option<DateTime<Utc>>` | `QuarantineEntry` |
| `auto_release_timeout_secs` | `Option<u64>` | `AegisConfig` |
| `agent_id` | `Option<String>` | `SecurityEvent` |
| `metadata` | `HashMap` (lazy) | `SecurityEvent` |
| `report_event` action return | `Option<QuarantineAction>` | daemon API |

`lib/tagged.cyr` provides a heap-allocated `Some`/`None` tagged-pointer type. Using it would faithfully model `Option<T>` but adds an allocation per Some, requires `is_some` / `unwrap` calls everywhere a value is read, and doesn't compose well with cyrius's i64-everywhere style.

## Decision

Use **value sentinels**, picked per field:

| Field | Sentinel for None | Why this value |
|-------|-------------------|----------------|
| `auto_release_at` (epoch s) | `-1` (`AEGIS_AUTO_RELEASE_NONE`) | 0 is a valid epoch (1970-01-01); -1 cannot be reached by valid timestamps. |
| `auto_release_timeout_secs` | `-1` (same constant) | 0 is a valid timeout (immediate release). |
| `agent_id` (Str ptr) | `0` | 0 is the universal null-pointer sentinel and cannot collide with any allocated Str. |
| `metadata` (map ptr) | `0` (lazy-init) | 0 means "no metadata yet"; the first `event_metadata_set` call allocates the map. |
| `QuarantineAction` return | `QA_NONE = 0` | Defined as the first enum variant so the return value can be used as a bool ("did anything fire?") without a separate Option wrapper. |

JSON serde maps every sentinel to `null` and back.

## Consequences

- **Positive** — every Option-shaped field stays a single i64 in storage; no allocations for None; checks are bare comparisons (`if (x == 0)` / `if (x != AEGIS_AUTO_RELEASE_NONE)`); JSON wire format matches rust serde's `null` rendering for free.
- **Negative** — the sentinel is per-field. Code reviewers must remember which constant means None for which field. Documented in field-offset comments above each ctor.
- **Neutral** — if a future field's natural domain includes the sentinel value (e.g., a counter that legitimately reaches `-1`), this approach won't extend; that field would need a tagged Option or a bit-flag side channel.

## Alternatives considered

- **`lib/tagged.cyr` `Some`/`None`** — rejected: heap-allocates per Some, adds two getter calls (`is_some` + `unwrap`) per read, and infects every accessor signature with the tagged shape. The boilerplate cost outweighed the type-safety win for the handful of optional fields aegis has.
- **Side-channel bit flags** (e.g., a "has_auto_release" bool field next to `auto_release_at`) — rejected: doubles the storage cost and creates a consistency invariant the ctor + setters must maintain. Sentinel encoding makes the invariant intrinsic.
- **`epoch=0` as None for timestamps** — rejected: 0 is a valid epoch, and rust-old callers could pass 1970-01-01 by accident. `-1` is unambiguous.
