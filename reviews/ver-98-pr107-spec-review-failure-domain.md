---
id: SR-17092
title: Failure-domain analysis of VER-98 PR 107
type: SpecReview
analysis: failure-domain
scope: 'agent-ix/filament-analyze; spec/functional/FR-009-package-runner.md; VER-98 PR #107'
review_set: subset
relationships:
- target: ix://agent-ix/filament-analyze/FR-009
  type: references
---

## Summary

Ticket: VER-98. Reviewed frozen PR #107 head 9186bc4403f4689b236c91345a40835b910f2975 against origin/main (diff origin/main...HEAD). Reviewer-only pass: no repo edits, no builds, tests, cargo or make were run (dispatch brief). A prior review run on an earlier head of this PR (SR-17081..SR-17086, run af323430) is on VER-98; this is a fresh pass on the new head, and prior findings that persist are restated here as new rows. Method: spec-review/failure-domain.

## Verdict

CONDITIONAL — one low finding. Prior SR-17085 findings (partial overlap beside a whole claim, reached-node source, path normalization, excluded-claim faults) are addressed on this head by the Partial overlap, Reached nodes, exact-comparison and Excluded claims rules and AC-30/33/35/37.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The numeric domain of `start` and `end` is not stated: a negative value, a fraction (`1.5`), a string, or a value above the source map's offset type is neither listed as a usage error nor covered by AC-31. Most decoders refuse them, but the spec should name them (non-negative integers within the region offset type) so the refusal is a requirement, not an accident. | spec/functional/FR-009-package-runner.md:58-63 |

## Examined Scope

```yaml
scope:
- id: FR-009-ClaimSelection-Document
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: 'A span names a byte interval under a source document in the vocabulary of the package''s own source map (`CheckedSourceRegion`): `authority` and `identity` are the two members of the region''s source reference, `start` is the inclusive byte offset and `end` the exclusive byte offset, with `start` below `end`.'
- id: FR-009-AC-31
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: A selection that is not valid JSON, has an unknown or missing member, a wrong `protocol`, an empty `authority` or `identity`, a `start` not below `end`, or is one byte over 1,048,576 bytes exits `2` with the cause on stderr and no report; `--select` given twice or without a value exits `2`.
- id: FR-009-ClaimSelection-Empty
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: '**Empty selection.** A document whose `spans` is empty is well formed and selects nothing. The runner shall exit `2` with the stderr cause `empty selection` and write no report'
- id: FR-009-AC-35
  path: spec/functional/FR-009-package-runner.md
  role: context_only
  excerpt: A selected claim whose body references an expression node, or a parameter, whose region lies outside every listed span, or that has no region, exits `2` naming `claim-N` and `semantic_graph.nodes[M]`; the same claim with a span that also contains that node is analysed.
```
