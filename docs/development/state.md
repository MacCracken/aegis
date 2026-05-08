# aegis — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**0.1.0** — ported from Rust (2026-05-08) via `cyrius port`. 1893 lines of Rust preserved at `rust-old/` for parity reference.

## Toolchain

- **Cyrius pin**: `5.10.0` (in `cyrius.cyml [package].cyrius`)

## Source

- Rust reference: 1893 lines at `rust-old/` (frozen, do not edit).
- Cyrius port (slices 1–3, 2026-05-08):
  - `src/lib.cyr` — `ThreatLevel`, `SecurityEventType`, `QuarantineAction` enums (the latter with `QA_NONE = 0` as a sentinel so `report_event` can return action-or-zero without tagged Option overhead) plus label fns. Counter-backed `aegis_next_id`. 72-byte `SecurityEvent` record (ctor takes cstr-or-0 for `agent_id`; getters + `event_set_resolved`). 56-byte `AegisConfig` record (defaults + getters + setters; `-1` sentinel for `auto_release_timeout_secs = None`). 48-byte `QuarantineEntry` record (ctor + accessors + `q_set_threat_level` / `q_set_reason` / `q_set_auto_release_at`). 72-byte `AegisSecurityDaemon` record (config + events vec + lazy-init quarantine map + scan-history slot + inline 5-slot threat-counts array).
  - Daemon API (cstrs for `agent_id` / `event_id` parameters): `aegis_new`, `aegis_report_event` (records + auto-quarantine; returns `QuarantineAction`), `aegis_recent_events`, `aegis_events_for_agent`, `aegis_events_by_threat`, `aegis_unresolved_events`, `aegis_resolve_event`, `aegis_threat_count`, `aegis_total_events`, `aegis_unresolved_count`, `aegis_quarantine_agent` (escalates threat / appends reason on re-quarantine), `aegis_release_agent`, `aegis_is_quarantined`, `aegis_get_quarantine`, `aegis_quarantined_agents`, `aegis_check_auto_releases`. `_aegis_prune_events` rebuilds the events vec with the kept suffix instead of an O(n²) `vec_remove(0)` loop.
  - `src/main.cyr` — thin entry that includes `src/lib.cyr`.
- Not yet ported: `ScanType`, `SecurityFinding`, `SecurityScanResult`, `DatabaseSecurityPolicy`, `KernelTuningRecommendation`, `AegisStats`. JSON serde, sakshi-full structured logging, agnostik UUIDs, nein firewall — all deferred (see `docs/architecture/cyrius-port-gaps.md`).

## Tests

`tests/aegis.tcyr` — 30 groups, **89 assertions, all passing** on `cyrius test tests/aegis.tcyr`:

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

Slice 3 (quarantine):

- `quarantine_action_labels` — round-trip for `QA_NONE / SUSPEND / TERMINATE / ISOLATE / RATELIMIT`.
- `quarantine_and_release` / `release_non_quarantined` / `get_quarantine_entry` / `quarantined_agents_list` — manual quarantine API and lazy-init map.
- `quarantine_no_downgrade` / `quarantine_escalates_and_appends_reason` — re-quarantining only escalates (never downgrades) threat; reasons append with `"; "` separator.
- `auto_quarantine_on_critical` / `auto_quarantine_on_high` / `no_quarantine_on_medium` / `no_quarantine_without_agent_id` — auto-quarantine policy in `report_event`.
- `quarantine_links_events` — multiple events for the same agent attach to one entry's event-id list.
- `auto_release_timeout_expired` / `auto_release_timeout_not_expired` / `auto_release_no_timeout_set` / `config_auto_release_populates_entry` — `check_auto_releases` against the config-driven `auto_release_at` field.

Bench / fuzz harnesses (`tests/aegis.bcyr`, `tests/aegis.fcyr`) remain stubs.

## Dependencies

Direct (declared in `cyrius.cyml`):

- stdlib — string, fmt, alloc, vec, str, syscalls, io, args, assert, tagged, chrono, hashmap

## Consumers

_None yet._

## Next

See [`roadmap.md`](roadmap.md). The first milestone is typically Rust→Cyrius surface parity for the 1893-line subset.
