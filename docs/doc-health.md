---
name: Aegis Documentation Health
description: Living state of doc currency in the aegis repo — fresh / stale / archived / open-question, refreshed as docs are touched
type: state
---

# Documentation Health — aegis

> **Last refresh**: 2026-05-10 (initial audit, paired with the 0.9.2 v1-prep cleanup pass) | **Refresh cadence**: when docs are touched, update the affected row.
> **Scope**: This repo only (`aegis`) — root-level files (README, CHANGELOG, CLAUDE.md, etc.) plus the entire `docs/` tree. Cross-repo cyrius pin / nein pin / agnostik pin lives in [`development/state.md`](development/state.md), not here.

This is a **ledger**, not a one-time audit. Rewrite-in-place as docs change. Aegis owns security policy enforcement for the AGNOS stack — wire shape (records, JSON serde, firewall ruleset format) is consumed by daimon / argonaut, so doc currency on the public surface carries weight. The doc surface is small (~18 files); most are load-bearing.

Pattern lifted from agnosys's ledger ([`agnosys/docs/doc-health.md`](https://github.com/MacCracken/agnosys/blob/main/docs/doc-health.md)) — same buckets, aegis-shaped tiers (smaller surface so fewer tier splits).

---

## At a glance — 2026-05-10 inventory

**18 markdown files** total (6 root + 12 under `docs/`). Bucket counts after the 0.9.2 v1-prep cleanup:

| Bucket | Count | What it means |
|---|---|---|
| ✅ **Fresh — touched in 0.8.3 → 0.9.3 cycle** | 14 | `README.md`, `CHANGELOG.md`, `CLAUDE.md` (refreshed in 0.9.2 alignment pass), `CONTRIBUTING.md`, `SECURITY.md` (out-of-scope bullet trimmed in 0.9.1), `state.md`, `roadmap.md`, `001-cyrius-port-gaps.md` (renamed to NNN convention in 0.9.2), `getting-started.md` (rust-old refs dropped 0.9.1), `doc-health.md` (this file — new in 0.9.2), `api-surface-1.0.snapshot` (new in 0.9.2 — gates CI), `audit/2026-05-10-audit.md` (new in 0.9.3 — first P(-1) audit report), the 5 ADRs (Accepted, all stable). |
| 🟡 **Stale — refresh in place** | 0 | None outstanding after the 0.8.3 → 0.9.2 sweep closed every dangling rust-spec / rust-old reference. |
| 🔵 **Probably evergreen** | 3 | `CODE_OF_CONDUCT.md`, `LICENSE`, `docs/adr/template.md`. No version-tied claims. Re-read pass annually (or when the ADR pattern changes). |
| 📦 **Archive / frozen by design** | 2 | `docs/adr/README.md` + `docs/architecture/README.md` are tier index pages — frozen until a new file lands in the relevant tier. |
| ❓ **Open strategic question** | 0 | None outstanding for the 0.9.2 cut. See [Open questions](#open-strategic-questions) for what would re-open it. |

**0.8.3 → 0.9.2 cleanup arc:**
- ✅ `state.md` — refreshed every release; carries VERSION cell, cyrius pin, dep tags, build sizes, test count, source layout.
- ✅ `roadmap.md` — 0.8.3 / 0.9.0 / 0.9.1 milestones added; v1-prep folded into 0.9.2; 1.0.0 reframed as "audit-only sign-off cut".
- ✅ `CHANGELOG.md` — entries through 0.9.2.
- ✅ `001-cyrius-port-gaps.md` — header note rewritten (rust source fully gone in 0.9.1); the `nein` row now reads "done in 0.9.0". Renamed in 0.9.2 to follow the `NNN-` numbering convention from first-party-documentation.md; `architecture/README.md` index populated.
- ✅ `README.md` — status block refreshed 0.8.2 → 0.9.2 across the 0.9.0/0.9.1/0.9.2 cuts; firewall surface section added to the API list (0.9.2); project-layout drops `docs/reference/`.
- ✅ `api-surface-1.0.snapshot` (new) — committed in 0.9.2 as the 1.0 baseline; CI gate `scripts/check-api-surface.sh` fails on unannounced public-fn additions or removals. 151 fns at 0.9.2 baseline (146 lib + 5 firewall).

---

## Tier 1 — Root files

| File | Last touched | Status | Notes |
|---|---|---|---|
| `README.md` | 2026-05-10 | ✅ Fresh | Status block at 0.9.2; API list covers all 151 public fns (records / enums / daemon API / firewall surface / ring primitive); project-layout reflects current src/ + docs/. |
| `CHANGELOG.md` | 2026-05-10 | ✅ Fresh | Source of truth for shipped work. Entries through 0.9.2. Historical entries (≤ 0.8.x) preserved verbatim — they describe past releases accurately. |
| `CLAUDE.md` | 2026-05-10 | ✅ Fresh | Durable rules. "Do not modify the rust spec" rule dropped 0.9.1 (the spec is gone). |
| `CONTRIBUTING.md` | 2026-05-10 | ✅ Fresh | Cyrius prereq points at `5.10.34`; src/firewall.cyr added to the surface map; rust-old parity-oracle guidance dropped (0.9.1) and replaced with an ADR rule for wire-shape divergences. |
| `SECURITY.md` | 2026-05-10 | ✅ Fresh | Reporting policy + scope. Out-of-scope "firewall integration" bullet dropped in 0.9.1 (no longer accurate since 0.9.0). |
| `CODE_OF_CONDUCT.md` | 2026-04-30 | 🔵 Evergreen | Standard. Re-read annually. |
| `VERSION` | 2026-05-10 | ✅ Fresh | `0.9.2` — single source of truth, read into `cyrius.cyml` via `${file:VERSION}`. |
| `LICENSE` | (initial commit) | 🔵 Evergreen | GPL-3.0-only. |

---

## Tier 2 — Project state (`docs/development/`)

| File | Last touched | Status | Notes |
|---|---|---|---|
| `state.md` | 2026-05-10 | ✅ Fresh | Live volatile state — VERSION line, cyrius pin, dep tags, source layout, test count, dependency block. Refreshed every release. |
| `roadmap.md` | 2026-05-10 | ✅ Fresh | Milestone table extended through 0.9.2; v1-prep deliverables folded into 0.9.2; 1.0.0 reframed as "clean review/audit before cut". |
| `api-surface-1.0.snapshot` | 2026-05-10 | ✅ Fresh — auto-gated | Machine-checkable companion (one `module::fn/arity` line per public fn, 151 lines). CI gate `scripts/check-api-surface.sh` diffs against this; unannounced additions/removals fail the build. Regen via `--update`. |

---

## Tier 3 — Architecture (`docs/architecture/`)

| File | Last touched | Status | Notes |
|---|---|---|---|
| `001-cyrius-port-gaps.md` | 2026-05-10 | ✅ Fresh | Cyrius-stdlib gotchas surfaced during the rust → cyrius port; per-row Status column tracks what's covered vs. deferred. Header note refreshed 0.9.1 (rust source fully gone); nein row refreshed 0.9.0 (done). Renamed from `cyrius-port-gaps.md` in 0.9.2 to follow `NNN-` convention. |
| `README.md` | 2026-04-30 | 📦 Tier index | Frozen until a second file lands in `architecture/`. |

---

## Tier 4 — ADRs (`docs/adr/`)

| File | Last touched | Status | Notes |
|---|---|---|---|
| `0001-sentinel-values-for-absent-state.md` | 2026-05-08 | ✅ Fresh | Accepted (0.5.0). Per-field `-1` / `0` / `QA_NONE` sentinels; JSON null in/out. Cross-ref'd from `src/lib.cyr` ThreatLevel comment. |
| `0002-cstr-api-boundary.md` | 2026-05-08 | ✅ Fresh | Accepted (0.5.0). Cstrs at the API, `Str*` in storage. Wire convention for every `agent_id` / `event_id` / path parameter. |
| `0003-integer-array-threat-counts.md` | 2026-05-08 | ✅ Fresh | Accepted (0.5.0). 5-slot inline counts vs. int-keyed hashmap; PascalCase string keys on the JSON wire. |
| `0004-hashmap-flavor-selection.md` | 2026-05-08 | ✅ Fresh | Accepted (0.5.0). `map_new()` (cstr-keyed) is the project default; `map_new_str()` is **not** used. Avoids the silent-segfault flavor mismatch. |
| `0005-fixed-cap-ring-buffer-events-log.md` | 2026-05-08 | ✅ Fresh | Accepted (0.8.1). Fixed-cap ring for the events log; cap captured at `aegis_new` time. Killed the O(n²) prune-and-rebuild. |
| `template.md` | 2026-04-30 | 🔵 Evergreen | Standard ADR skeleton. Touch only when the ADR pattern changes. |
| `README.md` | 2026-04-30 | 📦 Tier index | Frozen until a new ADR lands. |

**ADR posture**: low decision-velocity. 5 ADRs cover every load-bearing port-time decision. Re-evaluate at v1.0.0 cut whether any new decision needs an ADR (none currently outstanding).

---

## Tier 5 — Guides + examples

| File | Last touched | Status | Notes |
|---|---|---|---|
| `docs/guides/getting-started.md` | 2026-05-10 | ✅ Fresh | Build / layout / add-a-feature flow. rust-old references dropped 0.9.1; `src/firewall.cyr` added to layout. |
| `docs/examples/basic_consumer.cyr` | 2026-05-10 | ✅ Fresh | Stand-in downstream consumer (new in 0.9.2). Exercises the public surface end-to-end: aegis_new → report critical event → quarantine → firewall builder → render → validate. Builds clean. Not a real daimon, but proves nothing essential is private-by-accident. |

## Tier 6 — Audit reports (`docs/audit/`)

Date-stamped, frozen by design. Each P(-1) hardening pass per CLAUDE.md cadence lands a new report — old reports stay verbatim as the historical record.

| File | Date | Status | Notes |
|---|---|---|---|
| `2026-05-10-audit.md` | 2026-05-10 | ✅ Fresh | First P(-1) audit (initial cut 0.9.3; F-7 + F-9 fixed 0.9.4; F-6 fixed 0.9.5). All 9 findings closed (F-8 has a partial fix with the deeper depth-cap tracked as a `lib/json.cyr` upstream change). CVE landscape research 2024-2026 (Wazuh, osquery, ESET, VMware, Tomcat, Jackson, Spring, Android KEV). |

Next audit slot: at v1.0.0 sign-off cut (or sooner if a CVE pattern surfaces in nein, the cyrius json stdlib, or the agnostik UUID surface).

---

## Open strategic questions

None outstanding for the 0.9.2 cut. This section will repopulate when:

- **A real downstream consumer lands** (daimon or argonaut consuming `src/lib.cyr` end-to-end). Currently `docs/examples/basic_consumer.cyr` is the stand-in. When the real consumer exists, decide whether the example stays as a teaching artifact or gets deprecated in favor of pointing at the real code.
- **API surface drifts unexpectedly**. The CI gate (`scripts/check-api-surface.sh`) makes drift visible; if a slot generates many intentional drift updates, the gate's signal-to-noise drops and we may need a different convention (e.g. unstable-API namespace).
- **A new doc category appears** that doesn't fit an existing tier (e.g. `docs/operations/` if/when an operator-facing config story emerges).

---

## In-flight (blocked, not stale)

- **One downstream consumer green** (v1 deliverable from the original roadmap). Blocked on daimon or argonaut work outside this repo. `docs/examples/basic_consumer.cyr` is a partial stand-in that exercises the surface; the v1 deliverable proper requires a real consumer. Tracked as a post-1.0 follow-up rather than a 1.0 blocker — the surface itself is exercised by the example + tests + fuzz.

---

## Forward doc-policy commitments

| # | Commitment | Trigger | Source | Notes |
|---|---|---|---|---|
| 1 | **API-surface snapshot retention** — keep `docs/development/api-surface-1.0.snapshot` as the v1.0 baseline through at least v2.0.0; intentional public-fn additions regen via `scripts/check-api-surface.sh --update` and commit in the same PR. | v2.0.0 cut | This file | Pattern proven by agnosys (730 fns) and aegis (151 fns). |
| 2 | **CHANGELOG-as-historical-record** — never rewrite shipped CHANGELOG entries. Errata go in a follow-up entry, not in-place edits. (One in-flight rewrite happened in 0.9.1's editing phase before catching that 0.9.0 had already shipped — caught and reverted.) | always | This file | Avoids retroactive-history corruption. |
| 3 | **doc-health refresh cadence** — opportunistic, not periodic. Refresh in place when other docs are touched; the at-a-glance bucket counts only need a sync when they drift by more than ~2 in any cell. | When docs touched | This file | Aegis's small surface means the refresh load is light. |

---

## Refresh procedure

When docs are touched:

1. Find the affected row in the relevant tier table.
2. Update **Last touched** column to the new date.
3. Update **Status** column if the bucket changed.
4. Update **Notes** column if the next step changed.
5. If a doc moved or was archived, update its row to reflect the new home.
6. Re-anchor "Last refresh" date in the header.

When the bucket counts at the top drift by more than ~2 in any cell, refresh the at-a-glance table.

This file's refresh cadence is **opportunistic** (touched when other docs are touched), not periodic. The 0.8.3 → 0.9.2 cycle established the baseline; each subsequent cut's doc-sync step updates this file alongside CHANGELOG + roadmap + state.md.

---

## What this file is NOT

- Not a substitute for [`development/state.md`](development/state.md) (which holds live version / pin / dep state).
- Not a CHANGELOG (which records what shipped, not what's stale).
- Not a roadmap (forward work lives in [`development/roadmap.md`](development/roadmap.md)).
- Not a per-doc review log (we record the result of an audit pass, not the per-doc reasoning).

---

*Last refresh: 2026-05-10 (initial audit, paired with the 0.9.2 v1-prep cleanup pass). Refresh in place when docs are touched.*
