---
id: SR-20261113192664
title: "Gap analysis — filament-analyze real-tool lane"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-analyze; spec/, tests/, src/, Makefile, scripts/real_tool_test.py, spec/evidence/suites.md"
review_set: subset
---

## Summary

Planless matrix and reverse-gap review of the repository at PR #115. The new runner records honest missing-tool outcomes, but the installed matrix reports no run evidence, and the repository has existing untagged criteria and criteria without named verification symbols.

## Verdict

**FAIL** — the repository matrix contains untagged criteria and no run evidence; the complete matrix claim is limited by the installed Quoin version.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The computed matrix has 44 untagged criteria, 23 criteria whose verification method names no symbol, and no run evidence for any of 237 criteria; the SUITE-004 record is not recognized by the installed Quoin matrix. | spec/evidence/suites.md:8-25 |

## Coverage

- `quoin matrix --repo . --json` with installed Quoin 0.36.8: 28 requirements, 237 criteria; static status counts are 166 tagged, 44 untagged, 23 method-without-symbol, and 4 tagged-by-ignored-test. Evidence status is no run evidence for all 237 criteria.
- PR #115's generated 10-entry error/not-executed SUITE-004 record is untracked and not included in the PR file list. The installed Quoin matrix does not consume its RunEntry trace IDs. A Quoin matrix change is being developed separately; this review did not use its unbuilt worktree state.
- Reverse-gap and stub inspection was limited to the changed Make target, runner, and suite registry. No separate product behavior was added beyond running and recording the existing ignored integration tests.
- Plan completion: not assessed.
- Optional semantic review skipped; it was not requested.
- Matrix behavior has not been rechecked with the supplied release version (Quoin 0.30.0) because PATH resolves to Quoin 0.36.8.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | still-open | The updated SUITE-004 record is committed and has the corrected Cargo identity, but matrix validation is still pending: the installed Quoin 0.36.8 and available branch binary predate the trace-ID matrix change. The repository also has pre-existing untagged criteria outside this PR's scope. No matrix success is claimed. |
