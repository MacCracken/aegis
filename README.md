# Aegis

> **Aegis** (Greek: αἰγίς — Zeus's shield) — security daemon for AGNOS.

Aegis is the central security-policy daemon for the AGNOS stack. It records security events from peer subsystems, enforces auto-quarantine when threats cross a configured severity, scans agent binaries and package archives for basic integrity violations, audits database DDL operations, and surfaces aggregate threat statistics.

Consumers: **daimon** (security-policy enforcement), **argonaut** (boot hardening).

## Status

**1.0.0** — first stable. The 151-fn public API surface at [`docs/development/api-surface-1.0.snapshot`](docs/development/api-surface-1.0.snapshot) is the SemVer-stable contract; additions are non-breaking, removals or renames need a major bump. Ships the full surface built across 0.5.0 → 0.9.5: nein firewall integration (`aegis_isolate_agent` / `aegis_rate_limit_agent` / `aegis_hardened_host` + render/validate wrappers), JSON serde for all 8 records, sakshi-full structured logging, fixed-cap ring-buffer events log (`aegis_report_event` ≈ 4 µs avg at 50k iter), boundary-validated API (whitelist on `agent_id` + `agent_addr`; clamps on JSON-deserialized config; no-follow-symlink scanner). All 9 P(-1) audit findings closed — see [`docs/audit/2026-05-10-audit.md`](docs/audit/2026-05-10-audit.md). **326 passed / 0 failed** across 92 test groups + 1000-iter fuzz on every JSON parser.

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

- Construction & event reporting: `aegis_new`, `aegis_next_id`, `aegis_report_event`
- Event queries: `aegis_recent_events`, `aegis_events_for_agent`, `aegis_events_by_threat`, `aegis_unresolved_events`, `aegis_resolve_event`
- Threat counting: `aegis_threat_count`, `aegis_total_events`, `aegis_unresolved_count`
- Quarantine: `aegis_quarantine_agent`, `aegis_release_agent`, `aegis_is_quarantined`, `aegis_get_quarantine`, `aegis_quarantined_agents`, `aegis_check_auto_releases`
- Scanning: `aegis_scan_agent`, `aegis_scan_package`
- Database surface: `aegis_check_database_integrity`, `aegis_audit_ddl_operation`, `aegis_report_database_access_violation`, `aegis_database_kernel_recommendations`
- Stats: `aegis_stats`

Firewall (nein integration — `src/firewall.cyr`, exercised via `QA_ISOLATE` / `QA_RATELIMIT` quarantine actions):

- Builders: `aegis_isolate_agent(agent_id, agent_addr)`, `aegis_rate_limit_agent(agent_id, agent_addr, pps)`, `aegis_hardened_host()`
- Wrappers: `aegis_firewall_render(fw)` returns `Str*` (nftables source); `aegis_firewall_validate(fw)` returns `0` (ok) / `1` (invalid)

Ring primitive (events log; cap captured at `aegis_new` time — see [ADR 0005](docs/adr/0005-fixed-cap-ring-buffer-events-log.md)):

- `aegis_ring_new`, `aegis_ring_push`, `aegis_ring_get`, `aegis_ring_len`, `aegis_ring_cap`

JSON serde — every record gains `<name>_to_json` / `<name>_from_json` (rendered) plus `<name>_to_json_v` / `<name>_from_json_v` (typed-value tree). Wire format is consumed by daimon / argonaut; field names are snake_case and enum variants are PascalCase.

The full machine-checkable surface (151 public fns — the v1.0 SemVer-stable contract) lives at [`docs/development/api-surface-1.0.snapshot`](docs/development/api-surface-1.0.snapshot); CI gates additions/removals against it via [`scripts/check-api-surface.sh`](scripts/check-api-surface.sh).

## Project Layout

```
src/
  lib.cyr        — library surface (records, accessors, daemon API)
  firewall.cyr   — nein firewall builders for QA_ISOLATE / QA_RATELIMIT
  main.cyr       — daemon entry point
tests/
  aegis.tcyr     — test suite (cyrius test)
  aegis.bcyr     — benchmarks
  aegis.fcyr     — fuzz harness
docs/
  architecture/  — non-obvious constraints (cyrius-stdlib gotchas surfaced during the port)
  development/   — state.md (live), roadmap.md
  adr/           — Architectural Decision Records
  guides/        — task-oriented how-tos
```

## Documentation

- [`docs/development/state.md`](docs/development/state.md) — live project state (refreshed every release).
- [`docs/development/roadmap.md`](docs/development/roadmap.md) — milestones through v1.0; remaining work toward 1.0.0.
- [`docs/adr/`](docs/adr/) — Architectural Decision Records (sentinels, cstr API boundary, integer-array threat counts, hashmap flavor, ring buffer).
- [`docs/architecture/001-cyrius-port-gaps.md`](docs/architecture/001-cyrius-port-gaps.md) — non-obvious cyrius-implementation constraints found during the rust → cyrius port.
- [`docs/doc-health.md`](docs/doc-health.md) — living ledger of doc currency (fresh / stale / archived / open-question), refreshed when docs are touched.
- [`docs/examples/`](docs/examples/) — runnable consumer examples.
- [`bench-history.csv`](bench-history.csv) — perf baseline tracked across versions.

**Local audit**: [`./scripts/audit.sh`](scripts/audit.sh) runs every CI gate one-shot — use before pushing.

## License

GPL-3.0-only. See [LICENSE](LICENSE).
