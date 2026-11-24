---
id: AN-003
title: "Architecture description has stale CLI-scope claims"
type: Anomaly
status: identified
safety_impact: unassessed
---
# [AN-003] Architecture description has stale CLI-scope claims

## Description

The 2026-11-23 regulated-standards review found that AD-001 described the CLI
as exposing only `publish-report`, while `src/main.rs` also dispatches
`run-package`. This batch corrects the Interfaces and Trust text and the
CLI-scope disposition to describe both commands and their distinct roles.
Review is still pending before this anomaly can be resolved.

This record remains identified pending complete document correction and
verification. No severity or safety impact has been assessed.
