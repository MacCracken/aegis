# aegis — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile). Historical detail
> lives in [`../../CHANGELOG.md`](../../CHANGELOG.md); decision rationale
> lives in [`../adr/`](../adr/).

## Version

**1.0.1** — toolchain-refresh patch (2026-06-15): cyrius pin `5.10.34` → `6.2.11`, stdlib `json` → `bayan`, agnostik `1.2.1` → `1.3.1`, nein `1.5.0` → `1.5.3`. No aegis source changes — the 151-fn surface, all wire formats, and the firewall ruleset shape are byte-for-byte unchanged; 326 tests + fuzz pass identically.

**1.0.0** — first stable (2026-05-10). The 151-fn public API surface is the SemVer-stable contract; additions non-breaking, removals/renames need a major bump. No new functionality at the cut — freezes the surface built across 0.5.0 → 0.9.5: nein firewall integration, JSON serde for all 8 records, sakshi-full structured logging, fixed-cap ring-buffer events log, boundary-validated API (whitelist on `agent_id` + `agent_addr`; clamps on JSON config; no-follow-symlink scanner). All 9 P(-1) audit findings closed (F-8 has a partial fix with the deeper depth-cap tracked as `lib/json.cyr` upstream). Two pre-1.0 `### Breaking` contract changes shipped along the way (0.9.4 quarantine-API whitelist; 0.9.5 scanner-no-follow). Tests **326 passed / 0 failed** across 92 groups + 1000-iter fuzz. Sign-off checklist verified: audit green, snapshot matches, doc-health zero stale, ADRs Accepted, example consumer builds and runs.

## Toolchain

- **Cyrius pin**: `6.2.11` (in `cyrius.cyml [package].cyrius`). Bumped from
  `5.10.34` in the Unreleased toolchain-refresh — no aegis source changes;
  `[deps] stdlib` swapped `json` → `bayan` (6.2.x folds standalone `json` into
  the bundled `bayan` dist module, which re-exports the `json_v_*` value API).
  `lib/` is re-synced from the toolchain snapshot via `cyrius lib sync`.
  Composing agnostik + nein under the new pin surfaces benign cross-dep
  duplicate-symbol warnings (`ERR_*`/`err_*` defined in both `agnostik_error.cyr`
  and the transitive `agnosys-core.cyr`) and dead-path `exec_*` references — all
  DCE-dropped, not gated by `audit.sh`, and not fixable from aegis source.
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
| `tests/aegis.tcyr` | **326 passed / 0 failed** across 92 test groups (6 firewall in 0.9.0; 7 P(-1)-hardening in 0.9.3; 5 quarantine-validator in 0.9.4; 1 scan-no-follow-symlink in 0.9.5). |
| `tests/aegis.fcyr` | Real fuzz: 1000 random-byte iterations + ~30 curated edge-case JSON inputs through all 8 record-from-json parsers. Runs in ~1 s. |
| `tests/aegis.bcyr` | 3 benches: `aegis_next_id` ≈ 2 µs, `security_event_new` ≈ 3 µs, `aegis_report_event` ≈ 4 µs (avg, 50–100k iter). History in [`bench-history.csv`](../../bench-history.csv). |

## Dependencies

Direct (declared in `cyrius.cyml`):

- **stdlib** — `string`, `fmt`, `alloc`, `vec`, `str`, `syscalls`, `io`, `args`, `assert`, `tagged`, `chrono`, `hashmap`, `bench`, `fnptr`, `sakshi`, `bayan`, `random`. (`json` → `bayan` at the 6.2.11 pin; `bayan` re-exports the `json_v_*` value API.)
- **agnostik (v1.3.1)** — `src/types.cyr` for `agent_id_new` (UUID v4 over `getrandom`); `src/error.cyr` for `err_invalid_argument` (referenced by `types.cyr`'s parser paths we don't call, but the compiler needs the symbol). `lib/agnostik_*.cyr` is auto-resolved by `cyrius deps` from the version-pinned tag — not committed to the repo.
- **nein (v1.5.3)** — `dist/nein.cyr` single-file bundle (`firewall_*` / `table_*` / `chain_*` / `rule_*` / `match_*` / `verdict_*` API + constants). Used by `src/firewall.cyr` to build nftables rulesets for `QA_ISOLATE` / `QA_RATELIMIT` quarantine actions and the hardened-host baseline. Pulls `lib/agnosys-core.cyr` as a transitive dep (nein's own `[deps.agnosys]`); aegis doesn't reference agnosys-core symbols, so DCE drops them.

## Consumers

_None yet_ — daimon and argonaut are the planned downstream consumers; pull `src/lib.cyr` via `[deps.aegis]` once they're ready.

## Next

See [`roadmap.md`](roadmap.md). Remaining work: **0.10.x V1 prep** (API surface snapshot, full audit, doc polish, one downstream consumer green) → **1.0.0 freeze**. The API snapshot is now meaningful — the public surface does load-bearing enforcement (firewall builders generate real nftables rulesets) instead of placeholder enum-only behaviour.
