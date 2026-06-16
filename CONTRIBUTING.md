# Contributing to Aegis

Thanks for your interest. Aegis is the security daemon for the AGNOS stack — keep that constraint in mind: every change should be defensible against "what could a hostile agent do with this?"

## Prerequisites

- [Cyrius](https://github.com/MacCracken/cyrius) toolchain pinned in `cyrius.cyml [package].cyrius` (currently `6.2.11`)
- Linux x86_64 for now; aarch64 cross-build is best-effort in CI

## Workflow

1. Fork and clone.
2. Branch from `main`.
3. Make your change in the right place:
   - Library types / daemon API → `src/lib.cyr`
   - Firewall builders (nein integration) → `src/firewall.cyr`
   - Daemon entry → `src/main.cyr`
   - Tests → `tests/aegis.tcyr`
   - Benches → `tests/aegis.bcyr`
   - Fuzz → `tests/aegis.fcyr`
4. Run the local checks (see "Commands" below).
5. Update `CHANGELOG.md` under `[Unreleased]`. Performance claims need benchmark numbers; breaking changes get a `### Breaking` subsection.
6. Behavioural divergences from the documented contract (records, daemon API, JSON wire format) need an ADR under `docs/adr/` — wire shape is consumed by daimon / argonaut.
7. Open a PR. CI will run the same checks.

## Commands

| Command | Description |
|---------|-------------|
| `cyrius build src/main.cyr build/aegis` | Compile the daemon |
| `cyrius test tests/aegis.tcyr` | Run the test suite |
| `cyrius bench tests/aegis.bcyr` | Run benchmarks |
| `cyrius check src/lib.cyr` | Syntax check |
| `cyrius fmt src/lib.cyr` | Format (emits to stdout — diff to enforce) |
| `cyrius lint src/lib.cyr` | Static analysis |
| `cyrius vet src/main.cyr` | Include-graph audit |
| `cyrius deps` | Resolve dependencies into `lib/` |

## Style & Constraints

- **Cstrs at API boundaries.** Every daemon entry point that takes an `agent_id` / `event_id` / path takes a cstr. Internal storage on records is `Str*` (fat pointer). Wrap with `str_from(cstr)` when storing; unwrap with `str_data(s)` when comparing/keying maps.
- **Cstr-keyed maps use `map_new()`. Str-keyed use `map_new_str()`. They are not interchangeable** — feeding a cstr to a `map_new_str()` map segfaults silently. See `docs/architecture/001-cyrius-port-gaps.md`.
- **Records are byte-offset layouts.** Document the offsets in the comment above the constructor.
- **Sentinels**: `0` for None / absent pointer; `-1` for None on i64 timestamps where 0 is a valid value (e.g. `auto_release_timeout_secs`).
- **No `vec_remove(v, 0)` loops.** Drain-the-front is O(n²); rebuild a new vec with the kept suffix instead (see `_aegis_prune_events`).
- **Tests run in sequence.** If a test creates a tmp file, unlink it first and unlink it after.

## Adding a Daemon Method

1. Add the function in `src/lib.cyr`. Follow the `aegis_<verb>_<noun>` naming convention.
2. Add a test group in `tests/aegis.tcyr` and wire it into `main()`.
3. If you grew a record's size, update the byte-offset comment above its constructor.
4. Run `cyrius build` and `cyrius test` — both must be green.
5. Note the addition in `CHANGELOG.md` under `[Unreleased]`.

## License

By contributing, you agree your contributions will be licensed under GPL-3.0-only.
