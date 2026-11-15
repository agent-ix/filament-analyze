---
id: SR-17091
title: Integrity analysis of VER-98 PR 107
type: SpecReview
analysis: integrity
scope: 'agent-ix/filament-analyze; spec/functional/FR-009-package-runner.md; VER-98 PR #107'
review_set: subset
relationships:
- target: ix://agent-ix/filament-analyze/FR-009
  type: references
---

## Summary

Ticket: VER-98. Reviewed frozen PR #107 head 9186bc4403f4689b236c91345a40835b910f2975 against origin/main (diff origin/main...HEAD). Reviewer-only pass: no repo edits, no builds, tests, cargo or make were run (dispatch brief). A prior review run on an earlier head of this PR (SR-17081..SR-17086, run af323430) is on VER-98; this is a fresh pass on the new head, and prior findings that persist are restated here as new rows. Method: spec-review/integrity.

## Verdict

FAIL — one high finding: two Claim-selection rules contradict on a common input. Prior SR-17084 findings (AC-32 oracle, schema-vs-validator split, order rule, unreachable AC-30 case) are resolved on this head.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The Unmatched-span rule contradicts the Reached-nodes rule. A selected claim that reaches a parameter or expression node declared elsewhere in the file needs that node's region inside a listed span (Reached nodes, AC-35), but a span added to cover that declaration contains no claim and exits 2 as unmatched (AC-30). The only passing selection is one span stretching from the claim over the declaration, which also selects or partly overlaps every claim in between (draft rules included). Count a span as matched when it holds a selected claim or a node a selected claim reaches, and add an AC for a separate span over a shared parameter. | spec/functional/FR-009-package-runner.md:82-88 |
| FND-002 | medium | AC-33's last clause requires a selected claim with no occurrence to exit 2, but the Selected rule says a claim with no region is always excluded, so a selected occurrence-less claim cannot exist and that case cannot be built or fail. The same unreachable case appears in Source spans ("occurrences is empty for a selected claim", line 156). Drop the no-occurrence case from the selected half, keeping it for reached nodes only. | spec/functional/FR-009-package-runner.md:407 |

## Examined Scope

```yaml
scope:
- id: FR-009-ClaimSelection-Unmatched
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: '**Unmatched span.** If a listed span contains the region of no claim, the runner shall exit `2` naming the span and write no report.'
- id: FR-009-ClaimSelection-Reached
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: '**Reached nodes.** The translation inlines the target of every `Reference` and declares every `value/parameter` that a selected claim reaches. If the region of such a node does not lie inside a listed span, or the node has no region, the runner shall exit `2` naming `claim-N` and `semantic_graph.nodes[M]` of the node reached, and write no report.'
- id: FR-009-ClaimSelection-Selected
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: An `analysis_claim` node is selected exactly when its region lies inside a listed span. Every other `analysis_claim` node, including one with no region, is excluded.
- id: FR-009-AC-30
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: A span that contains the region of no claim exits `2` naming the span. A span that overlaps one claim's region without containing it exits `2` naming the span and the claim, also when another listed span contains that claim, and also when the span wholly contains a different claim. Each writes no report.
- id: FR-009-AC-33
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: A package holding one selected Boolean claim and, excluded, a claim whose body is non-Boolean, a claim whose term cannot be lowered, and a claim with no occurrence, run with `--select`, exits as the selected claim alone would; the same package without `--select` exits `2`. A selected claim with any of those three faults exits `2`.
- id: FR-009-AC-35
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: A selected claim whose body references an expression node, or a parameter, whose region lies outside every listed span, or that has no region, exits `2` naming `claim-N` and `semantic_graph.nodes[M]`; the same claim with a span that also contains that node is analysed.
```
