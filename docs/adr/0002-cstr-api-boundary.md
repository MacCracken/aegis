# 0002 — Cstrs at the API boundary, Str* in storage

**Status**: Accepted
**Date**: 2026-05-08

## Context

Cyrius has two string conventions in active use across stdlib and ecosystem:

- **C-string (cstr)** — null-terminated `char*`. What string literals lower to (`"hello"`). What `lib/string.cyr`'s `streq`/`strlen` operate on. What `lib/hashmap.cyr`'s `map_new()` keys are (cstr-keyed maps dispatch to `hash_str`/`streq` internally). What `lib/json.cyr`'s `json_v_obj_get(obj, key)` takes for the lookup arg.
- **Str (fat pointer)** — 16-byte `{data: ptr, len: i64}` heap-allocated record. What `lib/str.cyr` operates on. What `lib/json.cyr`'s `json_v_obj_set(obj, key, val)` stores as the key.

These are not interchangeable. Mixing them silently misbehaves: a `map_new_str()` map fed a cstr key reads the cstr's bytes as a Str struct via `load64` — segfault or random key-mismatch ([001-cyrius-port-gaps.md](../architecture/001-cyrius-port-gaps.md)). Worse: `json_v_obj_get(obj, str_from(...))` treats the Str as a cstr; `strlen` walks the struct bytes as if they were chars, returns garbage length, every field lookup misses (hit during 0.8.0).

aegis crosses both worlds: tests pass `"agent-x"` literals; storage on records carries `Str*` for length-cheap comparisons; hashmap keys are agent_id cstrs.

## Decision

A consistent split:

- **Daemon API parameters that look like strings — `agent_id`, `event_id`, paths, cstr keys for metadata — are cstrs.** Tests and consumers can pass string literals directly.
- **Record fields that store strings — `event.id`, `event.agent_id`, `q.reason`, `finding.category`, etc. — store `Str*`.** Length is constant-time, comparisons are length-prefixed.
- **Conversion at the boundary**: `str_from(cstr)` to wrap when storing into a record; `str_data(str)` to extract a cstr from a stored Str (safe because `str_from` and `str_builder_build` both null-terminate their backing buffers).
- **Hashmap convention**: cstr-keyed maps (`map_new()`) for everything aegis owns — quarantine map keyed by agent_id cstr, event metadata map keyed by metadata-key cstr. Never `map_new_str()`.
- **JSON helpers** (`_aegis_jv_set_*`, `_aegis_jv_get_*`) hide the asymmetry: setters wrap with `str_from`, getters pass cstr directly to `json_v_obj_get`. Documented in the helper comments.

## Consequences

- **Positive** — call sites stay clean: `aegis_quarantine_agent(d, "alice", "reason", THREAT_HIGH)`. No `str_from(...)` ceremony for callers. Internal hashmap lookups go through the cstr-keyed code path that matches how callers think about IDs.
- **Negative** — records that hold dynamic agent_id or metadata-key strings need to keep the backing cstr alive for the lifetime of the record. For literals that's free (static); for dynamically-built keys (rare in practice — most keys come from event constructors that take cstr args) the caller is responsible. Documented in `CONTRIBUTING.md` under the in-process metadata-map row of `SECURITY.md`.
- **Neutral** — when a new daemon API method needs a string parameter, the convention forces a quick cstr-vs-Str judgement: cstr at the boundary, wrap on store. Code reviewers should challenge any `str_from(...)` at a public API call site.

## Alternatives considered

- **All-Str API** — rejected: every test/consumer would need `str_from("alice")` per call. Mass churn, no win.
- **All-cstr storage** (skip Str entirely) — rejected: comparisons become `streq` (O(n)); aegis's events log iteration would re-walk null-terminator bytes per event on every filter call. Str's cached length is materially faster at 10k events.
- **Two parallel APIs** (`aegis_quarantine_agent_cstr` + `aegis_quarantine_agent_str`) — rejected: doubles the surface, no clear primary.
