# aegis — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**0.1.0** — ported from Rust (2026-05-08) via `cyrius port`. 1893 lines of Rust preserved at `rust-old/` for parity reference.

## Toolchain

- **Cyrius pin**: `5.10.0` (in `cyrius.cyml [package].cyrius`)

## Source

- Rust reference: 1893 lines at `rust-old/` (frozen, do not edit).
- Cyrius port (slices 1–2, 2026-05-08):
  - `src/lib.cyr` — `ThreatLevel` + `SecurityEventType` enums with label fns; counter-backed `aegis_next_id`; 72-byte `SecurityEvent` record (ctor + accessors + setter); 56-byte `AegisConfig` record (ctor + getters + setters; `auto_release_timeout_secs = -1` sentinel for None); 72-byte `AegisSecurityDaemon` record (config + events vec + quarantine slot + scan-history slot + inline 5-slot threat-counts array). Daemon API: `aegis_new`, `aegis_report_event` (records + increments threat count + prunes), `aegis_recent_events`, `aegis_events_for_agent`, `aegis_events_by_threat`, `aegis_unresolved_events`, `aegis_resolve_event`, `aegis_threat_count`, `aegis_total_events`, `aegis_unresolved_count`, `_aegis_prune_events` (rebuilds vec with kept suffix to avoid O(n²) `vec_remove(0)` loop).
  - `src/main.cyr` — thin entry that includes `src/lib.cyr`.
- Not yet ported: `QuarantineEntry`, `QuarantineAction`, `ScanType`, `SecurityFinding`, `SecurityScanResult`, `DatabaseSecurityPolicy`, `KernelTuningRecommendation`, `AegisStats`. Auto-quarantine logic in `report_event` returns 0 (action) until the quarantine slice lands. JSON serde, sakshi-full structured logging, agnostik UUIDs, nein firewall — all deferred (see `docs/architecture/cyrius-port-gaps.md`).

## Tests

`tests/aegis.tcyr` — 15 groups, **54 assertions, all passing** on `cyrius test tests/aegis.tcyr`:

Slice 1 (records):

- `threat_level_labels` / `threat_level_ordering` / `event_type_labels` — enum label round-trips and `Critical < High < … < Info` ordering matches rust-old's manual `Ord`.
- `security_event_new` / `event_set_resolved` / `event_ids_unique` — `SecurityEvent` ctor/accessors/setter and id uniqueness.

Slice 2 (daemon):

- `aegis_config_defaults` — defaults match rust-old's `Default for AegisConfig`.
- `aegis_new` — empty event log, zeroed threat counts.
- `aegis_report_event_basic` / `aegis_threat_counts_per_level` — events recorded, threat counts increment per level.
- `aegis_recent_events` / `aegis_events_for_agent` / `aegis_events_by_threat` — query filters, including the `agent_id == 0` (None) skip path.
- `aegis_unresolved_events_and_resolve` — `resolve_event` returns 1 on hit / 0 on miss; unresolved counts update.
- `aegis_max_events_prunes_oldest` — pruning at `max_events = 5` keeps the most recent 5 of 8 reports.

Bench / fuzz harnesses (`tests/aegis.bcyr`, `tests/aegis.fcyr`) remain stubs.

## Dependencies

Direct (declared in `cyrius.cyml`):

- stdlib — string, fmt, alloc, vec, str, syscalls, io, args, assert, tagged, chrono, hashmap

## Consumers

_None yet._

## Next

See [`roadmap.md`](roadmap.md). The first milestone is typically Rust→Cyrius surface parity for the 1893-line subset.
