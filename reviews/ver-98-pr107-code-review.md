---
id: SR-17087
title: Code review (with rust-review) of VER-98 PR 107
type: SpecReview
analysis: code-review
scope: 'agent-ix/filament-analyze; src/report.rs, tests/analysis_semantics.rs, schemas/differential-report-v2.schema.json; VER-98 PR #107'
review_set: subset
relationships:
- target: ix://agent-ix/filament-analyze/FR-005
  type: references
---

## Summary

Ticket: VER-98. Reviewed frozen PR #107 head 9186bc4403f4689b236c91345a40835b910f2975 against origin/main (diff origin/main...HEAD). Reviewer-only pass: no repo edits, no builds, tests, cargo or make were run (dispatch brief). A prior review run on an earlier head of this PR (SR-17081..SR-17086, run af323430) is on VER-98; this is a fresh pass on the new head, and prior findings that persist are restated here as new rows. Method: code-review.

## Verdict

CONDITIONAL — medium findings only. The validator now admits `selection` (the prior SR-17082 FND-001 split is closed), the closed-shape rules sit in the schema and the order/disjointness/assertion rules in `validate_selection`. No vendoring, no new digest, pin or ceremony was introduced; the protocol name carries a wire version only. Gates were not run (brief). Note: the PR body calls the change spec-only and the ticket says no cargo gates run, but the head changes src/report.rs and a test, so the full gates must run once on this head before merge.

## Assurance Context

AP-001 (spec/assurance/AP-001-analyze-release.md) is the repository's release AssuranceProfile and applies to the changed report validator. AD-001, CAC-001 and MP-001 exist under spec/assurance/ and were not re-measured. Unavailable: gate results (none run by brief), measurement records, independence and producer-reliance records. Active exceptions: none found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The disjointness check is quadratic (`selected.iter().any(/n/ excluded.contains(n))`) over lists the schema leaves unbounded (no maxItems), and it runs before the assertion-equality check. A crafted report under the 64 MiB MAX_REPORT_BYTES bound with millions of disjoint ascending ids in each list passes the schema and then does ~10^12 comparisons in the standalone validator. Both lists are already proven strictly ascending, so a linear merge walk (or a maxItems bound tied to the 10,000-node package limit) removes the unbounded work. | src/report.rs:886-888 |
| FND-002 | medium | Assertion clause ids are parsed leniently: `parse::<u64>()` accepts `claim-03` and `claim-+3` as 3, while the selection ids are held to the canonical spelling (`number.to_string() != digits`). A report whose assertion clause ends `claim-03` and whose `selected` is `["claim-3"]` validates, contrary to FR-005 (`selected` equals the set of `claim-N` clause ids) and FR-009 (one spelling). Apply the same canonical check to the assertion side. | src/report.rs:895-901 |
| FND-003 | medium | The FR-005-AC-7 test omits refusal cases the AC names: a repeated id, an id that is not `claim-N` at all (wrong prefix or non-string), and `selected` out of order (only `excluded` is tested); a missing `selected` list is also untested. The case labelled "id not claim-N" actually tests a leading zero (`claim-01`). Add the missing cases and label them by what they test. | tests/analysis_semantics.rs:1405-1421 |

## Examined Scope

```yaml
scope:
- id: src/report.rs#validate_selection
  path: src/report.rs
  role: examined
  excerpt: 'fn validate_selection(object: &serde_json::Map<String, Value>) -> Result<(), String>'
- id: src/report.rs#validate_report_document
  path: src/report.rs
  role: examined
  excerpt: if object.contains_key("selection") { root_keys.push("selection"); } require_exact_keys(object, &root_keys, "report root")?; validate_selection(object)?;
- id: schemas/differential-report-v2.schema.json#selection
  path: schemas/differential-report-v2.schema.json
  role: examined
  excerpt: '"selection": {"type": "object", "additionalProperties": false, "required": ["excluded", "selected"], ...}'
- id: tests/analysis_semantics.rs#differential_v2_validates_the_selection_member
  path: tests/analysis_semantics.rs
  role: examined
  excerpt: '#[trace("TC-007", "FR-005-AC-7")] fn differential_v2_validates_the_selection_member()'
- id: FR-005-AC-7
  path: spec/functional/FR-005-evidence-cli.md
  role: examined
  excerpt: '`validate_report_document` accepts a report without `selection` and a report with a `selection` whose lists are strictly ascending `claim-N` ids, disjoint, with non-empty `selected` equal to the `claim-N` clause ids of `assertions[]`. It refuses a `selection` with an extra member, a missing list, an empty `selected`, an id in both lists, an id that is not `claim-N` or has a leading zero, a list out of ascending order or with a repeated id, or `selected` ids that differ from the assertions'' clauses.'
```
