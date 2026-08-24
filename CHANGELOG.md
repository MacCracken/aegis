# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.6] — 2026-08-24

Second P(-1) hardening pass, paired with the 1.1.x minor per the CLAUDE.md
cadence — and the **first audit of `src/pam.cyr`**, which was folded in from
agnosys at 1.1.0, predates the 2026-05-10 audit, and had zero test coverage.
**18 findings (F-10..F-27), all fixed here.** Full report:
[`docs/audit/2026-08-24-audit.md`](docs/audit/2026-08-24-audit.md).

Test suite **326 → 414 assertions** across 104 groups. API snapshot regenerated
at **214 public fns** (was a stale 151) and renamed to a version-free filename.

### Security

- **F-11 (HIGH) — PAM rule validation was a denylist that omitted the render
  delimiters.** `pam_validate_rule` rejected ten shell metacharacters but not TAB,
  LF or CR — and `pam_render_rule` joins fields with TAB while
  `pam_render_config` joins lines with LF. A module or argument containing a
  newline therefore rendered as **additional PAM stack entries**; an injected
  `auth sufficient pam_permit.so` makes every password authenticate for that
  service. Replaced both loops with allowlists over printable ASCII plus an
  explicit punctuation set. Same class as F-2, worse consequence.
- **F-12 (HIGH) — `pam_read_service_config` concatenated an unvalidated service
  name into a path and opened it blocking.** `"../../tmp/fifo"` escaped
  `/etc/pam.d`, and a bare `O_RDONLY` open on a FIFO with no writer blocks in the
  kernel forever — a permanent daemon hang from an unprivileged local user.
  Added `_pam_valid_service_name` (allowlist, no `/`, no leading `.`) and put
  `O_CLOEXEC | O_NOFOLLOW` on all three opens in the file, matching the F-6
  scanner pattern.
- **F-13 (HIGH) — a partial config document silently disabled auto-quarantine.**
  `aegis_config_from_json_v` built from a zeroed record and could not distinguish
  an absent key from an explicit `false`, so omitting `quarantine_on_critical`
  turned off the daemon's primary enforcement switch. Deserialization now starts
  from `aegis_config_default()` and overwrites only keys actually present.
- **F-14 (MEDIUM) — F-7's agent_id whitelist was not enforced at the
  deserialization seam.** A wire event could carry an `agent_id` that
  `aegis_release_agent` would later refuse to act on, minting a quarantine entry
  nothing could clear. Recorded as F-7 reopening at a seam the original fix did
  not cover.
- **F-15 (MEDIUM) — unvalidated `ThreatLevel` indexed the 5-slot inline
  `threat_counts` array.** `-1` lands on the adjacent `scan_history` pointer.
  Both public entry points now bounds-check.
- **F-16 (MEDIUM) — a non-numeric UID parsed as `0` (root).** `str_to_int` skips
  non-digits silently, so `alice:x:root:root:…` read as uid 0 / gid 0 /
  `is_system=1`. The comment always claimed this was checked; now it is.
- **F-17 (MEDIUM) — one-byte heap write past the `who` capture buffer.**
  `run_capture` can return exactly its cap, so the NUL terminator landed one byte
  past a 16384-byte allocation. Now `alloc(cap + 1)`, matching
  `pam_read_fd_to_str`.
- **F-18 (MEDIUM) — parsed PAM and passwd fields were not NUL-terminated.**
  `str_new`/`str_sub` share the parent buffer without terminating, so
  `str_data(pam_user_username(u))` — the ADR 0002 unwrap — returned
  `"alice:x:1000:1000::/home/alice:/bin/sh"` instead of `"alice"`, disclosing the
  rest of the entry through a field that claims to be just the username. Every
  parsed value reaching a record field is now cloned. **Found by the new test
  suite on its first run**, after three review passes over the same code missed it.
- **F-20 (MEDIUM) — `scan_result_from_json_v` trusted the wire `clean` flag**
  instead of re-sealing from the findings it had just deserialized.
- **F-21 (MEDIUM) — `-EINTR` was treated as end-of-file**, silently truncating
  the security config being parsed.
- **F-22 (LOW) — `max_connections_per_agent` was the one config-shaped wire
  integer with no clamp.** Now bounded by `AEGIS_MAX_CONNS_HARD_CAP`.
- **F-8 is now fully closed.** It shipped in 0.9.3 as a partial fix (input-length
  cap only) with the per-depth cap deferred upstream. That cap has landed —
  bayan enforces `_JP_MAX_DEPTH = 128` on both descents — so aegis's length cap
  is now defence in depth rather than the only line.

### Fixed

- **F-10 (HIGH) — every JSON-deserialized timestamp was `-1`.** `iso8601_parse`
  takes a cstr and does `strlen`/`load8` directly; four sites in `src/lib.cyr`
  handed it a `Str*` fat pointer, so `strlen` walked the 16-byte header, always
  came out `< 19`, and the function returned its `-1` error sentinel. This hit
  `SecurityEvent.timestamp`, `SecurityScanResult.scanned_at` and
  `QuarantineEntry.auto_release_at` — and for `auto_release_at` the `-1` *is*
  `AEGIS_AUTO_RELEASE_NONE`, so a deserialized quarantine entry silently became
  one that never auto-releases. Exactly the boundary ADR 0002 exists to prevent;
  the untyped parameter meant the compiler could not catch it. Confirmed with a
  compiled probe: cstr → `1778416496`, `Str*` → `-1`.
- **F-19 (MEDIUM) — `pam_parse_config` rejected real PAM grammar.** It accepted
  neither the leading `-` on a type nor Linux-PAM's bracketed
  `[success=1 default=bad]` control form, and a rejection fails the whole file.
  On this host's own `/etc/pam.d` that is **14 lines plus the central
  `system-auth`** — aegis could not read the PAM stack it exists to audit.
- **F-27 — two quality gates were not gating.** The F-8 oversize-JSON regression
  passed a cstr where a `Str` was required, so `str_len` read character bytes as
  a length and the cap was never exercised — it passed for the wrong reason. And
  all three test harnesses called `syscall(60, …)`, the Linux exit literal, which
  is a no-op on agnos; now `SYS_EXIT`.

### Changed

- **`pam_rule` grows from 4 fields to 6** — `optional` (the `-` prefix) and
  `control_text` (the raw bracket expression), so a parsed real-world config
  renders back byte-for-byte instead of being flattened into a different policy.
  `PAM_CTL_BRACKET` added to `PamControlId`. `pam_rule_new/4` keeps its signature.
- **`pam_render_rule` / `pam_render_config` now fail closed**, returning `0`
  rather than emitting a rule that `pam_validate_rule` would reject — and
  refusing the whole document rather than silently dropping a rule, since
  dropping a `required` line weakens the stack as surely as injecting one. This
  is a **contract change** on two public functions, matching the F-2/F-3
  precedent where the firewall builders return `0` instead of emitting a
  dangerous ruleset.
- **API snapshot regenerated: 151 → 214 public fns** (145 `lib` + 63 `pam` +
  5 `firewall` + 1 `main`). The 151-fn file had been stale since the 1.1.0 PAM
  fold. Renamed `api-surface-1.0.snapshot` → **`api-surface.snapshot`**: it holds
  the current frozen surface, not the v1.0 one, and a version in the filename
  guarantees the label goes stale again. CHANGELOG history keeps the old name.

### Performance

- **F-23** — the eight debug log lines in `src/lib.cyr` were fully constructed
  before sakshi's level filter ran, then discarded at the default `SK_INFO`
  level. Now guarded by `_aegis_log_want_debug()`, mirroring the span guard that
  already existed.
- **F-24** — four `map_has` + `map_get` pairs replaced with a single `map_get`
  (values are pointers, never `0`, so non-zero *is* "present"). Two are on
  `aegis_report_event`'s path.
- **F-25** — `aegis_stats` counted quarantined agents by materializing the whole
  entry vec; `map_size` is O(1) and provably equal. `_aegis_uuid_to_string`
  ended with `str_from(hex)`, re-scanning 36 bytes whose length it had just
  computed; now `str_new(hex, pos)`.

Benchmarks (`tests/aegis.bcyr`, appended to `bench-history.csv`):

| Bench | 1.1.5 | 1.1.6 |
|---|---|---|
| `aegis_next_id` | 839 ns | **804 ns** |
| `security_event_new` | 2.372 µs | **2.288 µs** |
| `aegis_report_event` | 3.253 µs | **3.210 µs** |

Only `aegis_next_id` moves beyond run-to-run noise; the other two are reported
for continuity, not as a claimed win.

### Tests

- **326 → 414 assertions**, 92 → 104 groups. `src/pam.cyr` is now in the test TU
  — it had **zero** assertions before this cut, which is the direct reason six of
  the findings above survived a release.
- 12 new groups: timestamp round-trip, partial-config fail-closed, scan re-seal,
  connection clamp, threat-count bounds, serde agent_id whitelist, PAM delimiter
  rejection, PAM render fail-closed, PAM service-name traversal, PAM passwd
  numeric UID, PAM real-world grammar round-trip, and PAM parser/validator
  basics.
- `scripts/audit.sh` and `.github/workflows/ci.yml` gain nothing new here; the
  gate repairs from 1.1.5 are what made this pass's failures visible.

### Notes

- **Refuted during verification and deliberately not applied**: a claim that 37
  `stik_err_*` call sites in `pam.cyr` pass a cstr where a `Str` is required —
  the constructors accept that form and the mismatch does not exist. Recorded in
  the audit report so a future pass does not re-derive it.
- **Not a defect, recorded for consumers**: aegis never calls `free`, and the
  default bump allocator's `free` is a no-op. Steady-state memory therefore
  tracks *total events reported*, not the ring's capacity — the ADR 0005 ring
  bounds CPU and live-set size, not process RSS.


## [1.1.5] — 2026-08-24

Toolchain + dependency refresh onto the current AGNOS stack, plus a
manifest cleanup. Cyrius `6.4.66` → `6.5.35`, agnostik `1.3.4` → `1.4.0`,
nein `1.6.4` → `1.6.10`. **No behavioural change to aegis** — the public
surface, every wire format, and the firewall ruleset shape are unchanged;
326 assertions pass on the new stack.

The substantive part of this cut is that the nein-transitive dependency
workaround carried since 1.1.4 is **gone** — it was never load-bearing, and
it was actively harmful. Cross-bundle duplicate-symbol warnings drop from
**234 to 2**.

### Changed

- **Cyrius toolchain pin: 6.4.66 → 6.5.35.** Clears the manifest-pin
  drift — the installed wrapper was already 6.5.35, and aegis did not
  build clean against it (see Fixed below).
- **Dependency: agnostik 1.3.4 → 1.4.0.** Purely additive across the two
  modules aegis includes: 66 → 69 public fns, three added
  (`agnostik_err_kind_parse`, `message_type_parse`, `system_status_parse`),
  none removed, no signature changed. The three symbols aegis calls —
  `agent_id_new()`, `stik_err_invalid_argument(msg: Str)`,
  `stik_err_io(msg: Str)` — are byte-identical.
- **Dependency: nein 1.6.4 → 1.6.10.** Also purely additive on the surface
  `src/firewall.cyr` consumes: 100 → 104 `firewall_*` / `table_*` / `chain_*` /
  `rule_*` / `match_*` / `verdict_*` fns, four added (`firewall_deduplicate`,
  `rule_matching_addrs`, `rule_matching_addrs6`, `rule_matching_ports`), none
  removed, no signature changed. All 19 symbols `src/firewall.cyr` calls are
  present; the file needed no edits. (The dist bundle overall goes 359 → 370
  public fns — earlier docs in this repo cited "383 public fns stable" for the
  1.6.x line; that figure was never right and is not repeated here.)
- **`cyrius.cyml` is declarations only.** The manifest carried a
  release-by-release rationale ledger — stdlib-reorg history, per-module
  transitive-closure derivations, dep-version deltas. That is CHANGELOG and
  `state.md` material. Comments are now short statements of what each
  stanza is for, pointing at `docs/development/state.md` (Dependencies) for
  rationale and `CHANGELOG.md` for history.
- **Reformatted `src/lib.cyr` and `tests/aegis.tcyr`** to 6.5.x canonical
  continuation indent (2 spaces per open paren). Whitespace only —
  111 lines, line-for-line, no token changes.

### Removed

- **`[deps.libro]` and `[deps.bote]` declarations.** Added at 1.1.4 to
  short-circuit nein's source-graph walk — but `cyrius deps` has drained
  each resolved dep's own manifest since **cyrius v5.7.14**, so the
  short-circuit was never needed. (Confirmed: the pre-bump `cyrius.lock`
  already contained `majra`, `sigil`, `patra`, and `bote-core`, none of
  which aegis declared.) What the declarations *did* do was pin aegis's own
  libro alongside nein's, pulling libro's **thin sigil sub-bundles**
  (`sigil_hex`, `sigil_sha256`, `sigil_sha_ni`, `sigil-mldsa`) into the same
  translation unit as nein's **full** `sigil` pin — 232 of the 234
  duplicate-symbol warnings. Dropping them leaves nein's pins to win, and
  **2 warnings remain**: `majra._sub_new` vs `libro._sub_new`, and
  `sigil._hex_nibble` vs `agnostik_types._hex_nibble` — both benign
  last-definition-wins, both in DCE-dropped code.
- **Seven of the nine transitive-only `[deps].stdlib` entries** —
  `thread`, `thread_local`, `freelist`, `ct`, `keccak`, `slice`, `sync`.
  Declared at 1.1.4 to satisfy dist-bundle `.deps` sidecars by hand;
  `cyrius deps` reads those sidecars itself. **`process` and `fs` were
  kept**: `src/pam.cyr` includes them directly, so they are aegis's own
  requirement, not a transitive one — trimming them would have left aegis
  source depending on nein's sidecar to supply its own includes. The list
  is now exactly the 19 modules aegis source includes.

### Security

- **Picked up agnostik F-014 (medium), on aegis's own call path.** agnostik's
  `_fill_random` used `0 - 1` as its failure sentinel and compared it signed,
  so on failure the loop re-entered with `buf + (0 - 1)` and length `n + 1` —
  a one-byte heap underflow write plus a non-terminating loop. It is
  reachable from `agent_id_new()`, which aegis calls for every event ID, when
  `getrandom(2)` short-returns *and* the `/dev/urandom` fallback then fails.
  Fixed on agnostik's side and shipped inside the 1.4.0 tag (its CHANGELOG
  files it under a `[1.3.7]` heading that was never tagged separately). No
  aegis change required; noted because the bump is the delivery vehicle.

### Fixed

- **`cyrius fmt` gate was broken under 6.5.x.** 6.5.x changed `cyrius fmt
  <file>` to rewrite **in place** and print nothing; the old gate diffed
  the command's stdout against the file, so it reported *every* file as
  drifted (and silently rewrote two of them as a side effect of running the
  check). `scripts/audit.sh` and `.github/workflows/ci.yml` now use
  `cyrius fmt --check <file>`, which is non-mutating and exits non-zero
  naming the first differing line.
- **`audit.sh`'s test gate could not fail.** `cyrius test … | tail -2` gave
  the pipeline `tail`'s exit status (the script sets `set -e`, not
  `pipefail`), so a red suite still reported `audit: PASSED`. The gate now
  captures the output and branches on the runner's own status.
- **`audit.sh`'s lint gate could abort the audit silently.** `out=$(cyrius
  lint "$f" 2>&1)` under `set -e`: since cyrius 6.5.19 lint's syntax
  pre-pass can exit non-zero, which would kill the script *at the
  assignment*, before the warnings it was about to grep were ever printed.
  Now `|| true`.
- **`tests/aegis.tcyr` exit status truncated.** The harness returned
  `assert_summary()`'s raw failure count as the process exit code; exit
  status is 8-bit, so exactly 256 / 512 / 768 failing assertions would have
  reported success. Now clamped to 1.
- **`undefined function 'chan_try_send'` build warning.** The `thread`
  module vendored from the 6.4.66 pin predates `chan_try_send`, which the
  transitively-resolved `majra` bundle calls. Resolved by the pin bump —
  6.5.35's `thread` defines it.

### Performance

Benchmarks on 6.5.35 (`tests/aegis.bcyr`, appended to `bench-history.csv`):

| Bench | avg | min | max | iters |
|---|---|---|---|---|
| `aegis_next_id` | 839 ns | 772 ns | 2.029 µs | 100k |
| `security_event_new` | 2.372 µs | 2.112 µs | 5.390 µs | 100k |
| `aegis_report_event` | 3.253 µs | 2.981 µs | 6.543 µs | 50k |

**Not comparable to the 1.0.1 row.** 6.5.x's bench harness measures a timer
floor (1.254 µs per clock read here) and subtracts it from every sample;
earlier rows include it and are µs-rounded. Treat this as a new baseline,
not a speedup.

### Notes

- **Transitive dep versions now follow nein.** libro, majra, bote-core,
  sigil, and patra resolve from nein 1.6.10's manifest (libro 2.8.8,
  majra 2.6.7, bote-core 3.3.2, sigil 3.12.9, patra 1.13.9), not from
  aegis. They are unreachable from the firewall path and DCE-dropped;
  bumping them is nein's call, not aegis's.
- **nein 1.6.10 pins cyrius `6.5.33`**, two patches behind aegis's `6.5.35`.
  The `dist/nein.cyr` bundle aegis compiles was produced under a slightly
  older toolchain than the one compiling it. Benign, and normal for the
  stack, but worth knowing when reading nein-attributed warnings.
- **One new benign build warning.** `./lib/ shadows version-pinned … patra
  1.13.9 (pinned: 1.13.10)` — nein 1.6.10 pins patra one patch behind the
  6.5.35 stdlib snapshot. patra is dead code for aegis and advancing the pin
  is nein's call, not aegis's. Not gated by `audit.sh`.
- **`path` shadows `tag`.** A `[deps.X]` stanza with both uses the local
  sibling checkout outright and ignores the tag — verified by pointing
  agnostik at 1.3.4 with the override in place and getting 1.4.0 content.
  CI has no siblings and resolves the tag, so both must stay correct.
- **API-surface snapshot is stale, deliberately.** `cyrius api-surface`
  reports 210 public fns against a 151-fn snapshot — 59 additions, all from
  the 1.1.x PAM fold, all pre-dating this cut. The drift gate passes
  (additions are non-breaking under SemVer). Regenerating the v1.0 baseline
  is a separate decision and is not made here.


## [1.1.4] — 2026-07-17

Toolchain + dependency refresh onto the current AGNOS stack. Clears the
`6.3.37 → 6.4.66` manifest-pin drift and moves both git deps to their
latest tags. **No aegis source changes** — the 151-fn public surface, all
wire formats, and the firewall ruleset shape are byte-for-byte unchanged;
all 326 assertions pass on the new stack.

### Changed

- **Cyrius toolchain pin: 6.3.37 → 6.4.66.** Clears the "manifest-pin:
  6.3.37 (drift — wrapper is 6.4.66)" warning; the wrapper was already on
  6.4.66, so this only realigns the pin. The 6.4.x line changes the
  `cyrius lib sync` default to copy the declared `[deps].stdlib` subset
  (not the whole snapshot); the build sequence is now `cyrius lib sync`
  **then** `cyrius deps`.
- **Dependency: agnostik 1.3.3 → 1.3.4.** Toolchain-only bump on agnostik's
  side (public API + wire formats byte-for-byte unchanged); aegis's two PAM
  call sites (`stik_err_invalid_argument`, `stik_err_io`) are unaffected.
- **Dependency: nein 1.5.3 → 1.6.4.** The firewall API aegis consumes
  (`firewall_*` / `table_*` / `chain_*` / `rule_*` / `match_*` /
  `verdict_*`) is **byte-identical** across the whole 1.6.x line (383 public
  fns stable); the new MCP + Ed25519-signing surfaces are unused by aegis
  and DCE-dropped. `src/firewall.cyr` needed no edits.

### Dependencies

- **nein 1.6.x transitive dist-dep closure.** nein 1.6.x's
  `dist/nein.deps` sidecar declares `thread` / `thread_local` / `bote-core`
  fold requirements, and its `[deps.*]` graph pulls libro/patra/bote/majra/
  sigil dist bundles. To resolve these under the local-path dev overrides
  (offline-safe) without dragging nein's audit-chain **source** graph into
  aegis, `cyrius.cyml` now declares `[deps.libro]` (2.8.2, `dist/libro.cyr`)
  and `[deps.bote]` (3.1.4, `dist/bote-core.cyr`) as self-contained dist
  deps, and adds `thread`, `thread_local`, `freelist`, `fs`, `process`,
  `ct`, `keccak`, `slice`, `sync` to `[deps].stdlib` to satisfy the bundles'
  `.deps` sidecars. **All of this is dead code for aegis** — the firewall
  path references no libro/bote symbol (verified) and DCE strips it; the
  build carries only the two long-documented benign cross-bundle
  "duplicate fn (last-definition-wins)" warnings (`sigil_hex _hex_nibble`,
  `majra _sub_new`). Mirrors stiva's nein 1.6.x consumption pattern.
  `cyrius.lock` re-resolved (60 deps locked).

### Notes

- **Error-constant namespacing (no aegis change required).** The AGNOS
  stack is namespacing bare `ERR_*` enums to end cross-bundle symbol
  collisions (agnostik `STIK_ERR_*`, nein `NEIN_ERR_*`, bote `BOTE_ERR_*`,
  libro `LIBRO_ERR_*`). Aegis owns **no** bare `ERR_*` constants — its
  enums already use domain prefixes (`THREAT_*`, `EV_*`, `QA_*`, `ST_*`,
  `PAM_*`), so no rename applies here. The only `*ERR_*` tokens in the
  build come from vendored `lib/` bundles (`STIK_ERR_*`, `NEIN_ERR_*`),
  which are upstream and out of aegis's edit scope.

## [1.1.3] — 2026-07-03

Consumer step of the agnostik **1.3.3** error-namespace bump. agnostik
renamed its error-constructor API (`err_*` → `stik_err_*`) to disambiguate
it from the stdlib `result` helpers; aegis calls exactly two of them from
the PAM path, so this is a mechanical call-site rename plus a toolchain
pin and dependency refresh. All 326 assertions pass on the new stack.

### Changed

- **agnostik error API migration (`err_*` → `stik_err_*`)**: renamed the
  two agnostik error constructors aegis invokes — `err_invalid_argument`
  (33 call sites) and `err_io` (4 call sites), all in `src/pam.cyr` — to
  their `stik_err_`-prefixed forms. The stdlib `is_err_result` result
  helper is unaffected and left untouched.
- **Cyrius toolchain pin: 6.3.15 → 6.3.37.**
- **Dependency**: agnostik **1.3.3** (was 1.3.2). nein unchanged (1.5.3).
  Vendored agnostik error/types modules re-resolved to the 1.3.3 snapshot;
  `cyrius.lock` updated accordingly.

## [1.1.2] — 2026-07-01

AGNOS cross-build readiness — `aegis` now compiles cleanly under
`cyrius build --agnos`, joining the `--agnos`-clean set from the
base-security-stack agnos-readiness pass.

### Fixed

- **`--agnos` build**: guard the `O_CLOEXEC` open flag. It is exposed by the
  cyrius Linux / macOS / aarch64 syscall libs but not the agnos syscall lib
  (agnos uses `AO_*` flags), so the fstat-TOCTOU stat path
  (`_aegis_stat_modesize`, `src/lib.cyr`) failed to resolve the symbol on the
  agnos target. Now defined to the stable Linux value (524288) under an
  `#ifdef CYRIUS_TARGET_AGNOS` guard — agnos-only, to avoid a duplicate-symbol
  conflict with the stdlib enum on Linux. Compile-readiness only; the agnos
  `sys_open` ABI / close-on-exec runtime path is a separate step.

## [1.1.1] — 2026-06-30

Tier-4 (consumer) step of the coordinated base-security-stack migration
to cyrius **6.3.15**, over the migrated agnostik leaf (1.3.2). Toolchain
pin + dependency refresh; no aegis-side source changes. All 326
assertions pass on the new stack.

### Changed

- **Cyrius toolchain pin: 6.2.11 → 6.3.15.**
- **Dependency**: agnostik **1.3.2** (was 1.3.1). nein unchanged (1.5.x),
  vendored as source and compiled clean under 6.3.15.

### Known issues

- `src/pam.cyr` retains three unresolved references —
  `agnosys_read_fd_to_str` (from the decomposed `agnosys` library) plus
  `err_syscall_failed` / `err_unknown` — carried over from the 1.1.0 PAM
  fold. They are unreachable (DCE-pruned, so the binary builds and tests
  pass), but the PAM path is broken as written. Tracked for a dedicated
  fix/removal pass; the AGNOS auth posture (recognition over
  interrogation) may retire the `/etc/passwd`+PAM path entirely.

## [1.1.0] — 2026-06-19

**PAM folds into aegis.** Part of the `agnosys → agnodrm` ecosystem decomposition.
Auth questions are threats like any other, so PAM (Pluggable Authentication
Modules) belongs with the threat daemon. PAM is cross-platform (UNIX *and* DOS
lean on it) — the work is preserved, not dropped; the agnos-PAM form is **TBD**.

### Added

- **`src/pam.cyr`** (722 lines) — PAM service/config enumeration + user auth,
  folded from `agnosys`. It uses aegis's **existing** error surface (cyrius
  `result.cyr` `err_*`/`is_err_result` + the transitive agnos-core util for
  `read_fd_to_str`), so **no new support modules** were vendored — and the fold
  added **zero** new duplicate symbols (the 6 pre-existing transitive
  `agnostik_error`↔`agnos-core` `ERR_*` warnings are unrelated and unchanged).

### Notes

- The agnos-side `pam.cyr` removal couples to the agnodrm rename (agnosys's own
  `main.cyr` still references it). The transitive `agnos-core` error-system
  duplicate warnings are a separate downstream cleanup (agnostik's dep chain).
- Verified: `cyrius build src/main.cyr` builds; no pam-specific undefined/dup.

## [1.0.1] — 2026-06-15

Toolchain-refresh patch. No aegis source changes — wire formats, the 151-fn
API surface, and the firewall ruleset shape are byte-for-byte unchanged; the
326-assertion test suite + 1000-iter fuzz pass identically under the new pin.

### Toolchain

- **Cyrius pin `5.10.34` → `6.2.11`** (`cyrius.cyml [package].cyrius`). No
  aegis source changes — wire formats, the 151-fn API surface, and the
  firewall ruleset shape are byte-for-byte unchanged. Tracks the same
  toolchain-refresh migration agnostik shipped at 1.3.1 and sigil at the
  6.2.1 pin.
- **`[deps] stdlib` `json` → `bayan`.** 6.2.x folds the standalone `json`
  module (with `base64`/`csv`/`toml`) into the bundled `bayan` distribution
  module; `lib/json.cyr` no longer ships. `bayan` re-exports the full
  `json_v_*` value API aegis serdes all 8 records through, so the swap is
  transparent — all 326 test assertions + the 1000-iter fuzz pass unchanged.
  `lib/` re-synced from the 6.2.11 snapshot (`cyrius lib sync`).

### Dependencies

- **agnostik `1.2.1` → `1.3.1`** — toolchain-refresh patch; 871-fn API and
  all wire formats byte-for-byte unchanged (used for `agent_id_new` UUID v4).
- **nein `1.5.0` → `1.5.3`** — latest `dist/nein.cyr` bundle; firewall
  ruleset builders (`table_`/`chain_`/`rule_`/`match_`/`verdict_`) unchanged.

### Infrastructure

- **CI / release modernized to the canonical installer** (`ci.yml`,
  `release.yml`). Both workflows now install via
  `scripts/install.sh` (`curl … | CYRIUS_VERSION=<pin> sh`) instead of
  hand-unpacking a release tarball into a version-pinned layout. CI gains a
  `setup` job that parses the `cyrius.cyml [package].cyrius` pin once and
  feeds it to the build via `needs` (sigil pattern). All `cc5` references
  removed — the compiler driver is `cyrius` over `cycc` codegen, and the
  aarch64 cross-build probes `cycc_aarch64` (was `cc5_aarch64`); the same
  rename applied to the `scripts/audit.sh` header comment.

## [1.0.0] — 2026-05-10

**First stable release.** No new functionality — the cut freezes the API surface that 0.5.0 → 0.9.5 built up. The 151-fn machine-checkable surface at `docs/development/api-surface-1.0.snapshot` is now the SemVer-stable contract: additions are non-breaking, removals or renames need a major bump.

### Frozen at this cut

- **API surface**: 151 public fns across 2 modules (146 `lib`, 5 `firewall`). CI gate `scripts/check-api-surface.sh` enforces — every PR diffs against the committed snapshot.
- **Wire format**: JSON serde for all 8 records (snake_case fields, PascalCase enum variants, RFC 3339 timestamps, `null` for `Option::None`). Consumed by daimon / argonaut.
- **Firewall ruleset shape**: nftables table-name prefixes (`aegis_iso_`, `aegis_rl_`, `aegis_host`) and rule comments. Treat any change as breaking.
- **9/9 audit findings closed** (or partial-fix-with-tracked-deeper-fix). See [`docs/audit/2026-05-10-audit.md`](docs/audit/2026-05-10-audit.md).
- **Two pre-1.0 `### Breaking` contract changes** (0.9.4 quarantine-API whitelist, 0.9.5 scanner-no-follow-symlinks) — both intentional, documented in their respective entries. From here on out, breaking changes need a major bump.

### Verified at sign-off

Per the M10 sign-off checklist (formerly in `docs/development/roadmap.md`):

- `scripts/audit.sh` green end-to-end, every gate, zero warnings.
- API snapshot matches `cyrius api-surface --scope=project` (151 fns).
- `docs/doc-health.md`: zero rows in 🟡 stale bucket.
- All 5 ADRs (`0001`–`0005`) Accepted; `001-cyrius-port-gaps.md` table has no rows still marked deferred.
- `docs/examples/basic_consumer.cyr` builds and runs (`action=terminate events=1 ruleset_bytes=310`).

### Tracked for v1.x (post-stable, non-blocking)

- **Real downstream consumer integration** — daimon or argonaut consuming `src/lib.cyr` end-to-end. Stand-in `docs/examples/basic_consumer.cyr` exercises the surface but isn't a production consumer.
- **F-8 deeper fix** — JSON parser depth cap belongs in `lib/json.cyr` upstream (cyrius stdlib). Aegis ships the input-length cap as a partial mitigation today.
- **F-6 deeper fix** — pass open fd to consumer via `SecurityFinding` so the consumer's action operates on the same inode aegis stat'd. Adds API surface; held until a real consumer surfaces the requirement.
- **`O_NOFOLLOW` migration to stdlib** — currently defined locally as `_AEGIS_O_NOFOLLOW = 131072` in `src/lib.cyr` with an upstream-this comment. Single grep target.
- **Trace-ID propagation** (`sakshi_trace_set`) for cross-process correlation. Useful once a multi-process wire flow exists between aegis and a consumer.
- **`scripts/bench.sh`** — auto-append `bench-history.csv` after each bench run. Currently hand-maintained.

## [0.9.5] — 2026-05-10

**Closes F-6 from the 0.9.3 audit — every audit finding is now resolved or partial-fix-with-deferred-deeper-fix (F-8 only).** Scanner switched to `open(O_NOFOLLOW)+fstat+close`, refusing to dereference symlinks. Tests **326 passed / 0 failed** (was 322; +1 new test group with sys_symlink setup). API surface unchanged at 151 public fns. The 1.0.0 sign-off cut is now clear of open audit blockers.

### Breaking

- **`aegis_scan_agent` and `aegis_scan_package` no longer follow symlinks.** A symlinked binary path that would previously have scanned the target's metadata now surfaces an `unreadable_metadata` finding (the file is reachable per `file_exists`, but `_aegis_stat_modesize` refuses to follow). Migration: consumers that legitimately scan symlinked binaries (e.g. `/usr/bin/python` → `python3.11`) must canonicalize the path at their own boundary before passing it in. Same posture as `aegis_check_database_integrity`. Second `### Breaking` in 0.9.x — pre-1.0 contract changes are intentional; 1.0.0 freezes the surface.

### Security

- **F-6 (HIGH, CWE-367)** — TOCTOU + symlink follow class (CVE-2025-2425 ESET, CVE-2025-22224 VMware ESXi, CVE-2024-50379 Tomcat) closed. `_aegis_stat_modesize` now uses `sys_open(path, O_RDONLY | O_CLOEXEC | O_NOFOLLOW, 0)` → `sys_fstat(fd, sb)` → `sys_close(fd)`. On a symlink the open returns `-ELOOP`; aegis returns `0` (failure) and the existing scanner failure path emits `unreadable_metadata`.

### Added

- `_AEGIS_O_NOFOLLOW = 131072` local constant in `src/lib.cyr`. Stable POSIX flag value on x86_64 + aarch64 Linux (verified against `include/uapi/asm-generic/fcntl.h`). The cyrius stdlib's `lib/syscalls_*_linux.cyr` doesn't expose `O_NOFOLLOW` yet — defined locally with an upstream-this comment so the constant moves to the stdlib later and aegis swaps to the global. Single grep target for the migration.
- `test_p1_scan_refuses_to_follow_symlink` — sets up a real file + symlink → real file via `sys_symlink`, scans both: real file scans clean, symlink scan surfaces `unreadable_metadata` finding. Cleanup at end.

### Notes

- All 9 audit findings from `docs/audit/2026-05-10-audit.md` are now closed (F-1, F-2, F-3, F-4, F-5, F-6, F-7, F-9) or have a partial-fix-in-aegis with the deeper fix tracked as a future stdlib change (F-8: input-length cap is the partial; depth cap belongs in `lib/json.cyr`).
- 1.0.0 sign-off can proceed — no security-class blockers remain.

### Verification

- `scripts/audit.sh` green end-to-end.
- `scripts/check-api-surface.sh`: 151 public fns, surface matches snapshot exactly.
- All 326 tests pass; example consumer still produces `action=terminate events=1 ruleset_bytes=310`.

## [0.9.4] — 2026-05-10

**P(-1) follow-up — closes F-7 (Unicode quarantine bypass) and F-9 (sentinel audit) from the 0.9.3 audit report.** F-6 (TOCTOU + symlink follow) re-deferred to 0.9.5 once the cyrius stdlib gains a `sys_lstat` wrapper — see `docs/audit/2026-05-10-audit.md` F-6 update for the rationale. Tests **322 passed / 0 failed** (was 303; +5 new test groups, +19 new assertions). API surface unchanged at 151 public fns (helpers stay `_aegis_*` private).

### Breaking

- **Quarantine API now rejects non-whitelist `agent_id` cstrs.** `aegis_quarantine_agent` / `aegis_release_agent` / `aegis_is_quarantined` / `aegis_get_quarantine` apply `_aegis_valid_agent_id` (`^[a-zA-Z0-9_.-]{1,54}$`) at ingress; invalid input returns `0` (no entry created / no release performed / not quarantined / no entry returned). Migration: consumers passing IDs outside the whitelist must canonicalize at their own boundary first. In practice no consumer is in production (daimon / argonaut still planned), so this is breaking-in-name-only — but the `### Breaking` subsection documents the contract change for downstream awareness.
- `security_event_new(..., agent_id_cstr, ...)` now strips bad agent_ids to `aid=0` (anonymous event) rather than storing them. The event is still recorded; the auto-quarantine path then short-circuits per existing semantics. Silently dropping events would mask attack attempts — strip-and-record is the safer middle ground.

### Security

- **F-7 (MEDIUM, CWE-176/CWE-178)** — Unicode-normalization quarantine bypass class (CVE-2024-43093 Android KEV, CVE-2025-52488 DNN, CVE-2024-38820/38827 Spring family) is closed at the in-memory quarantine map. `_aegis_valid_agent_id` (introduced in 0.9.3 for the firewall path) moved from `src/firewall.cyr` to `src/lib.cyr` so both modules share the validator. Also covers case-variant + zero-width-insertion bypass since none of those bytes are in `[a-zA-Z0-9_.-]`.
- **F-9 (MEDIUM, CWE-690)** — Sentinel-collision audit pass. Introduced `_AEGIS_SERDE_INVALID = -1` named constant; replaced 4 magic-number `return 0 - 1` returns in `*_from_serde` parsers with the named form. `AEGIS_AUTO_RELEASE_NONE` and `QA_NONE` already had docstrings — confirmed clean. Re-affirmed precondition: aegis allocations never land at address 0 (relies on bump-allocator from `lib/alloc.cyr`); a future allocator swap would need to preserve this. No active code-behaviour change; the named constant is preventive — catches any future enum-value-collides-with-sentinel addition at code-review time.

### Changed

- `src/lib.cyr` — `_aegis_valid_agent_id`, `_aegis_valid_agent_id_or_empty` (new variant for security_event_new), `_aegis_valid_agent_addr` moved here from `src/firewall.cyr`. Validators now shared across both modules; firewall.cyr keeps a one-line note pointing at lib.cyr.
- `docs/audit/2026-05-10-audit.md` — F-7 status: Deferred → **Fixed in 0.9.4**. F-9 status: Deferred → **Fixed in 0.9.4** (annotation pass). F-6: Deferred to 0.9.5 with a sharper rationale (cyrius stdlib `sys_lstat` patch needed first; otherwise inconsistent with the project's "syscalls go through `SYS_*` constants" convention).

### Verification

- `scripts/audit.sh` green end-to-end.
- `scripts/check-api-surface.sh`: 151 public fns, surface matches snapshot exactly.
- 5 new test groups in `tests/aegis.tcyr`: `p1_quarantine_rejects_bad_agent_id`, `p1_quarantine_release_rejects_bad`, `p1_quarantine_is_get_reject_bad`, `p1_security_event_strips_bad_agent_id`, `p1_quarantine_accepts_good_input`.

## [0.9.3] — 2026-05-10

**P(-1) hardening pass — boundary validation against the 2024-2026 CVE landscape.** Six security-class fixes (F-1, F-2, F-3, F-4, F-5, F-8) closed at the API boundary. Three findings (F-6 TOCTOU, F-7 Unicode quarantine bypass, F-9 sentinel audit) deferred to 0.9.4 with concrete plans. Full audit report at [`docs/audit/2026-05-10-audit.md`](docs/audit/2026-05-10-audit.md). Tests **303 passed / 0 failed** (was 274; +7 new test groups, +29 new assertions). API surface unchanged (151 public fns) — all new helpers are `_aegis_*` private.

### Security

- **F-1 (MEDIUM, CWE-770)** — Unbounded `max_events` from JSON deserialization could request arbitrary heap alloc via `aegis_ring_new(cap*8)`. **Fixed**: `AEGIS_MAX_EVENTS_HARD_CAP = 1_000_000` (8 MB ring upper bound) clamp at JSON deserialization AND at `aegis_config_set_max_events` setter (defense-in-depth).
- **F-2 (HIGH, CWE-77)** — `aegis_isolate_agent` / `aegis_rate_limit_agent` interpolated `agent_id` into nftables table names + rule comments without validation. nein's `validate_identifier` only fired at `firewall_validate` time, after the firewall was built. **Fixed**: `_aegis_valid_agent_id` whitelist `^[a-zA-Z0-9_.-]{1,54}$` at the firewall builder seam — categorically rejects iptables-save comment-injection class (Shielder 2024) and every Unicode confusable / zero-width / control codepoint (CVE-2024-43093 class). Builders return `0` (null fw) on invalid input.
- **F-3 (HIGH, CWE-77)** — Same builders accepted `agent_addr` raw; nein doesn't validate the addr format. **Fixed**: `_aegis_valid_agent_addr` shape validator (`[0-9a-fA-F:./]`, length 1..43) at the seam. Rejects every nftables-injection vector (whitespace, braces, semicolons, newlines, quotes, comment markers). Typed parse + canonical re-emit deferred to 0.9.5.
- **F-4 (LOW, CWE-690)** — `auto_release_timeout_secs` from JSON accepted negatives other than the `-1` sentinel → past epoch + immediate auto-release on next `aegis_check_auto_releases`. **Fixed**: clamp to `{-1} ∪ [0, AEGIS_TIMEOUT_HARD_CAP]` (1 year of seconds).
- **F-5 (LOW)** — `periodic_scan_interval_secs` unbounded. **Fixed**: clamp to `[0, AEGIS_TIMEOUT_HARD_CAP]`.
- **F-8 (HIGH, CWE-674, partial fix)** — `lib/json.cyr`'s typed-value parser is recursive without a depth cap; deeply nested input could exhaust the stack (CVE-2025-52999 Jackson class). **Partial fix**: 256 KB input-length cap (`AEGIS_JSON_MAX_BYTES = 262_144`) at all 8 `*_from_json` seams bounds worst-case nesting at ~131K nodes, well past stack-exhaustion territory. A real depth cap upstream in `lib/json.cyr` (suggest 32 — record nesting in aegis is shallow) tracked for 0.9.5.

### Deferred to 0.9.4 (concrete plans, not skipped)

- **F-6 (HIGH, CWE-367)** — TOCTOU + symlink follow in scanner. `_aegis_stat_modesize` uses `sys_stat` (follows symlinks); the canonical anti-pattern from CVE-2025-2425 (ESET) / CVE-2025-22224 (VMware) / CVE-2024-50379 (Tomcat). 0.9.4 cheap fix: switch to `sys_lstat`, surface symlinks as findings rather than silently following. 0.9.5 deeper fix: rework around `sys_open(O_NOFOLLOW|O_CLOEXEC) → sys_fstat(fd)`. Why deferred: needs symlink test plumbing + new syscall surface.
- **F-7 (MEDIUM)** — Unicode-normalization quarantine bypass. The cstr-keyed quarantine map does byte-for-byte comparison; an attacker can use case variants, U+2011 hyphens, zero-width inserts, RTL overrides to register multiple "different" agents (CVE-2024-43093 class). 0.9.3 firewall-path validators block this on the firewall side (whitelist is ASCII-only); the in-memory quarantine map remains exposed. 0.9.4 fix: reuse `_aegis_valid_agent_id` on `aegis_quarantine_agent` / `aegis_release_agent` / `aegis_is_quarantined` / `aegis_get_quarantine` / `security_event_new`. Why deferred: API-contract change (existing consumers may pass weird IDs); needs `### Breaking` migration note.
- **F-9 (MEDIUM, CWE-690)** — Sentinel audit pass. ADR 0001 documented per-field `-1` / `0` / `QA_NONE` choices; risk is preventive — future fields added without going through the ADR may collide. 0.9.4: annotate every sentinel use with classification, re-evaluate at v1.0 freeze.

### Added

- `docs/audit/` directory + first audit report at `2026-05-10-audit.md` — 9 findings, severity-tagged, with concrete remediation per finding and `What aegis got right` section recording load-bearing audit-clean properties.
- `_aegis_valid_agent_id` / `_aegis_valid_agent_addr` validators in `src/firewall.cyr`.
- `_aegis_clamp_max_events` / `_aegis_clamp_interval_secs` / `_aegis_clamp_auto_release_timeout` / `_aegis_json_size_ok` helpers in `src/lib.cyr`.
- 7 new test groups in `tests/aegis.tcyr` covering each of the 6 fixes (F-2/F-3 share validators so + a regression-guard "valid input still accepted" group): **+29 assertions**, total **303 passed / 0 failed**.

### Verification

- `scripts/audit.sh` green end-to-end.
- API surface snapshot unchanged at 151 public fns (helpers are private; no contract change).
- `scripts/check-api-surface.sh`: `ok: 151 public fns, surface matches snapshot exactly`.

## [0.9.2] — 2026-05-10

**V1 prep — last work before the 1.0.0 cut.** Lands the four remaining v1 deliverables that don't require external consumers to ship: an API-surface CI gate, a doc-health ledger, a polished README API list, and an example consumer that exercises the public surface end-to-end. After this, 1.0.0 is a clean review/audit pass — no new functionality.

### Added

- **`scripts/check-api-surface.sh`** — thin wrapper around `cyrius api-surface --scope=project --snapshot=...`. Pattern lifted from agnosys; the cyrius CLI does the heavy lifting. Usage: `./scripts/check-api-surface.sh` (diff vs. committed snapshot) or `--update` (regenerate after intentional surface changes). Exits non-zero on drift.
- **`docs/development/api-surface-1.0.snapshot`** — committed v1.0 baseline. **151 public fns** across 2 modules: 146 in `lib`, 5 in `firewall`. CI gate fails on unannounced additions/removals.
- New CI step **API surface (drift gate)** in `.github/workflows/ci.yml`, between Vet and Build. Same gate added to `scripts/audit.sh` so local audits catch drift before push.
- **`docs/doc-health.md`** — living ledger of doc currency. Tier the 18-file aegis surface (6 root + 12 under `docs/`) into fresh / stale / read-through / evergreen / archive / open-question buckets. Refreshed in place when docs are touched. Pattern lifted from `agnosys/docs/doc-health.md`, right-sized for aegis's smaller surface.
- **`docs/examples/basic_consumer.cyr`** — small standalone consumer (~60 LOC) that exercises the public surface end-to-end: `aegis_new` → report a Critical event → auto-quarantine triggers → `aegis_isolate_agent` builds the firewall → `aegis_firewall_render` + `aegis_firewall_validate`. Builds with `cyrius build`; prints a one-line summary on success. Stand-in for the v1 deliverable "one downstream consumer green" until daimon / argonaut land — proves nothing essential is private-by-accident.

### Changed

- `README.md` — API list polished to cover all 151 public fns: added Firewall (nein integration) section, Ring primitive subsection (cross-ref'd to ADR 0005), JSON serde paragraph, pointer to the machine-checkable snapshot. Documentation index gains entries for `docs/doc-health.md` + `docs/examples/`.
- `docs/development/roadmap.md` — V1 prep deliverables marked shipped under 0.9.2; M10 (1.0.0) reframed as "clean review/audit before cut" with a concrete sign-off checklist (audit green / snapshot matches / doc-health zero stale rows / ADRs Accepted / `001-cyrius-port-gaps` zero deferred / example builds). "Real downstream consumer integration" moved to deferred-post-1.0.

### Notes

- **Lint warnings: 0** on every source file — the v1-prep "address every warning" deliverable trivially holds at the 0.9.2 baseline (recorded in `doc-health.md`).
- API freeze policy starts at 1.0.0: snapshot additions are non-breaking; removals or renames need a major bump.

### Alignment with first-party standards

- `docs/architecture/cyrius-port-gaps.md` renamed to **`docs/architecture/001-cyrius-port-gaps.md`** to follow the `NNN-kebab-case-title.md` convention from [first-party-documentation.md § Architecture Notes](https://github.com/MacCracken/agnosticos/blob/main/docs/development/planning/first-party-documentation.md#architecture-notes). All references updated (`README.md`, `CLAUDE.md`, `CONTRIBUTING.md`, ADR 0002 + 0005, `doc-health.md`, `roadmap.md`, `src/lib.cyr` header). `docs/architecture/README.md` index populated (was previously "Empty").
- **`CLAUDE.md` rewritten to align with `example_claude.md`** — Genesis repo link added to Project Identity, new Scaffolding section (project ported via `cyrius port`; rust-old retired in 0.6.0; firewall.rs.ref retired in 0.9.1), new Process section with Hardening (P(-1)) / Work Loop / Closeout Pass shaped to aegis's actual cadence, "Read the genesis repo's CLAUDE.md first" added as a hard rule, ADR cross-refs added to Key Principles. Durable content preserved; no volatile state inlined.

## [0.9.1] — 2026-05-10

**Rust scaffold fully retired.** With the firewall port shipped in 0.9.0, the last remaining piece of the original rust source — `docs/reference/firewall.rs.ref`, preserved through 0.5–0.8.x as the parity oracle for the deferred nein integration — is no longer load-bearing. This release deletes it along with the "do not modify the rust spec" guidance and the dangling references in CLAUDE.md / CONTRIBUTING.md / SECURITY.md / README.md / docs / inline comments. No source or behaviour change to the daemon library; tests still **274 passed / 0 failed**.

### Removed

- `docs/reference/firewall.rs.ref` — last file from the rust scaffold; preserved through 0.5–0.8.x as the parity oracle for nein integration. The cyrius port in 0.9.0 made it redundant.
- `/rust-old/target/` from `.gitignore` — `rust-old/` itself was deleted in 0.6.0; the gitignore line had been stale since.
- "Do not modify the frozen rust spec" rule in `CLAUDE.md`.
- "Outstanding rust surface" callout in `README.md` (status paragraph also refreshed from 0.8.2 → 0.9.0); `docs/reference/` removed from the project-layout tree.
- "Out of scope: firewall integration" bullet in `SECURITY.md` — no longer accurate since 0.9.0.
- "`rust-old/src/lib.rs` is the parity oracle" guidance in `CONTRIBUTING.md`; cyrius pin reference bumped to `5.10.34`; `src/firewall.cyr` added to the "make your change in the right place" map.
- `rust-old/` from the `docs/guides/getting-started.md` layout; "cross-check parity against rust-old/" step dropped.

### Changed

- `docs/architecture/001-cyrius-port-gaps.md` header note rewritten — the rust source is fully gone. The `nein` row updated from "deferred" to "done in 0.9.0".
- `docs/development/state.md` — version line and source list refreshed; bullet for the deleted reference file dropped.
- `src/firewall.cyr` header comment — removed the pointer at the deleted spec; replaced with a self-contained note that consumers (daimon) read the wire shape, so changes are breaking.
- 5 inline `# Mirrors rust-old…` / `# matches … in rust-old/src/lib.rs` comments in `src/lib.cyr` rewritten to describe the behaviour without dangling at deleted paths.

## [0.9.0] — 2026-05-10

**Nein firewall integration — `QA_ISOLATE` / `QA_RATELIMIT` are real now.** The deferred-since-0.5 firewall enforcement path lands as `src/firewall.cyr`, a faithful port of the frozen rust spec at `docs/reference/firewall.rs.ref`. nein dependency added at `[deps.nein]` = `1.5.0` (cyrius `5.10.34`). Three public builders + a render + validate wrapper; tests mirror the six `#[cfg(test)]` cases from the rust spec. Wire-level diffs against the rust output are zero (table-name prefixes `aegis_iso_` / `aegis_rl_` / `aegis_host` and rule comments preserved verbatim). Total tests: **274 passed / 0 failed** (was 256).

### Added

- `[deps.nein]` in `cyrius.cyml` — git/path/tag pointing at nein 1.5.0, pulls `dist/nein.cyr` (single-file bundle) plus the transitive `dist/agnosys-core.cyr` it requires.
- `src/firewall.cyr` — three public builders + two passthrough wrappers:
  - `aegis_isolate_agent(agent_id_cstr, agent_addr_cstr) → fw*` — drops all traffic to/from the agent address (inet table `aegis_iso_<agent_id>`, input + output chains, drop verdicts with `aegis isolate <agent_id>` comments).
  - `aegis_rate_limit_agent(agent_id_cstr, agent_addr_cstr, pps) → fw*` — accept up to `pps` packets/second (burst = `2*pps`), drop the rest. Symmetric on input + output. Comments `aegis rate-limit <agent_id>` / `aegis rate-limit drop <agent_id>`.
  - `aegis_hardened_host() → fw*` — baseline host posture: input default drop with allow-established/loopback/SSH/ICMP-echo carve-outs; output accept; forward drop. Table `aegis_host`.
  - `aegis_firewall_render(fw) → Str*` — passthrough to `firewall_render` so consumers can grab nftables source via the aegis surface.
  - `aegis_firewall_validate(fw) → i64` — converts nein's tagged `Result` (`Ok(0)` / `Err(code)`) to aegis's boundary convention: `0 = ok, 1 = invalid`.
- 6 new test groups in `tests/aegis.tcyr` (18 assertions): isolate/rate-limit/hardened-host × renders + validates. Each render assertion checks specific nftables clauses that match the rust spec's `assert!(rendered.contains(...))` calls verbatim.

### Architecture

- `src/main.cyr` now `include`s both `src/lib.cyr` and `src/firewall.cyr`. The new module is independent of the daemon record — it's a sibling slice of the public API, mirroring the rust spec's standalone-function shape. Coupling firewall generation into `aegis_quarantine_agent` would require adding an `agent_addr` to the `QuarantineEntry` record; the rust spec deliberately kept these decoupled (aegis decides the action, the consumer applies the ruleset with addresses it owns), so we follow suit.

### Notes

- Out-of-scope items from the prior 0.9.0 plan (API surface snapshot script + CI gate, full audit pass, doc polish, one downstream consumer green) shift to 0.10.x. The API snapshot in particular needed nein to land first — without it, `aegis_quarantine_agent` with `QA_ISOLATE` / `QA_RATELIMIT` was placeholder behaviour, and freezing a placeholder surface would have been wire-meaningless.
- nein pulls in `lib/agnosys-core.cyr` as a transitive dep (its own `[deps.agnosys]`). aegis doesn't reference any agnosys-core symbols directly; DCE drops everything not transitively reachable from `main`.

## [0.8.3] — 2026-05-10

**Toolchain + dependency refresh.** Cyrius pin moves to `5.10.34`; agnostik dep tag moves to `1.2.1` (was `1.0.0`). The `lib/` tree is now gitignored and repopulated by `cyrius deps` from the version-pinned snapshot — matches the agnosys/agnostik convention and prevents stale stubs from a prior cyrius version sitting in tree. CI and release workflows install the toolchain into `~/.cyrius/versions/<V>/{bin,lib}` with symlinks, which cc5 5.10.9+ requires for arch-peer include resolution (e.g. `syscalls_x86_64_linux.cyr`). No source / behaviour change.

### Changed

- `cyrius.cyml [package].cyrius` — `5.10.0` → `5.10.34`.
- `cyrius.cyml [deps.agnostik].tag` — `1.0.0` → `1.2.1`.
- `.github/workflows/ci.yml` + `release.yml` — toolchain install lays out `~/.cyrius/versions/<CYRIUS_VERSION>/{bin,lib}` and symlinks `~/.cyrius/{bin,lib}` to the versioned dir (cc5 5.10.9+ requires the version-pinned layout to find arch-peer includes). `Verify toolchain` stays as a separate step — `>> $GITHUB_PATH` only takes effect on subsequent steps, so inlining `cc5 --version` in the install step reports "cc5: command not found".

### Removed

- `lib/` no longer tracked in git. `cyrius deps` repopulates it from `cyrius.cyml [deps]`. `.gitignore` now lists `/lib/`.

## [0.8.2] — 2026-05-08

**Polish bucket — closes the remaining 0.8.x backlog.** Real fuzz targets, ADRs for the load-bearing decisions, baseline `bench-history.csv`, and a local `scripts/audit.sh` that mirrors the CI gates. Cyrius pin: `5.10.0`. No behaviour change; the audit script is the only thing consumers see.

### Added

- **`tests/aegis.fcyr` rewritten** as a real fuzz harness. 1000 random-byte iterations (length-explicit `Str` from `random_bytes`, 1–2048 bytes per iter) plus ~30 curated edge-case JSON inputs (empty, `[]`, `null`, malformed objects, wrong-type fields, truncated, whitespace-only) fed to all 8 record-from-json parsers. Pass criterion: process exits 0 within the CI's 10 s timeout — measured ~1 s locally.
- `random` to `[deps].stdlib` (for `random_bytes` from `lib/random.cyr` over `getrandom(2)`).
- **5 ADRs** in `docs/adr/`, all Accepted:
  - `0001` — Sentinel values for absent state (per-field `-1` / `0` / `QA_NONE`, JSON null in/out).
  - `0002` — Cstrs at the API boundary, `Str*` in storage; convert with `str_from`/`str_data` at the seam.
  - `0003` — Integer-array threat counts (5-slot inline) instead of an int-keyed hashmap; PascalCase string keys on the JSON wire.
  - `0004` — `map_new()` (cstr-keyed) is the project default; `map_new_str()` is **not** used in aegis source.
  - `0005` — Fixed-cap ring buffer for the events log; cap captured at `aegis_new` time.
- `bench-history.csv` baseline covering 0.7.0 → 0.8.1 with the three benches (`aegis_next_id`, `security_event_new`, `aegis_report_event`). Schema: `date,version,bench,avg_ns,min_ns,max_ns,iterations,notes`. Future versions append.
- `scripts/audit.sh` — local one-shot equivalent of `.github/workflows/ci.yml`: deps, syntax check (`--with-deps`), fmt-diff, lint, vet, DCE build, ELF magic, smoke, tests, fuzz, bench, security pattern scan, doc + version-consistency. Exits non-zero on the first failed gate.

### Notes

- The fuzz harness measures robustness against adversarial input (no crashes). Coverage isn't tracked — cyrius lacks coverage tooling. Each parser sees ~2k random + curated inputs per run.
- `bench-history.csv` is hand-maintained today. A future patch (probably 0.9.x) will add a `scripts/bench.sh` that runs the bench, parses output, and appends a row automatically.

## [0.8.1] — 2026-05-08

**Ring-buffer for the events log.** Replaces the v0.5–0.8 `vec*` + `_aegis_prune_events` rebuild (O(n) per push at cap) with a fixed-capacity ring (O(1) push, overwrite-oldest). `aegis_report_event` drops from **~220 µs → 4 µs avg at 50k iter** (≈ 55× speedup). Behaviour preserved: same observable order (oldest first), same drop-on-overflow semantics, same `aegis_total_events` answers.

### Added

- `aegis_ring_new(cap) → ring*`, `aegis_ring_push(rb, val)`, `aegis_ring_get(rb, i) → val`, `aegis_ring_len(rb)`, `aegis_ring_cap(rb)`. 32-byte header (slots/cap/head/count) + cap × 8-byte slot array. `cap <= 0` clamps to 1.
- 4 ring-specific test groups: basic push/get, overwrite-oldest at cap, iteration order after wrap, cap clamp. Total: **256 passed / 0 failed** across 73 groups.

### Changed

- `AegisSecurityDaemon.events` is now `ring*` (was `vec*`). Cap is captured from `config.max_events` at `aegis_new` time — runtime changes to `max_events` don't resize an existing daemon's ring (matches typical fixed-cap-ring practice; consumers that need to resize call `aegis_new` again).
- All callers refactored: `aegis_total_events`, `aegis_unresolved_count`, `aegis_recent_events`, `aegis_events_for_agent`, `aegis_events_by_threat`, `aegis_unresolved_events`, `aegis_resolve_event`, `aegis_report_event`. Each `vec_len`/`vec_get` became `aegis_ring_len`/`aegis_ring_get`; `vec_push + _aegis_prune_events` became single `aegis_ring_push`.

### Removed

- `_aegis_prune_events` — superseded by the ring's auto-overwrite. Dead code.

### Notes

- Memory: a daemon at default config now allocates 32 + 10000×8 = 80 KB upfront for the events ring (was: vec growing on demand). For embedded use cases on the small end, lower `max_events` before `aegis_new`.
- `scan_history` is still a `vec` and still grows unbounded — matches the rust-old behaviour. If that becomes a memory concern in production, ring-buffering it is an isolated future change.

## [0.8.0] — 2026-05-08

**JSON serde for the full record surface.** All 8 records (`SecurityEvent`, `QuarantineEntry`, `SecurityFinding`, `SecurityScanResult`, `AegisConfig`, `KernelTuningRecommendation`, `DatabaseSecurityPolicy`, `AegisStats`) gain `*_to_json` / `*_from_json` with roundtrip tests. Wire format mirrors rust-old's `serde_json` rendering. Cyrius pin: `5.10.0`.

### Added

- `json` to `[deps].stdlib`; `[deps.agnostik].modules` extended to include `src/error.cyr` (agnostik's `src/types.cyr` references `err_invalid_argument` from there in parser paths we don't call; pulling it in silences the link warning).
- 4 enum serde-label round-trips (PascalCase variant names, matching `serde`'s default for unit variants without `#[serde(rename_all = ...)]`):
  - `threat_level_serde` / `threat_level_from_serde` (`Critical / High / Medium / Low / Info`)
  - `event_type_serde` / `event_type_from_serde` (12 variants: `IntegrityViolation` … `DatabaseAccessViolation`)
  - `quarantine_action_serde` / `quarantine_action_from_serde` (`None / Suspend / Terminate / Isolate / RateLimit`)
  - `scan_type_serde` / `scan_type_from_serde` (`OnInstall / OnExecute / Periodic / Manual`)
- jv-tree helpers (`_aegis_jv_set_*`, `_aegis_jv_get_*`) layered over `lib/json.cyr`'s typed-value tree API (`json_v_obj_new`, `json_v_str_new`, `json_v_int_new`, `json_v_bool_new`, `json_v_arr_new`, `json_v_obj_set`, `json_v_arr_push`, `json_v_parse`, `json_v_build`).
- 16 record (de)serializers — for each record, both `<name>_to_json_v(rec) → json_v*` (tree) and `<name>_to_json(rec) → Str` (rendered) plus the parse-side equivalents.
- 16 new test groups, **84 new assertions** covering enum roundtrips, basic record roundtrips, edge cases (`Option::None → null` for `agent_id` / `auto_release_at` / `auto_release_timeout_secs`; nested `metadata` map; vec fields like `events` / `findings` / `kernel_tuning`; nested `threat_counts` object). Total: **239 passed / 0 failed** across 69 test groups.

### Wire format

- Field names: snake_case, exactly as the rust struct fields.
- Enum unit variants: PascalCase variant names.
- Timestamps: RFC 3339 / ISO 8601 strings via `iso8601(epoch)` / `iso8601_parse`.
- `Option<T>`: `null` when None, value otherwise.
- `HashMap<String, String>` (event metadata): nested JSON object.
- `HashMap<ThreatLevel, usize>` (threat counts): nested object with PascalCase keys.

### Notes / lessons

- `lib/json.cyr` has two key conventions, **easy to mix up**: `json_v_obj_set(obj, key, val)` stores `key` as a `Str`; `json_v_obj_get(obj, key)` looks up by **cstr**. Found this the hard way — passing `str_from(key_cstr)` to `obj_get` makes `strlen` walk the Str struct as if it were a cstr, returning a garbage length and silently missing every field. All `_aegis_jv_get_*` helpers pass cstr to `obj_get`; `_aegis_jv_set_*` wrap with `str_from` for `obj_set`.
- `aegis_audit_ddl_operation` and friends already populate `event.metadata` — the new SecurityEvent serde lights that wire path up automatically, so DB audit events ship to consumers as full nested-metadata JSON objects without extra glue.

## [0.7.0] — 2026-05-08

**Sakshi-full structured logging.** Spans wrap every mutating daemon entry point; severity-tagged logfmt-style messages emit on the major transitions (event reported, auto-quarantine, manual quarantine, release, scan skipped/started, db-integrity findings). Cyrius pin: `5.10.0`.

### Added

- `sakshi` to `[deps].stdlib` (uses the bundled v2.2.3 distribution).
- `_aegis_span_enter` / `_aegis_span_exit` wrappers — gate sakshi spans on the active level (`< SK_INFO` ⇒ skip), so tests/benches at `SK_ERROR` stay quiet without redirecting sakshi's output fd.
- `_aegis_log_emit_{info,warn,debug}` and `_aegis_log_kv_{cstr,str,int}` helpers for logfmt-style `"<msg> key=val key=val"` construction.
- Span + structured logging in 10 daemon entry points: `aegis_report_event`, `aegis_resolve_event`, `aegis_quarantine_agent`, `aegis_release_agent`, `aegis_check_auto_releases`, `aegis_scan_agent`, `aegis_scan_package`, `aegis_check_database_integrity`, `aegis_audit_ddl_operation`, `aegis_report_database_access_violation`. Each public fn is a thin trampoline: `_aegis_span_enter("…") → _<name>_inner(…) → _aegis_span_exit()` so no early-return path leaves the span stack unbalanced.
- `src/main.cyr` initialises sakshi at `SK_INFO` to stderr (operators can switch to a file later via `sakshi_output_file` once a config surface lands).
- Tests + benches set `sakshi_set_level(SK_ERROR)` at startup to keep stderr clean.

### Severity mapping (mirrors prior tracing! macros)

- `INFO` — events reported, agent released, db-integrity findings detected.
- `WARN` — auto-quarantine fires; quarantine-severity event without `agent_id`; manual quarantine.
- `DEBUG` — agent already quarantined (link/update); scan skipped (config disabled); scan started; db-integrity check passed.
- Library code never calls `sakshi_error` / `sakshi_fatal`.

### Notes

- Sakshi v2.2.3's actual severity values are `SK_FATAL=0, SK_ERROR=1, SK_WARN=2, SK_INFO=3, SK_DEBUG=4, SK_TRACE=5` — the comment in `lib/log.cyr` showing the inverse mapping is stale for this version. Filter is `emit if msg_severity ≤ active_level`.
- Bench impact: `aegis_report_event` ≈ 229 µs avg at `SK_ERROR` (no logging fires; the prune-and-rebuild dominates). At `SK_INFO` the per-call cost is dominated by `str_builder_*` + sakshi formatting — defer measurement until the ring-buffer perf fix lands in 0.8.x.
- One bug worth flagging: the wrapper `_aegis_span_exit` initially recursed because of an over-broad replace-all. Fix is documented; the wrapper now correctly delegates to `sakshi_span_exit`.

## [0.6.0] — 2026-05-08

**Cleanup + real UUIDs.** First post-parity slice. Cyrius pin: `5.10.0`.

### Added

- `[deps.agnostik]` (v1.0.0) — pulls `src/types.cyr` for the audit-reviewed `agent_id_new()` (RFC 4122 v4 over `getrandom` with `/dev/urandom` fallback).
- `_aegis_uuid_to_string(buf16)` — renders agnostik's 16-byte v4 UUID as a 36-char hyphenated lowercase hex string. Heap-allocated per call (no static-buffer aliasing across consecutive ids).

### Changed

- **Event IDs are real v4 UUIDs.** `aegis_next_id()` now produces `550e8400-e29b-41d4-a716-446655440000`-shaped strings instead of the placeholder `ev-1` / `ev-2` counter. Wire-format ready; collision-resistant.
- Removed the `_aegis_id_counter` global.

### Removed

- `rust-old/` is gone. `firewall.rs` was relocated to `docs/reference/firewall.rs.ref` as the spec for the (still-deferred) nein integration; the rest of the rust scaffolding (Cargo.lock, Cargo.toml, codecov.yml, deny.toml, rust-toolchain.toml, src/lib.rs, LINES_OF_RUST.txt) was deleted.

### Notes

- Bench impact: `aegis_next_id` ≈ 2 µs avg (was ≈ 1 µs with the counter). Extra microsecond is the `getrandom` syscall + hex formatting — within noise, not worth caching.
- aarch64 cross-build: agnostik's `_fill_random` hardcodes the x86_64 `getrandom` syscall number (318). On aarch64 it would be 278. CI's aarch64 cross-build is best-effort; expect a runtime path issue if the cross-build succeeds and is exercised. Will be addressed when agnostik gains arch dispatch.

## [0.5.0] — 2026-05-08

**Initial Cyrius release.** Full surface parity with the prior Rust scaffold (`rust-old/src/lib.rs`, 1893 lines). Cyrius pin: `5.10.0`.

### Added

- **Records**: `SecurityEvent` (72 B), `QuarantineEntry` (48 B), `SecurityFinding` (32 B), `SecurityScanResult` (40 B), `AegisConfig` (56 B), `KernelTuningRecommendation` (24 B), `DatabaseSecurityPolicy` (48 B), `AegisStats` (72 B), `AegisSecurityDaemon` (72 B). All accessors, all setters that the rust API exposed.
- **Enums** (integer-constant style): `ThreatLevel`, `SecurityEventType` (12 variants), `QuarantineAction` (with `QA_NONE = 0` so `aegis_report_event` returns action-or-zero), `ScanType`. All have label fns matching rust-old's `Display` / serde rendering.
- **Daemon API** (22 entry points covering all rust-old methods):
  - Event reporting + auto-quarantine: `aegis_new`, `aegis_report_event`.
  - Event queries: `aegis_recent_events`, `aegis_events_for_agent`, `aegis_events_by_threat`, `aegis_unresolved_events`, `aegis_resolve_event`.
  - Counters: `aegis_threat_count`, `aegis_total_events`, `aegis_unresolved_count`.
  - Quarantine management: `aegis_quarantine_agent`, `aegis_release_agent`, `aegis_is_quarantined`, `aegis_get_quarantine`, `aegis_quarantined_agents`, `aegis_check_auto_releases`.
  - Scanning: `aegis_scan_agent`, `aegis_scan_package`.
  - Database surface: `aegis_check_database_integrity`, `aegis_audit_ddl_operation`, `aegis_report_database_access_violation`, `aegis_database_kernel_recommendations`.
  - Snapshot: `aegis_stats`.
- **Tests**: `tests/aegis.tcyr` — 53 test groups, 155 assertions, all passing on `cyrius test`. Inline `_tmp_write` / `_tmp_unlink` helpers cover the empty-binary / world-writable scan-agent paths.
- **Docs**: `README.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, `docs/architecture/cyrius-port-gaps.md`, `docs/development/state.md`, `docs/development/roadmap.md`.

### Changed

- `cyrius.cyml` `version` is now sourced from `VERSION` via `${file:VERSION}` (single source of truth).

### Notes / Deferred

- **Network enforcement**: `firewall.rs` (rust) is preserved at `rust-old/src/firewall.rs` as the spec; the cyrius port is deferred until [nein](https://github.com/MacCracken/nein) bumps its language pin from `4.5.0` to a current Cyrius release.
- **Counter-backed event IDs**: `aegis_next_id` returns `ev-1`, `ev-2`, etc. To be replaced by [agnostik](https://github.com/MacCracken/agnostik)'s audit-reviewed `agent_id_new()` (RFC 4122 v4 over `getrandom`) post-release.
- **Logging**: ad-hoc `str_builder`-formed messages today. Switching to full sakshi (spans + trace IDs + structured fields) is scoped in `docs/architecture/cyrius-port-gaps.md`.
- **Wire format**: no JSON serde yet. Records are in-process only; per-record `*_to_json` / `*_from_json` hand-rolls land when a consumer needs wire interop.
