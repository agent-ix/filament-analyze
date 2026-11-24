---
id: AN-003
title: "Architecture description has stale CLI-scope claims"
type: Anomaly
status: identified
safety_impact: unassessed
---
# [AN-003] Architecture description has stale CLI-scope claims

## Description

The 2026-11-23 regulated-standards review found that AD-001 describes the CLI
as exposing only `publish-report`, while `src/main.rs` also dispatches
`run-package`. The current VER-115 change corrects the System Boundary text,
but AD-001 still contains stale publish-only statements in its Interfaces and
Trust section and CLI-scope disposition. Those statements require correction
and review before this anomaly can be resolved.

This record remains identified pending complete document correction and
verification. No severity or safety impact has been assessed.
