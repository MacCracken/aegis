# aegis — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile). Historical detail
> lives in [`../../CHANGELOG.md`](../../CHANGELOG.md); decision rationale
> lives in [`../adr/`](../adr/).

## Version

**1.1.4** — toolchain + dependency refresh (2026-07-17): cyrius pin `6.3.37` → `6.4.66` (clears manifest-pin drift), agnostik `1.3.3` → `1.3.4`, nein `1.5.3` → `1.6.4`. No aegis source changes — the 151-fn surface, all wire formats, and the firewall ruleset shape are byte-for-byte unchanged; 326 tests pass identically. nein 1.6.x's heavier dist-dep closure is absorbed by declaring libro `2.8.2` + bote `3.1.4` (bote-core) as dead-code dist deps and adding their sidecar stdlib (`thread`/`thread_local`/`freelist`/`fs`/`process`/`ct`/`keccak`/`slice`/`sync`) — all DCE-dropped. See CHANGELOG `[1.1.4]` and the earlier 1.1.0–1.1.3 entries for the intervening PAM-decouple and cross-build work.

**1.0.1** — toolchain-refresh patch (2026-06-15): cyrius pin `5.10.34` → `6.2.11`, stdlib `json` → `bayan`, agnostik `1.2.1` → `1.3.1`, nein `1.5.0` → `1.5.3`. No aegis source changes — the 151-fn surface, all wire formats, and the firewall ruleset shape are byte-for-byte unchanged; 326 tests + fuzz pass identically.

**1.0.0** — first stable (2026-05-10). The 151-fn public API surface is the SemVer-stable contract; additions non-breaking, removals/renames need a major bump. No new functionality at the cut — freezes the surface built across 0.5.0 → 0.9.5: nein firewall integration, JSON serde for all 8 records, sakshi-full structured logging, fixed-cap ring-buffer events log, boundary-validated API (whitelist on `agent_id` + `agent_addr`; clamps on JSON config; no-follow-symlink scanner). All 9 P(-1) audit findings closed (F-8 has a partial fix with the deeper depth-cap tracked as `lib/json.cyr` upstream). Two pre-1.0 `### Breaking` contract changes shipped along the way (0.9.4 quarantine-API whitelist; 0.9.5 scanner-no-follow). Tests **326 passed / 0 failed** across 92 groups + 1000-iter fuzz. Sign-off checklist verified: audit green, snapshot matches, doc-health zero stale, ADRs Accepted, example consumer builds and runs.

## Toolchain

- **Cyrius pin**: `6.4.66` (in `cyrius.cyml [package].cyrius`). Bumped from
  `6.3.37` at 1.1.4 to clear the manifest-pin drift (the wrapper was already
  6.4.66). Under 6.4.x the build sequence is explicit — `cyrius lib sync`
  copies the declared `[deps].stdlib` subset into `./lib/`, **then**
  `cyrius deps` resolves the git-dep bundles. `[deps].stdlib` grew a tail of
  transitive-only modules (`thread`/`thread_local`/`freelist`/`fs`/`process`/
  `ct`/`keccak`/`slice`/`sync`) to satisfy the nein 1.6.x dist-dep sidecars;
  none are referenced by aegis source. Composing nein 1.6.4 + its libro/bote/
  majra/sigil/patra dist bundles surfaces two benign cross-bundle
  duplicate-symbol warnings (`sigil_hex._hex_nibble`, `majra._sub_new`,
  last-definition-wins) plus ~3.1k unreachable fns — all DCE-dropped, not
  gated by `audit.sh`, and not fixable from aegis source.
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

- **stdlib** — core set: `string`, `fmt`, `alloc`, `vec`, `str`, `syscalls`, `io`, `args`, `assert`, `tagged`, `chrono`, `hashmap`, `bench`, `fnptr`, `sakshi`, `bayan`, `random` (`bayan` re-exports the `json_v_*` value API). Plus a transitive-only tail added at 1.1.4 purely to satisfy the nein 1.6.x dist-dep sidecars — `thread`, `thread_local`, `freelist`, `fs`, `process`, `ct`, `keccak`, `slice`, `sync` — none referenced by aegis source.
- **agnostik (v1.3.4)** — `src/types.cyr` for `agent_id_new` (UUID v4 over `getrandom`); `src/error.cyr` for `stik_err_invalid_argument` / `stik_err_io` (the two error constructors aegis's PAM path invokes). `lib/agnostik_*.cyr` is resolved by `cyrius deps` from the version-pinned tag — not committed to the repo.
- **nein (v1.6.4)** — `dist/nein.cyr` single-file bundle (`firewall_*` / `table_*` / `chain_*` / `rule_*` / `match_*` / `verdict_*` API + constants). Used by `src/firewall.cyr` to build nftables rulesets for `QA_ISOLATE` / `QA_RATELIMIT` quarantine actions and the hardened-host baseline. The 1.6.x line kept that firewall API byte-identical (383 public fns stable) and added unused MCP + Ed25519-signing surfaces. Its `dist/nein.deps` sidecar + `[deps.*]` graph now pull `thread`/`thread_local`/`bote-core` fold requirements and the libro/patra/bote/majra/sigil dist bundles (all DCE-dropped dead code for aegis).
- **libro (v2.8.2)** + **bote (v3.1.4, `dist/bote-core.cyr`)** — declared as top-level dist deps at 1.1.4 ONLY to satisfy nein 1.6.x's transitive closure under the local-path dev overrides (offline-safe), short-circuiting nein's audit-chain **source** walk. Neither is referenced by aegis source; both are DCE-stripped. Mirrors stiva's nein 1.6.x consumption pattern.

## Consumers

_None yet_ — daimon and argonaut are the planned downstream consumers; pull `src/lib.cyr` via `[deps.aegis]` once they're ready.

## Next

See [`roadmap.md`](roadmap.md). Remaining work: **0.10.x V1 prep** (API surface snapshot, full audit, doc polish, one downstream consumer green) → **1.0.0 freeze**. The API snapshot is now meaningful — the public surface does load-bearing enforcement (firewall builders generate real nftables rulesets) instead of placeholder enum-only behaviour.
