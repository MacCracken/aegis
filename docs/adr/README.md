# Architecture Decision Records

Decisions about aegis — what we chose, the context, and the consequences we accept. Use these when a future reader would reasonably ask *"why did we do it this way?"*

## Conventions

- **Filename**: `NNNN-kebab-case-title.md`, zero-padded to four digits. Never renumber.
- **One decision per ADR.** If a decision supersedes a prior one, add a new ADR and set the old one's status to `Superseded by NNNN`.
- **Status lifecycle**: `Proposed` → `Accepted` → (optionally) `Superseded` or `Deprecated`.
- Use [`template.md`](template.md) as the starting point.

## ADR vs. architecture note vs. guide

| Kind | Lives in | Answers |
|---|---|---|
| ADR | `docs/adr/` | *Why did we choose X over Y?* |
| Architecture note | `docs/architecture/` | *What non-obvious constraint is true about the code?* |
| Guide | `docs/guides/` | *How do I do X?* |

## Index

| ADR | Title | Status |
|-----|-------|--------|
| [0001](0001-sentinel-values-for-absent-state.md) | Sentinel values for absent state | Accepted |
| [0002](0002-cstr-api-boundary.md) | Cstrs at the API boundary, Str* in storage | Accepted |
| [0003](0003-integer-array-threat-counts.md) | Integer-array threat counts (not a hashmap) | Accepted |
| [0004](0004-hashmap-flavor-selection.md) | Hashmap flavor: `map_new()` (cstr) over `map_new_str()` (Str) | Accepted |
| [0005](0005-fixed-cap-ring-buffer-events-log.md) | Fixed-cap ring buffer for the events log | Accepted |
