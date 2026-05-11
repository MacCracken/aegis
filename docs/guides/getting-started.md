# Getting started with aegis

## Build

```sh
cyrius deps                              # resolve dependencies
cyrius build src/main.cyr build/aegis    # compile
cyrius test                              # run tests/*.tcyr
```

## Layout

- `src/lib.cyr` — library surface (records, accessors, daemon API).
- `src/firewall.cyr` — nein firewall builders for `QA_ISOLATE` / `QA_RATELIMIT`.
- `src/main.cyr` — entry point. Top-level `var r = main(); syscall(SYS_EXIT, r);`.
- `tests/` — test suite (`.tcyr` files, auto-discovered by `cyrius test`).

## Adding a feature

1. Edit `src/lib.cyr` (or `src/firewall.cyr` for firewall surface; add a new module and `include` it from `src/main.cyr` for anything else).
2. Add a test case to `tests/aegis.tcyr` and wire it into `main()`.
3. Run `cyrius test`.
4. Bump `VERSION` and add a CHANGELOG entry before tagging.

See [`../adr/template.md`](../adr/template.md) when a non-trivial design choice deserves an ADR.
