---
title: Analyze a quantified contract package
type: how-to
description: Run differential analysis on a contract package with finite collection quantifiers.
---

# Analyze a quantified contract package

Analyze a contract package with a finite collection quantifier using `filament-analyze run-package`.

The command expands each `forall` or `exists` domain before it sends the query to both solvers.

## Before you start

Start with a contract package from the upstream producer. Make sure Z3, cvc5 and `producer-execute` are available in your environment.

## Steps

1. **Choose a finite collection domain.** Use a collection quantifier over literal Boolean values, enum values or bounded integers. The analyzer expands the collection into the formula before making the solver query.

   A domain can contain at most 1,024 elements. A larger collection fails with an input error. Other collection forms are rejected.

2. **Run differential analysis.** Run this command from the workspace directory that contains `build/`. Use absolute paths for both solvers and `producer-execute`:

   ```sh
   filament-analyze run-package \
     --package ./build/contract-package.json \
     --output ./build/analysis-report.json \
     --z3 /usr/bin/z3 \
     --cvc5 /usr/bin/cvc5 \
     --producer-execute /usr/bin/producer-execute \
     --request consistency
   ```

3. **Read the report.** The JSON file contains both solver results and the differential outcome.

## Expected output

The command writes `build/analysis-report.json` as a `filament.differential-report/v2` JSON report.

## Exit status

The process exit status gives the outcome:

| Status | Meaning |
| --- | --- |
| `0` | Both solvers agree the clauses are consistent. |
| `1` | Both solvers agree the clauses are contradictory. |
| `2` | Usage or input error; no report is written. |
| `3` | Non-conclusive result; the report preserves both engine records. |
| `4` | Execution/publication failed; report is absent or durability is unknown. |

## Verify the result

Confirm the report file exists and contains both engine results. For exit `2`, check the quantifier domain and its element count.
