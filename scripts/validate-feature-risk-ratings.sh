#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
bundle_root=$(mktemp -d "${TMPDIR:-/tmp}/filament-analyze-rating-validation.XXXXXX")
trap 'rm -rf "$bundle_root"' EXIT HUP INT TERM

# Keep the complete FR and FRR corpus in the OKF graph. Other requirement and
# test documents live outside this focused bundle, so their trace edges cannot
# be resolved here; the normal repository validation target checks those docs.
mkdir -p "$bundle_root/spec"
cp -R "$repo_root/spec/functional" "$repo_root/spec/validation" "$bundle_root/spec/"

quire validate --okf \
    --severity trace:feature-without-risk-rating=error \
    --severity trace:dangling-trace-reference=off \
    --repo "agent-ix/filament-analyze=$bundle_root"
