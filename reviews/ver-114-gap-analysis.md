---
id: SR-17097
title: gap-analysis of VER-114 PR 113
type: SpecReview
analysis: gap-analysis
scope: 'agent-ix/filament-analyze; src/hybrid_tool.rs, README.md, reviews/; VER-114 PR #113'
review_set: subset
---

## Summary

Ticket: VER-114. This is a manual check of the ticket's two acceptance criteria against tests and code, labelled as manual. It is not the full `quoin:gap-analysis` matrix audit. The change touches only a `#[cfg(test)]` fixture and README. No spec FR/AC in spec/ owns it, so a matrix or reverse-gap pass has nothing to bind. Not run: the matrix computation, the stub scan and semantic review. All are out of scope for a test-fixture change.

## Verdict

FAIL: AC-2 is not met.

- AC-1 ("read from environment or configuration"): met and tested. `phaver_config_reads_tool_paths_from_the_environment` failed under a mutation that swapped which variable fed the executable. `phaver_config_names_the_unset_variable` failed under a mutation that added a library fallback.
- AC-2 ("no absolute home paths in the repo"): not met. Two tracked review records still contain `/home/peter` paths. The only test for AC-2 scans one file, so it cannot detect the remaining hits.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Ticket AC-2, "no absolute home paths in the repo", is unmet at the PR head. Two hits remain: reviews/2026-10-21-plat-1173-gap-analysis.md:25 and reviews/ir-730-gap-analysis.md:34 (`quoin matrix --repo /home/peter/...`). The test offered for AC-2, `source_has_no_absolute_home_paths`, only scans src/hybrid_tool.rs, so it passes while AC-2 fails. Close it by rewriting the two paths. A grep at review time is enough evidence for AC-2; no new content-scanning test is needed. | reviews/2026-10-21-plat-1173-gap-analysis.md:25, reviews/ir-730-gap-analysis.md:34, src/hybrid_tool.rs:1434-1439 |

## Coverage

Plan completion: not assessed

Manual check of the ticket's acceptance criteria against tests: AC-1 is bound to two unit tests, and both are mutation-verified. AC-2 is bound to one single-file test, which is too weak to prove it. Semantic review: skipped (optional, not requested).

## Examined Scope

```yaml
scope:
- id: VER-114-AC-1
  path: src/hybrid_tool.rs
  role: examined
  excerpt: read from environment or configuration
- id: VER-114-AC-2
  path: reviews/
  role: examined
  excerpt: no absolute home paths in the repo
```
