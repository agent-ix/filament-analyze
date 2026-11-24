---
id: SR-5644
title: "Code review — Filament build identity and anomalies"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-analyze; build.rs, src/report.rs, tests/analysis_semantics.rs, spec/functional/FR-005-evidence-cli.md, spec/test/TC-007-evidence-integrity.md, spec/assurance/AD-001-analyze-architecture.md, spec/anomalies/AN-001-main-ci-dispatch-only.md, spec/anomalies/AN-002-build-identity.md, spec/anomalies/AN-003-stale-cli-scope.md"
review_set: subset
---

## Summary

Reviewed PR #117 at frozen head `8bf13f859d815eb23763d20bf79c62b8887cc22a` for PLAT-1821 and VER-115, including its Rust build script, producer field, focused test, and changed specifications. The central Git identity path is implemented, but a newly created loose tag can leave Cargo's cached build identity stale; `build.rs` also omits the repository-required SPDX header.

## Assurance Context

AP-001 (`spec/assurance/AP-001-analyze-release.md`) applies: its scope is the filament-analyze v0.1 source candidate and it requires code review and gap analysis. The changed producer identity path affects report provenance and the `producer.version` field, but does not change solver classification or evidence conclusiveness. Reviewed AP-001 with AD-001, AA-001, CAC-001, and MP-001; AA-001 remains open and no producer-reliance decision or build-identity measurement record is present. No active exception applies. A focused test passed per the code lead; no additional build or test was run in this review.

## Verdict

**CONDITIONAL** — a newly created Git tag can change `git describe` without changing any watched path, so an incremental Cargo build can report a stale producer version.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `build.rs` watches existing refs individually but not the refs directory. Creating a new loose tag on the current commit does not change `HEAD`, `index`, `packed-refs`, or any previously watched ref file, so Cargo may reuse the old build-script output and keep reporting the pre-tag identity after `git describe` would return the new tag. | build.rs:33-35 |
| FND-002 | low | The newly added Rust build script has no SPDX license or copyright header, contrary to the Rust source-file convention. | build.rs:1 |

## Reviewed code and linked specification

- `build.rs`: derives normalized `git describe --tags --dirty --always`, package-version fallback, and Cargo rerun paths.
- `src/report.rs`: serializes `FILAMENT_BUILD_VERSION` as `producer.version`.
- `tests/analysis_semantics.rs`: focused trace-tagged assertion compares the report field with the current Git description.
- `FR-005` and `TC-007`: reviewed the build-identity behavior and test procedure for Git and source-archive cases.
- `AD-001`: reviewed CLI scope and the per-theory soundness and assumption discussion.
- `AN-001` through `AN-003`: reviewed lifecycle state, claims, and index entries.

## Disposition

FND-001 is addressed by watching the Git `refs` and `refs/tags` directories as well as existing
ref files. FND-002 is addressed by adding the standard SPDX and copyright header to `build.rs`.
This disposition records code changes only; no post-fix build or test was run.
