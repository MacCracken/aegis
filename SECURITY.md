# Security Policy

## Scope

Aegis is the central security-policy daemon for AGNOS. It records security events from peer subsystems, enforces auto-quarantine on high-severity events, scans agent binaries / package archives, and audits database operations. Every public function either mutates security-critical state (event log, quarantine map, scan history) or reports it.

## Attack Surface

| Area | Risk | Mitigation |
|------|------|------------|
| Event reporting | Hostile agent floods events to push older records out of the cap | Fixed-cap ring buffer (`config.max_events`) — O(1) push, oldest deterministically dropped. Auto-quarantine policy is evaluated **before** the slot overwrite, so a flood can still trip a Critical-tier quarantine on the inbound event. |
| Auto-quarantine policy | Bypass via missing `agent_id` on a Critical event | Critical / High events without an `agent_id` are recorded but **never** auto-quarantine. Threat counters and event log still update so an out-of-band investigator sees the activity. |
| Quarantine escalation | Re-quarantine downgrades the threat level | `aegis_quarantine_agent` only escalates (`new < existing`), never downgrades. Reasons append (`"old; new"`) so audit history is preserved. |
| Scan findings | Path traversal in scan target description | Targets are formatted as `agent:<id>` / `package:<path>`; no path is dereferenced through user input beyond `file_exists` + `sys_stat`. Findings are surfaced as data, not actioned. |
| World-writable / world-accessible checks | Wrong mask, false negatives | `mode & 0o002` for world-writable; `mode & 0o007` for any-other-bit on database directories. Test coverage: `scan_agent_world_writable` and `database_integrity_world_accessible_dir` create real fixtures with the relevant modes. |
| Database access violation | Unauthorized cross-tenant DB access slips through | `aegis_report_database_access_violation` raises a `THREAT_HIGH` event; default config quarantines High → `QA_SUSPEND`. |
| Event IDs | Predictable / replayable IDs | RFC 4122 v4 UUIDs via agnostik's `agent_id_new` (kernel `getrandom` + `/dev/urandom` fallback). Audit-reviewed (agnostik F-001, 2026-04-26 — deterministic-fallback removed). |
| In-process metadata maps | Hostile cstr keys outliving their backing buffer | Map values are `Str*` (heap-stable). Keys are cstrs that callers are expected to keep alive (string literals are static; dynamic keys must be kept by the caller until the map is dropped). |

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.8.x | Yes (current) |
| 0.7.x | Yes (previous minor) |
| < 0.7 | No |

## Reporting a Vulnerability

Email: **cyriusmaccken@gmail.com**

Please **do not** open public GitHub issues for security vulnerabilities. Include:

- Affected version (`VERSION` file).
- Reproducer (input, expected behavior, observed behavior).
- Whether the issue requires elevated privileges to exploit.

We will acknowledge within 72 hours and aim to ship a fix within 14 days for High / Critical severity.

## Out of Scope

- Network enforcement (firewall integration via `nein`) — not yet in cyrius. The frozen rust spec at `docs/reference/firewall.rs.ref` is the parity oracle for that work; the cyrius port lands when nein modernises its language pin.
- Cryptographic primitives — aegis enforces policy, **not** cryptography. All crypto belongs in [sigil](https://github.com/MacCracken/sigil).
