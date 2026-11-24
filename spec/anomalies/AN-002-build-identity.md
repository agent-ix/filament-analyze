---
id: AN-002
title: "Reports do not identify the application build"
type: Anomaly
status: identified
safety_impact: unassessed
---
# [AN-002] Reports do not identify the application build

## Description

The 2026-11-23 regulated-standards review found that a differential report's
`producer.version` identifies only the Cargo package version, not the build
that produced it. `src/report.rs` currently sets the value from
`CARGO_PKG_VERSION`, which is `0.1.0`; this does not distinguish tagged,
untagged, or dirty builds. VER-115 specifies a `git describe` build identity,
but the implementation and focused verification are not yet complete on this
branch.

This record remains identified until a person verifies the producer identity
change. No severity or safety impact has been assessed.
