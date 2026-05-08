# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.2] — 2026-05-08

**Polish bucket — closes the remaining 0.8.x backlog.** Real fuzz targets, ADRs for the load-bearing decisions, baseline `bench-history.csv`, and a local `scripts/audit.sh` that mirrors the CI gates. Cyrius pin: `5.10.0`. No behaviour change; the audit script is the only thing consumers see.

### Added

- **`tests/aegis.fcyr` rewritten** as a real fuzz harness. 1000 random-byte iterations (length-explicit `Str` from `random_bytes`, 1–2048 bytes per iter) plus ~30 curated edge-case JSON inputs (empty, `[]`, `null`, malformed objects, wrong-type fields, truncated, whitespace-only) fed to all 8 record-from-json parsers. Pass criterion: process exits 0 within the CI's 10 s timeout — measured ~1 s locally.
- `random` to `[deps].stdlib` (for `random_bytes` from `lib/random.cyr` over `getrandom(2)`).
- **5 ADRs** in `docs/adr/`, all Accepted:
  - `0001` — Sentinel values for absent state (per-field `-1` / `0` / `QA_NONE`, JSON null in/out).
  - `0002` — Cstrs at the API boundary, `Str*` in storage; convert with `str_from`/`str_data` at the seam.
  - `0003` — Integer-array threat counts (5-slot inline) instead of an int-keyed hashmap; PascalCase string keys on the JSON wire.
  - `0004` — `map_new()` (cstr-keyed) is the project default; `map_new_str()` is **not** used in aegis source.
  - `0005` — Fixed-cap ring buffer for the events log; cap captured at `aegis_new` time.
- `bench-history.csv` baseline covering 0.7.0 → 0.8.1 with the three benches (`aegis_next_id`, `security_event_new`, `aegis_report_event`). Schema: `date,version,bench,avg_ns,min_ns,max_ns,iterations,notes`. Future versions append.
- `scripts/audit.sh` — local one-shot equivalent of `.github/workflows/ci.yml`: deps, syntax check (`--with-deps`), fmt-diff, lint, vet, DCE build, ELF magic, smoke, tests, fuzz, bench, security pattern scan, doc + version-consistency. Exits non-zero on the first failed gate.

### Notes

- The fuzz harness measures robustness against adversarial input (no crashes). Coverage isn't tracked — cyrius lacks coverage tooling. Each parser sees ~2k random + curated inputs per run.
- `bench-history.csv` is hand-maintained today. A future patch (probably 0.9.x) will add a `scripts/bench.sh` that runs the bench, parses output, and appends a row automatically.

## [0.8.1] — 2026-05-08

**Ring-buffer for the events log.** Replaces the v0.5–0.8 `vec*` + `_aegis_prune_events` rebuild (O(n) per push at cap) with a fixed-capacity ring (O(1) push, overwrite-oldest). `aegis_report_event` drops from **~220 µs → 4 µs avg at 50k iter** (≈ 55× speedup). Behaviour preserved: same observable order (oldest first), same drop-on-overflow semantics, same `aegis_total_events` answers.

### Added

- `aegis_ring_new(cap) → ring*`, `aegis_ring_push(rb, val)`, `aegis_ring_get(rb, i) → val`, `aegis_ring_len(rb)`, `aegis_ring_cap(rb)`. 32-byte header (slots/cap/head/count) + cap × 8-byte slot array. `cap <= 0` clamps to 1.
- 4 ring-specific test groups: basic push/get, overwrite-oldest at cap, iteration order after wrap, cap clamp. Total: **256 passed / 0 failed** across 73 groups.

### Changed

- `AegisSecurityDaemon.events` is now `ring*` (was `vec*`). Cap is captured from `config.max_events` at `aegis_new` time — runtime changes to `max_events` don't resize an existing daemon's ring (matches typical fixed-cap-ring practice; consumers that need to resize call `aegis_new` again).
- All callers refactored: `aegis_total_events`, `aegis_unresolved_count`, `aegis_recent_events`, `aegis_events_for_agent`, `aegis_events_by_threat`, `aegis_unresolved_events`, `aegis_resolve_event`, `aegis_report_event`. Each `vec_len`/`vec_get` became `aegis_ring_len`/`aegis_ring_get`; `vec_push + _aegis_prune_events` became single `aegis_ring_push`.

### Removed

- `_aegis_prune_events` — superseded by the ring's auto-overwrite. Dead code.

### Notes

- Memory: a daemon at default config now allocates 32 + 10000×8 = 80 KB upfront for the events ring (was: vec growing on demand). For embedded use cases on the small end, lower `max_events` before `aegis_new`.
- `scan_history` is still a `vec` and still grows unbounded — matches the rust-old behaviour. If that becomes a memory concern in production, ring-buffering it is an isolated future change.

## [0.8.0] — 2026-05-08

**JSON serde for the full record surface.** All 8 records (`SecurityEvent`, `QuarantineEntry`, `SecurityFinding`, `SecurityScanResult`, `AegisConfig`, `KernelTuningRecommendation`, `DatabaseSecurityPolicy`, `AegisStats`) gain `*_to_json` / `*_from_json` with roundtrip tests. Wire format mirrors rust-old's `serde_json` rendering. Cyrius pin: `5.10.0`.

### Added

- `json` to `[deps].stdlib`; `[deps.agnostik].modules` extended to include `src/error.cyr` (agnostik's `src/types.cyr` references `err_invalid_argument` from there in parser paths we don't call; pulling it in silences the link warning).
- 4 enum serde-label round-trips (PascalCase variant names, matching `serde`'s default for unit variants without `#[serde(rename_all = ...)]`):
  - `threat_level_serde` / `threat_level_from_serde` (`Critical / High / Medium / Low / Info`)
  - `event_type_serde` / `event_type_from_serde` (12 variants: `IntegrityViolation` … `DatabaseAccessViolation`)
  - `quarantine_action_serde` / `quarantine_action_from_serde` (`None / Suspend / Terminate / Isolate / RateLimit`)
  - `scan_type_serde` / `scan_type_from_serde` (`OnInstall / OnExecute / Periodic / Manual`)
- jv-tree helpers (`_aegis_jv_set_*`, `_aegis_jv_get_*`) layered over `lib/json.cyr`'s typed-value tree API (`json_v_obj_new`, `json_v_str_new`, `json_v_int_new`, `json_v_bool_new`, `json_v_arr_new`, `json_v_obj_set`, `json_v_arr_push`, `json_v_parse`, `json_v_build`).
- 16 record (de)serializers — for each record, both `<name>_to_json_v(rec) → json_v*` (tree) and `<name>_to_json(rec) → Str` (rendered) plus the parse-side equivalents.
- 16 new test groups, **84 new assertions** covering enum roundtrips, basic record roundtrips, edge cases (`Option::None → null` for `agent_id` / `auto_release_at` / `auto_release_timeout_secs`; nested `metadata` map; vec fields like `events` / `findings` / `kernel_tuning`; nested `threat_counts` object). Total: **239 passed / 0 failed** across 69 test groups.

### Wire format

- Field names: snake_case, exactly as the rust struct fields.
- Enum unit variants: PascalCase variant names.
- Timestamps: RFC 3339 / ISO 8601 strings via `iso8601(epoch)` / `iso8601_parse`.
- `Option<T>`: `null` when None, value otherwise.
- `HashMap<String, String>` (event metadata): nested JSON object.
- `HashMap<ThreatLevel, usize>` (threat counts): nested object with PascalCase keys.

### Notes / lessons

- `lib/json.cyr` has two key conventions, **easy to mix up**: `json_v_obj_set(obj, key, val)` stores `key` as a `Str`; `json_v_obj_get(obj, key)` looks up by **cstr**. Found this the hard way — passing `str_from(key_cstr)` to `obj_get` makes `strlen` walk the Str struct as if it were a cstr, returning a garbage length and silently missing every field. All `_aegis_jv_get_*` helpers pass cstr to `obj_get`; `_aegis_jv_set_*` wrap with `str_from` for `obj_set`.
- `aegis_audit_ddl_operation` and friends already populate `event.metadata` — the new SecurityEvent serde lights that wire path up automatically, so DB audit events ship to consumers as full nested-metadata JSON objects without extra glue.

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
