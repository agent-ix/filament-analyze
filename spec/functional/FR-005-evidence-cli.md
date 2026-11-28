---
id: FR-005
title: "Provide evidence reports, differential checks, and CLI"
type: FR
relationships:
  - target: ix://agent-ix/filament-analyze/FR-004
    type: depends_on
  - target: ix://agent-ix/filament-analyze/interface_001
    type: implements
---
# FR-005: Provide evidence reports, differential checks, and CLI

## Description

The library and CLI shall produce the same normalized analysis report, derivation envelope, and
stable exit classification, and shall retain cross-engine discrepancies without hiding either result.

## Behavior

- Reports retain request, package, schema, encoding, canonical query identity, assertion map,
  engine, observed solver version, configured executable path and limits, raw response,
  normalized result, counterexample, and producer. The canonical proof-input identities bind
  the reported conclusion to the exact selected clauses, bindings, request, and query bytes.
- The report producer version identifies the application build. For a build made from a Git
  worktree, the report uses the normalized output of `git describe --tags --dirty --always`
  (leading `v` removed).
- Differential runs retain both engine records before computing agreement.
- Disagreement, unavailable comparison, or unverified model makes the differential result
  non-conclusive; it cannot become conclusive until a human-reviewed adjudication is retained.
- Every filed semantic defect receives a stable regression fixture that reproduces the pre-fix
  discrepancy and passes only with its reviewed disposition.
- CLI output is deterministic JSON; human-readable diagnostics go to stderr and do not alter it.
- Output publication is all-or-nothing and never edits developer-owned files.
- Linux publication does not report success until the staged file and parent directory have both
  been synchronized. A failure before atomic rename reports `destination-unmodified`; a parent
  directory sync failure after rename reports `published-durability-unknown`, because complete
  destination bytes can already be visible. Callers do not delete or retry that state as though the
  destination were absent.
- The analysis product contract names `filament.analysis-report/v1`; the differential runtime
  report emits `filament.differential-report/v2`. Canonical JSON is UTF-8, compact, recursively
  key-sorted, and has no insignificant whitespace. Validation re-derives canonical proof-input
  identities against retained material and the authoritative query where supplied; it does not
  add a hash of already retained raw streams or the report itself.
- The `filament.differential-report/v2` closed contract removes the duplicated
  `contractIrRevision` field. It retains the producer application name and version, complete clause
  and source identities with canonical clause digests, request and query identities, exact query
  bytes, and raw solver-stream bytes. The engine record retains its observed executable path and
  version only; it has no executable byte count or SHA, expected version or SHA, or configuration
  digest. The closed outcome set excludes executable-digest-mismatch, executable-changed, and
  version-mismatch because no pin or digest check exists.
  Dependency selection and resolution remain in `Cargo.toml` and `Cargo.lock`; their source revision
  is not copied into report metadata. The renderer and validator accept the v2 contract only, with
  no v1 fallback or compatibility reader. Differential reports do not claim an unavailable PGM-01 envelope.
- `run-package --select` ([FR-009](./FR-009-package-runner.md)) adds the optional member
  `selection`, `{"excluded":["claim-N"...],"selected":["claim-N"...]}`, to the v2 report, and only that
  member. The member is closed and absent when no selection was given. When present, each list is a
  strictly ascending sequence of `claim-N` ids by numeric `N` with no leading zeros, `selected` is
  non-empty, the lists are disjoint, and `selected` equals the set of `claim-N` clause ids of
  `assertions[]`, each id spelled exactly `claim-N`. The schema carries the shape (closed members, `claim-N` pattern, non-empty
  `selected`, unique items); `validate_report_document` in `src/report.rs` carries the order, disjointness
  and assertion-equality rules, and both are checked by the standalone validator without the
  authoritative query. This revises the v2 schema in place; it adds no version and no reader for an
  older shape.
- Differential status is the closed set `agreement`, `disagreement`, `unavailable`, and
  `inconclusive`. Agreement is conclusive only when both exact engines return the same conclusive
  status and every required sat model is replay-verified. Different conclusive statuses are
  disagreement; a missing engine is unavailable; every other combination is inconclusive.
- The v1 library executes caller-prepared queries with caller-supplied solver paths and limits and
  renders the authoritative report bytes. The `publish-report` subcommand validates and publishes those exact bytes; it does not
  reconstruct authoritative contract packages or solver configurations from JSON. The `run-package`
  subcommand of [FR-009](./FR-009-package-runner.md) is the CLI surface that decodes a package and solver paths. Publication
  creates a new destination through a same-directory temporary file, sync, and atomic rename; an
  existing destination is a developer-owned file and is refused. Recoverable pre-rename failures
  close and remove their staging file and synchronize that removal. An uncatchable process
  termination can leave a private staging file but never exposes partial destination bytes; the
  file name identifies it as non-authoritative staging rather than a report.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-005-AC-1 | CLI publication preserves the authoritative library report byte-for-byte and returns its stable exit class. | Test (TC-008) |
| FR-005-AC-2 | Every report validates against its closed, versioned schema. | Test (TC-007) |
| FR-005-AC-3 | Differential disagreement retains both results and cannot be reported as agreement. | Test (TC-006) |
| FR-005-AC-4 | Failed publication leaves no partial result or modified developer-owned file. | Test (TC-008) |
| FR-005-AC-5 | Every filed semantic defect maps to an executable retained regression fixture. | Inspection and test (TC-006) |
| FR-005-AC-6 | The closed `filament.differential-report/v2` schema requires `producer.name` and `producer.version`; every `assertions[].clause`, `assertions[].source`, and `assertions[].clauseDigest`; `bindingSetDigest`, `requestDigest`, `queryDigest`, and `queryHex`; and every engine's `configuredExecutable`, observed `identity.path` and `identity.version` when available, `stdoutHex`, `stderrHex`, and `modelHex`. It refuses `identity.bytes`, `identity.sha256`, `limits.executableBytes`, `expectedVersion`, `expectedExecutableSha256`, `configurationDigest`, `querySha256`, `stdoutSha256`, `stderrSha256`, `modelSha256`, and `reportDigest`, and the three removed mismatch outcomes. Standalone v2 validation derives `queryDigest` from retained `requestDigest` and query bytes. Validation bound to the authoritative `QueryBundle` and differential run rejects a report whose clause/source identities, clause/binding-set/request identities, or raw solver output differ from the authoritative reconstruction; standalone validation does not claim to derive those values from absent source content or authenticate self-consistent substituted raw output. Both validation paths reject missing named fields, `contractIrRevision`, and every `filament.differential-report/v1` document, with no v1 fallback. The closed schema requires `encodingProfile` `filament.smtlib2/v2` and a `logic` equal to the logic the query's feature set requires (`QF_LIA` for a query without datatype declarations; otherwise the logic FR-020 specifies; `LIA` or `ALL` for a query that holds a bounded solver quantifier per FR-032, without or with datatypes), and validation refuses a report that names `filament.smtlib2/v1` or `QF_UF`, with no fallback. `assertions[]` lists the named assertions of the query only; integer range constraints are not named assertions ([FR-010](./FR-010-linear-integer-arithmetic.md)). | Test (TC-007) |
| FR-005-AC-7 | `validate_report_document` accepts a report without `selection` and a report with a `selection` whose lists are strictly ascending `claim-N` ids, disjoint, with non-empty `selected` equal to the `claim-N` clause ids of `assertions[]`. It refuses a `selection` with an extra member, a missing list, an empty `selected`, an id in both lists, an id that is not exactly `claim-N` (a wrong prefix, a non-string, a leading zero or a sign), a list out of ascending order or with a repeated id, or `selected` ids that differ from the assertions' clauses, where an assertion clause counts only when its last `:` segment is exactly `claim-N`. Disjointness is checked in time linear in the two list lengths. | Test (TC-007) |
| FR-005-AC-8 | For the current Git worktree, `producer.version` equals the output of `git describe --tags --dirty --always` with one leading `v` removed. | Test (TC-007) |

## Dependencies

FR-004 and native issue #5.
