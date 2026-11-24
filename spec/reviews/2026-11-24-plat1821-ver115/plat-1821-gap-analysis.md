---
id: SR-5645
title: "Gap analysis — Filament repository audit"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-analyze; spec/, src/, tests/, build.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-analyze
    type: references
---

## Summary

Performed a planless repository audit of the Filament specification, computed matrix, trace tags, source, and tests for PLAT-1821 and VER-115. The new FR-005-AC-8 criterion is tagged, but the repository matrix also contains untagged and ignored-only criteria and untracked trace symbols.

## Verdict

**FAIL** — the repository matrix contains untagged criteria and criteria backed only by ignored tests.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The computed matrix contains 44 untagged criteria with no bound test symbols. | 44 criteria listed in Coverage |
| FND-002 | high | Four criteria are tagged only by ignored tests, so no runnable test backs them. | FR-010-AC-7, FR-012-AC-1, FR-012-AC-8, NFR-032-AC-3 |
| FND-003 | medium | The coverage report contains 80 untracked trace symbols whose IDs bind to no declared criterion. | quire coverage: untracked_symbols |

## Coverage

- Reconciliation: quoin matrix (quoin 0.36.8, quire 0.52.0 (engine 0.62.2)).
- Plan completion: not assessed
- Criteria: tagged 167 · untagged 44 · tagged-by-ignored-test 4 · method-without-symbol 23
- Untagged: FR-005-AC-5, FR-009-AC-10, FR-010-AC-11, FR-033-AC-5, FR-033-AC-8, FR-034-AC-1 through AC-8, FR-042-AC-1, NFR-001-M-1 through M-9, NFR-002-M-1 through M-3, NFR-003-M-1 through M-2, NFR-004-M-1 through M-5, NFR-020-M-1 through M-6, NFR-030-M-1 through M-2, NFR-031-M-1 through M-2, and NFR-032-M-1.
- Evidence: no run evidence for all 238 obligations; this is reported once and is not a per-criterion finding.
- Untracked symbols: 80; coverage also reports 32 unbacked rows, 18 no-symbol rows, and 314 diagnostics.
- Untraced behaviors / stubs: no new unowned behavior or source/test stub was found in the changed implementation; the full repository includes existing ignored real-solver and VER-82 lane tests.
- Semantic review: skipped (not requested; optional).
- The matrix extraction was engine-backed, but the command versions differ from the versions specified in the local Quoin/Quire release notice. No grep fallback was used.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | The 44 untagged criteria predate PLAT-1821 and VER-115 and do not describe behavior changed by this PR; the repository-wide gap remains visible under AP-001 for project-level follow-up. |
| FND-002 | deferred | These four ignored-only criteria predate PLAT-1821 and VER-115 and do not describe behavior changed by this PR; their coverage status remains visible under AP-001 for project-level follow-up. |
| FND-003 | deferred | The 80 untracked trace symbols predate PLAT-1821 and VER-115 and do not describe behavior changed by this PR; the repository-wide trace debt remains visible under AP-001 for project-level follow-up. |
| FND-001 | deferred | Round 2 confirms the 44 repository-wide untagged criteria are unchanged and outside the two ticket changes; the AP-001 follow-up remains visible. |
| FND-002 | deferred | Round 2 confirms the four repository-wide ignored-only criteria are unchanged and outside the two ticket changes; the AP-001 follow-up remains visible. |
| FND-003 | deferred | Round 2 confirms the 80 repository-wide untracked trace symbols are unchanged and outside the two ticket changes; the AP-001 follow-up remains visible. |
