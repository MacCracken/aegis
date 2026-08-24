# 0006 — Rendering refuses invalid input rather than emitting it

**Status**: Accepted
**Date**: 2026-08-24

## Context

The 1.1.6 P(-1) pass found that `pam_validate_rule` was a **denylist** of ten
shell metacharacters, while the functions that consume a rule —
`pam_render_rule` and `pam_render_config` — join fields with TAB and lines with
LF. Neither TAB nor LF was on the denylist, so a module path or argument
containing a newline rendered as *additional PAM stack entries*. An injected
`auth sufficient pam_permit.so` makes every password authenticate for that
service (audit finding F-11).

Two separate problems sat behind that:

1. The validator guarded the wrong threat model — it defended against a shell,
   but the sink is a tab-and-newline-delimited config file, so the delimiters
   are the load-bearing characters.
2. **Neither render function called the validator.** `pam_validate_rule` had no
   caller anywhere in the repo. A consumer was expected to know to call it
   first, and nothing enforced that.

Fixing only (1) would have left the second half standing: a correct validator
that nothing invokes still emits a dangerous config.

The same question had already been answered once in this codebase. Audit
findings F-2 and F-3 (2026-05-10) hardened the nein firewall builders, and the
resolution there was that `aegis_isolate_agent` / `aegis_rate_limit_agent`
**return `0`** on invalid input rather than building a ruleset that
`firewall_validate` would later reject — because `aegis_firewall_render` does
not call `aegis_firewall_validate` first, so "the caller will validate" is not
something the code can rely on.

## Decision

**Validation runs at the sink, and the sink fails closed.**

- `pam_render_rule` calls `pam_validate_rule` and returns `0` for a rule that
  would not validate.
- `pam_render_config` returns `0` for the whole document if *any* rule fails,
  rather than skipping the offending rule.
- The validators are **allowlists** over printable ASCII plus an explicit
  punctuation set, not denylists.

Rendering a config with one rule silently dropped is not a safe degradation: for
PAM, dropping a `required` line weakens the stack exactly as much as injecting a
`sufficient` one. All-or-nothing is the only behaviour a caller can reason about.

## Consequences

**Contract change.** `pam_render_rule` and `pam_render_config` previously always
returned a `Str`; they now return `0` on invalid input. Callers must check.
This mirrors the `0`-on-invalid convention the firewall builders have used since
0.9.3, so the library is now consistent: **every aegis function that emits
security-relevant text refuses rather than emits.**

Callers that previously relied on rendering never failing will see `0` where they
expected a `Str`. That is the intended outcome — the inputs that now return `0`
are exactly the inputs that previously produced an injection-capable document.

**Allowlists over denylists, permanently.** F-2 established this for `agent_id`
and `agent_addr`; F-11 showed pam.cyr had been written before that lesson landed.
The rule for this repo: any value that reaches generated security-relevant text
(nftables rulesets, PAM configs) is validated by allowlist, at the point of
emission, and emission fails closed.

**Cost.** `pam_render_rule` now re-validates on every call, including for rules
that a caller already validated. The rendering path is not hot — configs are
rendered on operator action, not per event — so the redundant scan is worth the
guarantee that no code path can skip it.

## Alternatives considered

- **Validate in the constructor (`pam_rule_new`).** Rejected: the constructor
  cannot return an error in this API shape, and a rule can be mutated after
  construction through the derived setters, so a constructor-time check is
  bypassable by design.
- **Escape rather than reject.** Rejected: PAM has no escaping mechanism for
  newlines within a rule line. There is no encoding of "a module path containing
  a newline" that Linux-PAM would read back as one field, so the value is not
  representable and rejecting it is the honest answer.
- **Skip invalid rules and render the rest.** Rejected as unsafe — see above.
