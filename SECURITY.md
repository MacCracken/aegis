# Security Policy

## Scope

Aegis is the central security-policy daemon for AGNOS. It records security events from peer subsystems, enforces auto-quarantine on high-severity events, scans agent binaries / package archives, and audits database operations. Every public function either mutates security-critical state (event log, quarantine map, scan history) or reports it.

## Attack Surface

| Area | Risk | Mitigation |
|------|------|------------|
| Event reporting | Hostile agent floods events to push older records out of the cap | Fixed-cap ring buffer (`config.max_events`) — O(1) push, oldest deterministically dropped. Auto-quarantine policy is evaluated **before** the slot overwrite, so a flood can still trip a Critical-tier quarantine on the inbound event. |
| Auto-quarantine policy | Bypass via missing `agent_id` on a Critical event | Critical / High events without an `agent_id` are recorded but **never** auto-quarantine. Threat counters and event log still update so an out-of-band investigator sees the activity. |
| Quarantine escalation | Re-quarantine downgrades the threat level | `aegis_quarantine_agent` only escalates (`new < existing`), never downgrades. Reasons append (`"old; new"`) so audit history is preserved. |
| Scan findings | Path traversal in scan target description; symlinks, FIFOs and other non-files at the scan path | Targets are formatted as `agent:<id>` / `package:<path>`. A path is checked with `lstat`, then opened `O_NOFOLLOW \| O_NONBLOCK` and `fstat`ed (the stdlib's per-arch `O_NOFOLLOW`; 1.2.1's x86_64 literal followed symlinks on aarch64). A symlink is `unreadable_metadata` and a FIFO, socket, device or directory is `not_regular_file`. A FIFO cannot block the scan. Findings are surfaced as data, not actioned. |
| World-writable / world-accessible checks | Wrong mask, false negatives | `mode & 0o002` for world-writable; `mode & 0o007` for any-other-bit on database directories. Test coverage: `scan_agent_world_writable` and `database_integrity_world_accessible_dir` create real fixtures with the relevant modes. |
| Database access violation | Unauthorized cross-tenant DB access slips through | `aegis_report_database_access_violation` raises a `THREAT_HIGH` event; default config quarantines High → `QA_SUSPEND`. |
| Event IDs | Predictable / replayable IDs | RFC 4122 v4 UUIDs via agnostik's `agent_id_new` (kernel `getrandom` + `/dev/urandom` fallback). Audit-reviewed (agnostik F-001, 2026-04-26 — deterministic-fallback removed). |
| Firewall rulesets | A value that nft parses differently than intended: a host name (resolved through DNS at apply time), a wide prefix, an IPv6 address in an `ip` match | The builders accept only a literal IPv4 / IPv6 address naming one host (no prefix, `/32`, `/128`), validated with nein's `validate_addr`, and a table name that passes nein's `validate_nft_name`. IPv6 renders `ip6 daddr` / `ip6 saddr`. `aegis_firewall_render` returns `0` for anything that does not validate (ADR 0006). |
| Event sink (`AEGIS_EVENT_LOG`) | A privileged appender aimed elsewhere: a symlink or hard link planted at the path, a FIFO, a file another user created first | Opened `O_NOFOLLOW \| O_NONBLOCK`, then required to be a regular file owned by the daemon's euid with one link. A record is one `write` under `O_APPEND`; after a short write, the next record starts on a new line. |
| PAM config audit | A stack Linux-PAM reads differently from aegis (line continuation, mid-line `#`, `[...]` shape); truncated reads; overflowing uids | Lines are assembled as Linux-PAM's `pam_line.c` does. Security files are read whole, or refused, never truncated. A uid / gid must be 1–10 digits and at most 4294967294. Rendering fails closed on any field outside its allowlist or range (ADR 0006). |
| In-process metadata maps | Hostile cstr keys outliving their backing buffer | Map values are `Str*` (heap-stable). Keys are cstrs that callers are expected to keep alive (string literals are static; dynamic keys must be kept by the caller until the map is dropped). |

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.2.x | Yes (current) |
| 1.1.x | Yes (previous minor) |
| < 1.1 | No |

## Reporting a Vulnerability

Email: **cyriusmaccken@gmail.com**

Please **do not** open public GitHub issues for security vulnerabilities. Include:

- Affected version (`VERSION` file).
- Reproducer (input, expected behavior, observed behavior).
- Whether the issue requires elevated privileges to exploit.

We will acknowledge within 72 hours and aim to ship a fix within 14 days for High / Critical severity.

## Out of Scope

- Cryptographic primitives — aegis enforces policy, **not** cryptography. All crypto belongs in [sigil](https://github.com/MacCracken/sigil).
