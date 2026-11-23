---
id: FR-009
title: "Run analysis over a FSL contract package and write a report"
type: FR
relationships:
  - target: ix://agent-ix/filament-analyze/FR-005
    type: depends_on
  - target: ix://agent-ix/filament-analyze/StR-001
    type: implements
  - target: ix://agent-ix/engineering-assurance/FR-029
    type: depends_on
---
# FR-009: Run analysis over a FSL contract package and write a report

## Description

The `filament-analyze` CLI shall provide a `run-package` subcommand that runs one differential consistency
analysis over one FSL package in its lowered form and publishes one `filament.differential-report/v2`
report, so a caller obtains a report and a pass/fail exit status from a single invocation. The
subcommand runs the Z3 and cvc5 solvers only through `producer-execute`.

## Input: the FSL package as a contract package

FSL source (in fences of spec documents or in standalone files) is parsed and checked by
`filament-spec-language` into a checked package. `filament-contract-ir` lowers a checked package
(`CheckedPackageV2::lower` in `filament-contract-model`) into the target-neutral contract package
`filament.contract-package/v1`.

The runner input `--package FILE` is that `filament.contract-package/v1` document. The runner reads it
with `ContractPackageWireV1::read(bytes, limits, dependencies)` of `filament-contract-model`, with
`limits = CheckedPackageReadLimits::bounded()` (1,048,576 canonical bytes, 10,000 nodes) and
`dependencies` empty. The reader returns `Admitted`, `Refused`, `Incomplete` or `CatalogUnavailable`:
`Admitted` proceeds, `Refused` and `Incomplete` exit `2` naming the refusal or the exhausted limit, and
`CatalogUnavailable` exits `4` because it is a reader and catalog skew, not a verdict on the input. A
package that names dependencies is refused by the empty dependency map and exits `2`.

A contract package names no analysis request: it holds a semantic graph, a source map and per-request
lowering accounting. This requirement therefore fixes the request, and no FSL-side declaring field is
needed. The analysable clauses are the nodes of `semantic_graph.nodes` of kind `claim` with form
`analysis_claim`. If a package holds none, or a selected `analysis_claim` has a body that is not a
Boolean-typed term, the runner exits `2`; without `--select` every `analysis_claim` is selected. The
runner builds ONE consistency request whose statements are the selected analysable clauses (every
analysable clause without `--select`, the clauses the Claim selection section selects with it), with no
assumptions and no binding groups. One invocation yields exactly one request, which is what the one-request report of
[FR-005](./FR-005-evidence-cli.md) holds. Other analysis kinds, assumption selection and binding groups
are out of scope. This definition closes the selection question for a run without `--select`; the
selection that `--select` adds is defined in the next section, and the translation of a term into an
analysis statement after it.

## Claim selection (`--select`)

This section is the canonical definition of the claim-selection request. Producers of a selection
conform to it. Callers that must analyse only approved formal rules pass `--select FILE`. The file is a
`filament-analyze.claim-selection/v1` JSON document:

```json
{"protocol":"filament-analyze.claim-selection/v1","spans":[{"authority":"quoin","identity":"rules/a.qsl","start":120,"end":340}]}
```

A span names a byte interval under a source document in the vocabulary of the package's own source map
(`CheckedSourceRegion`): `authority` and `identity` are the two members of the region's source
reference, `start` is the inclusive byte offset and `end` the exclusive byte offset. `start` and `end` are
non-negative JSON integers within the offset type of `CheckedSourceRegion`, with `start` below `end`. The package carries byte regions and no line numbers, so the selection speaks bytes and the runner
reads no source text. The document is closed: an unknown member, a missing member, a wrong `protocol`,
an empty `authority` or `identity`, a `start` or `end` that is negative, fractional, not a number or
outside the offset type, or a `start` not below `end` is a usage error. `spans` may be empty.
The file is read with a bound of 1,048,576 bytes.

### Resolving a node to a region

- The region of a node `N` is `source_map.nodes[N].occurrences[0].regions[0]`, the region of Source spans.
  A node with no occurrence has no region.
- A region lies inside a listed span exactly when its `authority` and `identity` equal the span's by
  exact string comparison, with no normalization of case, separators or path segments, and
  `span.start <= region.start` and `region.end <= span.end`. Two regions overlap when they share a
  source reference and `max(start) < min(end)`.

### Which claims are selected

- An `analysis_claim` node is selected exactly when its region lies inside a listed span. Every other
  `analysis_claim` node, including one with no region, is excluded.
- **Partial overlap.** If a listed span overlaps the region of a claim without containing it, the runner
  shall exit `2` naming the span and `claim-N` and write no report, whether or not another span contains
  that claim. A caller therefore never has half a claim judged.
- **Reached nodes.** The translation inlines the target of every `Reference` and declares every
  `value/parameter` that a selected claim reaches. If the region of such a node does not lie inside a
  listed span, or the node has no region, the runner shall exit `2` naming `claim-N` and
  `semantic_graph.nodes[M]` of the node reached, and write no report. A selected claim is therefore
  never judged through source the caller did not select. A separate span may cover such a node, for
  example a shared parameter declared elsewhere in the source, without selecting any claim between the
  two: only claims whose own region lies inside a listed span are selected.
- **Unmatched span.** A listed span is matched when it contains the region of an `analysis_claim` or the
  region of a node that a selected claim reaches. If a listed span is matched by neither, the runner
  shall exit `2` naming the span and write no report.
- **Empty selection.** A document whose `spans` is empty is well formed and selects nothing. The
  approval workflow that produces the selection does not run an analysis when no approved rule exists,
  so it does not invoke the runner for an empty list. If the runner is invoked with one, it shall exit
  `2` with the stderr cause `empty selection` and write no report, so a caller can tell zero approved
  rules from a malformed document by that cause.
- **Excluded claims.** An excluded claim is not examined beyond its region: the runner shall not check
  that its body is Boolean-typed, shall not translate or lower it, and shall not fail the run because its
  body is non-Boolean, cannot be lowered, or has no occurrence (a claim with no region is always
  excluded). It adds no assertion and no value
  declaration, and contributes nothing to the bounds of the Terms section. A package that holds no
  `analysis_claim` node at all still exits `2` as the Input section states.
- **Claim ids.** Selected and excluded claims are named by the `ClauseId` `claim-N` of the Identity
  section, so the ids of the report `selection` member and of the assertions use one spelling.
- **Without `--select`**, every analysable clause is selected and the report holds no `selection` member.

### Boundary with the producer of `analysis_claim` nodes

This requirement defines selection over the package data that exists: source references and byte regions.
It does not define what an `analysis_claim` node's region covers for a QSL declaration, nor what a source
reference's `authority` and `identity` spell for a QSL file. Those belong to the producer of
`analysis_claim` nodes (QSL-1900, cited by VER-98) and are gap 7. A caller derives the span of an
approved declaration from the same source identity and bytes the package was produced from.

### Producers and consumers of the selection and the report

This requirement owns the request schema: `filament-analyze.claim-selection/v1` with the span members
`authority`, `identity`, `start` and `end` as byte regions, and an empty `spans` accepted as well formed.
The producer of a selection is the quoin FR-190 claim-selection writer (repository `agent-ix/quoin`), which
shall write exactly this document; any different protocol name or span vocabulary is refused by the Claim
selection rules above, and aligning the producer is a change in the quoin repository.

The consumer of the report is the quoin FR-151 adapter (repository `agent-ix/quoin`), which projects a
`filament.differential-report/v2` report for the caller. For a selected run it shall accept the
`selection` member of the Outputs section and shall project its `selected` and `excluded` lists with the
other members it projects; a report without `selection` is projected as before. The `selection` member is
defined only by this requirement and FR-005; the adapter's acceptance of it is verified in the quoin
repository against a report this runner or the FR-005 validator accepts.

The `--select` runner behavior of this requirement is implemented by VER-100 and VER-102. The change that
introduced this section changes report validation and the schema only (FR-005-AC-7); the runner itself
does not yet accept `--select`.

The runner reads the selection as data and judges nothing about how it was built. This requirement stores
no approval state.

## Translation of an analysis_claim to an analysis statement

The analysis API takes `StatementInput::from_clause(package, requirement, clause, environment)` over a
`ContractPackage<TypedExpression>`, and `ContractPackageV1` holds a graph of `CheckedTerm` bodies. The
runner therefore builds, in memory, one `ContractPackage<TypedExpression>` that holds one requirement
and one clause per analysable clause, and takes each statement from it. Nothing of this package is
written to disk. In this section `N` is the zero-based index of a node in `semantic_graph.nodes`, and a
node is named on stderr as `semantic_graph.nodes[N]`.

### Identity and anchor

- The runner shall build the package with `PackageId::new` of `ContractProvenanceV1.source.name`,
  `SchemaVersion::V1_1`, and the source identity of document `contract-package`, revision `1`. If
  `PackageId::new` refuses the name, the runner shall exit `2` naming the package name.
- The runner shall build exactly one requirement, with id `analysis`, revision `1` and source span
  `0..0` (see Source spans). The `RequirementRef` of every statement is therefore
  (`source.name`, `analysis`, `1`). One shared requirement is required for the claims to share
  variables: the variable key of `src/smt.rs` contains the owner `RequirementRef` and the execution
  point, and a request with no binding groups equates two variables only when their keys are equal.
- The runner shall build one clause per analysable clause, in the order of `semantic_graph.nodes`, with
  `ClauseId` `claim-N` for node `N`, `ClauseKind::Assertion`, and the anchor
  `ExecutionPoint::Handler` named `analysis`. An `analysis_claim` node carries no anchor and
  `from_clause` refuses a clause without one, so the anchor is a fixed synthetic point. `Assertion`
  accepts every execution point, and `analysis` satisfies the identifier grammar that
  `Clause::new` applies to an anchor name.
- The runner shall build one `DeclarationEnvironment`, owned by the requirement above, with no type
  declarations, no function declarations and the value declarations of the next subsection, and shall
  type each clause body with `DeclarationEnvironment::check_expression` against `ValueType::Boolean`
  at the clause anchor with `clause_root` true.

### Source spans

- The span of a node `N` shall be the span built from the first `CheckedSourceRegion` of the first
  occurrence of `source_map.nodes[N]`, that is `source_map.nodes[N].occurrences[0].regions[0]`: both ends
  at line `1` and column `1`, with `byte_offset` the region `start` for the start and the region `end` for
  the end, in the source identity above. The source map holds byte regions only, without lines or
  columns, and a `SourceLocation` requires line and column of at least `1`. The region's own source
  reference is not carried over because `SourceDocumentId` requires an identifier-grammar spelling that a
  source reference need not have.
- The clause source shall be the span of its claim node. Every `Expression` of the clause body shall
  carry the span of the claim node. A value declaration shall carry the span of the node it
  declares. The requirement source shall be the span with both byte offsets `0`.
- The reader guarantees one `source_map.nodes` entry per graph node, in graph order, and a non-empty
  `regions` list for every occurrence, so a missing entry or an empty region list cannot reach the
  translation. If `source_map.nodes[N].occurrences` is empty for a claim without `--select`, or for a
  selected claim's reached node, the runner shall exit `2` naming the node; with `--select` a claim whose
  occurrences are empty has no region and is excluded.

### Terms

The runner shall translate the body of an analysis claim, a `CheckedTerm`, recursively. An
`Application` stands only at the root of a node body, so operations nest through `Reference` terms to
other nodes, and the translation follows those references. Let the type of a term be: for a
`Literal`, the kind of its `type` node; for a `Reference`, the kind of the `semantic_type` node of its
target; for an `Application`, the kind of its `result_type` node. The type is Boolean when that node is
`scalar_type/boolean`. A claim body whose type is not Boolean, whose type node cannot be resolved, or
that is an `Aggregate`, `Binding`, `DependencyReference` or any other form, is a non-Boolean claim and
exits `2` as the Input section states. For every other term:

| Term | Result |
|---|---|
| `Literal` of value kind `boolean` and Boolean type | `ExpressionKind::BooleanLiteral` |
| `Reference` to a node of `expression` family or of `value/literal` | the translation of that node's body |
| `Reference` to a `value/parameter` node whose `semantic_type` is Boolean | `ExpressionKind::ValueReference`, name `nN`, observation `StateObservation::Current` |
| `Application` of `filament.op.boolean.not` | `ExpressionKind::BooleanNot` |
| `Application` of `filament.op.boolean.and` | `ExpressionKind::Boolean` with `BooleanOperator::TotalAnd` |
| `Application` of `filament.op.boolean.or` | `ExpressionKind::Boolean` with `BooleanOperator::TotalOr` |
| `Application` of `filament.op.boolean.implies` | `ExpressionKind::Boolean` with `BooleanOperator::Implication` |
| `Application` of `filament.op.boolean.eq` | `ExpressionKind::Compare` with `ComparisonOperator::Equal` |
| `Application` of `filament.op.boolean.ne` | `ExpressionKind::Compare` with `ComparisonOperator::NotEqual` |

The operation identity is `CheckedApplicationOperation::identity()`, an entry of
`filament.checked-operation-catalog/v1` of `filament-verification-contracts` (`CheckedApplicationOperation::read()` returns `NotAdmitted` for a package read by `ContractPackageWireV1::read`, which installs no admitted operation views; an application with no identity is one the translation cannot lower), and the arguments of an
application are translated in order as left and right (the one argument of `not` as the operand).
This Boolean subset is the Boolean part of the `exact_constructs` of `CAPABILITY_CONTRACT` in `src/smt.rs`; the integer terms are those of [FR-012](./FR-012-integer-claim-translation.md). `and` and `or`
become the total operators because this subset has no partial operation, so evaluation order carries no
meaning. Every other operation, term form and node family is one the translation cannot lower, and
so is a reference chain that returns to a node it is already translating. The runner shall exit `2`
naming the node at which the translation stopped. This includes every rational, text, enum, collection, field
and conditional operation, every non-linear integer operation, and every quantifier whose domain
[FR-030](./FR-030-bounded-quantifier-domains.md) does not admit. Linear integer terms are translated as
[FR-012](./FR-012-integer-claim-translation.md) defines, and bounded quantifiers as
[FR-034](./FR-034-quantifier-package-translation.md) defines.

Two bounds apply to the translated `Expression` tree, because a `Reference` inlines the translation of its
target and so a node shared by several references expands once per reference: a chain `x1 = and(p, p)`,
`xk = and(x(k-1), x(k-1))` of 61 nodes, well inside the 10,000-node limit of the reader, expands to about
2^61 `Expression` nodes.

- **Node bound.** The translation shall count every `Expression` it produces, each inlined copy counted
  again, and shall stop at the first count above 10,000, the node limit of `check_expression` in
  `filament-contract-model` (smaller than `MAX_EXPRESSION_NODES` (100,000) of `src/smt.rs`). The translation
  shall do work proportional to this bound, never to the size of the expansion it refuses. The runner
  shall exit `2` naming the node `semantic_graph.nodes[N]` whose translation produced the 10,001st
  `Expression`. `Expression` nodes are counted in pre-order, an application before its operands, and a
  `ValueReference` or `BooleanLiteral` is produced by the node whose body holds the reference or literal.
  The count shall also be kept across all claims of the package, without reset between claims, and the
  translation shall stop at the first total above 100,000 and exit `2` naming the node
  `semantic_graph.nodes[N]` whose translation produced that `Expression`, so that many claims sharing
  one large tree cost no more than 100,000 `Expression` nodes.
- **Depth bound.** The depth of the translated tree shall be the nesting depth of its `Expression` nodes,
  with the root `Expression` of a clause body at depth `1`, each operand one level deeper than its
  application, and a `Reference` adding no level of its own. A tree deeper than 128 (`MAX_EXPRESSION_DEPTH`
  of `src/smt.rs`, counted from `1` in `lower_expression`) shall be refused: the runner shall exit `2`
  naming the node `semantic_graph.nodes[N]` whose translation produced the `Expression` at depth `129`. A
  count of reference hops is not the bound.

### Value declarations

- For each distinct `value/parameter` node `N` reached by any claim, the runner shall declare one
  `ValueDeclaration` named `nN` of kind `ValueDeclarationKind::Input`, of type `ValueType::Boolean` (or the
  integer type FR-012 defines for an `integer_range` parameter),
  with the span of node `N`, in ascending order of `N`. The name satisfies `SymbolName` (it starts
  with a letter) and cannot collide, because `N` is unique.
- The kind is `Input` and the observation is `Current` because `check_value_reference` admits an
  `Input` only with `Current`, and admits that at every execution point, so the choice does not depend
  on the synthetic anchor. A parameter reached from two claims is one declaration and one SMT symbol.
- Integer types, `IntegerDomain` and `OverflowPolicy` are mapped as FR-012 defines; a declaration of any
  type other than Boolean or integer is not produced.

### Refusals

- If `check_expression`, `from_clause`, the environment or package constructors, or
  `lower_boolean_statements` return an error for a clause, the runner shall exit `2` naming the clause
  `claim-N` and the error's path and message on stderr, and shall write no report.
- The translation shall be a pure function of the admitted package: the same package bytes yield the
  same clause order, identities, declarations and spans.

## Producer-execute

`producer-execute` is the `engineering-assurance producer-execute` command, the bounded
producer-execution component of the `engineering-assurance` repository. Its contract is
`FR-019` (bounded producer execution) and `FR-029` (the command) of that repository, and its request and
result protocols are `engineering-assurance.producer-execution-request/v1` and
`engineering-assurance.producer-execution-result/v1`. This spec cites that contract and does not restate
it. `filament-analyze` is the runner that executes the solvers.

## Inputs

- `--package FILE`: the `filament.contract-package/v1` document.
- `--z3 PATH` and `--cvc5 PATH`: the solver executables, each an absolute path.
- `--producer-execute PATH`: the absolute path of the `engineering-assurance` executable; it is never
  searched for on `PATH`.
- `--output NEW_REPORT.json`: the report destination, which does not exist beforehand.
- `--select FILE`: optional; the claim-selection document of the previous section. It may be given at
  most once.

Solver limits are `AdapterLimits::default()` of [NFR-001](../nonfunctional/NFR-001-determinism-bounds.md).

## Outputs

- One report in the `filament.differential-report/v2` contract of [FR-005](./FR-005-evidence-cli.md),
  rendered by the library renderer and published by the FR-005 publication discipline. The runner
  neither converts from nor accepts any other report version. With `--select`, the report also holds
  the `selection` member `{"excluded":[claim ids],"selected":[claim ids]}`, each list a strictly ascending
  sequence of `claim-N` by numeric `N` with no leading zero, `selected` non-empty and the two lists
  disjoint. `selected` names exactly the clauses of the report's assertions.
- Exit status: `0` satisfied, `1` refuted, `2` usage or input error, `3` non-conclusive, `4`
  publication or executor failure.
- Human diagnostics on stderr only.

## Behavior

- When invoked with the five required arguments and an admitted package, the runner shall build one consistency
  request over the selected analysable clauses of the package: every analysable clause without
  `--select`, the clauses the Claim selection section selects with it.
- The runner shall lower the request, run each solver through one `producer-execute` invocation for the
  query and one for its version, and render the report with the library renderer.
- The runner shall not start a solver by any means other than `producer-execute`.
- For each invocation the runner shall pass one `producer-execution-request/v1` document to
  `producer-execute --request -` on its stdin, and read its stdout.
- The runner shall fill the request fields as follows: `protocol` is the request protocol constant;
  `procedure` is `direct`; `producer.executable` is the absolute solver path and `producer.name` is
  `z3` or `cvc5`; `caller` is kind `filament-analyze.run-package`, version `1`; `arguments` are the literal
  query arguments (`-in -smt2` for Z3, `--lang=smt2 --no-incremental` for cvc5) or version arguments
  (`-version`, `--version`) of the engine; `environment` is empty; `outputs` and `outputTrees` are
  empty; `containment` is `process-group-v1` with contract kind `filament.solver-process`, version
  `v1`; `cancellation` is `disabled`; `response` is protocol `engineering-assurance.process-evidence`
  version `1`, adapter `engineering-assurance.cli-process-evidence-adapter` version `1`, exit codes `any`.
- The runner shall run the version invocation of an engine first, with `producer.version` `unprobed`,
  stdin `null` and no inputs, and shall use the trimmed version stdout, bounded by the solver version
  limit, as `producer.version` of that engine's query invocation. When the version invocation is not
  `completed` or yields empty text, the runner shall skip the query invocation and record the
  engine outcome given by the table.
- The runner shall run the query invocation with stdin bound to the input artifact role `query`, whose
  file is the lowered query staged in the capability root.
- The runner shall set the budget as: `timeoutMillis` the solver wall-time limit, `maxStdoutBytes` the
  lesser of the solver stdout limit and 8,388,608, `maxStderrBytes` the lesser of the solver stderr
  limit and 8,388,608, `maxInputBytes` the solver stdin limit, `maxOutputArtifacts` `0`,
  `maxOutputBytes` `0`, `maxDescendants` `16` and `maxConcurrency` `1`. The stdout ceiling of 8,388,608
  bytes is `MAX_STREAM_BYTES` of `engineering-assurance`; it is below the 16,777,216-byte
  `AdapterLimits` default, so a solver whose stdout exceeds it is reported as truncated and
  non-conclusive. The other values are the smallest that a single sequential solver needs and lie
  inside the request schema's caps.
- The runner shall create a runner-owned capability root as a fresh directory with mode `0700`,
  created exclusively under the parent directory of `--output` with a name unique to the run, shall
  pass it as `capabilityRoot` and shall remove it, with the staged query, on every exit path, including
  failure and non-conclusive outcomes. If the removal fails, the runner shall name the path and the
  cause on stderr and shall keep the exit status the run would otherwise have had.
- When `producer-execute` returns state `completed`, the runner shall take the solver's exit code,
  stdout and stderr from the retained process evidence.
- For every `producer-execute` result state the runner shall record the engine outcome given by the
  table below, a value of the closed `filament.differential-report/v2` engine outcome set, and shall never
  record `sat` or `unsat` from a state other than `completed`.

| Result state | Engine outcome recorded | Differential disposition |
|---|---|---|
| `completed` | classified from the solver exit code, stdout and stderr as the library classifies a direct run (`sat`, `unsat`, `unknown`, `solver-error`, `malformed-output`, `nonzero-exit`, and the other closed values) | by `compare_solver_records` |
| `unavailable` | `missing-executable` | unavailable |
| `timed_out` | `timed-out` | inconclusive |
| `failed`, exit code rejected | `nonzero-exit` | inconclusive |
| `failed`, signal | `signaled` | inconclusive |
| `failed`, stdout too large | `stdout-limit` | inconclusive |
| `failed`, stderr too large | `stderr-limit` | inconclusive |
| `failed`, observation or output artifact | `io-error` | inconclusive |
| `refused` | `spawn-error` | inconclusive |
| `malformed_response` | `malformed-output` | inconclusive |
| `containment_failure` | `unsupported-platform` when the runner's host is not Linux, otherwise `cleanup-failed` | unavailable on a non-Linux host, otherwise inconclusive |
| `cancelled` | `cancelled` | inconclusive |
| query skipped after a bad version run | the outcome the table gives for the failed version invocation; `identity-error` when the version text is empty | as for that outcome |

- The exit status is `3` for every row whose disposition is not agreement; only `missing-executable` and
  `unsupported-platform` make the disposition unavailable, and the exit status stays `3`.
- If `producer-execute` cannot be started, exits with any non-zero status (including `2`), is ended by
  a signal, or does not print exactly one newline-terminated result line that decodes as a
  `producer-execution-result/v1` document, the runner shall exit `4`, name the cause on stderr and
  write no report.
- When the agreed status is `satisfied`, the runner shall exit `0` and record `satisfied` in the report.
- When the report records a conclusive agreement on `refuted`, the runner shall exit `1`.
- If the differential disposition is not agreement, the runner shall publish the report retaining both
  engine records and exit `3`.
- When an argument is unknown, repeated, missing or lacks a value, the runner shall exit `2`, print the
  usage on stderr and write no report.
- When the package file is unreadable, is refused by the reader, or contains no analysable clause or a
  non-Boolean analysis claim, the runner shall exit `2`, name the cause on stderr and write no report.
- When a solver or `producer-execute` path is relative, empty, non-UTF-8 or over the path bound, the
  runner shall exit `2`, name the cause on stderr and write no report.
- When a solver path, the `producer-execute` path or the parent directory of `--output` has a symbolic
  link in any component, the runner shall exit `2`, name the path on stderr and write no report,
  because `producer-execute` refuses such paths.
- When an `analysis_claim` body is a Boolean-typed term that the translation of the previous section
  cannot lower, the runner shall exit `2`, name the node as `semantic_graph.nodes[N]` on stderr and
  write no report.
- When the destination exists, its parent directory is missing or unwritable, or publication fails
  ahead of the atomic rename, the runner shall exit `4` and leave the destination unmodified.
- If publication fails with `published-durability-unknown`, the runner shall exit `4` and report that
  state on stderr.
- The runner shall produce identical report bytes for identical package bytes, identical solver
  executables and identical limits, except for the `elapsedMs` fields, which carry the measured
  `producer-execute` execution time and are excluded from determinism as in
  [NFR-001](../nonfunctional/NFR-001-determinism-bounds.md).
- The runner shall write no file other than the capability root and its staged query, which it removes, and the staged and published report.
- The runner shall modify no developer-owned file as defined in [FR-005](./FR-005-evidence-cli.md).

## Constraints

| ID | Constraint | Type | Validation |
|---|---|---|---|
| FR-009-CON-1 | The runner shall emit `filament.differential-report/v2` as defined by [FR-005](./FR-005-evidence-cli.md) and `schemas/differential-report-v2.schema.json`. The schema is revised by the optional closed `selection` member only; no other member, outcome or rule changes, and a report without `selection` validates as before. | Interface | Inspection |
| FR-009-CON-2 | The runner shall store no approval state, approval list, key or signature material. A selection is data supplied by the caller for one run. | Interface | Inspection |
| FR-009-CON-3 | The runner shall verify no signature. | Interface | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-009-AC-1 | With a stub `producer-execute` that records every request it receives, a `run-package` run issues exactly two requests per engine (version, query), each a valid `producer-execution-request/v1` naming the configured absolute solver path, an empty environment and the built-in response binding, and no solver process is started by any other path. | Test |
| FR-009-AC-2 | With a stub `producer-execute` returning `completed` results in which both solvers answer `sat` with a model that replays against the assertions of consistent clauses, `run-package` exits `0` and writes a report that `validate_report_document` accepts with agreed status `satisfied`. | Test |
| FR-009-AC-3 | With results in which both solvers answer `unsat` for contradictory clauses, `run-package` exits `1` and the report records agreed status `refuted`. | Test |
| FR-009-AC-4 | For each of the states `unavailable`, `timed_out`, `failed`, `refused`, `malformed_response`, `containment_failure` and `cancelled` returned for one engine, and for `sat` against `unsat` across engines, `run-package` exits `3` and the report retains both engine records. | Test |
| FR-009-AC-5 | A `producer-execute` that cannot be started, that exits with status `2`, with status `1`, or with any other non-zero status, that is ended by a signal, that prints no result line, two lines, or a line that is not a `producer-execution-result/v1` document makes `run-package` exit `4` with the cause on stderr and no report file. | Test |
| FR-009-AC-6 | A missing package file, a file `ContractPackageWireV1::read` refuses, a package that names a dependency, a package with no `analysis_claim`, a package with a non-Boolean `analysis_claim`, and a document one byte over the 1,048,576-byte `CheckedPackageReadLimits::bounded()` limit (an `Incomplete` result) each exit `2`, write no report and name the cause on stderr. A document exactly at the limit is not refused for size. | Test |
| FR-009-AC-7 | An unknown, repeated or missing argument and an option without a value each exit `2` with the usage on stderr and no report. | Test |
| FR-009-AC-8 | Each of a relative, an empty, a non-UTF-8 and an over-long path, given in turn as `--z3`, `--cvc5` and `--producer-execute`, exits `2` and writes no report. | Test |
| FR-009-AC-9 | An existing `--output` file, a missing parent directory and an unwritable parent directory each exit `4`; an existing destination keeps its bytes. | Test |
| FR-009-AC-10 | A unit test of the runner function, injecting a directory-sync failure after rename through the publication seam of [TC-008](../test/TC-008-library-cli-atomic.md), shows the runner maps the `published-durability-unknown` publication error to exit `4` with that state named on stderr. | Test |
| FR-009-AC-11 | Two runs over the same package bytes, executables and stub results, whose stub timings differ, write report files that are byte-identical once every `elapsedMs` field is set to `0`, and that differ in no other field. | Test |
| FR-009-AC-12 | After a run the package file and its directory hold no added, removed or modified entry; the only entries added to the `--output` parent directory are the published report, and during the run a capability-root directory of mode `0700` that is gone after every exit path (conclusive, non-conclusive, exit `2`, exit `4`). | Test |
| FR-009-AC-13 | `run-package --help` lists the five required arguments (`--package`, `--z3`, `--cvc5`, `--producer-execute`, `--output`), the one optional argument `--select`, six in all, and the exit classes `0` to `4`. | Inspection |
| FR-009-AC-17 | For each row of the result-state table other than `completed`, a stub result of that state for one engine yields the stated engine outcome in the published report, the stated disposition, and exit `3`. For the `completed` row, the engine outcome is the classification the library gives the same bytes run directly and the exit status is the one the comparison gives (`0`, `1` or `3`). | Test |
| FR-009-AC-18 | A symlinked component in `--z3`, `--cvc5`, `--producer-execute` or the `--output` parent each exits `2` and writes no report. | Test |
| FR-009-AC-19 | A package whose Boolean `analysis_claim` term the translation cannot lower exits `2` naming the node as `semantic_graph.nodes[N]`, with no report. | Test |
| FR-009-AC-20 | For a package with analysis claims at nodes `i` and `j`, the statements the translation builds have `ClauseRef` (`source.name`, `analysis`, `1`) with clause ids `claim-i` and `claim-j` in node order, one `RequirementRef` for both, and the execution point `Handler` named `analysis`; `from_clause` accepts both. A `source.name` that `PackageId::new` refuses exits `2` naming the package name. | Test |
| FR-009-AC-21 | Each of `filament.op.boolean.not`, `and`, `or`, `implies`, `eq` and `ne`, applied to Boolean parameters, translates to the `ExpressionKind` and operator of the Terms table, and a Boolean `Literal` to `BooleanLiteral`; a claim that is a `Reference` to an expression node translates to the body of that node. | Test |
| FR-009-AC-22 | Two claims that reference the same Boolean `value/parameter` node `N` yield one `ValueDeclaration` named `nN` of kind `Input` and type Boolean, referenced with observation `Current`, and the lowered query holds one variable symbol for it; two distinct parameter nodes yield two declarations. | Test |
| FR-009-AC-23 | Each of a quantifier over a text domain or an unbounded integer domain, a `filament.op.integer.div`, `filament.op.integer.rem` or `filament.op.integer.mod` application, an integer-typed operand of a Boolean operation (`not`, `and`, `or`, `implies`, Boolean `eq` or `ne`), a reference to a node that is neither an expression, a literal nor a parameter, a reference cycle, a tree deeper than 128 (128 nested `not` applications over a Boolean parameter, depth 129, are refused; 127, depth 128, are not) and a shared-subterm chain `x1 = and(p, p)`, `xk = and(x(k-1), x(k-1))` of 61 nodes (about 2^61 expanded `Expression` nodes; a tree of 10,000 `Expression` nodes is not refused for size and one of 10,001 is) and claims sharing one 10,000-node tree (the 11th, whose first `Expression` is the 100,001st in total, is refused) each exit `2` naming `semantic_graph.nodes[N]` of the node where translation stopped, with no report. The shared-subterm run finishes within the time of a normal run. | Test |
| FR-009-AC-24 | The `SourceSpan` of a claim is built from `source_map.nodes[N].occurrences[0].regions[0]` of its node: line `1`, column `1`, byte offsets the region start and end, document `contract-package`, revision `1`. A claim node whose `source_map.nodes[N].occurrences` is empty exits `2` naming the node. | Test |
| FR-009-AC-25 | By inspection, an error from `check_expression`, `from_clause` or `lower_boolean_statements` for a clause maps to exit `2` naming `claim-N`, the path and the message, with no report; a Boolean claim does not reach these errors, because the translation bounds are tighter, but an integer claim can: the IR refuses, for example, an addition over a range of more than one point (FR-012), which exits `2` naming `claim-N`. | Inspection |
| FR-009-AC-26 | Two translations of the same package bytes yield equal clause order, ids, declarations and spans. | Test |
| FR-009-AC-28 | When the capability root cannot be removed (an entry inside it is unreadable), the runner names the root path on stderr and exits with the status the run would otherwise have had, and the report is still published. | Test |
| FR-009-AC-27 | Every contract-package document that a test of `run-package` reads is built at test time through the public APIs of the `&es (a `filament.checked-package/v2` document built in test code, read with `CheckedPackageV2::read`, lowered with `CheckedPackageV2::lower`, and the canonical bytes of the resulting `CompleteContractPackageV2`), or by byte-level mutation of such a built document (malformed, oversized, at-limit); no contract-package document or copy of another repository's file is committed to this repository. | Inspection |
| FR-009-AC-15 | Every request the runner logs validates against the request schema published by `producer-execute --schema request`, and the budget it carries has `maxStdoutBytes` and `maxStderrBytes` at most 8,388,608, `maxOutputArtifacts` and `maxOutputBytes` `0`, `maxDescendants` `16` and `maxConcurrency` `1`. | Test |
| FR-009-AC-16 | The version request of an engine carries `producer.version` `unprobed` and stdin `null`; its query request carries the trimmed version stdout of the version result; a version result that is not `completed` or is empty yields no query request, the engine is recorded with the outcome the result-state table gives for that version result (`identity-error` when the text is empty), and the exit status is `3`. | Test |
| FR-009-AC-29 | With a package of two analysable claims at distinct regions and `--select` naming a span that contains the region of one, only that claim is analysed: the report's assertions come from that claim alone, `selection.selected` is that claim's id and `selection.excluded` is the other's, and the report validates. | Test |
| FR-009-AC-30 | A span that contains neither the region of a claim nor the region of a node a selected claim reaches exits `2` naming the span. A span that overlaps one claim's region without containing it exits `2` naming the span and the claim, also when another listed span contains that claim, and also when the span wholly contains a different claim. Each writes no report. | Test |
| FR-009-AC-31 | A selection that is not valid JSON, has an unknown or missing member, a wrong `protocol`, an empty `authority` or `identity`, a `start` or `end` that is negative, fractional, not a number or outside the region offset type, a `start` not below `end`, or is one byte over 1,048,576 bytes exits `2` with the cause on stderr and no report; `--select` given twice or without a value exits `2`. | Test |
| FR-009-AC-32 | A run with a selection that names every claim writes a report that differs from the report of a run without `--select` over the same package only in its `selection` member (once every `elapsedMs` is `0`); the run without `--select` writes no `selection` member. | Test |
| FR-009-AC-33 | A package holding one selected Boolean claim and, excluded, a claim whose body is non-Boolean, a claim whose term cannot be lowered, and a claim with no occurrence, run with `--select`, exits as the selected claim alone would; the same package without `--select` exits `2`. A selected claim whose body is non-Boolean or cannot be lowered exits `2`. | Test |
| FR-009-AC-34 | By inspection, `src/` holds no code that writes, reads or verifies approval state, a key or a signature for `--select`; the runner reads the selection file only as the document of the Claim selection section. | Inspection |
| FR-009-AC-35 | A selected claim whose body references an expression node, or a parameter, whose region lies outside every listed span, or that has no region, exits `2` naming `claim-N` and `semantic_graph.nodes[M]`; the same claim with a span that also contains that node is analysed, also when that span is a separate span over a shared parameter and an intermediate claim between the two stays excluded and is not examined. | Test |
| FR-009-AC-36 | A selection whose `spans` is empty is well formed: invoked with it, the runner exits `2` with the stderr cause `empty selection` and no report, and that cause differs from the one printed for a malformed selection. | Test |
| FR-009-AC-37 | Regions are matched by exact comparison of `authority` and `identity`: a span differing in case, a leading `./`, a doubled separator or a trailing separator does not contain the region, and a span that then matches nothing exits `2` as an unmatched span. | Test |
| FR-009-AC-38 | The six cases RP-SAT, RP-UNSAT, RP-UNSUPPORTED, RP-TIMEOUT, RP-MISSING-ENGINE and RP-DISAGREE of [TC-006](../test/TC-006-differential-counterexample.md), each over a package built at test time (FR-009-AC-27) and stub `producer-execute` results, run through `run_package` and through the spawned `run-package` binary with the same `--package`, `--z3`, `--cvc5`, `--producer-execute` and `--output` paths (the output removed between the two runs), give the same exit status (`0`, `1`, `2`, `3`, `3`, `3` in that order), no report for RP-UNSUPPORTED and a valid report for the others whose bytes are equal once every `elapsedMs` is `0`; for RP-UNSUPPORTED `RunnerError::message()` and the CLI stderr each name `semantic_graph.nodes[N]` and no `producer-execute` request is made; RP-TIMEOUT, RP-MISSING-ENGINE and RP-DISAGREE retain both engine records, record the dispositions `inconclusive`, `unavailable` and `disagreement`, and are never reported as `satisfied` or `refuted`. | Test |
| FR-009-AC-39 | With the real `engineering-assurance` executable and real Z3 and cvc5, the RP-SAT package (`consistent()`) exits `0` with agreed status `satisfied` and the RP-UNSAT package (`contradictory()`) exits `1` with agreed status `refuted`, each with a report that validates; the test is skipped with a stated reason when the executable or either solver is absent, and a skip is not evidence for FR-009-AC-38. | Test |
| FR-009-AC-14 | A run with the real `engineering-assurance` executable and real Z3 and cvc5 on a contract-package fixture built at test time (FR-009-AC-27) exits `0` or `1` and its report validates; the test is skipped with a stated reason when either solver or the executable is absent. | Test |

## Dependencies

 [FR-005](./FR-005-evidence-cli.md) for report rendering, validation and publication;
`engineering-assurance` FR-019 and FR-029 for `producer-execute`; `filament-contract-ir` for
`filament.contract-package/v1` and `ContractPackageWireV1::read`; `filament-spec-language` for the FSL checked
package that is lowered to the contract package.

### Gaps

The following gaps were measured against the current code. They are preconditions of an implementing
change, and none is closed by invented behaviour here.

1. `filament-spec-language` has no production step that produces a contract package. The only call of
   `CheckedPackageV2::lower` in its `fsl-package` sources is the `cfg(test)` module
   `fsl-package/src/emit/extent_agreement.rs`, and no `filament.contract-package/v1` document is committed.
   FSL's `checked_handoff::write` writes a checked-package handoff directory. FSL emits no `analysis_claim` node today: the only use of `ClaimForm::AnalysisClaim` in its sources
   is the admission-corpus test `fsl-package/src/emit/tests/admission_corpus.rs`, so a package lowered
   from FSL's `examples/checked-handoff/` units has no analysable clause and the runner exits `2` on
   it. Fixtures that reach exit `0` or `1` are built at test time (FR-009-AC-27) and are not
   committed.
2. The analysis API takes `StatementInput` built from `ContractPackage<TypedExpression>` and a
   `DeclarationEnvironment`, and `ContractPackageV1` holds a semantic graph of checked terms. Selection
   of the analysable clauses and the translation of an `analysis_claim` body into a statement are
   defined above; the code that implements them does not exist yet and is the implementing change.
3. `execute_solver` in `src/solver.rs` is the only builder of a `SolverRecord`, and it runs a solver
   process directly. `run-package` needs a library constructor that builds a `SolverRecord` from
   `producer-execute` process evidence and the outcome table above; without it the report cannot be
   rendered. Whether `execute_solver` itself migrates is outside this requirement.
4. `producer-execute` is a separate executable. The runner depends on it through a process boundary
   only, with no Cargo dependency on `engineering-assurance`.
5. Integer types, operation rows and overflow policy are defined by FR-010 and FR-012, which extend this
   requirement; the package has no overflow policy field, and FR-012 states the mapping.
6. The `filament-contract-model` revision in `Cargo.lock` predates the `CatalogUnavailable` variant of
   `ContractPackageReadResult` that `Cargo.toml` (branch `main`) delivers. The implementing change updates
   the lock to current `main` of `filament-contract-ir` so the reader has the four variants the Input section
   handles.
7. Whether the `source` reference and byte region of an `analysis_claim` node of a package lowered from QSL
   spell the QSL file and cover the declaration text that a caller selects is decided by the producer of
   `analysis_claim` nodes (QSL-1900, cited by VER-98), which is not defined here. Until it holds, a
   selection built from real QSL source cannot be exercised end to end; tests build packages whose
   regions are set at test time (FR-009-AC-27).
8. `run-package --select` is specified in this requirement and implemented by VER-100 and VER-102
   (Backlog). Until they land, FR-009-AC-29 to AC-37 have no implementation or tagged test; only the
   report validation and schema behavior of FR-005-AC-7 is implemented.
