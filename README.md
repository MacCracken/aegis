# Aegis

> **Aegis** (Greek: αἰγίς — Zeus's shield) — security daemon for AGNOS.

Aegis is the central security-policy daemon for the AGNOS stack. It records security events from peer subsystems, enforces auto-quarantine when threats cross a configured severity, scans agent binaries and package archives for basic integrity violations, audits database DDL operations, and surfaces aggregate threat statistics.

Consumers: **daimon** (security-policy enforcement), **argonaut** (boot hardening).

## Status

**0.8.2** — polish bucket. Real fuzz harness (1000 random + curated inputs across all 8 JSON parsers), 5 ADRs in [`docs/adr/`](docs/adr/), `bench-history.csv` baseline, [`scripts/audit.sh`](scripts/audit.sh) mirroring the CI gates locally. Carries forward the 0.8.1 ring-buffer perf (`aegis_report_event` ≈ 4 µs avg at 50k iter), 0.8.0 JSON serde for the full record surface, 0.7.0 sakshi-full structured logging. **256 passed / 0 failed** across 73 test groups. Network-layer enforcement (firewall integration via [nein](https://github.com/MacCracken/nein)) is deferred until nein bumps its language pin to a current Cyrius release; the rust spec is preserved at [`docs/reference/firewall.rs.ref`](docs/reference/firewall.rs.ref).

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
- [`docs/development/roadmap.md`](docs/development/roadmap.md) — milestones through v1.0; remaining work for 0.9.0 / 1.0.0.
- [`docs/adr/`](docs/adr/) — Architectural Decision Records (sentinels, cstr API boundary, integer-array threat counts, hashmap flavor, ring buffer).
- [`docs/architecture/cyrius-port-gaps.md`](docs/architecture/cyrius-port-gaps.md) — non-obvious cyrius-implementation constraints found during the rust → cyrius port.
- [`bench-history.csv`](bench-history.csv) — perf baseline tracked across versions.

**Local audit**: [`./scripts/audit.sh`](scripts/audit.sh) runs every CI gate one-shot — use before pushing.

The only outstanding rust surface is [`docs/reference/firewall.rs.ref`](docs/reference/firewall.rs.ref), pending [nein](https://github.com/MacCracken/nein)'s language-version bump.

## License

GPL-3.0-only. See [LICENSE](LICENSE).
