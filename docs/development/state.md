# aegis — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**0.1.0** — ported from Rust (2026-05-08) via `cyrius port`. 1893 lines of Rust preserved at `rust-old/` for parity reference.

## Toolchain

- **Cyrius pin**: `5.10.0` (in `cyrius.cyml [package].cyrius`)

## Source

- Rust reference: 1893 lines at `rust-old/` (frozen, do not edit).
- Cyrius port (slices 1–4, 2026-05-08):
  - `src/lib.cyr` — `ThreatLevel`, `SecurityEventType`, `QuarantineAction`, `ScanType` enums (with label fns; `QA_NONE = 0` sentinel so `report_event` returns action-or-zero without tagged Option overhead). Counter-backed `aegis_next_id`. Records: 72-byte `SecurityEvent`, 56-byte `AegisConfig` (`-1` sentinel for `auto_release_timeout_secs = None`), 48-byte `QuarantineEntry`, 32-byte `SecurityFinding`, 40-byte `SecurityScanResult`, 72-byte `AegisStats` (with inline 5-slot threat-counts array), 72-byte `AegisSecurityDaemon` (config + events vec + lazy-init quarantine map + lazy-init scan-history vec + inline 5-slot threat-counts array). Stat helper `_aegis_stat_modesize(path, out)` wraps `sys_stat` and pulls `STAT_MODE`/`STAT_SIZE` into a 16-byte caller-owned scratch.
  - Daemon API (cstrs for `agent_id` / `event_id` / path parameters): `aegis_new`, `aegis_report_event` (records + auto-quarantine; returns `QuarantineAction`), `aegis_recent_events`, `aegis_events_for_agent`, `aegis_events_by_threat`, `aegis_unresolved_events`, `aegis_resolve_event`, `aegis_threat_count`, `aegis_total_events`, `aegis_unresolved_count`, `aegis_quarantine_agent`, `aegis_release_agent`, `aegis_is_quarantined`, `aegis_get_quarantine`, `aegis_quarantined_agents`, `aegis_check_auto_releases`, `aegis_scan_agent` (missing / empty / world-writable / unreadable findings; respects `scan_on_execute`), `aegis_scan_package` (missing / empty / oversized >500 MB / unreadable; respects `scan_on_install`), `aegis_stats`. `_aegis_prune_events` rebuilds the events vec with the kept suffix instead of an O(n²) `vec_remove(0)` loop.
  - `src/main.cyr` — thin entry that includes `src/lib.cyr`.
- Not yet ported: `DatabaseSecurityPolicy`, `KernelTuningRecommendation`, `check_database_integrity`, `audit_ddl_operation`, `report_database_access_violation`. JSON serde, sakshi-full structured logging, agnostik UUIDs, nein firewall — all deferred (see `docs/architecture/cyrius-port-gaps.md`).

## Tests

`tests/aegis.tcyr` — 42 groups, **123 assertions, all passing** on `cyrius test tests/aegis.tcyr`:

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

Slice 4 (scans + stats):

- `scan_type_labels` — round-trip for `ST_ON_INSTALL / ON_EXECUTE / PERIODIC / MANUAL`.
- `scan_agent_missing_binary` / `scan_agent_disabled_by_config` / `scan_agent_empty_binary` / `scan_agent_world_writable` / `scan_agent_clean_file` — finding categories `missing_binary`, `empty_binary`, `world_writable`; config flag respected; clean file scans clean. Inline `_tmp_write` / `_tmp_unlink` helpers (using `file_write_all` + `sys_chmod` + `sys_unlink`) cover the parity gap noted in `docs/architecture/cyrius-port-gaps.md`.
- `scan_package_missing` / `scan_package_disabled_by_config` / `scan_package_empty` — finding categories `missing_package`, `empty_package`; oversized-package path (>500 MB threshold) exists in production but not exercised in tests.
- `scan_results_recorded_in_history` — every scan appends to `scan_history`.
- `aegis_stats_empty_daemon` / `aegis_stats_accuracy` — snapshot reflects events / unresolved / quarantined / scans-completed / per-level threat counts.

Bench / fuzz harnesses (`tests/aegis.bcyr`, `tests/aegis.fcyr`) remain stubs.

## Dependencies

Direct (declared in `cyrius.cyml`):

- stdlib — string, fmt, alloc, vec, str, syscalls, io, args, assert, tagged, chrono, hashmap

## Consumers

_None yet._

## Next

See [`roadmap.md`](roadmap.md). The first milestone is typically Rust→Cyrius surface parity for the 1893-line subset.
