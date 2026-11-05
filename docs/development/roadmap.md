# aegis — Roadmap

> Forward-looking work only. Shipped milestones live in
> [`../../CHANGELOG.md`](../../CHANGELOG.md) — that's the historical
> record per Keep-a-Changelog. Live state lives in
> [`state.md`](state.md). Doc currency in
> [`../doc-health.md`](../doc-health.md).
>
> v1.0.0 cut on 2026-05-10. The API at
> [`api-surface.snapshot`](api-surface.snapshot) is the SemVer-stable
> contract from that point forward — **217 fns** as of the 1.1.7
> regeneration. Additions are non-breaking; removals and renames need a
> major bump.

## How the lines work

Lines run in order. Inside a line, work ships as patches; the line's first
release (`X.Y.0`) is a minor cut and pairs with a P(-1) pass (CLAUDE.md). An
item that adds public API needs a minor and a regenerated snapshot. The last
section is concrete work with no release yet: each item names what it waits on.

## 1.3.x — the daemon reports its own events

`src/main.cyr` is still a stub: it sets up logging, opens the NDJSON sink when
`AEGIS_EVENT_LOG` is set, prints `aegis ready` and exits. Every event in the
sink today was reported by a library caller. This line gives aegis a run loop.

- **ADR first: where scan targets come from.** The library can scan agent
  binaries and package archives and has a config record with
  `periodic_scan_interval_secs`, but nothing says what the daemon scans. The
  choices are a config file listing paths, or registration by daimon. Decide
  before writing the loop, because the second option needs a wire protocol.
- **Run loop.** Load `AegisConfig` (`aegis_config_from_json`), scan on
  `periodic_scan_interval_secs`, and report each finding through
  `aegis_report_event` so it reaches the sink and the auto-quarantine path.
  Exit cleanly on SIGTERM and close the sink.
- **Test it as a process.** A smoke test that starts the daemon, plants a
  world-writable binary in a scanned path, and reads the finding back from the
  sink, under both x86_64 and qemu-aarch64.
- **PAM grammar coverage (audit F-47, deferred from 1.2.2).** Each of these
  makes `pam_parse_config` return Err for the whole file, so aegis cannot
  audit hosts that use them:
  - upper-case keywords (`AUTH Required`), which Linux-PAM accepts;
  - the `substack` control (Fedora/RHEL sshd and login);
  - `@include` (Debian);
  - authselect's symlinked `/etc/pam.d/system-auth`, which the no-follow open
    refuses. Following it safely means resolving the link and checking the
    target is under `/etc/authselect/`, not dropping `O_NOFOLLOW`.

  It fails closed today. `substack` needs a new `PAM_CTL_SUBSTACK` constant
  and `@include` a directive record, so it is minor-sized. Match keywords
  case-insensitively, as `pam_handlers.c` does with `strcasecmp`. While here:
  `pam_list_sessions` answers Ok with no sessions when `/usr/bin/who` is
  missing, because `run_capture` returns Ok.
- **`scripts/bench.sh`** — run the benches and append `bench-history.csv`.
  Today the CSV is appended by hand at each release. Patch-sized: lands any
  time in the line.

## 1.4.x — runs on AGNOS

`cyrius build --agnos` compiles today, but that is all it proves. The file
paths are Linux-only at runtime:

- **Opens.** Every `sys_open(path, flags, mode)` in `src/` uses the Linux
  argument order. agnos's `sys_open` is `(name, namelen, flags)` and takes
  `AO_*` flags. The stdlib's `file_open` bridges both, so move the scanner, the
  PAM reads and the event sink onto it.
- **No `fstat`.** agnos has no fstat-by-fd syscall, and the stdlib's
  `sys_fstat` returns -1 there. Three paths depend on it:
  - `_aegis_stat_modesize` reports every scanned file as unreadable;
  - the event sink's ownership check refuses every file (1.2.2, F-29);
  - the PAM reads size their buffer from it (1.2.2, F-42), so they refuse
    every file.

  The stdlib note (`lib/syscalls_x86_64_agnos.cyr`, above `sys_fstat`) names
  the two ways out: gate the path, or fall back to path-based `sys_stat` and
  document the race that reopens (F-6).
- **Prove it.** Run the test suite under agnos in QEMU, the same way 1.2.2
  added the qemu-aarch64 gate.

## 1.x — waiting on a consumer

Concrete, but not scheduled. Each waits on the consumer it names.

- **A real downstream consumer, end to end.** daimon or argonaut linking
  `src/lib.cyr` in production, including a quarantine path that calls
  `aegis_isolate_agent` and applies the rendered ruleset. Today chakshu
  (0.8.0+) reads the NDJSON event stream, but nothing links the library;
  `docs/examples/basic_consumer.cyr` is the stand-in. When the real one lands,
  decide whether the example stays as a teaching artifact.
- **F-6, the consumer half.** Pass the open fd to the consumer through
  `SecurityFinding`, so the consumer acts on the same inode aegis stat'd.
  That closes the TOCTOU window on the consumer's side. It adds API, so it
  ships in a minor, once a consumer acts on findings.
- **Trace-ID propagation** (`sakshi_trace_set`). Lets an event in the sink be
  matched to the consumer request that caused it. Waits on a consumer that
  calls `aegis_report_event` and has a trace ID to pass in.

## What this file is NOT

- Not a CHANGELOG (that's [`../../CHANGELOG.md`](../../CHANGELOG.md)).
- Not a state snapshot (that's [`state.md`](state.md)).
- Not a v1.0 sign-off checklist — that ran during the 1.0.0 cut and
  passed; the verification record is in CHANGELOG `[1.0.0] § Verified at sign-off`.

Add work here only when it is concrete enough to act on, and remove it in the
release that ships it. Don't pre-populate empty lines ("Future", "2.0") — those
rot.
