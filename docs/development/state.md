# aegis — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**0.1.0** — ported from Rust (2026-05-08) via `cyrius port`. 1893 lines of Rust preserved at `rust-old/` for parity reference.

## Toolchain

- **Cyrius pin**: `5.10.0` (in `cyrius.cyml [package].cyrius`)

## Source

- Rust reference: 1893 lines at `rust-old/` (frozen, do not edit).
- Cyrius port (first slice, 2026-05-08):
  - `src/lib.cyr` — `ThreatLevel` + `SecurityEventType` enums with label fns; counter-backed `aegis_next_id`; 72-byte `SecurityEvent` record with constructor, accessors, and `event_set_resolved` setter.
  - `src/main.cyr` — thin entry that includes `src/lib.cyr`.
- Not yet ported: `QuarantineEntry`, `QuarantineAction`, `ScanType`, `SecurityFinding`, `SecurityScanResult`, `AegisConfig`, `DatabaseSecurityPolicy`, `KernelTuningRecommendation`, `AegisStats`, `AegisSecurityDaemon`. JSON serde, sakshi-full structured logging, agnostik UUIDs, nein firewall — all deferred (see `docs/architecture/cyrius-port-gaps.md`).

## Tests

`tests/aegis.tcyr` — 6 groups, 25 assertions, all passing on `cyrius test tests/aegis.tcyr`:

- `threat_level_labels` — Display round-trip for all 5 levels.
- `threat_level_ordering` — `Critical < High < Medium < Low < Info` matches rust-old's manual `Ord`.
- `event_type_labels` — snake_case labels for all 12 event types (parity with serde rendering).
- `security_event_new` — constructor + accessors round-trip; defaults (`resolved=0`, `agent_id=0`, `metadata=0`).
- `event_set_resolved` — mutator.
- `event_ids_unique` — counter advances across constructions.

Bench / fuzz harnesses (`tests/aegis.bcyr`, `tests/aegis.fcyr`) remain stubs.

## Dependencies

Direct (declared in `cyrius.cyml`):

- stdlib — string, fmt, alloc, vec, str, syscalls, io, args, assert, tagged, chrono, hashmap

## Consumers

_None yet._

## Next

See [`roadmap.md`](roadmap.md). The first milestone is typically Rust→Cyrius surface parity for the 1893-line subset.
