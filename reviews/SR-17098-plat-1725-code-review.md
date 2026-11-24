---
id: SR-17098
title: "Code review of README polarity and hybrid naming fix and hello removal"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-analyze; README.md, CLAUDE.md, docs/images/figures.json, docs/images/kinds-light.svg, docs/images/kinds-dark.svg, src/lib.rs, tests/integration.rs"
review_set: subset
---
# Code Review of README Polarity and Hybrid Naming Fix and Hello Removal

## Summary

Ticket: PLAT-1725 (and PLAT-1726). PR agent-ix/filament-analyze#114 at head
0c915b54ef4f34646baf0d58d8b8a7b97f9db2ca, diff `origin/main...HEAD`, seven files.
Includes the rust-review lane for `src/lib.rs` and `tests/integration.rs`.

Verified against the code: in `src/analysis.rs` consistency (102-107), contradiction
(139-142) and dead antecedent (215-220) assert every group Positive; implication negates
the consequent (171) and redundancy the candidate (197). `reach_hybrid` is
`src/hybrid_tool.rs:159`; `synthesize` and `validate_candidate` are in
`src/hybrid_synthesis.rs`; no `reach` or `replay_hybrid` exists in `src`. The README, the
img alt text, `figures.json` `label` and both SVG `aria-label`/`title` texts now agree with
each other and with the code. After removing `hello`, no reference remains anywhere in the
repo (source, tests, docs, Makefile, `.github`). The removed tests covered only the
placeholder, so no real behavior loses coverage. The CLAUDE.md layout line is accurate.

## Verdict

**FAIL (one medium).** `rustfmt --check src/lib.rs` reports a diff: the file now ends with a
blank line after `pub use solver::*;`, so `make fmt-check` (part of `make ci`) fails. One
low on the figure's box text.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `src/lib.rs` ends with `pub use solver::*;` followed by an empty line (`\n\n` at EOF); `rustfmt --check` reports the diff, so `make fmt-check` and `make ci` fail. Delete the trailing blank line. | `src/lib.rs:25-26` |
| FND-002 | low | Figure `kinds` box text (unchanged) shows "antecedents + NOT consequent" and "peers + NOT candidate", but implication and redundancy also assert the assumptions positive (`src/analysis.rs:160-171`, `190-197`), unlike the contradiction and dead-antecedent rows which show them. The new alt text says "a fixed set of statements" and does not conflict, but the figure under-states what is asserted. | `docs/images/figures.json:53,59` |
