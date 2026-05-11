# aegis — Claude Code Instructions

> **Core rule**: this file is **preferences, process, and procedures** —
> durable rules that change rarely. Volatile state (current version,
> module line counts, port progress, test counts, consumers) lives in
> [`docs/development/state.md`](docs/development/state.md).
> Do not inline state here.

## Project Identity

**aegis** — security daemon for AGNOS. Records security events, enforces auto-quarantine, scans agent binaries / package archives, audits database operations.

- **Type**: Daemon + library (binary at `src/main.cyr`, consumer-facing surface at `src/lib.cyr`)
- **License**: GPL-3.0-only
- **Language**: Cyrius (toolchain pinned in `cyrius.cyml [package].cyrius`)
- **Version**: `VERSION` at the project root is the source of truth — do not inline the number here. `cyrius.cyml` reads it via `${file:VERSION}`.
- **Consumers**: daimon (security-policy enforcement), argonaut (boot hardening)
- **Standards**: [First-Party Standards](https://github.com/MacCracken/agnosticos/blob/main/docs/development/applications/first-party-standards.md) · [First-Party Documentation](https://github.com/MacCracken/agnosticos/blob/main/docs/development/applications/first-party-documentation.md)

## Goal

Aegis owns *security policy enforcement* for the AGNOS stack: collect events from peer subsystems, decide who to quarantine, surface scan/integrity findings, never store secrets. Cryptographic primitives belong in sigil; aegis enforces policy, not crypto.

## Current State

> Volatile state lives in [`docs/development/state.md`](docs/development/state.md) —
> port progress, version, test counts, in-flight work. Refreshed every release.

This file (`CLAUDE.md`) is durable rules.

## Quick Start

```sh
cyrius deps                              # resolve dependencies
cyrius build src/main.cyr build/aegis    # compile the daemon
cyrius test tests/aegis.tcyr             # run the test suite
cyrius bench tests/aegis.bcyr            # run benchmarks
```

## Key Principles

- **Correctness over cleverness** — if behaviour diverges silently from the documented contract, the bugs win.
- **Cstrs at the API boundary, Str* in storage.** Daemon API parameters that look like strings (`agent_id`, `event_id`, paths) are cstrs. Records store `Str*` (fat pointer) — `str_from(cstr)` to wrap, `str_data(s)` to unwrap.
- **Test after every change**, not after the feature is "done".
- **ONE change at a time** — never bundle unrelated changes.
- Build with `cyrius build`, not raw `cat file | cc5` — the manifest auto-resolves deps.
- Source files only need project includes — stdlib auto-resolves from `cyrius.cyml`.
- `var buf[N]` = N **bytes**, not N entries. **Function-scope `var buf[N]` is static data** — consecutive calls share the backing memory; alloc on the heap if the buffer's contents need to outlive the call.
- For non-obvious cyrius-implementation constraints surfaced during the port, see [`docs/architecture/cyrius-port-gaps.md`](docs/architecture/cyrius-port-gaps.md).

## Rules (Hard Constraints)

- **Do not commit or push** — the user handles all git operations.
- **Never use `gh` CLI** — use `curl` to the GitHub API if needed.
- Do not skip tests before claiming changes work.
- Do not modify `lib/` files (vendored stdlib / dep symlinks).
- Do not hardcode toolchain versions in CI YAML — `cyrius = "X.Y.Z"` in `cyrius.cyml` is the source of truth.

## Documentation

- [`docs/adr/`](docs/adr/) — Architecture Decision Records (*why X over Y?*)
- [`docs/architecture/`](docs/architecture/) — Non-obvious constraints
- [`docs/guides/`](docs/guides/) — Task-oriented how-tos
- [`docs/examples/`](docs/examples/) — Runnable examples
- [`docs/development/state.md`](docs/development/state.md) — Live state
- [`docs/development/roadmap.md`](docs/development/roadmap.md) — Milestones through v1.0

