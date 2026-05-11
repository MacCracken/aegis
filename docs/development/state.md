# aegis — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile). Historical detail
> lives in [`../../CHANGELOG.md`](../../CHANGELOG.md); decision rationale
> lives in [`../adr/`](../adr/).

## Version

**0.9.3** — P(-1) hardening pass (2026-05-10). Six security-class fixes (F-1 max_events DoS clamp, F-2 nftables identifier injection via agent_id, F-3 match-value injection via agent_addr, F-4/F-5 timeout/interval bounds, F-8 JSON input-length cap) closed at the API boundary against the 2024-2026 CVE landscape (Wazuh, osquery, ESET, VMware, Tomcat, Jackson, Spring, Android KEV). Three findings deferred to 0.9.4 with concrete plans (F-6 TOCTOU + symlink follow, F-7 Unicode quarantine bypass, F-9 sentinel audit). Full report: [`docs/audit/2026-05-10-audit.md`](../audit/2026-05-10-audit.md). Tests **303 passed / 0 failed** (was 274; +7 groups +29 assertions). API surface unchanged (151 public fns; all new helpers are `_aegis_*` private). After this release, 0.9.4 closes F-6/F-7/F-9; 1.0.0 is the sign-off cut with no new functionality. Carries forward 0.9.2's V1 prep + first-party-standards alignment, 0.9.1's rust-scaffold retirement, 0.9.0's nein firewall integration.

## Toolchain

- **Cyrius pin**: `5.10.34` (in `cyrius.cyml [package].cyrius`).
- **CI**: [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) — toolchain install, `cyrius deps`, syntax check (`--with-deps`), fmt-diff, lint, vet, **api-surface drift gate** (0.9.2+), DCE build, ELF check, aarch64 cross-build (best-effort), smoke, full test suite, fuzz, benchmarks, security pattern scan, doc + version-consistency gates.
- **Release**: [`.github/workflows/release.yml`](../../.github/workflows/release.yml) — runs CI, verifies tag matches `VERSION`, builds x86_64 + aarch64 (best-effort), packages source tarball + `aegis-<ver>-lib.cyr` + binaries + `SHA256SUMS`. `0.x` tags ship as prerelease.
- **Local one-shot**: [`scripts/audit.sh`](../../scripts/audit.sh) — every CI gate locally.

## Source

- `src/lib.cyr` — core library: 4 enums, 9 records, 22 daemon API methods, JSON serde for all 8 records, sakshi-full logging on 10 mutating entry points, fixed-cap ring buffer for the events log, agnostik-backed v4 UUID event IDs.
- `src/firewall.cyr` — nein integration. Three public builders (`aegis_isolate_agent`, `aegis_rate_limit_agent`, `aegis_hardened_host`) + `aegis_firewall_render` / `aegis_firewall_validate` wrappers. Standalone surface — not coupled to `QuarantineEntry`; the rust spec keeps the same shape. Consumers (daimon) decide when to call the builder based on the `QuarantineAction` they read from the entry.
- `src/main.cyr` — thin daemon entry: `alloc_init`, sakshi level config, prints `"aegis ready"`. Includes both `lib.cyr` and `firewall.cyr`.

## Tests / fuzz / bench

| Harness | Status |
|---------|--------|
| `tests/aegis.tcyr` | **303 passed / 0 failed** across 86 test groups (6 firewall groups added in 0.9.0; 7 P(-1)-hardening groups added in 0.9.3). |
| `tests/aegis.fcyr` | Real fuzz: 1000 random-byte iterations + ~30 curated edge-case JSON inputs through all 8 record-from-json parsers. Runs in ~1 s. |
| `tests/aegis.bcyr` | 3 benches: `aegis_next_id` ≈ 2 µs, `security_event_new` ≈ 3 µs, `aegis_report_event` ≈ 4 µs (avg, 50–100k iter). History in [`bench-history.csv`](../../bench-history.csv). |

## Dependencies

Direct (declared in `cyrius.cyml`):

- **stdlib** — `string`, `fmt`, `alloc`, `vec`, `str`, `syscalls`, `io`, `args`, `assert`, `tagged`, `chrono`, `hashmap`, `bench`, `fnptr`, `sakshi`, `json`, `random`.
- **agnostik (v1.2.1)** — `src/types.cyr` for `agent_id_new` (UUID v4 over `getrandom`); `src/error.cyr` for `err_invalid_argument` (referenced by `types.cyr`'s parser paths we don't call, but the compiler needs the symbol). `lib/agnostik_*.cyr` is auto-resolved by `cyrius deps` from the version-pinned tag — not committed to the repo.
- **nein (v1.5.0)** — `dist/nein.cyr` single-file bundle (`firewall_*` / `table_*` / `chain_*` / `rule_*` / `match_*` / `verdict_*` API + constants). Used by `src/firewall.cyr` to build nftables rulesets for `QA_ISOLATE` / `QA_RATELIMIT` quarantine actions and the hardened-host baseline. Pulls `lib/agnosys-core.cyr` as a transitive dep (nein's own `[deps.agnosys]`); aegis doesn't reference agnosys-core symbols, so DCE drops them.

## Consumers

_None yet_ — daimon and argonaut are the planned downstream consumers; pull `src/lib.cyr` via `[deps.aegis]` once they're ready.

## Next

See [`roadmap.md`](roadmap.md). Remaining work: **0.10.x V1 prep** (API surface snapshot, full audit, doc polish, one downstream consumer green) → **1.0.0 freeze**. The API snapshot is now meaningful — the public surface does load-bearing enforcement (firewall builders generate real nftables rulesets) instead of placeholder enum-only behaviour.
