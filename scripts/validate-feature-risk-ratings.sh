#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
bundle_root=$(mktemp -d "${TMPDIR:-/tmp}/filament-analyze-rating-validation.XXXXXX")
trap 'rm -rf "$bundle_root"' EXIT HUP INT TERM

if [ "$#" -eq 0 ]; then
    echo "rating validation requires explicit module paths; pass QUIRE_MODULE_ARGS='--module <spec-artifacts-process> --module <spec-artifacts-iso>' to make spec" >&2
    exit 2
fi

module_count=0
has_process_module=0
has_feature_risk_rating=0
has_iso_module=0
expect_module_path=0
for arg do
    if [ "$expect_module_path" -eq 1 ]; then
        if [ ! -d "$arg" ] || [ ! -f "$arg/manifest.yaml" ]; then
            echo "rating validation module path is missing or has no manifest.yaml: $arg" >&2
            exit 2
        fi
        module_count=$((module_count + 1))
        if grep -Eq '^name:[[:space:]]*spec-artifacts-process[[:space:]]*$' "$arg/manifest.yaml"; then
            has_process_module=1
            if grep -Eq '^[[:space:]]*-[[:space:]]*name:[[:space:]]*FeatureRiskRating[[:space:]]*$' "$arg/manifest.yaml"; then
                has_feature_risk_rating=1
            fi
        fi
        if grep -Eq '^name:[[:space:]]*spec-artifacts-iso[[:space:]]*$' "$arg/manifest.yaml"; then
            has_iso_module=1
        fi
        expect_module_path=0
    elif [ "$arg" = "--module" ]; then
        expect_module_path=1
    else
        echo "rating validation accepts only repeatable --module <path> arguments; got: $arg" >&2
        exit 2
    fi
done
if [ "$expect_module_path" -eq 1 ]; then
    echo "rating validation received --module without a path" >&2
    exit 2
fi
if [ "$module_count" -eq 0 ] || [ "$has_process_module" -ne 1 ] || [ "$has_feature_risk_rating" -ne 1 ] || [ "$has_iso_module" -ne 1 ]; then
    echo "rating validation requires explicit spec-artifacts-process (with FeatureRiskRating) and spec-artifacts-iso module paths" >&2
    exit 2
fi

# Keep the complete FR and FRR corpus in the OKF graph. Other requirement and
# test documents live outside this focused bundle, so their trace edges cannot
# be resolved here; the normal repository validation target checks those docs.
mkdir -p "$bundle_root/spec"
cp -R "$repo_root/spec/functional" "$repo_root/spec/validation" "$bundle_root/spec/"

quire validate "$@" --okf \
    --severity trace:feature-without-risk-rating=error \
    --severity trace:dangling-trace-reference=off \
    --repo "agent-ix/filament-analyze=$bundle_root"
