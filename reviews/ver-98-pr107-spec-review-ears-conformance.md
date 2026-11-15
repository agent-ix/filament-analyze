---
id: SR-17090
title: EARS conformance of VER-98 PR 107
type: SpecReview
analysis: ears-conformance
scope: 'agent-ix/filament-analyze; spec/functional/FR-009-package-runner.md, spec/functional/FR-005-evidence-cli.md; VER-98 PR #107'
review_set: subset
relationships:
- target: ix://agent-ix/filament-analyze/FR-009
  type: references
---

## Summary

Ticket: VER-98. Reviewed frozen PR #107 head 9186bc4403f4689b236c91345a40835b910f2975 against origin/main (diff origin/main...HEAD). Reviewer-only pass: no repo edits, no builds, tests, cargo or make were run (dispatch brief). A prior review run on an earlier head of this PR (SR-17081..SR-17086, run af323430) is on VER-98; this is a fresh pass on the new head, and prior findings that persist are restated here as new rows. Method: spec-review/ears-conformance.

## Verdict

PASS — the new and edited requirement statements are singular and use an EARS form. FR-009-CON-2 and FR-009-CON-3 are now separate single-shall constraints (prior SR-17083 FND-001 is resolved). The Claim selection rules use event/state forms with one response each. `quire validate` reports no ears diagnostic.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined Scope

```yaml
scope:
- id: FR-009-CON-1
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: The runner shall emit `filament.differential-report/v2` as defined by FR-005 and `schemas/differential-report-v2.schema.json`. The schema is revised by the optional closed `selection` member only; no other member, outcome or rule changes, and a report without `selection` validates as before.
- id: FR-009-CON-2
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: The runner shall store no approval state, approval list, key or signature material. A selection is data supplied by the caller for one run.
- id: FR-009-CON-3
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: The runner shall verify no signature.
- id: FR-009-Behavior-1
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: When invoked with the five required arguments and an admitted package, the runner shall build one consistency request over the analysable clauses of the package.
- id: FR-009-ClaimSelection-Unmatched
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: '**Unmatched span.** If a listed span contains the region of no claim, the runner shall exit `2` naming the span and write no report.'
- id: FR-009-ClaimSelection-Empty
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: '**Empty selection.** A document whose `spans` is empty is well formed and selects nothing. The runner shall exit `2` with the stderr cause `empty selection` and write no report'
```
