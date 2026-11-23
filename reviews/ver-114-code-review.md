---
id: SR-17096
title: code-review of VER-114 PR 113
type: SpecReview
analysis: code-review
scope: 'agent-ix/filament-analyze; src/hybrid_tool.rs, README.md, reviews/; VER-114 PR #113'
review_set: subset
---

## Summary

Ticket: VER-114. Reviewed PR #113 head f4861b7a8ae53bb6ad2177c31732df49e53420f7 (one commit) against origin/main, with the rust-review lane folded in (rust-style loaded). The diff replaces the hard-coded PHAVer, engineering-assurance and runtime-library paths in the test helper `phaver_config()` with three required environment variables, documents them in README.md, and adds three unit tests. The repository is public. No branch edits were made; the mutation run was reverted with `git checkout` and the tree is clean.

## Verdict

FAIL: not mergeable yet. The source change itself is correct: no fallback, the unset variable is named in the panic, and the PHAVer tests stay `#[ignore]`. But the ticket's acceptance criterion "no absolute home paths in the repo" is not met. Two tracked review records still contain home-directory paths (FND-001). Everything else is low severity.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The repo still contains absolute home paths after this PR, so the ticket's acceptance criterion "no absolute home paths in the repo" fails. a grep of HEAD for home-directory paths hits two tracked review records: reviews/2026-10-21-plat-1173-gap-analysis.md:25 (`quoin matrix --repo <home>/... --json`) and reviews/ir-730-gap-analysis.md:34 (`quoin matrix --repo <home>/... --json`). Both ship in the public repo, and neither path is needed: the command reads the same with `--repo .`. Fix: rewrite the path in place. Do not delete the review files. | reviews/2026-10-21-plat-1173-gap-analysis.md:25, reviews/ir-730-gap-analysis.md:34 |
| FND-002 | low | `source_has_no_absolute_home_paths` greps the text of one file (src/hybrid_tool.rs) for the home-directory prefix. It passes at this head while the repo still violates the criterion it stands for (FND-001). It also misses other home forms (macOS user, root and tilde home forms). It checks file content, not behaviour. The two config tests plus review already cover the regression it targets. Recommend deleting it rather than widening it into a repo-wide content scanner. | src/hybrid_tool.rs:1434-1439 |
| FND-003 | low | The injectable `phaver_config_from(lookup)` seam and its two unit tests add about 40 lines to test a test-only fixture. The repo's own pattern for the same need is one line per variable: `std::env::var_os("FILAMENT_Z3").expect("FILAMENT_Z3 is required")` (tests/analysis_semantics.rs:756, tests/datatype_replay.rs:963, tests/enum_lowering.rs:725). The behaviour is correct; the cost is extra surface for no product behaviour. Optional simplification: inline `var_os(..).expect(..)` and drop `phaver_config_reads_tool_paths_from_the_environment` and `phaver_config_names_the_unset_variable`. | src/hybrid_tool.rs:1381-1432 |
| FND-004 | low | Tracked review records also name machine-local scratch logs: `<tmp>/ir1211-fmt.status` (reviews/ir-1211-code-review.md:26), `<tmp>/ir730-msrv-focused.log` (reviews/ir-730-code-review.md:38) and `<tmp>/.../ver95-review-logs/...` (reviews/ver-95-pr104-code-review.md:38, reviews/ver-95-pr104-gap-analysis.md:45). These are not home paths, so they fall outside the literal criterion. They are still machine-specific absolute paths in a public repo, and no reader can resolve them. Fix: rewrite them as plain "local log" wording in the same pass as FND-001. | reviews/ir-1211-code-review.md:26, reviews/ir-730-code-review.md:38, reviews/ver-95-pr104-code-review.md:38 |

## Checks

All checks ran under the shared `locked-build.sh` build-lane wrapper with RUSTUP_TOOLCHAIN=1.98.1, CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0 and CARGO_INCREMENTAL=0. Focused runs only.

- `cargo test --lib -- hybrid_tool::tests::phaver_config hybrid_tool::tests::source_has_no_absolute_home_paths`: 3 passed.
- Mutation run (reverted afterwards with `git checkout`; tree clean): the executable was switched to read FILAMENT_PRODUCER_EXECUTE, the library variable got a `/usr/lib` fallback, and a `<home>/...` literal was re-added. All 3 new tests failed. `reads_tool_paths` failed with left `/opt/ea`, right `/opt/phaver`. `names_the_unset_variable` failed on `unwrap_err` of an Ok result. `source_has_no_absolute_home_paths` failed with 1 hit. Every mutant is killed.
- `cargo fmt --all -- --check`: exit 0. `cargo clippy --lib --tests -- -D warnings` on the unmutated head: exit 0, no warnings.
- Error text: the panic reads `FILAMENT_PHAVER_LIBRARY_PATH is required` (or `... must be valid UTF-8`). It names the variable, which matches the README.

Grep sweep over all tracked files at HEAD for home-directory prefixes (Linux, macOS, root and tilde forms), WSL drive mounts, user-local bin/share directories and named temp-dir paths. Hits are listed in FND-001 and FND-004. Synthetic `/opt` and temp-dir fixture literals in src/ and tests/ that are never executed, so they are fine.

Rust lane: no new public API (all items are in `mod tests`); no unsafe; the error is `String`, which fits a test helper; panic on a missing variable is the intended behaviour for an ignored integration test; the non-UTF-8 library path is rejected rather than lossily converted. No `#[trace]` is needed: these tests exercise test fixtures, not a specified AC.

## Coverage

Plan completion: not assessed

## Examined Scope

```yaml
scope:
- id: src/hybrid_tool.rs#phaver_config
  path: src/hybrid_tool.rs
  role: examined
  excerpt: phaver_config_from(|name| std::env::var_os(name)).unwrap_or_else(|error| panic!("{error}"))
- id: README.md#phaver-env
  path: README.md
  role: examined
  excerpt: The ignored PHAVer tests in src/hybrid_tool.rs read FILAMENT_PHAVER, FILAMENT_PRODUCER_EXECUTE and FILAMENT_PHAVER_LIBRARY_PATH. An unset variable fails the test with its name.
- id: reviews/
  path: reviews/
  role: examined
  excerpt: tracked review records scanned for machine-specific absolute paths
```

## Dispositions

Disposition pass 1 reviewed fix commit cfafff26136441aae2dce59b9b0c2714c3fd19a8 against the original head f4861b7a8ae53bb6ad2177c31732df49e53420f7. The only other change to this file is the one allowed edit: machine-specific paths in the text were replaced with neutral placeholders before the file is committed to the public repo. The findings are otherwise unchanged. A re-grep of every tracked file at cfafff2 for home-directory prefixes (Linux, macOS, root and tilde forms) and named temp-dir paths finds no machine-specific path. The only hits are the two reviews/ver-114-* files, which this cleaned version replaces, and the existing synthetic `src/solver.rs` fixture literal. `cargo fmt --all -- --check` and `cargo clippy --lib --tests -- -D warnings` both exit 0 on cfafff2. Verdict after this pass: every finding is fixed, so PR #113 is mergeable.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Both `quoin matrix --repo` paths now read `--repo .`; the review files were kept. |
| FND-002 | fixed | `source_has_no_absolute_home_paths` was deleted. |
| FND-003 | fixed | The seam and both helper tests were removed. `phaver_config()` now uses `std::env::var(name).unwrap_or_else(\|_\| panic!("{name} is required"))` per variable. |
| FND-004 | fixed | The temp-dir log paths in ir-1211, ir-730 and ver-95-pr104 reviews were reworded as "the recorded ... log/status". |
