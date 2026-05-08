# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
