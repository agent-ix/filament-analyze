---
id: SR-17094
title: Spec review of VER-17 PR 110
type: SpecReview
analysis: base
scope: 'agent-ix/filament-analyze; spec/functional/FR-009-package-runner.md, spec/test/TC-006-differential-counterexample.md; VER-17 PR #110'
review_set: subset
relationships:
- target: ix://agent-ix/filament-analyze/FR-009
  type: references
- target: ix://agent-ix/filament-analyze/TC-006
  type: references
---

## Summary

Ticket: VER-17. Reviewed frozen PR #110 head 09672942cb0f4a28ccb3e3e603ec12aa0c68aa14 against origin/main (diff origin/main...HEAD; two spec files, +37/-1). Spec-only PR, so one review (spec-review/base) and `quire validate` on the two changed files, which passed clean. Evidence read: `tests/package_runner.rs` (builders `consistent()`, `contradictory()`, `untranslatable()`, the stub `producer-execute` script, the result-state table rows), `src/package_runner.rs` and `src/package_runner/execution.rs` (`parse_run_package_arguments`, `run_package`, `RunnerError::class`), and `src/report.rs` (report members).

## Verdict

CHANGES — three medium findings and two low ones. What holds up: all three builders exist with the stated semantics. The result-state outcome and disposition names (`timed-out`/`inconclusive`, `missing-executable`/`unavailable`, `disagreement`) match `rows()` and the current runner. Exits 0/1/2/3/3/3 match `exit_status`. The library boundary uses only the existing public `parse_run_package_arguments` + `run_package` + `RunnerError::class`, so no new API or consumer is introduced. FR-009-AC-27 is preserved, because every package is built at test time. Stub and real-engine cases are separated. No pin, digest or manifest is added. The PR body handles PGM-01 consistently with the 2026-11-23 VER-17 scope comment.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The library/CLI byte-parity claim holds only if both runs pass the same paths. The report embeds `configuredExecutable`, `argv` and `identity.path` for each engine (src/report.rs:816-823), but TC-006 only says "the same package bytes and the same stub results". Two separately built fixtures (two temp dirs) would give different bytes on the main path. State that both boundaries run with the same argument vector (same `--package`, `--z3`, `--cvc5` and `--producer-execute` paths, with the output removed between runs, as `identical_inputs_give_identical_bytes_and_leave_no_trace` does). | spec/test/TC-006-differential-counterexample.md:36-41, spec/functional/FR-009-package-runner.md:442 |
| FND-002 | medium | The new text says an absent executable or solver is "reported as unavailable with a stated reason, never as a pass". The real-engine repeats it names (FR-009-AC-14, NFR-003-AC-1) are the existing tests that `eprintln!("skipped: ...")` and `return`, which cargo reports as `ok`, and FR-009-AC-14 itself says "the test is skipped". The new sentence contradicts AC-14 and the tests it points to. Reword it to "skipped with a stated reason (FR-009-AC-14)", or specify a mechanism that is not a pass. | spec/test/TC-006-differential-counterexample.md:58-62, spec/functional/FR-009-package-runner.md:443 |
| FND-003 | medium | "RP-SAT and RP-UNSAT are repeated with the real ... Z3 and cvc5 (FR-009-AC-14)" has no criterion behind it. AC-14 covers one package and accepts exit `0` or `1`, and the existing real test runs only `consistent()`. Nothing requires real RP-SAT to exit `0` or real RP-UNSAT to exit `1`, so two implementers would read "repeated" differently. Either give AC-14 the exact per-case exits or drop the RP-UNSAT real repeat. | spec/test/TC-006-differential-counterexample.md:58-60, spec/functional/FR-009-package-runner.md:443 |
| FND-004 | low | The RP-UNSUPPORTED row requires "stderr names `semantic_graph.nodes[N]`", but the library boundary has no stderr. The library equivalent is `RunnerError::message()`, and FR-009-AC-38 omits node naming entirely. Say that the node is named in the library's `RunnerError::message()` and on the CLI's stderr, or drop it from the row (AC-19/AC-23 already cover it on the CLI). | spec/test/TC-006-differential-counterexample.md:47 |
| FND-005 | low | Ceremony that predates this PR is now part of every corpus report. Every report carries `pgm01Envelope {dependency: "agent-ix/filament-contract-ir#20", status: "unavailable"}` (src/report.rs:277-280; FR-005, TC-007, schema v2), a placeholder for a proposal that is closed NOT_PLANNED with no consumer. The repo's CLAUDE.md marks this kind of member for removal. This PR correctly adds nothing to it, but byte parity now asserts over it. Removing it is a cross-repo schema change (Quoin keeps differential-report-v2 fixtures that carry the member), so it needs a follow-up ticket. | src/report.rs:277-280, spec/functional/FR-005-evidence-cli.md:74 |

## Examined Scope

```yaml
scope:
- id: FR-009-AC-38
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: 'The six cases RP-SAT, RP-UNSAT, RP-UNSUPPORTED, RP-TIMEOUT, RP-MISSING-ENGINE and RP-DISAGREE of TC-006, each over a package built at test time (FR-009-AC-27) and stub `producer-execute` results, give through `run_package` and through the spawned `run-package` binary the same exit status (`0`, `1`, `2`, `3`, `3`, `3` in that order), no report for RP-UNSUPPORTED and a valid report for the others whose bytes are equal once every `elapsedMs` is `0`; ...'
- id: FR-009-AC-27
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: Every contract-package document that a test of `run-package` reads is built at test time through the public APIs ...; no contract-package document or copy of another repository's file is committed to this repository.
- id: FR-009-AC-14
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: A run with the real `engineering-assurance` executable and real Z3 and cvc5 on a contract-package fixture built at test time (FR-009-AC-27) exits `0` or `1` and its report validates; the test is skipped with a stated reason when either solver or the executable is absent.
- id: FR-009-AC-17
  path: spec/functional/FR-009-package-runner.md
  role: context_only
  excerpt: For each row of the result-state table other than `completed`, a stub result of that state for one engine yields the stated engine outcome in the published report, the stated disposition, and exit `3`.
- id: TC-006
  path: spec/test/TC-006-differential-counterexample.md
  role: examined
  excerpt: Six named cases, each run through the library boundary (`filament_analyze::run_package` with the parsed `RunPackageArguments`, mapping an `Err` to `RunnerError::class()`) and through the existing CLI boundary ..., over the same package bytes and the same stub `producer-execute` results.
- id: TC-015
  path: spec/test/TC-015-package-runner.md
  role: context_only
  excerpt: Spawn the built binary as a child process. ... Both `sat` with a replaying model exit `0` ... Each non-completed state for one engine, and `sat` against `unsat` exit `3`, both records retained.
- id: NFR-003-AC-1
  path: spec/nonfunctional/NFR-003-runner-invocation.md
  role: context_only
  excerpt: A solver that stalls past its wall-time limit or floods past its output limit, run through the real `producer-execute`, yields a published report with a typed non-conclusive outcome and exit `3`.
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, head 776496d. FR-009-AC-38 and TC-006 now require the same `--package`, `--z3`, `--cvc5`, `--producer-execute` and `--output` paths, with the output removed between the two runs. |
| FND-002 | fixed | Round 1. "never as a pass" was removed. TC-006 now separates RP-MISSING-ENGINE (stub `unavailable`) from real-tool skips, and FR-009-AC-39 says a skip is not evidence for FR-009-AC-38. |
| FND-003 | fixed | Round 1. FR-009-AC-39 requires real RP-SAT (`consistent()`) to exit `0` / `satisfied` and real RP-UNSAT (`contradictory()`) to exit `1` / `refuted`. |
| FND-004 | fixed | Round 1. FR-009-AC-38 and TC-006 require that `RunnerError::message()` and the CLI stderr each name `semantic_graph.nodes[N]`. |
| FND-005 | deferred: VER-113 | Tracked as VER-113, "Remove the unused PGM-01 placeholder from differential reports" (Backlog). Out of scope for this spec-only PR, which does not extend the placeholder. |
| FND-006 | still-open: no fix yet | Round 2, same head 776496d. Found in disposition pass 2 and not yet addressed. TC-006:31-32 still says "unavailability is a result, never a skipped pass". |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | medium | A sentence that predates this PR still says "Real-engine execution is a measured local lane and unavailability is a result, never a skipped pass." It contradicts this PR's own text: FR-009-AC-14/AC-39 and TC-006:63-66 say real-engine tests skip with a stated reason when a tool is absent. This is the same skip-semantics conflict as FND-002. Round 1 recorded FND-002 as fixed, which is true for the sentence FND-002 quoted, but that round missed this residual sentence. Reword it so a real tool's absence is a stated skip and not evidence, and engine unavailability inside a run is a recorded result (RP-MISSING-ENGINE). | spec/test/TC-006-differential-counterexample.md:31-32 |
