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
  - target: ix://agent-ix/filament-analyze/FR-001
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-002
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-003
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-004
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-005
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-008
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-009
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-010
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-011
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-012
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-020
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-021
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-022
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-030
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-031
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-032
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-033
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-034
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-041
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-042
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

## Assurance activities

| Activity | Acceptance criterion |
|---|---|
| Check the independent analysis truth table, bindings, identities, and invalid-request cases. | FR-001-AC-1 through FR-001-AC-5 |
| Compare canonical query fixtures, capability refusals, and complete source mappings. | FR-002-AC-1 through FR-002-AC-5 |
| Exercise adapter bounds, timeout/cancellation cleanup, malformed responses, and both configured solvers. | FR-003-AC-1 through FR-003-AC-6 |
| Check classification, source mapping, incomplete explanations, and model replay with adversarial results. | FR-004-AC-1 through FR-004-AC-5 |
| Validate report schema/envelope, differential outcomes, atomic publication, and retained regression cases. | FR-005-AC-1 through FR-005-AC-8 |
| Check canonical candidate identity/order and keep candidate discovery separate from independent validation. | FR-008-AC-1 through FR-008-AC-3 |
| Run package fixtures across input, selection, solver, output, and failure boundaries. | FR-009-AC-1 through FR-009-AC-39 |
| Verify integer query logic, exact lowering, solver outcomes, and refusal cases. | FR-010-AC-1 through FR-010-AC-11 |
| Replay integer witnesses against all assertions and declared bounds. | FR-011-AC-1 through FR-011-AC-11 |
| Translate supported integer package terms and verify source identity and refusal behavior. | FR-012-AC-1 through FR-012-AC-9 |
| Verify datatype declarations, range constraints, and model replay. | FR-020-AC-1 through FR-020-AC-10 |
| Replay datatype witnesses and reject incomplete or invalid models. | FR-021-AC-1 through FR-021-AC-7 |
| Translate datatype package terms and verify exact mapping and refusals. | FR-022-AC-1 through FR-022-AC-5 |
| Verify refusal of unsupported and unbounded quantifier domains before solver execution. | FR-030-AC-1 through FR-030-AC-6 |
| Compare finite-domain expansion with expected instances, order, and bounds. | FR-031-AC-1 through FR-031-AC-6 |
| Check bounded integer quantifier selection, guards, solver logic, and failure boundaries. | FR-032-AC-1 through FR-032-AC-9 |
| Replay quantified models across their full bound and verify typed failure outcomes. | FR-033-AC-1 through FR-033-AC-9 |
| Translate admitted package quantifiers and verify refused forms and bounded expansion. | FR-034-AC-1 through FR-034-AC-8 |
| Inspect the independent reachability enumeration, review findings, and exactly one selected disposition. | FR-041-AC-1 through FR-041-AC-7 |
| Exercise the recorded path, tool boundary, error results, and no-fallback behavior. | FR-042-AC-1 through FR-042-AC-10 |
