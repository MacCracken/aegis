# Architecture notes

Non-obvious constraints, quirks, and invariants that a reader cannot derive from the code alone. Numbered chronologically — never renumber.

Not decisions (those live in [`../adr/`](../adr/)) and not guides (those live in [`../guides/`](../guides/)). An item here describes *how the world is*, not *what we chose* or *how to do something*.

## Items

| # | File | Affects | One-line hook |
|---|------|---------|---------------|
| 001 | [`001-cyrius-port-gaps.md`](001-cyrius-port-gaps.md) | every `src/` module — stdlib quirks, type translations, behavioral divergences | Cyrius-stdlib gotchas surfaced during the rust → cyrius port; per-row Status column tracks what's covered vs. deferred. |
