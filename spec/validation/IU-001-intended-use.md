---
id: IU-001
title: Intended use of filament-analyze
type: IntendedUse
tool: filament-analyze
relationships: []
---
# Intended use of filament-analyze

## Purpose

`filament-analyze` is a Rust library and command-line tool that analyzes admitted
requirement contracts. It translates supported contract expressions into
solver queries, runs configured external solvers, classifies their results,
and retains source-mapped evidence. Unsupported constructs and inconclusive
solver outcomes remain distinct from a satisfied or refuted analysis.

## Records produced

The library returns typed analysis and method results to its caller. The CLI
writes or publishes these records:

| Record kind | Command that writes it |
|---|---|
| Differential analysis report (`filament.differential-report/v2`) | `filament-analyze run-package --package <file> --output <new-file> --z3 <path> --cvc5 <path> --producer-execute <path>` |
| Validated report copy | `filament-analyze publish-report --input <file> --output <new-file>` |

The report retains the request and query identities, solver observations,
normalized outcomes, and any replayed model. Library calls may also return
bounded synthesis candidates or hybrid-reachability outcomes in memory; they
do not write those results to a file.

## Who relies on them

Contract authors and maintainers use the results to find contradictions,
check implications, inspect counterexamples, and review whether two configured
solvers agree. Reviewers may use a retained report when assessing a contract
change. A report is evidence about one input package and one configured run;
it does not replace the review or approval of a responsible person.

## Limits

`filament-analyze` is not a compliance oracle. Its results do not establish
regulatory compliance, product safety, clinical safety, or acceptance of a
product or process. It does not assign a feature-risk rating or decide whether
a risk is acceptable. A candidate from synthesis is not proof unless
independently validated. Unsupported inputs, missing tools, timeouts, malformed
solver results, and disagreement remain non-conclusive. People remain
responsible for interpreting reports and making acceptance decisions.

## Operating context

The library reads local contract-package data and configuration supplied by its
caller. The CLI reads local package and report files and writes a requested new
report path. Solver execution is supported on Linux; on other operating systems
the adapter reports `unsupported`.

The solver adapter invokes the configured Z3 and cvc5 executables as separate
processes. The `run-package` command uses the configured
`engineering-assurance` executable through its `producer-execute` interface to
run those solver processes. When a caller selects hybrid reachability, the
configured external PHAVer executable may be run through the same
`producer-execute` interface with its runtime library directory supplied by
the caller. On Unix, cancellation may invoke `/bin/kill` to stop the child
process group. These paths and settings are supplied by the caller; the
analysis input cannot select or modify executable paths.
