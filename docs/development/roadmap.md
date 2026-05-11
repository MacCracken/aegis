# aegis — Roadmap

> Forward-looking work only. Shipped milestones live in
> [`../../CHANGELOG.md`](../../CHANGELOG.md) — that's the historical
> record per Keep-a-Changelog. Live state lives in
> [`state.md`](state.md). Doc currency in
> [`../doc-health.md`](../doc-health.md).
>
> v1.0.0 cut on 2026-05-10. The 151-fn API at
> [`api-surface-1.0.snapshot`](api-surface-1.0.snapshot) is the
> SemVer-stable contract from this point forward.

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
