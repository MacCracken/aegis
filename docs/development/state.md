# aegis — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile). Historical detail
> lives in [`../../CHANGELOG.md`](../../CHANGELOG.md); decision rationale
> lives in [`../adr/`](../adr/).

## Version

**0.8.3** — toolchain + dep refresh (2026-05-10). Cyrius pin `5.10.0` → `5.10.34`; agnostik dep `1.0.0` → `1.2.1`. `lib/` now gitignored and repopulated by `cyrius deps`. CI/release install the toolchain into the version-pinned `~/.cyrius/versions/<V>/{bin,lib}` layout that cc5 5.10.9+ requires for arch-peer include resolution. No source / behaviour change. Carries forward 0.8.2's fuzz harness + audit script, 0.8.1's ring-buffer events log, and 0.8.0's JSON serde surface.

## Toolchain

- **Cyrius pin**: `5.10.34` (in `cyrius.cyml [package].cyrius`).
- **CI**: [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) — toolchain install, `cyrius deps`, syntax check (`--with-deps`), fmt-diff, lint, vet, DCE build, ELF check, aarch64 cross-build (best-effort), smoke, full test suite, fuzz, benchmarks, security pattern scan, doc + version-consistency gates.
- **Release**: [`.github/workflows/release.yml`](../../.github/workflows/release.yml) — runs CI, verifies tag matches `VERSION`, builds x86_64 + aarch64 (best-effort), packages source tarball + `aegis-<ver>-lib.cyr` + binaries + `SHA256SUMS`. `0.x` tags ship as prerelease.
- **Local one-shot**: [`scripts/audit.sh`](../../scripts/audit.sh) — every CI gate locally.

## Source

- `src/lib.cyr` — full library surface: 4 enums, 9 records, 22 daemon API methods, JSON serde for all 8 records, sakshi-full logging on 10 mutating entry points, fixed-cap ring buffer for the events log, agnostik-backed v4 UUID event IDs.
- `src/main.cyr` — thin daemon entry: `alloc_init`, sakshi level config, prints `"aegis ready"`.
- `docs/reference/firewall.rs.ref` — frozen rust spec for the (deferred) nein firewall integration. Read-only; do not modify.

## Tests / fuzz / bench

| Harness | Status |
|---------|--------|
| `tests/aegis.tcyr` | **256 passed / 0 failed** across 73 test groups. |
| `tests/aegis.fcyr` | Real fuzz: 1000 random-byte iterations + ~30 curated edge-case JSON inputs through all 8 record-from-json parsers. Runs in ~1 s. |
| `tests/aegis.bcyr` | 3 benches: `aegis_next_id` ≈ 2 µs, `security_event_new` ≈ 3 µs, `aegis_report_event` ≈ 4 µs (avg, 50–100k iter). History in [`bench-history.csv`](../../bench-history.csv). |

## Dependencies

Direct (declared in `cyrius.cyml`):

- **stdlib** — `string`, `fmt`, `alloc`, `vec`, `str`, `syscalls`, `io`, `args`, `assert`, `tagged`, `chrono`, `hashmap`, `bench`, `fnptr`, `sakshi`, `json`, `random`.
- **agnostik (v1.2.1)** — `src/types.cyr` for `agent_id_new` (UUID v4 over `getrandom`); `src/error.cyr` for `err_invalid_argument` (referenced by `types.cyr`'s parser paths we don't call, but the compiler needs the symbol). `lib/agnostik_*.cyr` is auto-resolved by `cyrius deps` from the version-pinned tag — not committed to the repo.

## Consumers

_None yet_ — daimon and argonaut are the planned downstream consumers; pull `src/lib.cyr` via `[deps.aegis]` once they're ready.

## Next

See [`roadmap.md`](roadmap.md). Remaining work: **0.9.0 V1 prep** (API surface snapshot, full audit, doc polish, one downstream consumer green) → **1.0.0 freeze**. The nein firewall integration stays out of v1.0 scope until nein modernises its language pin.
