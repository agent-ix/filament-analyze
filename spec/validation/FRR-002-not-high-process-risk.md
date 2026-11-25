---
id: FRR-002
title: Proposed not-high process-risk supplier rating for filament-analyze vulnerability reporting
type: FeatureRiskRating
scope: supplier
rating: not_high_process_risk
rated_by: Peter Krenesky
relationships:
  - target: ix://agent-ix/filament-analyze/IU-001
    type: uses_intended_use
  - target: ix://agent-ix/filament-analyze/FR-043
    type: rates_feature
---
# Proposed not-high process-risk supplier rating for vulnerability reporting

## Feature assessments

| Requirement | Failure mode | Detection |
|---|---|---|
| FR-043 | A reporting route could be missing or direct a reporter to a public forum, delaying a vulnerability report or exposing details. | Inspect the repository `SECURITY.md` route and verify the private reporting link works without asking a reporter to disclose details publicly. |

## Rationale

This is a proposed supplier-side rating for IU-001. The `rated_by` field names
Peter Krenesky as the person assigned to review and accept the proposal. This
record is unsigned and unaccepted pending that decision through the
signed-commit workflow.

The reporting policy provides a route for a security concern but does not
compute, classify, or validate an analysis result or contract report. Its
failure can obstruct or expose a report, but it does not itself change the
solver evidence produced for the intended use. The proposed rating is a human
decision and is not a computed score.

## Assurance activities

| Activity | Acceptance criterion |
|---|---|
| Inspect the published private reporting route and confirm it avoids public disclosure instructions. | FR-043-AC-1 through FR-043-AC-2 |
