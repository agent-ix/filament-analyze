# PLAT-1817 matrix revalidation after Quoin #1052

Quoin #1052 merged at `33b0c6f3`. The committed SUITE-004 run was evaluated
with the Quoin worktree binary:

```text
/home/peter/dev/quoin/.worktrees/wave2-evidence-mutation/target/debug/quoin matrix --repo . --json
```

The command exited 0. It returned 28 requirements and 237 criteria. All 14
criterion IDs carried by the 10 SUITE-004 entries were recognized and bound to
their run entries; none were unrecognized. Their evidence status is `stale`,
because every entry in the retained run is `error` / `not-executed` while Z3 and
cvc5 are unavailable. Across the matrix, 14 criteria are stale and 223 remain
undischarged. This verifies trace-ID recognition, not a passing real-tool run.
