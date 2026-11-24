---
id: AN-001
title: "Main CI verification is dispatch-only"
type: Anomaly
status: identified
safety_impact: unassessed
---
# [AN-001] Main CI verification is dispatch-only

## Description

The 2026-11-23 regulated-standards review found that this repository does not
run code-validation CI automatically on pushes or pull requests. In this
repository, `.github/workflows/ci.yml` runs only on `workflow_dispatch`; the
separate `.github/workflows/cla.yml` runs on pull requests and comments for CLA
processing, not code validation. No `v*` tag-attestation workflow is present
under `.github/workflows/`.

This record describes the configured workflow triggers and does not claim a
failure of a manually dispatched run or assign a severity or safety impact.
