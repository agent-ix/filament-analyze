---
id: SR-5650
title: "Evidence review — Filament build identity criterion"
type: SpecReview
analysis: evidence
scope: "agent-ix/filament-analyze; spec/functional/FR-005-evidence-cli.md, spec/test/TC-007-evidence-integrity.md, tests/analysis_semantics.rs, spec/evidence/suites.md, spec/evidence/runs/SUITE-004/2026-11-24T20~3A38~3A04Z.json"
review_set: subset
---

## Summary

Ran the installed deterministic `quoin advise --json` for the declared method and inspected the computed matrix, test binder, TC-007 procedure, and available evidence records. The authored `Test` method is compatible with the catalog's Test recommendation, but the attached procedure spans tagged/untagged/dirty Git states and source archives while the new test exercises only the current Git worktree state.

## Verdict

**CONDITIONAL** — the new test binding does not demonstrate all cases required by its criterion and test procedure.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-005-AC-8's `Test` method is bound to one test that compares a clean, untagged worktree's report version with `git describe`. It does not exercise a tagged build, dirty-tree suffix, or the source-archive package-version fallback that AC-8 and TC-007 require, leaving those verification outcomes without evidence. | FR-005-AC-8, tests/analysis_semantics.rs:1341 |

## Evidence

The computed quoin matrix reports FR-005-AC-8 as `tagged` by `differential_report_producer_version_identifies_git_build`; all 238 obligations show `no run evidence` in the evidence store. The code lead reports the focused exact test passed (1 passed, 12 filtered); no further tests or builds were run in this review. `quoin advise` authored `Test` without a method mismatch and also recommended `property-based-testing` and `runtime-monitoring` under matched rules; the Test recommendation is applicable to this build identity behavior. The `quoin` and `quire` binaries report 0.36.8 and 0.52.0 (engine 0.62.2), respectively, while the workspace release notice expects 0.30.0 and 0.40.0 (engine 0.56.0); these results are recorded from the resolved installed executables.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The requirement and test procedure were narrowed to the current Git-worktree comparison covered by the tagged test. |
| FND-001 | fixed | Round 2 confirms the current Git-worktree-only criterion and procedure match the focused test binding. |
