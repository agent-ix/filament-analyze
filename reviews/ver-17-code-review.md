---
id: SR-17095
title: Code review of VER-17 PR 111
type: SpecReview
analysis: code-review
scope: 'agent-ix/filament-analyze; tests/package_runner.rs (PR #111 head 851a136261a8696ae3b3e867d95f2a7adc5cfca5, diff origin/main...HEAD); checked against FR-009-AC-38, FR-009-AC-39 and TC-006 on origin/main'
review_set: subset
relationships:
- target: ix://agent-ix/filament-analyze/FR-009
  type: references
- target: ix://agent-ix/filament-analyze/TC-006
  type: references
---

## Summary

Ticket: VER-17. Reviewed PR #111 (one file, tests/package_runner.rs, +239) against FR-009-AC-38, FR-009-AC-39 and the TC-006 `run-package` corpus on origin/main. Method: code-review with the rust-review lane folded in. Ran `cargo test --test package_runner -- rp_` on toolchain 1.98.1: 8 passed, 0 failed. Both FR-009-AC-39 tests skipped locally with the stated reason "z3 is not on PATH", so the real-engine lane was not exercised here. gap-analysis was not run: the diff has no production code.

## Verdict

FIX NEEDED. The six cases exist, each runs through `run_package` and the spawned binary with one argument vector, the exit codes are 0, 1, 2, 3, 3, 3, RP-UNSUPPORTED names `semantic_graph.nodes[N]` in `message()` and stderr with no producer-execute request, the dispositions are checked, and every case has a healthy control. The real-engine tests skip with a stated reason. The diff adds no input format, digest, pin, manifest, PGM-01 or CI change. Two medium gaps remain: RP-DISAGREE omits the swapped-engine run that TC-006 requires, and the report comparison normalises through `serde_json::Value`, so it is not the byte comparison FR-009-AC-38 states.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | RP-DISAGREE runs only Z3 `sat` / cvc5 `unsat`. TC-006 states the case as that "then the same with the engines swapped", so an implementation that resolves a disagreement in favour of one engine, or orders engines asymmetrically, is not caught. | tests/package_runner.rs:2532-2541 |
| FND-002 | medium | `through_both` compares `serde_json::to_vec(&normalised(bytes))` for the two reports. Re-serialising through `Value` erases key order, whitespace and number formatting, so two reports that differ in bytes still compare equal. FR-009-AC-38 requires bytes equal once every `elapsedMs` is `0`. Zero `elapsedMs` in the text and compare raw bytes, or also assert the raw bytes differ only in `elapsedMs` values. | tests/package_runner.rs:2386-2395 |
| FND-003 | low | RP-DISAGREE asserts only that `agreedStatus` is neither `satisfied` nor `refuted`; TC-006 states "no agreed status". The file already asserts `agreedStatus == Value::Null` elsewhere (line 2212); assert it here too. | tests/package_runner.rs:2429-2433 |
| FND-004 | low | `real_tools` / `real_case` repeat the tool lookup and the `--z3`/`--cvc5` argument substitution already written twice in the file (lines 2019-2033, 2116-2132). Extract one helper and reuse it. | tests/package_runner.rs:2544-2563 |

## Examined Scope

```yaml
scope:
- id: FR-009-AC-38
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: The six cases RP-SAT, RP-UNSAT, RP-UNSUPPORTED, RP-TIMEOUT, RP-MISSING-ENGINE and RP-DISAGREE of TC-006 ... run through `run_package` and through the spawned `run-package` binary with the same ... paths ... give the same exit status (`0`, `1`, `2`, `3`, `3`, `3` in that order), no report for RP-UNSUPPORTED and a valid report for the others whose bytes are equal once every `elapsedMs` is `0`
- id: FR-009-AC-39
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: With the real `engineering-assurance` executable and real Z3 and cvc5, the RP-SAT package exits `0` with agreed status `satisfied` and the RP-UNSAT package exits `1` with agreed status `refuted`, each with a report that validates; the test is skipped with a stated reason when the executable or either solver is absent
- id: TC-006
  path: spec/test/TC-006-differential-counterexample.md
  role: examined
  excerpt: 'RP-DISAGREE | `consistent()` | Z3 `completed` `sat` with the replaying model, cvc5 `completed` `unsat`; then the same with the engines swapped | `3` | published, valid, outcomes `sat` and `unsat`, disposition `disagreement`, both engine records retained and no agreed status'
```
