# aegis — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**0.7.0** — sakshi-full structured logging (2026-05-08). Spans wrap 10 mutating daemon entry points; logfmt `key=value` messages emit at INFO/WARN/DEBUG mirroring the prior tracing! macros. Spans gated below `SK_INFO` so tests/benches stay silent at `SK_ERROR`. Sakshi v2.2.3's actual severity scale is `FATAL=0..TRACE=5` (`lib/log.cyr`'s inverse mapping comment is stale).

## Toolchain

- **Cyrius pin**: `5.10.0` (in `cyrius.cyml [package].cyrius`)
- **CI**: `.github/workflows/ci.yml` — toolchain install, syntax check, deps + lockfile verify, fmt-diff gate, lint (warnings fail), vet, DCE build, ELF check, aarch64 cross-build (best-effort), smoke test, full test suite, fuzz, benchmarks, security pattern scan, doc + version-consistency gates.
- **Release**: `.github/workflows/release.yml` — runs CI, verifies tag matches `VERSION`, builds x86_64 + aarch64 (best-effort), packages source tarball + `aegis-<ver>-lib.cyr` + binaries + `SHA256SUMS`, creates GitHub release with body sliced from `CHANGELOG.md`. `0.x` tags are tagged prerelease.

## Source

- Rust reference: removed in 0.6.0. Only `firewall.rs` survives at `docs/reference/firewall.rs.ref` (frozen spec for the deferred nein integration).
- Cyrius port (slices 1–5 complete, 2026-05-08): **full surface parity** with the prior `rust-old/src/lib.rs` (1893 lines) — all 13 public types and 22 daemon methods.
  - Enums: `ThreatLevel`, `SecurityEventType`, `QuarantineAction` (`QA_NONE = 0` so `report_event` returns action-or-zero), `ScanType`. All have label fns.
  - Records: 72-byte `SecurityEvent` (with lazy-init `metadata` cstr-keyed map), 56-byte `AegisConfig` (`-1` sentinel for `auto_release_timeout_secs = None`), 48-byte `QuarantineEntry`, 32-byte `SecurityFinding`, 40-byte `SecurityScanResult`, 24-byte `KernelTuningRecommendation`, 48-byte `DatabaseSecurityPolicy`, 72-byte `AegisStats`, 72-byte `AegisSecurityDaemon` (config + events vec + lazy-init quarantine map + lazy-init scan-history vec + inline 5-slot threat-counts array).
  - Helpers: `aegis_next_id` calls agnostik's `agent_id_new()` (RFC 4122 v4) and stringifies via local `_aegis_uuid_to_string(buf16)`; `_aegis_stat_modesize(path, out16)` wraps `sys_stat` (`STAT_MODE` + `STAT_SIZE`); `event_metadata_set` / `event_metadata_get` lazy-init the metadata map; `_aegis_prune_events` rebuilds the events vec with the kept suffix to avoid O(n²) `vec_remove(0)` (still O(n) per push at cap — ring buffer is the planned fix in 0.8.x).
  - Daemon API (cstrs for `agent_id` / `event_id` / path parameters): `aegis_new`, `aegis_report_event` (records + auto-quarantine; returns `QuarantineAction`), `aegis_recent_events`, `aegis_events_for_agent`, `aegis_events_by_threat`, `aegis_unresolved_events`, `aegis_resolve_event`, `aegis_threat_count`, `aegis_total_events`, `aegis_unresolved_count`, `aegis_quarantine_agent`, `aegis_release_agent`, `aegis_is_quarantined`, `aegis_get_quarantine`, `aegis_quarantined_agents`, `aegis_check_auto_releases`, `aegis_scan_agent`, `aegis_scan_package`, `aegis_stats`, `aegis_check_database_integrity`, `aegis_audit_ddl_operation`, `aegis_report_database_access_violation`, `aegis_database_kernel_recommendations`.
  - `src/main.cyr` — thin entry that includes `src/lib.cyr`.
- Still deferred (post-parity polish):
  - **0.8.0** — JSON serde (hand-rolled `*_to_json` / `*_from_json` per record).
  - **0.8.x** — ring-buffer for events (kills the O(n) prune-and-rebuild perf hit), real fuzz targets, ADRs for load-bearing decisions, `bench-history.csv` baseline, `scripts/audit.sh`. Optional: trace-ID propagation (`sakshi_trace_set`) once a wire format exists.
  - **0.9.0** — V1 prep: API-surface snapshot, full audit, doc polish.
  - **1.0.0** — first stable. API freeze. Firewall stays out per user call (lands in 1.x once nein modernises its `cyrius = "4.5.0"` pin).
  - All scoped in `docs/architecture/cyrius-port-gaps.md`.

## Tests

`tests/aegis.tcyr` — 53 groups, **155 assertions, all passing** on `cyrius test tests/aegis.tcyr`:

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

Slice 5 (database surface):

- `database_security_policy_defaults` / `database_kernel_recommendations` — defaults match rust-old (`/var/lib/postgresql/data`, `/var/lib/redis`, audit_ddl on, max 10 conns, socket-perm checks on, 4 kernel-tuning recs covering `vm.overcommit_memory` / `vm.swappiness` / `net.core.somaxconn` / `kernel.shmmax`).
- `database_integrity_check_nonexistent_dirs` / `database_integrity_check_records_scan` — quiet on missing dirs; scan recorded with target `database-services`, `ST_PERIODIC`.
- `database_integrity_world_accessible_dir` — creates `/tmp/aegis_pgdata_test` at `0o777`, points policy at it, asserts `database_permissions` finding (severity `High`).
- `audit_ddl_operation_creates_event` / `audit_ddl_operation_no_agent` — emits `EV_DATABASE_INTEGRITY` events at `THREAT_INFO` with `ddl_operation` + `ddl_object` metadata; works with `agent_id = 0` (None).
- `database_access_violation_quarantines_agent` / `database_access_violation_metadata` — `EV_DATABASE_ACCESS_VIOLATION` at `THREAT_HIGH` triggers auto-quarantine (`QA_SUSPEND`) under default config; metadata carries `database` + `violation_reason`.
- `database_event_types_distinguishable` — sanity check that the two database event-type constants differ from each other and from generic integrity.

Bench / fuzz harnesses (`tests/aegis.bcyr`, `tests/aegis.fcyr`) remain stubs.

## Dependencies

Direct (declared in `cyrius.cyml`):

- stdlib — string, fmt, alloc, vec, str, syscalls, io, args, assert, tagged, chrono, hashmap, bench, fnptr, sakshi
- agnostik (v1.0.0) — `src/types.cyr` for `agent_id_new` (UUID v4 over `getrandom`)

## Consumers

_None yet._

## Next

See [`roadmap.md`](roadmap.md). The first milestone is typically Rust→Cyrius surface parity for the 1893-line subset.
