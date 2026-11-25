---
id: FRR-001
title: Proposed high process-risk supplier ratings for filament-analyze
type: FeatureRiskRating
scope: supplier
rating: high_process_risk
rated_by: Peter Krenesky
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

## Feature assessments

| Requirement | Failure mode | Detection |
|---|---|---|
| FR-001 | The analysis request could be interpreted under the wrong truth conditions or identity, producing a conclusion for a different contract question. | Exercise the truth table, scope/binding boundaries, identity changes, and invalid-request fixtures; compare requests and outcomes with the stated analysis algebra. |
| FR-002 | A construct could be approximated, or a symbol/assertion could lose its source identity, so a solver result is attached to the wrong semantics. | Validate canonical query fixtures, exact capability/refusal cases, and complete injective source maps against their golden results. |
| FR-003 | A timeout, malformed response, or process-cleanup failure could be treated as conclusive or leave a solver process running. | Exercise timeout, cancellation, malformed output, nonzero exit, signal, truncation, and configured-path cases; inspect outcome and process-group cleanup records. |
| FR-004 | An invalid, incomplete, or incorrectly replayed solver model could be published as verified or mapped to the wrong source clause. | Run the independent truth table, incomplete-model, invalid-model, and source-mapping fixtures; confirm only complete replayed models are marked verified. |
| FR-005 | A report could omit or alter evidence, hide solver disagreement, or publish partial output as a completed record. | Validate report schemas and envelope identities; run disagreement and publication-failure fixtures and verify the destination and retained evidence. |
| FR-008 | A synthesis candidate could be reported as proof, or bounded-search exhaustion could be mistaken for no candidate. | Compare canonical candidate order and identity for equivalent inputs; exercise unvalidated candidates, validation failures, exhaustive searches, and incomplete searches. |
| FR-009 | A package could be misread, an analysis claim omitted, or a partial/incorrect report published with a successful result. | Run admitted, refused, incomplete, and boundary package fixtures through the command; compare claim selection, solver requests, exit class, and atomic output. |
| FR-010 | Integer expressions could be lowered under the wrong logic or an unsupported expression approximated, changing a solver decision. | Check integer query goldens, both solver outcomes, arithmetic boundaries, logic declarations, and unsupported-expression refusals. |
| FR-011 | An integer witness could be marked verified without satisfying every assertion or its declared bounds. | Replay complete, failing, missing, malformed, and out-of-range witness models; compare the first failing assertion and the resulting verification status. |
| FR-012 | Integer expressions from a contract package could map to different variables or semantics than their source clauses. | Translate package fixtures with supported, boundary, and refused integer terms; compare source spans, lowered queries, and diagnostics. |
| FR-020 | Datatype declarations or range predicates could be incorrect, letting invalid values satisfy a query or making a valid query appear sound. | Compare enum, record, and option query fixtures and range constraints with expected declarations; exercise invalid and boundary model replay. |
| FR-021 | A datatype witness could omit a value, violate a range, or fail to satisfy the original assertion but still be marked verified. | Replay valid and adversarial enum, record, and option models and compare every assertion result with the expected typed values. |
| FR-022 | Datatype expressions in a package could be translated with altered constructor, field, or source identity. | Translate admitted datatype package fixtures and refused forms; compare canonical query bytes, mappings, and refusal locations with expected results. |
| FR-030 | An unbounded or unsupported quantifier domain could be accepted with an unsound approximation. | Exercise each supported and refused domain shape and verify refusal occurs before a solver query is produced. |
| FR-031 | Expansion of finite domains could omit, duplicate, reorder, or misbind instances and change the quantified result. | Compare expanded queries and source maps for enum and collection fixtures, including empty, singleton, repeated-variable, and expansion-limit cases. |
| FR-032 | A bounded integer quantifier could use an incorrect guard, expansion choice, or logic declaration and change the result. | Check literal and declared bounds around the expansion budget, query guards, logic, and refusal cases against the criterion fixtures. |
| FR-033 | A returned model could pass replay without satisfying every quantified instance, and be presented as verified evidence. | Exercise universal and existential replay across full bounds, first-failure cases, malformed bounds, and overflow; compare verified status and reported instance. |
| FR-034 | A package quantifier could be lowered differently from the declared operation or accepted outside its supported domain. | Translate package fixtures for every admitted domain and refusal boundary; compare bound-variable mapping, query, and diagnostics with expected results. |
| FR-041 | The recorded reachability review or disposition could omit an unsound case or select a path inconsistent with the evidence. | Inspect the adversarial-case set, independent enumeration, finding classification, and single recorded disposition against the acceptance criteria. |
| FR-042 | The selected reachability path could be applied incorrectly, or a failed external invocation could be turned into an enclosure. | Exercise both recorded disposition paths, malformed requests, tool failures, output limits, schema failures, and zero-tool-call cases; compare typed outcomes and invocation counts. |

## Rationale

These are proposed supplier-side ratings for IU-001. The `rated_by` field names
Peter Krenesky as the person assigned to review and accept the proposals. This
record is unsigned and unaccepted pending that decision through the
signed-commit workflow.

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
