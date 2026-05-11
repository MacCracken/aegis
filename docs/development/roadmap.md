# aegis — Roadmap

> Sequencing through v1.0. Live state lives in [`state.md`](state.md);
> this file is the milestone plan — what ships, in what order, against
> what dependency gates.

## v1.0 criteria

- [x] Surface parity with the prior Rust scaffold (13 records/enums + 22 daemon methods)
- [x] Test suite covering every public method (≥ 250 assertions; targeted edge cases)
- [x] Real fuzz harness against malformed input (no crashes in 1000 random + curated inputs)
- [x] JSON wire format compatible with the consuming daimon / argonaut stack
- [x] Structured logging (sakshi-full spans + logfmt) on every mutating entry point
- [x] CHANGELOG complete from 0.5.0 onward (Keep-a-Changelog format)
- [x] CI green: build, test, fuzz, bench, fmt, lint, vet, security scan, doc gate
- [x] Local audit script (`scripts/audit.sh`) mirrors CI gates one-shot
- [x] `bench-history.csv` baseline so perf regressions surface
- [x] ADRs for every load-bearing design decision (sentinels, cstr API, hashmap flavor, ring buffer)
- [x] **0.9.0 nein firewall integration** — `aegis_isolate_agent` / `aegis_rate_limit_agent` / `aegis_hardened_host` via nein 1.5.0; `QA_ISOLATE` / `QA_RATELIMIT` are no longer placeholder actions
- [x] **0.9.1 rust scaffold retired** — `docs/reference/firewall.rs.ref` deleted; supporting "do not modify the rust spec" guidance removed across CLAUDE.md / CONTRIBUTING.md / SECURITY.md / README.md / docs / inline comments
- [x] **0.9.2 V1 prep** — API surface snapshot + CI gate (`scripts/check-api-surface.sh`); doc-health ledger (`docs/doc-health.md`); README API list polished to cover all 151 public fns; example consumer at `docs/examples/basic_consumer.cyr` exercising the surface end-to-end (stand-in until daimon/argonaut consume directly)
- [ ] **1.0.0 release** — clean review/audit before cut; no new deliverables

## Shipped milestones

| Version | Theme | Date |
|---------|-------|------|
| **0.5.0** | First cyrius release — full Rust→Cyrius surface parity | 2026-05-08 |
| **0.6.0** | Cleanup + real UUIDs (rust-old removed; agnostik `agent_id_new`) | 2026-05-08 |
| **0.7.0** | Sakshi-full structured logging (spans + logfmt on 10 entry points) | 2026-05-08 |
| **0.8.0** | JSON serde for the full record surface | 2026-05-08 |
| **0.8.1** | Ring-buffer events log (`aegis_report_event` ~220 µs → 4 µs avg) | 2026-05-08 |
| **0.8.2** | Polish — fuzz harness, ADRs, `bench-history.csv`, `scripts/audit.sh` | 2026-05-08 |
| **0.8.3** | Toolchain + dep refresh: cyrius `5.10.0` → `5.10.34`, agnostik `1.0.0` → `1.2.1`, `lib/` gitignored, versioned CI toolchain layout | 2026-05-10 |
| **0.9.0** | Nein firewall integration — `aegis_isolate_agent` / `aegis_rate_limit_agent` / `aegis_hardened_host` against nein 1.5.0; `QA_ISOLATE` / `QA_RATELIMIT` actions become real | 2026-05-10 |
| **0.9.1** | Rust scaffold retired — `docs/reference/firewall.rs.ref` and supporting guidance fully removed | 2026-05-10 |
| **0.9.2** | V1 prep — API surface CI gate, doc-health ledger, README API polish, example consumer | 2026-05-10 |

## Upcoming milestones

### M10 — First stable (v1.0.0)

Clean review / audit pass before the cut — no new deliverables. Confirms:

- Audit script green (every gate, zero warnings tolerated).
- API surface snapshot matches `cyrius api-surface --scope=project`; no in-flight drift.
- doc-health ledger has zero rows in the 🟡 stale bucket.
- All 5 ADRs are still Accepted and the [`001-cyrius-port-gaps`](../architecture/001-cyrius-port-gaps.md) table has no rows still marked deferred.
- Example consumer (`docs/examples/basic_consumer.cyr`) builds and runs.

Then: API freeze (additions non-breaking; removals / renames need a major bump), tag, push, ship.

**Deferred to post-1.0 (not blockers for the cut):**

- **Real downstream consumer integration** — daimon or argonaut consuming `src/lib.cyr` end-to-end. The 0.9.2 example covers the surface but isn't a real production consumer. Tracked as a v1.x point release.
- **Trace-ID propagation** (`sakshi_trace_set` for cross-process correlation) — not useful until a multi-process wire flow exists between aegis and a consumer.
- **`scripts/bench.sh` to auto-append `bench-history.csv`** — nice-to-have; can land in any v1.x patch.
