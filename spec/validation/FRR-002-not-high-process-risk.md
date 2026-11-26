---
id: FRR-002
title: Proposed not-high process-risk supplier ratings for filament-analyze
type: FeatureRiskRating
scope: supplier
rating: not_high_process_risk
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
  - target: ix://agent-ix/filament-analyze/FR-008
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-012
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-020
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-022
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-030
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-031
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-032
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-034
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-041
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-042
    type: rates_feature
  - target: ix://agent-ix/filament-analyze/FR-043
    type: rates_feature
---
# Proposed not-high process-risk supplier ratings for filament-analyze

## Rationale

These are proposed supplier-side ratings for IU-001. They are unsigned and
unaccepted pending a human decision through the signed-commit workflow.

The reporting policy provides a route for a security concern but does not
compute, classify, or validate an analysis result or contract report. Its
failure can obstruct or expose a report, but it does not itself change the
solver evidence produced for the intended use.

The other features in this record can affect analysis inputs, translation, or
usability, but their specified behavior does not itself issue a false claim
that a result is verified, traced, discharged, or approved. This proposed
rating is a human decision for IU-001, not a computed score.

## Assurance activities

| Activity | Acceptance criterion |
|---|---|
| Check the independent analysis truth table, bindings, identities, and invalid-request cases. | FR-001-AC-1 through FR-001-AC-5 |
| Compare canonical query fixtures, capability refusals, and complete source mappings. | FR-002-AC-1 through FR-002-AC-5 |
| Exercise adapter bounds, timeout/cancellation cleanup, malformed responses, and both configured solvers. | FR-003-AC-1 through FR-003-AC-6 |
| Check canonical candidate identity/order and keep candidate discovery separate from independent validation. | FR-008-AC-1 through FR-008-AC-3 |
| Translate supported integer package terms and verify source identity and refusal behavior. | FR-012-AC-1 through FR-012-AC-9 |
| Verify datatype declarations, range constraints, and model replay. | FR-020-AC-1 through FR-020-AC-10 |
| Translate datatype package terms and verify exact mapping and refusals. | FR-022-AC-1 through FR-022-AC-5 |
| Verify refusal of unsupported and unbounded quantifier domains before solver execution. | FR-030-AC-1 through FR-030-AC-6 |
| Compare finite-domain expansion with expected instances, order, and bounds. | FR-031-AC-1 through FR-031-AC-6 |
| Check bounded integer quantifier selection, guards, solver logic, and failure boundaries. | FR-032-AC-1 through FR-032-AC-9 |
| Translate admitted package quantifiers and verify refused forms and bounded expansion. | FR-034-AC-1 through FR-034-AC-8 |
| Inspect the independent reachability enumeration, review findings, and exactly one selected disposition. | FR-041-AC-1 through FR-041-AC-7 |
| Exercise the recorded path, tool boundary, error results, and no-fallback behavior. | FR-042-AC-1 through FR-042-AC-10 |
| Inspect the published private reporting route and confirm it avoids public disclosure instructions. | FR-043-AC-1 through FR-043-AC-2 |
