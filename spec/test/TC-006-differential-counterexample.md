---
id: TC-006
title: "Verify engine parity and counterexamples"
type: TC
relationships:
  - target: ix://agent-ix/filament-analyze/FR-003
    type: verifies
  - target: ix://agent-ix/filament-analyze/FR-004
    type: verifies
  - target: ix://agent-ix/filament-analyze/FR-005
    type: verifies
  - target: ix://agent-ix/filament-analyze/FR-009
    type: verifies
---
# TC-006: Verify engine parity and counterexamples

## Description

Verify engines agree on supported cases and every published counterexample replays.

## Test Procedure

For the issue #4 slice, decode bounded Boolean models and independently re-evaluate every asserted
expression using the query's sealed replay map. Reject missing, duplicate, unknown, non-Boolean, and
predicate-refuting assignments as verified evidence. In Task-007, run Z3 and cvc5
over the seeded supported corpus, inject a controlled disagreement fixture, and verify every filed
semantic defect has a stable executable regression fixture and retained disposition.

The issue #5 corpus includes satisfied/refuted examples for all five kinds, unsupported lowering,
timeout, missing-engine, and controlled sat/unsat disagreement fixtures. Both engine records are
retained before disposition. Real-engine execution is a measured local lane. An absent real tool is a
stated skip and is not evidence; an engine that a run observes as unavailable is a result (the
RP-MISSING-ENGINE case, exit `3`).

The retained real-engine corpus has ten case identities. Each Boolean formula has an independently
enumerated truth result, the expected solver answer, and the expected analyzer status:

| Case | Analysis kind | Solver answer | Analysis status |
|---|---|---|---|
| `consistency-sat` | consistency of `x` | sat | satisfied |
| `consistency-unsat` | consistency of `x AND NOT x` | unsat | refuted |
| `contradiction-satisfied` | left `true`, right `false` | unsat | satisfied |
| `contradiction-refuted` | left `true`, right `true` | sat | refuted |
| `implication-satisfied` | antecedent `true`, consequent `true` | unsat | satisfied |
| `implication-refuted` | antecedent `true`, consequent `false` | sat | refuted |
| `redundancy-satisfied` | peer `true`, candidate `true` | unsat | satisfied |
| `redundancy-refuted` | peer `true`, candidate `false` | sat | refuted |
| `dead-antecedent-satisfied` | antecedent `x AND NOT x` | unsat | satisfied |
| `dead-antecedent-refuted` | antecedent `x` | sat | refuted |

Both pinned engines must return the listed solver answer for every case. The test independently checks
the ten formula truth cells, requires agreement and the listed status, validates each report, and can
retain the observed engine records and report bytes as `filament-analyze.tc006-corpus-run/v1` JSON.
The pinned executable digests are checked before any case runs.

The non-conclusive corpus fixtures are also executable: unsupported lowering is checked by
`quantification_and_data_types_are_explicitly_unsupported`; a bounded wall timeout by
`timeout_and_cancellation_cleanup_are_measured`; missing-engine and controlled sat/unsat disagreement
by `differential_disposition_requires_two_verified_conclusive_results`. The latter checks both engine
records remain present and disagreement has no agreed status.

### `run-package` corpus

Six named cases, each run through the library boundary (`filament_analyze::run_package`
with the parsed `RunPackageArguments`, mapping an `Err` to `RunnerError::class()`) and through the
existing CLI boundary (the built `filament-analyze run-package` binary spawned as a child process), over
the same package bytes and the same stub `producer-execute` results. No package document is committed
(FR-009-AC-27): each package is built at test time by the builder named here in
`tests/package_runner.rs`, and each stub result is the `producer-execution-result/v1` line the case
states. A case id names the case; it is not a digest of its inputs.

| Case | Package, built at test time | Engine results | Exit | Report |
|---|---|---|---|---|
| RP-SAT | `consistent()`: two asserted Boolean parameters | both `completed`, `sat` with the replaying model | `0` | published, valid, agreed status `satisfied` |
| RP-UNSAT | `contradictory()`: a parameter and its negation | both `completed`, `unsat` | `1` | published, valid, agreed status `refuted` |
| RP-UNSUPPORTED | `untranslatable()`: a Boolean claim over an integer `div` the translation cannot lower | none: no `producer-execute` request is made | `2`, stderr names `semantic_graph.nodes[N]` | none |
| RP-TIMEOUT | `consistent()` | one engine `timed_out`, the other `completed` `sat` | `3` | published, valid, engine outcome `timed-out`, disposition `inconclusive`, both engine records retained |
| RP-MISSING-ENGINE | `consistent()` | one engine `unavailable`, the other `completed` `sat` | `3` | published, valid, engine outcome `missing-executable`, disposition `unavailable`, both engine records retained |
| RP-DISAGREE | `consistent()` | Z3 `completed` `sat` with the replaying model, cvc5 `completed` `unsat`; then the same with the engines swapped | `3` | published, valid, outcomes `sat` and `unsat`, disposition `disagreement`, both engine records retained and no agreed status |

For each case, the library run and the CLI run receive the same complete argument vector: the same
`--package`, `--z3`, `--cvc5`, `--producer-execute` and `--output` paths, because the report embeds
these values. The `--output` path is reused only after the prior run's output has been removed. With
that vector, the two runs give the same exit status, and when a report is published their reports are
byte-identical once every `elapsedMs` is `0`; when no report is published neither run writes one. For
RP-UNSUPPORTED the library boundary returns a `RunnerError` whose `message()` names
`semantic_graph.nodes[N]`, and the CLI writes that node name on stderr. RP-DISAGREE is a controlled
disagreement: real Z3 and cvc5 do not disagree on a built package, so the two answers are stub results.
A disagreement is never adjudicated into agreement and its exit status never becomes `0` or `1`.

RP-MISSING-ENGINE, by its stub `unavailable` result, is the corpus evidence for the unavailable
disposition and exit `3`. The absence of a real tool is a different matter: the real-engine tests
(FR-009-AC-14, FR-009-AC-39, NFR-003-AC-1) skip with a stated reason when the `engineering-assurance`
executable, Z3 or cvc5 is absent, and such a skip is not positive evidence for any corpus case.

Real engines are a measured local lane under FR-009-AC-39: RP-SAT and RP-UNSAT are repeated with the real
`engineering-assurance` executable and real Z3 and cvc5, and RP-TIMEOUT with a stalled stub solver
through the real executable (NFR-003-AC-1). The run identity (the application version and the solver
identities) is the one the report already carries; the corpus adds no pin, digest or manifest of its
inputs or outputs.

## Expected Results

Every `run-package` corpus case gives its stated exit status and report outcome through both
boundaries. Every published decoded counterexample satisfies the required truth condition. Later engines
agree on supported cases; a disagreement retains both raw and normalized results and cannot be
classified as conclusive before human-reviewed adjudication. Every filed semantic defect remains
reproducible by its regression fixture.
