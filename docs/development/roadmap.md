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
- [ ] **0.10.x V1 prep** — API surface snapshot + freeze, full audit, doc polish, one downstream consumer green
- [ ] **1.0.0 release** — first stable

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

## Upcoming milestones

### M9 — V1 prep (v0.10.x)

Stabilise. Goal: be in a state where v1.0 is a tag, not a slice of work. Originally planned as 0.9.0 but the API snapshot piece had no anchor without a load-bearing consumer surface — nein 0.9.0 lands the firewall builders, so the snapshot now describes real enforcement instead of placeholder enum-only behaviour.

- Public-API-surface snapshot (`scripts/check-api-surface.sh` analogous to agnosys's): commit a frozen list of public fn names, gate CI to fail on unannounced removals.
- Full audit pass — re-run `scripts/audit.sh`, address every warning even if currently below the failure threshold.
- Documentation polish — every public daemon fn covered in either `README.md`'s API list or a guide; cross-references between ADRs and architecture notes complete.
- One downstream consumer green (daimon or argonaut consuming `src/lib.cyr` end-to-end, including a quarantine path that calls `aegis_isolate_agent` and applies the rendered ruleset).

### M10 — First stable (v1.0.0)

- API freeze. Public-fn additions are non-breaking; removals or renames need a major bump.
- Tag, push, ship.

**Out of scope for v1.0:**

- **Trace-ID propagation** (`sakshi_trace_set` for cross-process correlation) — not useful until a multi-process wire flow exists between aegis and a consumer.
- **`scripts/bench.sh` to auto-append `bench-history.csv`** — nice-to-have; can land in any v1.x patch.
