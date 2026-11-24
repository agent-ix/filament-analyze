---
id: SR-5648
title: "Integrity review — Filament build identity and soundness scope"
type: SpecReview
analysis: integrity
scope: "agent-ix/filament-analyze; spec/functional/FR-005-evidence-cli.md, spec/test/TC-007-evidence-integrity.md, spec/assurance/AD-001-analyze-architecture.md, spec/anomalies/index.md, spec/anomalies/AN-001-main-ci-dispatch-only.md, spec/anomalies/AN-002-build-identity.md, spec/anomalies/AN-003-stale-cli-scope.md"
review_set: subset
---

## Summary

Reviewed the changed requirement and architecture claims for atomicity, consistency, scope boundaries, and completeness. The theory table distinguishes SMT results from hybrid reachability and states the external assumptions; the new test procedure includes build-identity branches that the implementation test does not verify.

## Verdict

**CONDITIONAL** — the changed Test case claims source-archive fallback verification without a test that exercises the fallback.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-007 requires checking the package-version fallback for a source archive without Git metadata, but the added test invokes `git describe` in the current worktree and compares only that result. The criterion's verification contract is therefore not fully demonstrated by its referenced test procedure and implementation. | spec/test/TC-007-evidence-integrity.md:35-38 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-005-AC-8 and TC-007 no longer claim tagged, dirty, or source-archive cases beyond the focused test. |
| FND-001 | fixed | Round 2 confirms FR-005-AC-8 and TC-007 contain no tagged-build, dirty-build, or source-archive verification claim. |
