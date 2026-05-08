# Aegis

> **Aegis** (Greek: αἰγίς — Zeus's shield) — security daemon for AGNOS.

Aegis is the central security-policy daemon for the AGNOS stack. It records security events from peer subsystems, enforces auto-quarantine when threats cross a configured severity, scans agent binaries and package archives for basic integrity violations, audits database DDL operations, and surfaces aggregate threat statistics.

Consumers: **daimon** (security-policy enforcement), **argonaut** (boot hardening).

## Status

**0.6.0** — first post-parity release. 13 public records / enums and 22 daemon entry points covered by 53 test groups (155 assertions). Event IDs are real RFC 4122 v4 UUIDs via [agnostik](https://github.com/MacCracken/agnostik)'s `agent_id_new`. Network-layer enforcement (firewall integration via [nein](https://github.com/MacCracken/nein)) is deferred until nein bumps its language pin to a current Cyrius release; the rust spec is preserved at [`docs/reference/firewall.rs.ref`](docs/reference/firewall.rs.ref).

## Quick Start

```sh
cyrius deps                              # resolve dependencies
cyrius build src/main.cyr build/aegis    # compile the daemon
cyrius test tests/aegis.tcyr             # run the test suite
./build/aegis                            # run the daemon (currently prints "aegis ready")
```

The library API lives in `src/lib.cyr`; consumers pull it via `[deps.aegis] modules = ["src/lib.cyr"]` in their own `cyrius.cyml`.

## Library Surface

Records:

| Record | Purpose |
|--------|---------|
| `SecurityEvent` | One reported security event (id, timestamp, kind, source, agent_id, threat_level, description, metadata, resolved). |
| `QuarantineEntry` | One quarantined agent (id, reason, threat_level, linked event ids, optional auto-release-at). |
| `SecurityFinding` / `SecurityScanResult` | A finding within a scan and its containing result. |
| `AegisConfig` | Daemon configuration (scan toggles, max events, auto-release timeout). |
| `DatabaseSecurityPolicy` / `KernelTuningRecommendation` | Database integrity policy and kernel-tuning advice for argonaut. |
| `AegisStats` | Snapshot of aggregate counters. |
| `AegisSecurityDaemon` | Top-level daemon record. |

Enums (integer constants — see `src/lib.cyr`):

- `ThreatLevel` (`THREAT_CRITICAL` … `THREAT_INFO`, lower number = more severe)
- `SecurityEventType` (12 categories)
- `QuarantineAction` (`QA_NONE` sentinel + `SUSPEND` / `TERMINATE` / `ISOLATE` / `RATELIMIT`)
- `ScanType` (`ON_INSTALL` / `ON_EXECUTE` / `PERIODIC` / `MANUAL`)

Daemon API (cstrs for `agent_id` / `event_id` / path parameters):

- Construction & event reporting: `aegis_new`, `aegis_report_event`
- Event queries: `aegis_recent_events`, `aegis_events_for_agent`, `aegis_events_by_threat`, `aegis_unresolved_events`, `aegis_resolve_event`
- Threat counting: `aegis_threat_count`, `aegis_total_events`, `aegis_unresolved_count`
- Quarantine: `aegis_quarantine_agent`, `aegis_release_agent`, `aegis_is_quarantined`, `aegis_get_quarantine`, `aegis_quarantined_agents`, `aegis_check_auto_releases`
- Scanning: `aegis_scan_agent`, `aegis_scan_package`
- Database surface: `aegis_check_database_integrity`, `aegis_audit_ddl_operation`, `aegis_report_database_access_violation`, `aegis_database_kernel_recommendations`
- Stats: `aegis_stats`

## Project Layout

```
src/
  lib.cyr        — library surface (records, accessors, daemon API)
  main.cyr       — daemon entry point
tests/
  aegis.tcyr     — test suite (cyrius test)
  aegis.bcyr     — benchmarks
  aegis.fcyr     — fuzz harness
docs/
  architecture/  — non-obvious constraints (Rust → Cyrius port gaps)
  development/   — state.md (live), roadmap.md
  adr/           — Architectural Decision Records
  guides/        — task-oriented how-tos
  reference/     — frozen reference material (firewall.rs.ref — pending nein modernisation)
```

## Documentation

- [`docs/development/state.md`](docs/development/state.md) — live project state (refreshed every release).
- [`docs/development/roadmap.md`](docs/development/roadmap.md) — milestones through v1.0.
- [`docs/architecture/cyrius-port-gaps.md`](docs/architecture/cyrius-port-gaps.md) — Rust → Cyrius translation map; what's done, what's deferred (sakshi-full structured logging, agnostik UUIDs, JSON serde, nein firewall).

## License

GPL-3.0-only. See [LICENSE](LICENSE).
