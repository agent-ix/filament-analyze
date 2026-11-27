---
id: SUR-001
title: Filament Analyze evidence suite registry
type: SuiteRegistry
---

# Filament Analyze evidence suite registry

## Suites

| ID | Name | Command | Tool | Evidence Kind |
|---|---|---|---|---|
| SUITE-004 | Real Z3 and cvc5 solver integration tests | `make real-tool-test` | cargo test with caller-supplied Z3 and cvc5 | Integration |

SUITE-004 runs the repository's real-solver tests with Z3 and cvc5 executables.
Set `FILAMENT_Z3` and `FILAMENT_CVC5` to absolute executable paths. The
range and domain tests for FR-033-AC-8 are required tests and fail with the
missing solver-path variable named when either path is unset. Other optional
real-tool tests remain ignored for ordinary `cargo test` runs and are selected
by this suite. The dose-package test also requires `engineering-assurance` on
`PATH` or at `FILAMENT_ENGINEERING_ASSURANCE`.

The lane records each selected test's result, the exact configured solver paths
and reported versions, Cargo and Quoin versions, test source, source trace IDs,
and relevant solver limits. Criterion IDs from each test's authoritative
`#[trace]` attributes populate `RunEntry.traceIds`; the complete source tag
list is retained in `config.source_trace_ids`. Test case and measurement IDs
remain visible there without being presented to Quoin as obligation IDs. The
registry does not maintain a parallel test-to-obligation map.

Unavailable or invalid required inputs produce `outcome: error` and
`config.execution_state: not-executed`. Test failures produce `outcome: fail`.
The lane returns nonzero after recording any error or failure. All Cargo
commands execute under `locked-build.sh` admission.
