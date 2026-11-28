---
id: TC-007
title: "Verify evidence schemas, identities, and truthfulness"
type: TC
relationships:
  - target: ix://agent-ix/filament-analyze/FR-005
    type: verifies
  - target: ix://agent-ix/filament-analyze/NFR-002
    type: verifies
---
# TC-007: Verify evidence schemas, identities, and truthfulness

## Description

Verify retained evidence detects missing, altered, extra, and contradicted records.

## Test Procedure

Validate every report and available envelope against their schemas. Remove and mutate each required
proof-input identity, artifact, status, limitation, engine field, and output. Contradict raw responses
and declared outcomes, and alter the artifact census.

Runtime report validation checks canonical JSON, canonical query identity, and differential
disposition. Differential reports make no PGM-01 envelope claim.

For the `filament.differential-report/v2` lane, validate a genuine v2 document both standalone
and against its authoritative query and differential run. Omit each required producer field,
`assertions[].clause`, `assertions[].source`, every canonical proof-input identity, and every
retained-byte field; standalone validation must reject each omission. Mutate retained query bytes
without updating `queryDigest`; standalone validation must reject the mismatch. Mutate raw solver
bytes alone and check that a contradiction with the declared result is refused. Then change the raw
bytes and all dependent result fields consistently: standalone validation may admit this self-consistent
record, while validation against the unchanged authoritative run must refuse the substitution.
For a build made from the current Git worktree, check that `producer.version` equals the output of
`git describe --tags --dirty --always` after removing one leading `v`.
Separately mutate
the clause/source identities and clause, binding-set, or request digests, then recompute dependent
query identity so the document is internally consistent; validation against the unchanged
authoritative query and run must still reject it. Add `contractIrRevision` to a v2 document and pass
a valid v1 document to each v2 validation path; both paths must reject them. Add each removed solver
pin and redundant hash field to a v2 document; the closed schema must reject those fields and
the removed mismatch outcomes.

For the `selection` member (FR-005-AC-7), validate a v2 document with no `selection` and one whose
assertion clauses are `claim-N` clauses and whose `selection` is well formed; both must pass. Then
add, one at a time, an extra `selection` member, a missing `selected`, a missing `excluded`, an empty
`selected`, an id in both lists, an id with a wrong prefix, a non-string id, an id with a leading zero,
an assertion clause with a leading zero or a sign while `selected` is canonical, an `excluded` list out
of order, a `selected` list out of order, a repeated id in each list, and a `selected` set that differs
from the clauses of the assertions; the standalone validator must reject each. A pair of disjoint lists
of 200,000 ids each must be accepted or refused without a quadratic number of comparisons.

## Expected Results

The valid corpus passes. Every omission, canonical proof-input identity mismatch, outcome
contradiction, or census change fails closed with a stable diagnostic. A standalone report cannot
authenticate substituted raw solver bytes when the dependent result fields agree with them;
authoritative-run comparison rejects that substitution. Optional explanation evidence is never
inferred as present.
The v2 standalone lane checks the query identity derivable from retained bytes; the
authoritative-input lane rejects substituted clause/source identities and clause, binding-set, or
request digests even when the document's own digests were recomputed. Both reject the duplicate
revision field, omitted required fields, and v1 documents without a fallback. A well-formed `selection`
passes standalone validation; each malformed `selection` of the procedure fails it.
