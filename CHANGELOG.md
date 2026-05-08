# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.0] — 2026-05-08

**Sakshi-full structured logging.** Spans wrap every mutating daemon entry point; severity-tagged logfmt-style messages emit on the major transitions (event reported, auto-quarantine, manual quarantine, release, scan skipped/started, db-integrity findings). Cyrius pin: `5.10.0`.

### Added

- `sakshi` to `[deps].stdlib` (uses the bundled v2.2.3 distribution).
- `_aegis_span_enter` / `_aegis_span_exit` wrappers — gate sakshi spans on the active level (`< SK_INFO` ⇒ skip), so tests/benches at `SK_ERROR` stay quiet without redirecting sakshi's output fd.
- `_aegis_log_emit_{info,warn,debug}` and `_aegis_log_kv_{cstr,str,int}` helpers for logfmt-style `"<msg> key=val key=val"` construction.
- Span + structured logging in 10 daemon entry points: `aegis_report_event`, `aegis_resolve_event`, `aegis_quarantine_agent`, `aegis_release_agent`, `aegis_check_auto_releases`, `aegis_scan_agent`, `aegis_scan_package`, `aegis_check_database_integrity`, `aegis_audit_ddl_operation`, `aegis_report_database_access_violation`. Each public fn is a thin trampoline: `_aegis_span_enter("…") → _<name>_inner(…) → _aegis_span_exit()` so no early-return path leaves the span stack unbalanced.
- `src/main.cyr` initialises sakshi at `SK_INFO` to stderr (operators can switch to a file later via `sakshi_output_file` once a config surface lands).
- Tests + benches set `sakshi_set_level(SK_ERROR)` at startup to keep stderr clean.

### Severity mapping (mirrors prior tracing! macros)

- `INFO` — events reported, agent released, db-integrity findings detected.
- `WARN` — auto-quarantine fires; quarantine-severity event without `agent_id`; manual quarantine.
- `DEBUG` — agent already quarantined (link/update); scan skipped (config disabled); scan started; db-integrity check passed.
- Library code never calls `sakshi_error` / `sakshi_fatal`.

### Notes

- Sakshi v2.2.3's actual severity values are `SK_FATAL=0, SK_ERROR=1, SK_WARN=2, SK_INFO=3, SK_DEBUG=4, SK_TRACE=5` — the comment in `lib/log.cyr` showing the inverse mapping is stale for this version. Filter is `emit if msg_severity ≤ active_level`.
- Bench impact: `aegis_report_event` ≈ 229 µs avg at `SK_ERROR` (no logging fires; the prune-and-rebuild dominates). At `SK_INFO` the per-call cost is dominated by `str_builder_*` + sakshi formatting — defer measurement until the ring-buffer perf fix lands in 0.8.x.
- One bug worth flagging: the wrapper `_aegis_span_exit` initially recursed because of an over-broad replace-all. Fix is documented; the wrapper now correctly delegates to `sakshi_span_exit`.

## [0.6.0] — 2026-05-08

**Cleanup + real UUIDs.** First post-parity slice. Cyrius pin: `5.10.0`.

### Added

- `[deps.agnostik]` (v1.0.0) — pulls `src/types.cyr` for the audit-reviewed `agent_id_new()` (RFC 4122 v4 over `getrandom` with `/dev/urandom` fallback).
- `_aegis_uuid_to_string(buf16)` — renders agnostik's 16-byte v4 UUID as a 36-char hyphenated lowercase hex string. Heap-allocated per call (no static-buffer aliasing across consecutive ids).

### Changed

- **Event IDs are real v4 UUIDs.** `aegis_next_id()` now produces `550e8400-e29b-41d4-a716-446655440000`-shaped strings instead of the placeholder `ev-1` / `ev-2` counter. Wire-format ready; collision-resistant.
- Removed the `_aegis_id_counter` global.

### Removed

- `rust-old/` is gone. `firewall.rs` was relocated to `docs/reference/firewall.rs.ref` as the spec for the (still-deferred) nein integration; the rest of the rust scaffolding (Cargo.lock, Cargo.toml, codecov.yml, deny.toml, rust-toolchain.toml, src/lib.rs, LINES_OF_RUST.txt) was deleted.

### Notes

- Bench impact: `aegis_next_id` ≈ 2 µs avg (was ≈ 1 µs with the counter). Extra microsecond is the `getrandom` syscall + hex formatting — within noise, not worth caching.
- aarch64 cross-build: agnostik's `_fill_random` hardcodes the x86_64 `getrandom` syscall number (318). On aarch64 it would be 278. CI's aarch64 cross-build is best-effort; expect a runtime path issue if the cross-build succeeds and is exercised. Will be addressed when agnostik gains arch dispatch.

## [0.5.0] — 2026-05-08

**Initial Cyrius release.** Full surface parity with the prior Rust scaffold (`rust-old/src/lib.rs`, 1893 lines). Cyrius pin: `5.10.0`.

### Added

- **Records**: `SecurityEvent` (72 B), `QuarantineEntry` (48 B), `SecurityFinding` (32 B), `SecurityScanResult` (40 B), `AegisConfig` (56 B), `KernelTuningRecommendation` (24 B), `DatabaseSecurityPolicy` (48 B), `AegisStats` (72 B), `AegisSecurityDaemon` (72 B). All accessors, all setters that the rust API exposed.
- **Enums** (integer-constant style): `ThreatLevel`, `SecurityEventType` (12 variants), `QuarantineAction` (with `QA_NONE = 0` so `aegis_report_event` returns action-or-zero), `ScanType`. All have label fns matching rust-old's `Display` / serde rendering.
- **Daemon API** (22 entry points covering all rust-old methods):
  - Event reporting + auto-quarantine: `aegis_new`, `aegis_report_event`.
  - Event queries: `aegis_recent_events`, `aegis_events_for_agent`, `aegis_events_by_threat`, `aegis_unresolved_events`, `aegis_resolve_event`.
  - Counters: `aegis_threat_count`, `aegis_total_events`, `aegis_unresolved_count`.
  - Quarantine management: `aegis_quarantine_agent`, `aegis_release_agent`, `aegis_is_quarantined`, `aegis_get_quarantine`, `aegis_quarantined_agents`, `aegis_check_auto_releases`.
  - Scanning: `aegis_scan_agent`, `aegis_scan_package`.
  - Database surface: `aegis_check_database_integrity`, `aegis_audit_ddl_operation`, `aegis_report_database_access_violation`, `aegis_database_kernel_recommendations`.
  - Snapshot: `aegis_stats`.
- **Tests**: `tests/aegis.tcyr` — 53 test groups, 155 assertions, all passing on `cyrius test`. Inline `_tmp_write` / `_tmp_unlink` helpers cover the empty-binary / world-writable scan-agent paths.
- **Docs**: `README.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, `docs/architecture/cyrius-port-gaps.md`, `docs/development/state.md`, `docs/development/roadmap.md`.

### Changed

- `cyrius.cyml` `version` is now sourced from `VERSION` via `${file:VERSION}` (single source of truth).

### Notes / Deferred

- **Network enforcement**: `firewall.rs` (rust) is preserved at `rust-old/src/firewall.rs` as the spec; the cyrius port is deferred until [nein](https://github.com/MacCracken/nein) bumps its language pin from `4.5.0` to a current Cyrius release.
- **Counter-backed event IDs**: `aegis_next_id` returns `ev-1`, `ev-2`, etc. To be replaced by [agnostik](https://github.com/MacCracken/agnostik)'s audit-reviewed `agent_id_new()` (RFC 4122 v4 over `getrandom`) post-release.
- **Logging**: ad-hoc `str_builder`-formed messages today. Switching to full sakshi (spans + trace IDs + structured fields) is scoped in `docs/architecture/cyrius-port-gaps.md`.
- **Wire format**: no JSON serde yet. Records are in-process only; per-record `*_to_json` / `*_from_json` hand-rolls land when a consumer needs wire interop.
