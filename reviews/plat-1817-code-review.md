---
id: SR-20261113192663
title: "Code review — filament-analyze real-tool lane"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-analyze; Makefile, scripts/real_tool_test.py, spec/evidence/suites.md"
review_set: subset
---

## Summary

Reviewed the real-solver test runner, its suite registry, and the focused validation recorded in PR #115. The missing-tool path correctly records error/not-executed entries, but the runner's pass/fail/record-failure behavior has no focused tests, and the only retained record predates the Cargo shim fix.

## Verdict

**CONDITIONAL** — add focused runner tests and retain a post-fix SUITE-004 record with the correct Cargo identity before treating the evidence lane as complete.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The runner's outcome classification and Quoin recording failure paths have no focused tests; only the missing-solver path was exercised. | scripts/real_tool_test.py:100-336 |
| FND-002 | medium | The sole retained SUITE-004 record predates the Cargo shim fix and records `rustup 1.29.3` as Cargo; no post-fix record is included in the PR. | spec/evidence/runs/SUITE-004/2026-11-24T20~3A19~3A28Z.json |
