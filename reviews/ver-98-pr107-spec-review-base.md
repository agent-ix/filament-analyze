---
id: SR-17089
title: Spec review of VER-98 PR 107
type: SpecReview
analysis: base
scope: 'agent-ix/filament-analyze; spec/functional/FR-009-package-runner.md, spec/functional/FR-005-evidence-cli.md, spec/interface/interface-001-analysis-api.md, spec/test/TC-007-evidence-integrity.md, spec/test/TC-015-package-runner.md, schemas/differential-report-v2.schema.json; VER-98 PR #107'
review_set: subset
relationships:
- target: ix://agent-ix/filament-analyze/FR-009
  type: references
---

## Summary

Ticket: VER-98. Reviewed frozen PR #107 head 9186bc4403f4689b236c91345a40835b910f2975 against origin/main (diff origin/main...HEAD). Reviewer-only pass: no repo edits, no builds, tests, cargo or make were run (dispatch brief). A prior review run on an earlier head of this PR (SR-17081..SR-17086, run af323430) is on VER-98; this is a fresh pass on the new head, and prior findings that persist are restated here as new rows. Method: spec-review/base.

## Verdict

CONDITIONAL — one medium finding. `quire validate` on the five changed spec files is clean (only the unrelated module-level semantic.unused-import notice). The FR-005 `selection` text, interface-001, TC-007 and TC-015 now agree with each other; the prior SR-17081 findings (line vs byte spans, AC-13 help text, AC order) are resolved on this head. The ticket's line-based spans were replaced by byte regions, as the PR body explains; that is a reasoned spec choice, judged in SR-17093.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The Input section still says the one request's selected statements are "all analysable clauses", and Behavior still says the runner builds the request "over the analysable clauses of the package", with no `--select` qualifier. With `--select` only selected claims enter the request (Claim selection, AC-29), so both sentences contradict the new section as written. Qualify both as "every selected analysable clause". | spec/functional/FR-009-package-runner.md:41-43 |

## Examined Scope

```yaml
scope:
- id: FR-009
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: The runner builds ONE consistency request whose selected statements are all analysable clauses, with no assumptions and no binding groups, and, without `--select`, no per-clause selection.
- id: FR-009-Behavior-1
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: When invoked with the five required arguments and an admitted package, the runner shall build one consistency request over the analysable clauses of the package.
- id: FR-009-CON-1
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: The runner shall emit `filament.differential-report/v2` as defined by FR-005 and `schemas/differential-report-v2.schema.json`. The schema is revised by the optional closed `selection` member only; no other member, outcome or rule changes, and a report without `selection` validates as before.
- id: FR-009-AC-29
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: 'With a package of two analysable claims at distinct regions and `--select` naming a span that contains the region of one, only that claim is analysed: the report''s assertions come from that claim alone, `selection.selected` is that claim''s id and `selection.excluded` is the other''s, and the report validates.'
- id: FR-005-AC-7
  path: spec/functional/FR-005-evidence-cli.md
  role: examined
  excerpt: '`validate_report_document` accepts a report without `selection` and a report with a `selection` whose lists are strictly ascending `claim-N` ids, disjoint, with non-empty `selected` equal to the `claim-N` clause ids of `assertions[]`. It refuses a `selection` with an extra member, a missing list, an empty `selected`, an id in both lists, an id that is not `claim-N` or has a leading zero, a list out of ascending order or with a repeated id, or `selected` ids that differ from the assertions'' clauses.'
- id: TC-007
  path: spec/test/TC-007-evidence-integrity.md
  role: examined
  excerpt: For the `selection` member (FR-005-AC-7), validate a v2 document with no `selection` and one whose assertion clauses are `claim-N` clauses and whose `selection` is well formed; both must pass.
- id: TC-015
  path: spec/test/TC-015-package-runner.md
  role: examined
  excerpt: '23. `--select` cases (AC-29 to AC-33, AC-35 to AC-37): a two-claim built package whose source map regions are set at test time, with a selection naming one region; ...'
- id: interface-001
  path: spec/interface/interface-001-analysis-api.md
  role: context_only
  excerpt: and, for a selected run only, the closed `selection` member of FR-005 naming selected and excluded clause ids.
```
