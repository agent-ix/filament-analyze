---
id: FRR-002
title: Proposed not-high process-risk supplier rating for filament-analyze vulnerability reporting
type: FeatureRiskRating
scope: supplier
rating: not_high_process_risk
status: proposed
relationships:
  - target: ix://agent-ix/filament-analyze/IU-001
    type: uses_intended_use
  - target: ix://agent-ix/filament-analyze/FR-043
    type: rates_feature
---
# Proposed not-high process-risk supplier rating for vulnerability reporting

## Rationale

This is a proposed supplier-side rating for IU-001. It is unsigned and
unaccepted pending a human decision through the signed-commit workflow.

The reporting policy provides a route for a security concern but does not
compute, classify, or validate an analysis result or contract report. Its
failure can obstruct or expose a report, but it does not itself change the
solver evidence produced for the intended use. The proposed rating is a human
decision and is not a computed score.

## Assurance activities

| Activity | Acceptance criterion |
|---|---|
| Inspect the published private reporting route and confirm it avoids public disclosure instructions. | FR-043-AC-1 through FR-043-AC-2 |
