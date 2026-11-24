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

SUITE-004 runs the repository's ignored tests that invoke real Z3 and cvc5
executables. Set `FILAMENT_Z3` and `FILAMENT_CVC5` to absolute executable
paths. The dose-package test also requires `engineering-assurance` on `PATH`
or at `FILAMENT_ENGINEERING_ASSURANCE`.

The lane records each selected test's result, the exact configured solver paths
and reported versions, Cargo and Quoin versions, test source, and relevant
solver limits. Trace IDs come from each test's authoritative `#[trace]`
attributes and are copied into the retained `RunEntry`; the registry does not
maintain a parallel test-to-obligation map.

Unavailable or invalid required inputs produce `outcome: error` and
`config.execution_state: not-executed`. Test failures produce `outcome: fail`.
The lane returns nonzero after recording any error or failure. All Cargo
commands execute under `locked-build.sh` admission.
