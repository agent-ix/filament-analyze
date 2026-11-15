---
id: SR-17088
title: Gap analysis of VER-98 PR 107
type: SpecReview
analysis: gap-analysis
scope: 'agent-ix/filament-analyze; spec/functional/FR-009-package-runner.md, spec/functional/FR-005-evidence-cli.md, src/, tests/; VER-98 PR #107'
review_set: subset
relationships:
- target: ix://agent-ix/filament-analyze/FR-009
  type: references
---

## Summary

Ticket: VER-98. Reviewed frozen PR #107 head 9186bc4403f4689b236c91345a40835b910f2975 against origin/main (diff origin/main...HEAD). Reviewer-only pass: no repo edits, no builds, tests, cargo or make were run (dispatch brief). A prior review run on an earlier head of this PR (SR-17081..SR-17086, run af323430) is on VER-98; this is a fresh pass on the new head, and prior findings that persist are restated here as new rows. Method: gap-analysis.

## Verdict

CONDITIONAL — one medium coverage gap. FR-005-AC-7 has a tagged test (tests/analysis_semantics.rs:1364, #[trace("TC-007", "FR-005-AC-7")]); its case gaps are SR-17087 FND-003. Plan completion: not assessed. Tooling: `quoin matrix --repo .` (quoin 0.35.0) reports every Test criterion in the repo as untagged, including long-tagged ones such as FR-005-AC-6, so the matrix was not used as evidence; this matches the known #[trace] matrix regression recorded as PLAT-1672 in earlier reviews. A manual AC-to-test check (grep of #[trace] tags and src/) was used instead. No unowned production behaviour was added: the only src change is `validate_selection`, owned by FR-005-AC-7.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-009-AC-29..37 have no tagged test and no implementation: src/ has no `--select` argument, no claim-selection reader and no `selection` rendering (grep of src/package_runner* and src/main.rs). The runner implementation is named in VER-100/VER-102 (both Backlog), but FR-009 and the PR do not record that these ACs are specified ahead of code, so after merge the spec states a `run-package --select` the binary refuses. Expected disposition: deferred to the implementing ticket, stated in the PR. | spec/functional/FR-009-package-runner.md:403-411 |

## Examined Scope

```yaml
scope:
- id: FR-005-AC-7
  path: spec/functional/FR-005-evidence-cli.md
  role: examined
  excerpt: '`validate_report_document` accepts a report without `selection` and a report with a `selection` whose lists are strictly ascending `claim-N` ids, disjoint, with non-empty `selected` equal to the `claim-N` clause ids of `assertions[]`. It refuses a `selection` with an extra member, a missing list, an empty `selected`, an id in both lists, an id that is not `claim-N` or has a leading zero, a list out of ascending order or with a repeated id, or `selected` ids that differ from the assertions'' clauses.'
- id: FR-009-AC-29
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: 'With a package of two analysable claims at distinct regions and `--select` naming a span that contains the region of one, only that claim is analysed: the report''s assertions come from that claim alone, `selection.selected` is that claim''s id and `selection.excluded` is the other''s, and the report validates.'
- id: FR-009-AC-30
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: A span that contains the region of no claim exits `2` naming the span. A span that overlaps one claim's region without containing it exits `2` naming the span and the claim, also when another listed span contains that claim, and also when the span wholly contains a different claim. Each writes no report.
- id: FR-009-AC-31
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: A selection that is not valid JSON, has an unknown or missing member, a wrong `protocol`, an empty `authority` or `identity`, a `start` not below `end`, or is one byte over 1,048,576 bytes exits `2` with the cause on stderr and no report; `--select` given twice or without a value exits `2`.
- id: FR-009-AC-33
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: A package holding one selected Boolean claim and, excluded, a claim whose body is non-Boolean, a claim whose term cannot be lowered, and a claim with no occurrence, run with `--select`, exits as the selected claim alone would; the same package without `--select` exits `2`. A selected claim with any of those three faults exits `2`.
- id: FR-009-AC-35
  path: spec/functional/FR-009-package-runner.md
  role: examined
  excerpt: A selected claim whose body references an expression node, or a parameter, whose region lies outside every listed span, or that has no region, exits `2` naming `claim-N` and `semantic_graph.nodes[M]`; the same claim with a span that also contains that node is analysed.
- id: src/report.rs#validate_selection
  path: src/report.rs
  role: examined
  excerpt: 'fn validate_selection(object: &serde_json::Map<String, Value>) -> Result<(), String>'
```
