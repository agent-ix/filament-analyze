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
`producer.version` identified only the Cargo package version, not the build
that produced it. The implementation now embeds normalized `git describe`
output for Git worktrees and uses the Cargo package version when Git metadata is
unavailable. The archive fallback is implementation behavior and is outside the
focused Git-worktree criterion. Focused verification is pending.

This record remains identified until a person verifies the producer identity
change. No severity or safety impact has been assessed.
