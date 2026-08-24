# aegis — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile). Historical detail
> lives in [`../../CHANGELOG.md`](../../CHANGELOG.md); decision rationale
> lives in [`../adr/`](../adr/).

## Version

**1.1.6** — second P(-1) hardening pass (2026-08-24), paired with the 1.1.x minor.
**18 findings (F-10..F-27), all fixed** — see
[`../audit/2026-08-24-audit.md`](../audit/2026-08-24-audit.md). Four HIGH: every
JSON-deserialized timestamp was the `-1` error sentinel (`iso8601_parse` fed a
`Str*` where a cstr is required); PAM rule validation was a denylist that omitted
the TAB/LF render delimiters, making the render path a PAM-stack injection
primitive; `pam_read_service_config` concatenated an unvalidated service name into
a path and opened it blocking; and a partial config document silently disabled
auto-quarantine. The first audit of `src/pam.cyr` — folded in at 1.1.0, it
predated the 2026-05-10 pass and had **zero** test assertions, which is why six of
the findings survived a release. Tests 326 → 414. API snapshot regenerated at 214
fns and renamed version-free. F-8 fully closed (bayan's depth cap landed).

**1.1.5** — toolchain + dependency refresh and manifest cleanup (2026-08-24):
cyrius pin `6.4.66` → `6.5.35`, agnostik `1.3.4` → `1.4.0`, nein `1.6.4` →
`1.6.10`. No aegis behavioural change — the public surface, all wire formats,
and the firewall ruleset shape are unchanged; 326 tests pass. The nein-transitive
workaround carried since 1.1.4 is removed — it was never load-bearing (`cyrius
deps` has walked each dep's own manifest since v5.7.14) and it was pulling
libro's thin `sigil_*` sub-bundles alongside nein's full sigil pin;
cross-bundle duplicate-symbol warnings drop 234 → 2. `cyrius fmt`
changed to in-place-plus-`--check` in 6.5.x, so the fmt gate in `audit.sh` and
`ci.yml` was rewritten. See CHANGELOG `[1.1.5]`.

**1.1.4** — toolchain + dependency refresh (2026-07-17): cyrius pin `6.3.37` → `6.4.66` (clears manifest-pin drift), agnostik `1.3.3` → `1.3.4`, nein `1.5.3` → `1.6.4`. No aegis source changes — the 151-fn surface, all wire formats, and the firewall ruleset shape are byte-for-byte unchanged; 326 tests pass identically. nein 1.6.x's heavier dist-dep closure is absorbed by declaring libro `2.8.2` + bote `3.1.4` (bote-core) as dead-code dist deps and adding their sidecar stdlib (`thread`/`thread_local`/`freelist`/`fs`/`process`/`ct`/`keccak`/`slice`/`sync`) — all DCE-dropped. See CHANGELOG `[1.1.4]` and the earlier 1.1.0–1.1.3 entries for the intervening PAM-decouple and cross-build work.

**1.0.1** — toolchain-refresh patch (2026-06-15): cyrius pin `5.10.34` → `6.2.11`, stdlib `json` → `bayan`, agnostik `1.2.1` → `1.3.1`, nein `1.5.0` → `1.5.3`. No aegis source changes — the 151-fn surface, all wire formats, and the firewall ruleset shape are byte-for-byte unchanged; 326 tests + fuzz pass identically.

**1.0.0** — first stable (2026-05-10). The 151-fn public API surface is the SemVer-stable contract; additions non-breaking, removals/renames need a major bump. No new functionality at the cut — freezes the surface built across 0.5.0 → 0.9.5: nein firewall integration, JSON serde for all 8 records, sakshi-full structured logging, fixed-cap ring-buffer events log, boundary-validated API (whitelist on `agent_id` + `agent_addr`; clamps on JSON config; no-follow-symlink scanner). All 9 P(-1) audit findings closed (F-8 has a partial fix with the deeper depth-cap tracked as `lib/json.cyr` upstream). Two pre-1.0 `### Breaking` contract changes shipped along the way (0.9.4 quarantine-API whitelist; 0.9.5 scanner-no-follow). Tests **326 passed / 0 failed** across 92 groups + 1000-iter fuzz. Sign-off checklist verified: audit green, snapshot matches, doc-health zero stale, ADRs Accepted, example consumer builds and runs.

## Toolchain

- **Cyrius pin**: `6.5.35` (in `cyrius.cyml [package].cyrius`). Bumped from
  `6.4.66` at 1.1.5. Two 6.5.x behaviours matter to this repo:
  - **`cyrius fmt <file>` rewrites in place and prints nothing.** The
    non-mutating gate is `cyrius fmt --check <file>` (non-zero exit, names
    the first differing line). The pre-6.5 idiom of diffing the command's
    stdout against the file reports every file as drifted *and* rewrites it
    as a side effect. `scripts/audit.sh` and `ci.yml` use `--check`.
  (Note: `cyrius deps` draining each dep's own manifest is **not** new in
  6.5.x — that BFS has been in place since v5.7.14. It is why the libro/bote
  declarations removed at 1.1.5 were never needed; see Dependencies below.)
  Composing nein 1.6.10's bundle set leaves three benign build warnings, none
  gated by `audit.sh` and none fixable from aegis source:
  - two cross-bundle duplicate symbols, last-definition-wins —
    `majra._sub_new` vs `libro._sub_new`, and `sigil._hex_nibble` vs
    `agnostik_types._hex_nibble`;
  - `./lib/ shadows version-pinned … patra 1.13.9 (pinned: 1.13.10)` — nein
    pins patra one patch behind the 6.5.35 stdlib snapshot. Dead code for
    aegis; nein's pin to advance;
  - ~5.1k unreachable fns, all DCE-dropped.
- **CI**: [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) — toolchain install, `cyrius deps`, syntax check (`--with-deps`), fmt (`--check`), lint, vet, **api-surface drift gate** (0.9.2+), DCE build, ELF check, aarch64 cross-build (best-effort), smoke, full test suite, fuzz, benchmarks, security pattern scan, doc + version-consistency gates.
- **Release**: [`.github/workflows/release.yml`](../../.github/workflows/release.yml) — runs CI, verifies tag matches `VERSION`, builds x86_64 + aarch64 (best-effort), packages source tarball + `aegis-<ver>-lib.cyr` + binaries + `SHA256SUMS`. `0.x` tags ship as prerelease.
- **Local one-shot**: [`scripts/audit.sh`](../../scripts/audit.sh) — every CI gate locally.

## Source

- `src/lib.cyr` — core library: 4 enums, 9 records, 22 daemon API methods, JSON serde for all 8 records, sakshi-full logging on 10 mutating entry points (debug construction gated behind `_aegis_log_want_debug` since 1.1.6), fixed-cap ring buffer for the events log, agnostik-backed v4 UUID event IDs.
- `src/firewall.cyr` — nein integration. Three public builders (`aegis_isolate_agent`, `aegis_rate_limit_agent`, `aegis_hardened_host`) + `aegis_firewall_render` / `aegis_firewall_validate` wrappers. Standalone surface — not coupled to `QuarantineEntry`; consumers (daimon) decide when to call the builder based on the `QuarantineAction` they read from the entry.
- `src/pam.cyr` — PAM surface, folded in from agnosys during the agnosys → agnodrm decomposition (1.1.0). 63 public fns. Parses `/etc/passwd`, `/etc/pam.d/*` and `who` output; audited for the first time at 1.1.6, which added allowlist validation on module / arg / service-name, `O_NOFOLLOW` on every open, fail-closed rendering, and support for the `-` type prefix and bracketed control form that real PAM configs use.
- `src/main.cyr` — thin daemon entry: `alloc_init`, sakshi level config, prints `"aegis ready"`. Includes `lib.cyr`, `firewall.cyr`, and `pam.cyr`.

**Allocator note for consumers**: aegis never calls `free`, and the default bump
allocator's `free` is a no-op. Steady-state memory tracks *total events reported*,
not the ring's capacity — [ADR 0005](../adr/0005-fixed-cap-ring-buffer-events-log.md)
bounds CPU and live-set size, not process RSS.

## Tests / fuzz / bench

| Harness | Status |
|---------|--------|
| `tests/aegis.tcyr` | **414 passed / 0 failed** across 104 test groups (326/92 before 1.1.6). 12 groups added at 1.1.6 covering every F-10..F-27 regression; `src/pam.cyr` entered the test TU at 1.1.6 with 7 dedicated groups after having zero assertions. |
| `tests/aegis.fcyr` | Real fuzz: 1000 random-byte iterations + ~30 curated edge-case JSON inputs through all 8 record-from-json parsers. Runs in ~1 s. |
| `tests/aegis.bcyr` | 3 benches on 6.5.35: `aegis_next_id` 804 ns, `security_event_new` 2.288 µs, `aegis_report_event` 3.210 µs (avg, 50–100k iter). Only `aegis_next_id` moved beyond noise at 1.1.6 (F-25). 6.5.x's harness subtracts a measured timer floor, so these are not comparable to pre-1.1.5 rows. History in [`bench-history.csv`](../../bench-history.csv). |

## Dependencies

`cyrius.cyml` is a build manifest and holds declarations only; this section is
where the rationale lives. Note that a `[deps.X]` stanza carrying both `path`
and `tag` uses the **local sibling checkout** and ignores the tag entirely —
CI has no siblings and resolves the tag, so both must stay correct.

Direct (declared in `cyrius.cyml`):

- **stdlib** — the 19 modules aegis source includes: `string`, `fmt`, `alloc`,
  `vec`, `str`, `syscalls`, `io`, `args`, `assert`, `tagged`, `chrono`,
  `hashmap`, `bench`, `fnptr`, `sakshi`, `bayan`, `random`, `process`, `fs`
  (`bayan` re-exports the `json_v_*` value API; `process` and `fs` are
  included directly by `src/pam.cyr`). The invariant: every
  `include "lib/X.cyr"` in `src/` must have its `X` declared here — a module
  that only arrives via a dep's `.deps` sidecar is a dependency on that dep's
  packaging, not a declaration. Seven transitive-only entries added at 1.1.4
  (`thread`, `thread_local`, `freelist`, `ct`, `keccak`, `slice`, `sync`) were
  removed at 1.1.5; `cyrius deps` reads the dist bundles' sidecars itself.
- **agnostik (v1.4.0)** — `src/types.cyr` for `agent_id_new` (UUID v4 over
  `getrandom`); `src/error.cyr` for `stik_err_invalid_argument` /
  `stik_err_io`, the two constructors aegis's PAM path invokes. `lib/agnostik_*.cyr`
  is resolved by `cyrius deps` — not committed.
- **nein (v1.6.10)** — `dist/nein.cyr` single-file bundle (`firewall_*` /
  `table_*` / `chain_*` / `rule_*` / `match_*` / `verdict_*` API + constants).
  Used by `src/firewall.cyr` to build nftables rulesets for `QA_ISOLATE` /
  `QA_RATELIMIT` quarantine actions and the hardened-host baseline.

Transitive (resolved from nein's manifest, **not declared by aegis**):

nein 1.6.10 pins its own audit-chain and MCP deps — libro `2.8.8`, majra
`2.6.7`, bote-core `3.3.2`, sigil `3.12.9`, patra `1.13.9` — and `cyrius deps`
walks that graph. None is referenced by aegis source; all are DCE-stripped.
Bumping them is nein's call — note nein 1.6.10 itself pins cyrius `6.5.33`,
two patches behind aegis. aegis declared libro and bote directly at 1.1.4 as a
workaround for a source-graph walk that `cyrius deps` had not done since
v5.7.14 anyway; the declarations' only real effect was to pin aegis's own libro
alongside nein's, dragging libro's thin `sigil_*` sub-bundles into the same
translation unit as nein's full `sigil` pin. Removed at 1.1.5: 234 → 2
duplicate-symbol warnings.

## Consumers

_None yet_ — daimon and argonaut are the planned downstream consumers; pull `src/lib.cyr` via `[deps.aegis]` once they're ready.

## API surface

[`api-surface.snapshot`](api-surface.snapshot) holds the frozen public surface —
**214 public fns** (145 `lib` + 63 `pam` + 5 `firewall` + 1 `main`), regenerated
at 1.1.6. It was stale at 151 fns from 1.0.0 until then: the 1.1.0 PAM fold added
its symbols without a snapshot update, and the drift gate passes additions
silently because they are non-breaking under SemVer, so nothing forced the issue.

Renamed from `api-surface-1.0.snapshot` at 1.1.6 — the file holds the *current*
frozen surface, not the v1.0 one, and a version number in the filename guarantees
the label goes stale again.

The contract is unchanged: additions are non-breaking, removals and renames need
a major bump. The gate is [`scripts/check-api-surface.sh`](../../scripts/check-api-surface.sh);
intentional additions regenerate with `--update` and commit in the same PR.

## Next

See [`roadmap.md`](roadmap.md). The v1.0 freeze shipped 2026-05-10; the 1.1.x
line has been toolchain/dependency refreshes plus the PAM fold from the
agnosys → agnodrm decomposition. The outstanding v1.x deliverable is **one
real downstream consumer green** — daimon or argonaut consuming `src/lib.cyr`
end-to-end; [`docs/examples/basic_consumer.cyr`](../examples/basic_consumer.cyr)
is the stand-in until then. Regenerating the API snapshot is gated on that
consumer existing.
