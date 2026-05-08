# Rust → Cyrius port — differences and gaps

> Reference oracle: `rust-old/src/lib.rs` (1703 lines) + `rust-old/src/firewall.rs` (192 lines).
> Toolchain target: cyrius 5.10.0. (Manifest currently pins 5.9.43 — bump or accept.)
>
> Verified against `~/Repos/cyrius/lib/*.cyr` and `~/Repos/cyrius/docs/stdlib-reference.md`.
> Verified against `~/Repos/nein/` (cyrius port v1.0.0).

## Dependency map

| Rust dep | Cyrius equivalent | Status |
|----------|------------------|--------|
| `chrono` (DateTime/Duration/Utc) | `lib/chrono.cyr` — `clock_epoch_secs`, `iso8601(epoch)`, `iso8601_now`, `dur_new(secs, nsecs)`, `sleep_ms` | covered |
| `serde` + `serde_json` | `lib/json.cyr` — `json_parse`, `json_get`, `json_build`. **No derive.** | partial — every record needs hand-rolled `*_to_json` / `*_from_json` |
| `tracing` (`info!`/`warn!`/`debug!` + spans) | `lib/sakshi.cyr` (full v2.2.3 bundle — used directly, **not** via `lib/log.cyr`) | **done in 0.7.0**. Spans wrap 10 mutating daemon entry points; logfmt `key=value` messages via `_aegis_log_emit_{info,warn,debug}` + `_aegis_log_kv_*` helpers. Trampoline pattern keeps the span stack balanced across all early-return paths. Spans gated below `SK_INFO` so tests/benches at `SK_ERROR` stay silent. Note: sakshi v2.2.3's actual severity scale is `SK_FATAL=0, SK_ERROR=1, SK_WARN=2, SK_INFO=3, SK_DEBUG=4, SK_TRACE=5` — `lib/log.cyr`'s mapping comment is stale for this version. |
| `uuid` (`Uuid::new_v4`) | **agnostik** v1.0.0 `src/types.cyr` — `agent_id_new()` returns a 16-byte v4 UUID buffer (getrandom + `/dev/urandom` fallback, version/variant nibbles set, audit-reviewed Apr 2026). | **done in 0.6.0**. `[deps.agnostik]` declared in `cyrius.cyml`; `aegis_next_id` calls `agent_id_new()` and renders via the local `_aegis_uuid_to_string(buf16)` helper (heap-allocated 37-byte buffer per call, 8-4-4-4-12 hyphenated lowercase hex). |
| `nein` (firewall feature) | `~/Repos/nein` cyrius port v1.0.0 — pinned to **cyrius 4.5.0**, ~5 minor versions stale | **deferred.** Wait until nein bumps to a modern cyrius pin before porting `rust-old/src/firewall.rs`. Until then: `firewall.rs` stays in `rust-old/` as the spec, no cyrius equivalent shipped, no `[deps.nein]` in the manifest, no firewall module in `src/`. The aegis daemon ships without quarantine-via-firewall enforcement until that lands. |
| `tempfile` (dev-dep) | none | gap — tests use a manual `/tmp/aegis_test_<pid>_<n>` scratch helper |

## Type-by-type translation

| Rust | Cyrius |
|------|--------|
| `enum ThreatLevel` (5 variants, custom `Ord` so `Critical < High < … < Info`) | `enum ThreatLevel { CRITICAL = 0; HIGH = 1; MEDIUM = 2; LOW = 3; INFO = 4; }` — integer constants double as rank. Comparison `<` matches Rust's manual `Ord`. |
| `enum SecurityEventType` (12 variants) | integer constants; add `event_type_label(t) → cstr` for `Display`. |
| `enum QuarantineAction` (4) / `enum ScanType` (4) | integer constants. |
| `struct SecurityEvent { id, timestamp, event_type, source, agent_id, threat_level, description, metadata, resolved }` | record at offsets: `id@0` (Str ptr), `timestamp@8` (i64 epoch s), `event_type@16` (i64), `source@24` (Str ptr), `agent_id@32` (Str ptr or 0), `threat_level@40` (i64), `description@48` (Str ptr), `metadata@56` (map ptr), `resolved@64` (i64 0/1). Total 72 B. |
| `struct QuarantineEntry { agent_id, reason, quarantined_at, threat_level, events: Vec<String>, auto_release_at: Option<DateTime> }` | record. `events` → `vec` of Str ptrs. `auto_release_at` → tagged Option (`lib/tagged.cyr`'s `Some`/`None`) **not** epoch=0 sentinel — 1970-01-01 collides. |
| `struct SecurityFinding`, `SecurityScanResult`, `AegisConfig`, `KernelTuningRecommendation`, `DatabaseSecurityPolicy`, `AegisStats` | each a fixed-offset record with `*_new` / accessors / setters. |
| `HashMap<String, String>` (event metadata) | cstr-keyed `map_new()` from `lib/hashmap.cyr`, values are Str ptrs. **Note:** there are two hashmap flavors — `map_new()` is cstr-keyed, `map_new_str()` is `Str`-keyed (fat-pointer keys built via `str_from`). They are NOT interchangeable: a `map_new_str()` map dispatches to `hash_str_v` which calls `str_data`/`str_len` on the key, so feeding it a cstr does `load64` on chars and segfaults silently. Pick the one that matches what callers actually pass. (Hit during the quarantine slice — agent_id keys flow through as cstrs, so `map_new()`.) |
| `HashMap<ThreatLevel, usize>` (threat counts) | int-keyed not native — simplest is a 5-slot `var counts[40]` indexed by `threat_level` (0..4). Avoids the hash overhead and gives O(1) iteration. |
| `Vec<SecurityEvent>` (events log) | `vec` of event-record pointers. `vec_remove(v, 0)` per drop is O(n²) for `prune_events` — instead implement a ring-buffer or `vec_drain_front(v, n)` helper. |
| `AegisSecurityDaemon { config, events, quarantine, scan_history, threat_counts }` | record. `quarantine` is `map_new_str()` keyed by agent_id. |

## Behavioral / API divergences to expect

1. **UUID** — Rust's `Uuid::new_v4().to_string()` is `"550e8400-e29b-41d4-a716-446655440000"`. Cyrius port should match the format; otherwise event IDs are wire-incompatible with anything that round-trips through Rust callers. Acceptance: produce a 36-char string, set version nibble (byte 6 high = 0x40) and variant bits (byte 8 high two = 0b10).
2. **JSON shape** — `serde_json` field names default to struct-field names. Cyrius hand-roll must use the same keys (`id`, `timestamp`, `event_type`, `agent_id`, `threat_level`, `description`, `metadata`, `resolved`). Enum encoding: serde's default for unit variants is the variant name as a string (`"Critical"`, `"SandboxEscape"`). Match that.
3. **Timestamps** — `serde_json` writes `chrono::DateTime<Utc>` as RFC3339 like `"2026-05-08T12:46:00Z"`. `iso8601(epoch)` from `lib/chrono.cyr` produces a compatible shape — confirm exact format on first parity test.
4. **Logging payloads** — `tracing::info!(event_id = %event.id, threat = %event.threat_level, …, "msg")` becomes a `str_builder` that emits e.g. `"security event reported event_id=abc-123 threat=CRITICAL kind=sandbox_escape"`, passed to `sakshi_info(buf, len)`. Use sakshi spans (`sakshi_span_enter("report_event", 12)` / `sakshi_span_exit()`) around the public daemon entry points so log records carry the call context that `tracing` would have surfaced as `#[instrument]`. Set a per-daemon trace ID at construction (`sakshi_trace_set(id)`) for correlation. Output routing: default stderr (`sakshi_set_output_fd(2)`); operators can redirect with `sakshi_output_file(path)`. Decide on a stable key=value formatting convention up front so downstream parsers stay consistent — pick `key=value` (logfmt) since that's what tracing's text layer also emits.
5. **`std::fs::metadata` + Unix mode bits** — no `lib/fs.cyr` wrapper for stat. Use `sys_stat(path, buf)` (x86_64 syscall 4, `lib/syscalls_x86_64_linux.cyr` v5.8.6+) into a 144-byte buffer, then read offsets via the `Stat` enum (`STAT_MODE = 24`, `STAT_SIZE = 48`). World-writable check: `(mode & 0o002) != 0`. World-readable dir check: `(mode & 0o007) != 0`.
6. **`#[cfg(unix)]` blocks** — drop entirely. Linux is the only target.
7. **`#[non_exhaustive]` / `#[must_use]`** — language has neither. Remove the rules from `CLAUDE.md` post-port; forward-compat for enums is handled by reserving stable integer values and never reusing them.
8. **Error handling** — Rust `Result<T, E>` ⇒ cyrius `Result` (`lib/result.cyr`, v5.8.28) with `?` operator (v5.8.29). `Ok = 0`, `Err = 1` tag layout. For pattern `if let Some(q) = self.quarantine.get_mut(aid)` — split into `if (map_has(q, aid)) { var q = map_get(q, aid); ... }` since cyrius `map_get` returns the value directly with 0 sentinel on miss.
9. **Test framework** — no rust-style `#[test]`. Tests live in `tests/aegis.tcyr` (already scaffolded). Each test is a function called from `main`, using `assert`/`assert_eq` from `lib/assert.cyr`. `cyrius test` runs the binary, expects exit 0.
10. **Tempfile in `scan_disabled_by_config` / `scan_empty_binary_flagged` / `scan_world_writable_flagged` tests** — write a small `tmp_dir_new()` helper using `sys_mkdir` + pid + counter, plus `tmp_dir_drop()` for cleanup. ~25 LOC.
11. **`prune_events` cost** — Rust's `Vec::drain(..n)` is O(n). Cyrius `vec` only exposes `vec_pop` (O(1) tail) and `vec_remove(idx)` (O(n) shift). Drop-the-front from a 10 000-cap event log via repeated `vec_remove(v, 0)` would be O(n²). Either implement a ring-buffer record (head, tail, mask, slots) or add a `vec_drain_front` helper. Match the Rust observable behavior: oldest drops, ordering preserved.
12. **`Duration::seconds(n)` in `auto_release_at`** — `lib/chrono.cyr`'s `dur_new` is for nanosecond-resolution durations. For `auto_release_at = now + secs`, just do `clock_epoch_secs() + secs` (i64). Skip the duration record entirely.
13. **Cargo features (`firewall`)** — moot for now. nein is deferred (see dep table); the cyrius port ships without firewall enforcement until nein modernises its language pin. When it does land, cyrius has no feature flags — either always link nein or split firewall into a sibling project. Decide then.

## Missing functionality to write (estimated effort)

| Item | Lines | Notes |
|------|-------|-------|
| ~~`uuid_to_string(buf16) → Str`~~ | done | `_aegis_uuid_to_string` shipped in 0.6.0. |
| `vec_drain_front(v, n)` or ring buffer | ~40 | for `prune_events` correctness at 10k cap |
| `tmp_dir_new()` / `tmp_dir_drop(path)` | ~25 | tests/scaffolding |
| `stat_mode(path) → mode_or_-1` | ~15 | sys_stat wrapper |
| Per-record JSON serialize/deserialize | ~250 | 8 records × ~30 LOC each, mechanical against `lib/json.cyr` |
| Enum `*_label(t) → cstr` for `ThreatLevel`, `SecurityEventType`, `ScanType`, `QuarantineAction` | ~80 | matches Rust `Display` |
| `event_type_from_label(cstr) → t` (deserialize) | ~80 | inverse of above |

## Toolchain / manifest fixups

- `cyrius.cyml` pins `cyrius = "5.9.43"`; user is on 5.10.0 — bump.
- Manifest is missing several stdlib modules the port will need: `tagged`, `result`, `hashmap`, `json`, `chrono`, `random`, `sakshi`, `bench`, `fnptr`. Add them under `[deps].stdlib`. (Skipping `log` — going straight to sakshi.)
- Add `[deps.agnostik]` (git `https://github.com/MacCracken/agnostik.git`, tag `1.0.0`, modules `["src/types.cyr", "src/error.cyr"]`) for the audited UUID v4 generator and the agnos-family error record. nein pinned `0.97.1`; we want current.
- **Not** adding `[deps.agnosys]` — it's a parallel stdlib (its own `alloc.cyr`/`fs.cyr`/`syscalls_linux.cyr`/`vec.cyr`/…). nein uses it instead of the cyrius stdlib; aegis uses the cyrius stdlib, so pulling agnosys would just duplicate every primitive. Revisit only if nein returns and forces alignment.
- No `[deps.nein]` until nein's cyrius pin gets bumped from `4.5.0` to a current release.

## Logging layer (sakshi-full) — call-site shape

Pattern to use throughout the daemon, illustrated for `report_event`:

```cyr
fn aegis_report_event(d, ev) {
    sakshi_span_enter("report_event", 12);

    var sb = str_builder_new();
    str_builder_add_cstr(sb, "security event reported");
    str_builder_add_cstr(sb, " event_id=");
    str_builder_add(sb, event_id(ev));
    str_builder_add_cstr(sb, " threat=");
    str_builder_add_cstr(sb, threat_level_label(event_threat(ev)));
    str_builder_add_cstr(sb, " kind=");
    str_builder_add_cstr(sb, event_type_label(event_kind(ev)));
    var msg = str_builder_build(sb);
    sakshi_info(str_data(msg), str_len(msg));

    # ... actual reporting logic ...

    sakshi_span_exit();
    return action;
}
```

Convention:

- One span per public daemon entry point (`report_event`, `quarantine_agent`, `release_agent`, `scan_agent`, `scan_package`, `check_database_integrity`, `audit_ddl_operation`, `report_database_access_violation`, `check_auto_releases`).
- logfmt-style `key=value` pairs, space-separated, message first.
- Common keys: `event_id`, `agent_id`, `threat`, `kind`, `path`, `findings`, `released`, `target`.
- Severity mapping vs. Rust:
  - `tracing::info!` → `sakshi_info`
  - `tracing::warn!` → `sakshi_warn`
  - `tracing::debug!` → `sakshi_debug`
  - panic-class only via `sakshi_fatal`; aegis library code shouldn't panic.
