---
id: SR-17099
title: "Gap analysis of README polarity and hybrid naming fix and hello removal"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-analyze; README.md, docs/images, src/lib.rs, tests/integration.rs, spec/assurance/AD-001-analyze-architecture.md"
review_set: subset
---
# Gap Analysis of README Polarity and Hybrid Naming Fix and Hello Removal

## Summary

Ticket: PLAT-1725 (and PLAT-1726). PR agent-ix/filament-analyze#114 at head
0c915b54ef4f34646baf0d58d8b8a7b97f9db2ca, diff-scoped. Plan completion: not assessed.
The diff changes no production behavior: it removes a placeholder `pub fn hello()` with
no owning requirement and its two placeholder tests, and edits docs. No acceptance
criterion loses a test; no stub or coverage inflation is introduced. The matrix was not
recomputed because no spec or traced test changed. Every `PLAT-1725` expected outcome
holds in the README. One stale claim of the same kind remains outside the README.

## Verdict

**PASS with one low.** The README now matches the code; the architecture description
still names the removed provider.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AD-001 still lists `src/hybrid_synthesis.rs` `reach` as a separate method provider; `reach` was removed (FR-042-AC-2) and the provider is `reach_hybrid` in `src/hybrid_tool.rs`. Same stale claim the PR fixes in the README; outside this diff, may be deferred to a follow-up. | `spec/assurance/AD-001-analyze-architecture.md:45` |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, ccb59fc: AD-001 line 45 now lists `src/hybrid_tool.rs` `reach_hybrid` (delegated PHAVer tool path) and `src/hybrid_synthesis.rs` `synthesize`, `validate_candidate`. |
