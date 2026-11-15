---
id: SR-17093
title: Dependency analysis of VER-98 PR 107
type: SpecReview
analysis: dependency
scope: 'agent-ix/filament-analyze; spec/functional/FR-009-package-runner.md, spec/functional/FR-005-evidence-cli.md; cross-repo context agent-ix/quoin FR-190, FR-151 on origin/main; VER-98 PR #107'
review_set: subset
relationships:
- target: ix://agent-ix/filament-analyze/FR-009
  type: references
---

## Summary

Ticket: VER-98. Reviewed frozen PR #107 head 9186bc4403f4689b236c91345a40835b910f2975 against origin/main (diff origin/main...HEAD). Reviewer-only pass: no repo edits, no builds, tests, cargo or make were run (dispatch brief). A prior review run on an earlier head of this PR (SR-17081..SR-17086, run af323430) is on VER-98; this is a fresh pass on the new head, and prior findings that persist are restated here as new rows. Method: spec-review/dependency.

## Verdict

FAIL — the selection contract and its only named producer still disagree. The empty-selection outcome (prior SR-17086 FND-002) is now a distinct `empty selection` cause and is resolved. Gap 7 correctly names QSL-1900 as the producer of `analysis_claim` regions without defining it.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-009 requires protocol `filament-analyze.claim-selection/v1` with spans `{authority, identity, start, end}` in bytes and makes a wrong protocol or unknown/missing member exit 2. quoin FR-190 on quoin origin/main (50643ccc) still emits `quire-analyze.claim-selection/v1` with `{path, start_line, end_line}`. Every document the specified producer writes is refused, so no run over approved rules can succeed. One side must change, with a linked quoin ticket; persists from SR-17086 FND-001. | spec/functional/FR-009-package-runner.md:56-66 |
| FND-002 | medium | VER-98's FR-009 amendment AC 4 (the quoin FR-151 adapter accepts the revised report) is specified nowhere. The PR body says it lives in the quoin PR, but quoin FR-151 on origin/main does not mention `selection` and its projection is a closed allowlist. A selected-run report therefore has no specified consumer. Persists from SR-17086 FND-003; needs a quoin ticket or an FR-151 amendment. | spec/functional/FR-009-package-runner.md:368 |

## Examined Scope

```yaml
scope:
- id: FR-009-ClaimSelection
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: Callers that must analyse only approved formal rules pass `--select FILE`. The file is a `filament-analyze.claim-selection/v1` JSON document
- id: FR-009-ClaimSelection-Document
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: 'A span names a byte interval under a source document in the vocabulary of the package''s own source map (`CheckedSourceRegion`): `authority` and `identity` are the two members of the region''s source reference, `start` is the inclusive byte offset and `end` the exclusive byte offset, with `start` below `end`.'
- id: FR-009-ClaimSelection-Empty
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: '**Empty selection.** A document whose `spans` is empty is well formed and selects nothing. The runner shall exit `2` with the stderr cause `empty selection` and write no report'
- id: FR-009-CON-1
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: The runner shall emit `filament.differential-report/v2` as defined by FR-005 and `schemas/differential-report-v2.schema.json`. The schema is revised by the optional closed `selection` member only; no other member, outcome or rule changes, and a report without `selection` validates as before.
- id: quoin/FR-190
  path: agent-ix/quoin spec/functional/FR-190-formal-rule-analysis-claim-selection.md
  role: context_only
  excerpt: report, a `quire-analyze.claim-selection/v1` document that lists the source spans of `approved` declarations only ... with one entry `{path, start_line, end_line}` per `approved` declaration
- id: quoin/FR-151
  path: agent-ix/quoin spec/functional/FR-151-differential-report-v2-adapter.md
  role: context_only
  excerpt: projection is a closed allowlist of data the FR-116 qualifier consumes
```
