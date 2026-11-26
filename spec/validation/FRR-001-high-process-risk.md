---
id: FRR-001
title: Proposed high process-risk supplier ratings for filament-analyze
type: FeatureRiskRating
scope: supplier
rating: high_process_risk
status: proposed
relationships:
  - target: ix://agent-ix/filament-analyze/IU-001
    type: uses_intended_use
  - target: ix://agent-ix/filament-analyze/FR-004
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-005
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-009
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-010
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-011
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-021
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-033
    type: rates_feature
---
# Proposed high process-risk supplier ratings for filament-analyze

## Rationale

These are proposed supplier-side ratings for IU-001. They are unsigned and
unaccepted pending a human decision through the signed-commit workflow.

A failure in these features could make an analysis, witness, solver agreement,
source mapping, package run, or reachability result appear conclusive or
accurate when the supplied evidence does not support it. They can therefore
change or misrepresent records relied on when reviewing requirement contracts.
The proposed rating is a human decision for this intended use, not a computed
score or product-safety classification.

These reasons describe the false assurance each feature could emit under IU-001; the rating is a human proposal, not a computed score.

## Feature-specific reasons for retained high ratings

| FR | False-claim reason |
| --- | --- |
| FR-004 | Result classification can claim a proof or counterexample state unsupported by solver or replay evidence. |
| FR-005 | Evidence report and differential output can state a verified result unsupported by the recorded analysis. |
| FR-009 | Package analysis can report a requirement result contrary to the solver evidence. |
| FR-010 | Integer arithmetic decision can report a proof or refutation result that is invalid. |
| FR-011 | Witness replay can claim a model satisfies all assertions when it does not. |
| FR-021 | Enum, record, or options witness replay can claim an invalid model is valid. |
| FR-033 | Bounded-quantifier replay can claim an assertion holds when the witness violates it. |

## Assurance activities

| Activity | Acceptance criterion |
|---|---|
| Check classification, source mapping, incomplete explanations, and model replay with adversarial results. | FR-004-AC-1 through FR-004-AC-5 |
| Validate report schema/envelope, differential outcomes, atomic publication, and retained regression cases. | FR-005-AC-1 through FR-005-AC-8 |
| Run package fixtures across input, selection, solver, output, and failure boundaries. | FR-009-AC-1 through FR-009-AC-39 |
| Verify integer query logic, exact lowering, solver outcomes, and refusal cases. | FR-010-AC-1 through FR-010-AC-11 |
| Replay integer witnesses against all assertions and declared bounds. | FR-011-AC-1 through FR-011-AC-11 |
| Replay datatype witnesses and reject incomplete or invalid models. | FR-021-AC-1 through FR-021-AC-7 |
| Replay quantified models across their full bound and verify typed failure outcomes. | FR-033-AC-1 through FR-033-AC-9 |
