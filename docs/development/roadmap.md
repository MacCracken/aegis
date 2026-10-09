# aegis — Roadmap

> Forward-looking work only. Shipped milestones live in
> [`../../CHANGELOG.md`](../../CHANGELOG.md) — that's the historical
> record per Keep-a-Changelog. Live state lives in
> [`state.md`](state.md). Doc currency in
> [`../doc-health.md`](../doc-health.md).
>
> v1.0.0 cut on 2026-05-10. The API at
> [`api-surface.snapshot`](api-surface.snapshot) is the SemVer-stable
> contract from that point forward — **214 fns** as of the 1.1.6
> regeneration. Additions are non-breaking; removals and renames need a
> major bump.

## Open work (post-1.0)

Each item is non-blocking — v1.0 ships without it. Pulled forward when a
real consumer surfaces the requirement, when an upstream stdlib change
lands, or when a P(-1) follow-up cycle batches them.

### Consumer integration

- **Real downstream consumer end-to-end**: daimon or argonaut consuming
  `src/lib.cyr` in production, including a quarantine path that calls
  `aegis_isolate_agent` and applies the rendered ruleset. The
  `docs/examples/basic_consumer.cyr` stand-in exercises the surface but
  isn't a production consumer. When the real one lands, decide whether the
  example stays as a teaching artifact or gets deprecated.

### Audit follow-ups (deeper fixes for items closed at the boundary in 0.9.x)

- **F-8 deeper fix** — JSON parser depth cap belongs in `lib/json.cyr`
  upstream (cyrius stdlib). Aegis ships an input-length cap as a partial
  mitigation today (`AEGIS_JSON_MAX_BYTES = 262_144` at every
  `*_from_json` seam). Tracked in `docs/audit/2026-05-10-audit.md` F-8.
- **F-6 deeper fix** — pass open fd to consumer via `SecurityFinding` so
  the consumer's action operates on the same inode aegis stat'd. Closes
  the consumer-side TOCTOU window. Adds API surface, so it's a v1.x
  feature held until a real consumer surfaces the requirement.

### Stdlib migrations (currently shimmed locally)

- **`O_NOFOLLOW` → `lib/syscalls_*_linux.cyr`**: defined locally as
  `_AEGIS_O_NOFOLLOW = 131072` in `src/lib.cyr` with an upstream-this
  comment. Single grep target. Land the constant in cyrius stdlib, then
  swap the local for the global.

### Observability

- **Trace-ID propagation** (`sakshi_trace_set` for cross-process
  correlation). Useful once a multi-process wire flow exists between
  aegis and a consumer — currently aegis logs are single-daemon.

### Tooling

- **`scripts/bench.sh`** — auto-append `bench-history.csv` after each
  bench run. Currently hand-maintained; not a blocker for any release.

## What this file is NOT

- Not a CHANGELOG (that's [`../../CHANGELOG.md`](../../CHANGELOG.md)).
- Not a state snapshot (that's [`state.md`](state.md)).
- Not a v1.0 sign-off checklist — that ran during the 1.0.0 cut and
  passed; the verification record is in CHANGELOG `[1.0.0] § Verified at sign-off`.

When a v1.x or v2.0 milestone is being planned, add a section above with
the scope and a few load-bearing requirements. Don't pre-populate empty
buckets ("Future") — those rot. Add work items here only when they're
concrete enough to act on.

## Moving the cyrius pin to 6.6.6

Current pin: `cyrius = "6.6.2"` (`cyrius.cyml`).

aegis is the repo in this group whose correctness argument leans hardest on `O_APPEND`, so the
Windows half of 6.6.6 is worth reading even though it does not currently reach here. The audit
log opens at `src/lib.cyr:1020` with a raw
`sys_open(path_cstr, _AEGIS_O_WRONLY | _AEGIS_O_CREAT | _AEGIS_O_APPEND | O_CLOEXEC, 384)`
(`_AEGIS_O_APPEND = 1024`, `src/lib.cyr:1011`), and the comment immediately above it at
`:1002-1005` states the invariant plainly: *"O_APPEND is what makes it safe without a lock …
the record and its newline are written in ONE sys_write."* Before 6.6.6 a PE build's `O_APPEND`
did not append — it overwrote from offset 0 — which would have destroyed that invariant and
the security log with it.

**aegis is not exposed today**, and that was checked rather than assumed: no PE target
(`CYRIUS_TARGET_AGNOS` is the only `CYRIUS_TARGET_*` in `src/`), CI is `ubuntu-latest` only,
and `release.yml` ships x86_64 + aarch64 with no `windows-*` runner or `.exe` asset. The
practical instruction is forward-looking: **do not build aegis for Windows on a cyrius below
6.6.6.** If a PE target is ever added, 6.6.6 is the floor, and this is the first thing to
re-verify.

Everything else is a plain pin bump. What was checked, and found empty:

- The only other opens in `src/` are read-only — `src/pam.cyr:685`, `:703`, `:877`, and
  `src/lib.cyr:1063`, all `O_RDONLY | O_CLOEXEC | _PAM_O_NOFOLLOW`. No `O_TRUNC` anywhere
  outside the vendored `lib/`.
- None of the new compile errors in item 3 have sites: no `async fn`, no `operator` fn, no
  `ret2`/`rethi`, no SIMD intrinsics, and no struct- or vector-typed parameter or `var`
  declaration in `src/`. The three structs (`pam_rule`, `pam_user`, `pam_session`,
  `src/pam.cyr:255`/`:279`/`:320`) are accessor-style over heap offsets and are never passed or
  assigned by value, so item 5's by-value-struct-param deep copy is a no-op here as well.
- No top-level bare `{` blocks (item 4), no duplicated global declarations (item 6), no
  `regression_*` call sites (item 8), and no own `vec_*` function colliding with the 14 names
  `lib/vec.cyr` exports (item 9), so the new transitive `lib/assert.cyr` → `lib/vec.cyr`
  include changes nothing.
- 7 `file_exists` / `file_read_all` call sites — those gain read-only-volume support only on
  PE, so no change on the targets aegis actually ships.

After bumping, verify the audit-log append path directly: write two records, restart the
daemon, write a third, and confirm all three are present in order. That is cheap and it is the
one behaviour this repo cannot afford to get wrong.

## Moved from cyrius's roadmap (2026-10-08) — for the next cyrius pin move

cyrius no longer carries this repo's follow-ups; the items below were moved here from its roadmap. Each
applies from the cyrius version it names. Everything else that changes between this repo's pin and cyrius
6.7.6 is in cyrius's `CHANGELOG.md` and `docs/development/ecosystem-migration.md`.

- **Same-arity duplicate fns** — cyrius's 6.6.3 ecosystem sweep (2026-09-12) recorded **1** same-arity
  duplicate-fn warning(s) in this repo's build (not re-derived since). Since cyrius **6.6.20** every call binds
  the LAST definition (before, a non-tail call to an already-defined fn bound the first), so a duplicate whose
  body differs changes what runs at the pin bump. Remove the duplicates or confirm the last definition is the
  intended one.
