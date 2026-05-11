# aegis — Claude Code Instructions

> **Core rule**: this file is **preferences, process, and procedures** —
> durable rules that change rarely. Volatile state (current version,
> module line counts, port progress, test counts, consumers) lives in
> [`docs/development/state.md`](docs/development/state.md).
> Do not inline state here.

## Project Identity

**aegis** — security daemon for AGNOS. Records security events, enforces auto-quarantine, scans agent binaries / package archives, audits database operations, and (since 0.9.0) generates nftables rulesets via [nein](https://github.com/MacCracken/nein) for `QA_ISOLATE` / `QA_RATELIMIT` quarantine actions.

- **Type**: Daemon + library (binary at `src/main.cyr`, consumer-facing surface at `src/lib.cyr` + `src/firewall.cyr`)
- **License**: GPL-3.0-only
- **Language**: Cyrius (toolchain pinned in `cyrius.cyml [package].cyrius`)
- **Version**: `VERSION` at the project root is the source of truth — do not inline the number here. `cyrius.cyml` reads it via `${file:VERSION}`.
- **Genesis repo**: [agnosticos](https://github.com/MacCracken/agnosticos)
- **Consumers**: daimon (security-policy enforcement), argonaut (boot hardening)
- **Standards**: [First-Party Standards](https://github.com/MacCracken/agnosticos/blob/main/docs/development/planning/first-party-standards.md) · [First-Party Documentation](https://github.com/MacCracken/agnosticos/blob/main/docs/development/planning/first-party-documentation.md)

## Goal

Aegis owns *security policy enforcement* for the AGNOS stack: collect events from peer subsystems, decide who to quarantine, generate the firewall ruleset that enforces the quarantine, surface scan/integrity findings, never store secrets. Cryptographic primitives belong in sigil; aegis enforces policy, not crypto.

## Current State

> Volatile state lives in [`docs/development/state.md`](docs/development/state.md) —
> port progress, version, test counts, dep tags, in-flight work. Refreshed every release.
> Doc currency lives in [`docs/doc-health.md`](docs/doc-health.md).

This file (`CLAUDE.md`) is durable rules.

## Scaffolding

Project was ported via `cyrius port` from a prior rust scaffold; the rust scaffold itself was retired in 0.6.0 (last surviving file `firewall.rs` removed in 0.9.1 once its cyrius port shipped). Do not manually create project structure — use `cyrius init` / `cyrius port`. If the tools are missing something, fix the tools.

## Quick Start

```sh
cyrius deps                              # resolve dependencies
cyrius build src/main.cyr build/aegis    # compile the daemon
cyrius test tests/aegis.tcyr             # run the test suite
cyrius bench tests/aegis.bcyr            # run benchmarks
./scripts/audit.sh                       # one-shot equivalent of CI
./scripts/check-api-surface.sh           # diff public surface vs snapshot
```

## Key Principles

- **Correctness over cleverness** — if behaviour diverges silently from the documented contract, the bugs win.
- **Cstrs at the API boundary, Str* in storage.** Daemon API parameters that look like strings (`agent_id`, `event_id`, paths) are cstrs. Records store `Str*` (fat pointer) — `str_from(cstr)` to wrap, `str_data(s)` to unwrap. ([ADR 0002](docs/adr/0002-cstr-api-boundary.md))
- **Test after every change**, not after the feature is "done".
- **ONE change at a time** — never bundle unrelated changes.
- Build with `cyrius build`, not raw `cat file | cc5` — the manifest auto-resolves deps.
- Source files only need project includes — stdlib auto-resolves from `cyrius.cyml`.
- `var buf[N]` = N **bytes**, not N entries. **Function-scope `var buf[N]` is static data** — consecutive calls share the backing memory; alloc on the heap if the buffer's contents need to outlive the call.
- For non-obvious cyrius-implementation constraints surfaced during the port, see [`docs/architecture/001-cyrius-port-gaps.md`](docs/architecture/001-cyrius-port-gaps.md).

## Rules (Hard Constraints)

- **Read the genesis repo's CLAUDE.md first** — [agnosticos/CLAUDE.md](https://github.com/MacCracken/agnosticos/blob/main/CLAUDE.md)
- **Do not commit or push** — the user handles all git operations.
- **NEVER use `gh` CLI** — use `curl` to the GitHub API if needed.
- Do not skip tests before claiming changes work.
- Do not modify `lib/` files (vendored stdlib / dep symlinks; `cyrius deps` repopulates them).
- Do not hardcode toolchain versions in CI YAML — `cyrius = "X.Y.Z"` in `cyrius.cyml` is the source of truth.
- Do not bypass the API-surface drift gate — intentional public-fn additions / removals regen via `scripts/check-api-surface.sh --update` and commit the new snapshot in the same PR.

## Process

### P(-1): Hardening (before features and at minor cuts)

Aegis ran the original P(-1) pass during 0.5–0.8.x; the pre-1.0 hardening cycle (0.9.3 → 0.9.5) closed the audit at `docs/audit/2026-05-10-audit.md`. Subsequent runs pair with each minor cut.

1. **Cleanliness** — `./scripts/audit.sh` green; `cyrius lint`, `cyrius vet`, `cyrius fmt` clean.
2. **Benchmark baseline** — `cyrius bench tests/aegis.bcyr`; append to `bench-history.csv`.
3. **Internal review** — gaps, optimizations, correctness, doc currency.
4. **Security audit** — input handling, syscall usage, buffer sizes; log findings under `docs/audit/YYYY-MM-DD-audit.md` if surfaced.
5. **API surface diff** — `./scripts/check-api-surface.sh`; update snapshot only for intentional changes.
6. **Documentation audit** — refresh `docs/doc-health.md`, ADRs for any new decisions, README API list against the snapshot.
7. **Repeat if heavy** — keep drilling until clean.

### Work Loop

1. Make the change in the right place (`src/lib.cyr` for core surface; `src/firewall.cyr` for nein integration; `src/main.cyr` only for daemon entry).
2. Build (`cyrius build`).
3. Tests + benchmarks for new code.
4. Run `./scripts/audit.sh` before claiming done — every gate including api-surface drift.
5. Update `CHANGELOG.md` `[Unreleased]`, `state.md`, `doc-health.md` rows for any docs touched.
6. Verify `VERSION`, `cyrius.cyml`, CHANGELOG header in sync before tagging.

### Closeout Pass (before each minor / `X.Y.0` and the v1.0.0 cut)

Per the v1.0.0 sign-off checklist in [`docs/development/roadmap.md`](docs/development/roadmap.md):

- `./scripts/audit.sh` green, every gate, zero warnings tolerated.
- API snapshot matches `cyrius api-surface --scope=project`.
- `docs/doc-health.md`: zero rows in 🟡 stale bucket.
- All ADRs still Accepted; `001-cyrius-port-gaps` table has no rows still marked deferred.
- `docs/examples/basic_consumer.cyr` builds and runs.
- `VERSION`, `cyrius.cyml`, CHANGELOG header, intended git tag all match.

## Documentation

- [`docs/adr/`](docs/adr/) — Architecture Decision Records (*why X over Y?*)
- [`docs/architecture/`](docs/architecture/) — Non-obvious constraints (`NNN-` numbered)
- [`docs/guides/`](docs/guides/) — Task-oriented how-tos
- [`docs/examples/`](docs/examples/) — Runnable examples (incl. `basic_consumer.cyr`)
- [`docs/development/state.md`](docs/development/state.md) — Live state
- [`docs/development/roadmap.md`](docs/development/roadmap.md) — Milestones through v1.0
- [`docs/development/api-surface-1.0.snapshot`](docs/development/api-surface-1.0.snapshot) — Machine-checkable v1.0 baseline (gated by CI)
- [`docs/doc-health.md`](docs/doc-health.md) — Doc-currency ledger; refresh in place when docs are touched
