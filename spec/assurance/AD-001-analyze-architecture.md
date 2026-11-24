---
id: AD-001
title: "Contract analysis architecture"
type: ArchitectureDescription
status: accepted
owner: analyze-maintainers
system: filament-analyze v0.1
relationships:
  - target: ix://agent-ix/filament-analyze/AP-001
    type: realizes
---
# Contract analysis architecture

## System Boundary

The Rust library consumes typed expressions and clause identities from `filament-contract-ir`. It owns
closed-request validation, Boolean-v1 lowering, bounded solver execution, conclusion classification,
source mapping, and differential report rendering. The solver engines and operating-system process
behavior remain external. The CLI currently validates and publishes an existing library report;
it also runs an analysis from a supplied FSL contract package through `run-package`. Human reviewers retain release and consuming-project
qualification authority. Architecture acceptance is scoped to this implemented structure and the
explicit limitations below; it does not qualify an engine or authorize source release.
AD-001 and interface_001 are accepted by the independent IR-1294 architecture review for the
implemented structure and explicit limitations described here.

## Views

The implemented Boolean-v1 derivation path is typed contract IR → closed `AnalysisRequest` →
canonical `QueryBundle` with assertion and variable maps → bounded Z3/cvc5 child process →
`SolverRecord` → `classify_analysis` status and model replay record → differential report bytes.
`classify_analysis` sets `Satisfied` or `Refuted` from the solver outcome before replaying a SAT
model. On replay error it retains that logical status and records `Incomplete` with a diagnostic,
without representing a model or source value as verified. This is the FR-004 distinction between
logical classification and explanation evidence. `AnalysisConclusion::is_conclusive` reads the
logical status; `compare_solver_records` separately requires verified SAT explanations before
same-status differential `Agreement`, as FR-005 requires.

| Boundary | Owner and current entry point | Responsibility |
|---|---|---|
| Typed contract input | `filament-contract-ir` dependency; `src/smt.rs` `StatementInput`, `BindingGroup` | Consume authoritative expressions, clause identity, and explicit cross-requirement bindings; refuse unsupported shapes. |
| Request and exact lowering | `src/analysis.rs` `AnalysisRequest`, `lower_analysis_request`; `src/smt.rs` `lower_boolean_statements` | Validate closed analysis kinds and produce the Boolean-v1 SMT-LIB2 query plus source maps. |
| Bounded engine execution | `src/solver.rs` `execute_solver` | Run the external engine under resource bounds and retain a normalized record; never certify its answer. |
| Conclusion classification | `src/analysis.rs` `classify_analysis` | Apply analysis-kind-specific sat/unsat rules; preserve the logical status while recording a failed SAT model replay as incomplete explanation evidence. |
| Differential evidence and publication | `src/report.rs` `execute_differential`, `render_differential_report`, `validate_report_document`, `publish_report_new`; `src/main.rs` `run-package` and `publish-report` | Compare and retain both engine records, render and validate report bytes, and either analyze a package or atomically publish a pre-existing report through the CLI. |
| Separate method providers | `src/hybrid_tool.rs` `reach_hybrid` (delegated PHAVer tool path), `src/hybrid_synthesis.rs` `synthesize`, `validate_candidate`; `crates/method-contract` `AnalyzeOutcome` | Return bounded hybrid reachability and synthesis outcomes through their own method-result seam; these do not become SMT proof classifications. |

The proof identity view binds selected clause content, explicit bindings, query bytes and maps,
engine execution record, classified conclusion, and canonical report content. These content digests
bind a proof to what it analyzed. Cargo.toml declares `filament-contract-ir` and Cargo.lock records
its resolved source; they are the dependency identity authorities. Report producer identity is the
application name and version. The differential report is a closed v2 contract without a copied
dependency revision. It retains canonical proof-to-content digests and producer application
identity. The renderer and validators reject v1 without a fallback.

The failure view keeps invalid, unsupported, unknown, timed-out, cancelled, unavailable, malformed,
truncated, and internal solver states distinct from logical `Satisfied` or `Refuted`. A failed SAT
model replay leaves the logical status intact and marks explanation evidence incomplete; it cannot
publish an unverified source value or yield same-status differential `Agreement`. Discrepant
engine statuses retain both records with a non-conclusive differential disposition. Adapters cannot
construct conclusions directly.

The evidence view is incomplete: `src/report.rs` explicitly marks its PGM-01 envelope
`unavailable` pending `filament-contract-ir#20`. FR-005-AC-2 and TC-007 cannot be claimed complete
from that placeholder. Source publication and release qualification remain a human PGM-02 Wave 4
decision.

## Decisions

- One engine-neutral Boolean query bundle prevents adapters from reinterpreting clauses.
- Complete semantic identity, not display name, controls variable sharing and source mapping.
- Canonical SMT-LIB2 and explicit assertion names make derivation reviewable.
- External engines run as bounded child processes with argv, never a shell command.
- Retained raw records are inputs to normalization and differential comparison.
- Exact encoding is required for conclusive v0.1 results; approximation is fail-closed.
- Hybrid reachability and synthesis have separate bounded providers and method outcomes; their
  results do not imply a Boolean SMT proof.

## Interfaces and Trust

Solver answers are evidence, not self-authenticating truth: finite-model and cross-engine campaigns
reduce risk but do not qualify engines. The OS process API, filesystem atomicity, content-digest
implementation, and schema validator remain named external dependencies. The interface's
prepare/lower/execute/conclude/compare sequence describes callable library steps; the current CLI
exposes `run-package` and `publish-report`.

## Risks

Encoding drift, solver defects, platform process differences, unbounded output, identity collisions,
model-decoding errors, and unavailable cross-platform execution can weaken evidence. The measurement
plan retains these outcomes and no automated control may waive them into success. The upstream PGM-01 envelope, engine qualification, and source-release authority are excluded
from architecture acceptance. Their absence remains visible and cannot be waived by accepting
this description.

## Acceptance Scope and Decision Disposition

IR-1294 reviews this description and interface_001 against the current implementation as one
architecture decision. ADR-0010 accepts the algebra and identity rules; its acceptance does not
accept these two artifacts. The independent IR-1294 architecture review accepts these two
artifacts for this scope. Acceptance records conformance of the described boundaries, not completion of
all requirements, measurement campaigns, platform lanes, or release qualification.

| Concern | Decision and current implementation | Disposition |
|---|---|---|
| Semantic authority and aliasing | Retain typed `filament-contract-ir` expressions and explicit structurally validated binding groups; never infer cross-requirement aliases from names. `src/smt.rs` owns lowering and source maps. | Resolved by ADR-0010; accepted scope is the implemented Boolean-v1 subset. |
| Request validation timing | `AnalysisRequest` constructors reject missing groups and duplicate statement identities. `lower_analysis_request` delegates remaining semantic/binding/capability checks to lowering before execution. | Resolved: a separate fully validated `AnalysisModel` prepare result is not part of the current API contract. |
| Status and explanation | `classify_analysis` preserves the logical SAT-derived status on replay failure; no verified model is returned. Differential agreement requires verified SAT evidence. | Resolved: logical classification and explanation evidence are distinct, as FR-004/FR-005 require. |
| Failure detail | `SolverRecord` retains cancellation, absence and protocol/process outcomes; `AnalysisStatus` maps cancellation and timeout to `Timeout`, unsupported platform to `Unsupported`, and other failures to `ToolError`. | Resolved: consumers inspect the retained solver record for detail; interface_001 states both levels. |
| CLI scope | `run-package` orchestrates bounded package analysis and `publish-report` validates and publishes existing v2 report bytes. | Resolved: the documented CLI scope names both commands and their distinct roles. |
| Shared assurance envelope | `src/report.rs` requires the explicit unavailable envelope placeholder. | Dependency work remains `agent-ix/filament-contract-ir#20`; this architecture accepts the visible limitation, not envelope compliance. |
| Separate methods | Hybrid reachability and synthesis return method outcomes through `crates/method-contract`. | Resolved: these providers do not establish Boolean SMT conclusions or engine qualification. |
| Qualification and release | AP-001 retains the human source-release boundary; MP-001 supplies evidence only. | Resolved boundary: no architecture acceptance, automated status or zero-count observation grants qualification or release authority. |

The conformance rule for this scope is that each callable library stage and the publication CLI
retain the ownership, source identities, failure detail and evidence distinction above. Changes
that compose a new analysis CLI, widen the exact fragment, alter variable sharing, change report
compatibility, or promote another method's outcomes into SMT conclusions require a separately
reviewed contract change. No such behavior is implied by this acceptance.

## Soundness Scope and Assumptions

For the SMT analysis path, `sat` is evidence that the emitted query has a satisfying model; `unsat`
is evidence that the emitted query has no model. Neither answer alone proves a source-level contract
claim. A source-level conclusion also depends on the applicable FR's translation being exact for the
selected constructs, the upstream typed contract IR preserving its declared meaning, and the solver
implementing the declared SMT-LIB logic. Z3/cvc5 agreement is not a proof that both engines or the
translation are correct. A SAT explanation is published as verified only after the model is replayed
against the lowered assertions and source map under FR-004, FR-011, FR-021 and FR-033.

| Theory or method | Meaning of a result | Translation and external assumptions |
|---|---|---|
| Linear integer arithmetic (FR-010) | `sat` gives an integer model of the exact linear query; `unsat` means no integer assignment satisfies that query. | Arithmetic is over mathematical integers. Lowering emits only the listed exact operators; unsupported terms refuse before execution. Definedness and reject-overflow obligations are established by the upstream IR validator. The configured solver implements LIA as specified. |
| Enumerated datatypes (FR-020–FR-022) | `sat` gives an assignment in the emitted enum, record or option algebra; `unsat` means no assignment in those declared sorts satisfies the query. | Datatype constructors, selectors, equality and presence operations must follow the complete selected declaration environment. Unsupported types and unproved option unwraps refuse. The solver implements the selected algebraic datatype logic. |
| Bounded quantifiers (FR-030–FR-034) | `sat` gives a model satisfying the finite-domain or guarded quantified query; `unsat` means no such model under the exact emitted domain and guard. | Expanded domains must enumerate every member, and bounded solver quantifiers must preserve both endpoints and the formula body. Replay checks a returned quantified model over the original finite range. No unbounded, sampled, truncated or relaxed domain is accepted. These claims assume the solver correctly implements the emitted logic; no decidability claim is made for quantified formulas with datatypes. |
| Delegated hybrid reachability (FR-041–FR-042) | This method does not return SMT `sat` or `unsat`. An enclosure is the configured PHAVer tool's reported interval enclosure for every requested step; incomplete or refused execution returns no enclosure. | The request and result schema are checked locally, but enclosure soundness still assumes the configured PHAVer executable computes sound enclosures for the submitted model. The finite-execution oracle and adversarial cases in FR-041/TC-041 test selected cases; they do not establish universal tool correctness. The tool path, producer-execute path and runtime library path come from explicit configuration, not a workstation-specific home directory. |

These are conditional soundness arguments for the implemented translation boundaries, not a formal
proof of the implementation or an independent qualification of Z3, cvc5, PHAVer, the IR validator,
or the process adapter. Unsupported, unavailable, unknown, timed-out and incomplete outcomes remain
non-conclusive under the relevant requirements.
