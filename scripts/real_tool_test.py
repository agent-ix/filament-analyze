#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX

"""Run ignored real-solver tests and retain their outcomes in Quoin."""

from __future__ import annotations

import datetime as dt
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
SUITE = "SUITE-004"
TRACE_ID = re.compile(r'"([A-Z]+-[0-9]+(?:-(?:AC-[0-9]+|M-[0-9]+))?)"')
TEST_LINE = re.compile(r"^test\s+(.+?)\s+\.\.\.\s+(ok|FAILED)$")
REAL_TESTS = {
    "official_z3_cvc5_differential_corpus_agrees": "analysis_semantics",
    "the_official_engines_decide_the_enum_membership_fixtures": "datatype_translation",
    "real_z3_cvc5_decide_membership_fixtures": "enum_lowering",
    "real_z3_cvc5_models_replay": "datatype_replay",
    "real_z3_cvc5_decide_integer_fixtures_identically": "integer_lowering",
    "real_z3_cvc5_decide_record_and_option_fixtures": "record_option_lowering",
    "real_z3_cvc5_decide_range_fixtures": "range_quantifier_lowering",
    "real_solver_timeout_on_a_quantified_query_is_inconclusive": "range_quantifier_lowering",
    "real_z3_cvc5_decide_domain_fixtures": "range_quantifier_lowering",
    "real_solvers_agree_with_the_controlled_run_on_the_dose_fixtures": "package_runner",
}
SOLVER_TESTS = set(REAL_TESTS) - {
    "real_solvers_agree_with_the_controlled_run_on_the_dose_fixtures"
}


def utc_timestamp() -> str:
    return dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def executable_version(path: Path, args: list[str]) -> tuple[str | None, str | None]:
    try:
        result = subprocess.run([str(path), *args], capture_output=True, text=True, check=False, timeout=10)
    except (OSError, subprocess.TimeoutExpired) as error:
        return None, str(error)
    text = (result.stdout or result.stderr).strip()
    if result.returncode != 0 or not text:
        return None, f"version command exited {result.returncode}: {text}"
    return text.splitlines()[0], None


def configured_executable(
    variable: str, fallback: str, *, require_absolute: bool = True, canonicalize: bool = True
) -> tuple[Path | None, str | None]:
    configured = os.environ.get(variable)
    if configured:
        path = Path(configured).expanduser()
        if require_absolute and not path.is_absolute():
            return None, f"{variable} must be an absolute executable path, got {configured!r}"
        if path.is_absolute():
            candidate = path
        else:
            found = shutil.which(configured)
            if found is None:
                return None, f"{variable} executable is not on PATH: {configured}"
            candidate = Path(found)
    else:
        found = shutil.which(fallback)
        if found is None:
            return None, f"{variable} is unset and {fallback} is not on PATH"
        candidate = Path(found)
    if not candidate.is_file() or not os.access(candidate, os.X_OK):
        return None, f"{variable} is not an executable file: {candidate}"
    return (candidate.resolve() if canonicalize else candidate.absolute()), None


def ignored_tests() -> dict[str, dict[str, Any]]:
    """Read test identities and trace IDs from authoritative Rust attributes."""
    found: dict[str, dict[str, Any]] = {}
    for name, binary in REAL_TESTS.items():
        source = ROOT / "tests" / f"{binary}.rs"
        lines = source.read_text(encoding="utf-8").splitlines()
        for index, raw in enumerate(lines):
            line = raw.strip()
            if not line.startswith("fn ") or line[3:].split("(", 1)[0].strip() != name:
                continue
            prefix = lines[:index]
            attrs: list[str] = []
            cursor = index - 1
            while cursor >= 0 and prefix[cursor].strip().startswith("#["):
                attrs.append(prefix[cursor].strip())
                cursor -= 1
            attributes = " ".join(reversed(attrs))
            if "#[test]" not in attributes or "#[ignore" not in attributes:
                raise ValueError(f"{source}: {name} is no longer an ignored test")
            source_trace_ids = sorted(set(TRACE_ID.findall(attributes)))
            trace_ids = [trace_id for trace_id in source_trace_ids if "-AC-" in trace_id]
            if not trace_ids:
                raise ValueError(f"{source}: ignored test {name} has no criterion IDs in #[trace]")
            found[name] = {
                "binary": binary,
                "symbol": f"{binary}::{name}",
                "traceIds": trace_ids,
                "sourceTraceIds": source_trace_ids,
                "source": source.relative_to(ROOT).as_posix(),
            }
            break
        if name not in found:
            raise ValueError(f"{source}: ignored test {name} not found")
    return found


def entry(test: dict[str, Any], outcome: str, config: dict[str, str]) -> dict[str, Any]:
    return {"symbol": test["symbol"], "outcome": outcome, "traceIds": test["traceIds"], "config": config}


def run_plan(path: Path) -> int:
    plan = json.loads(path.read_text(encoding="utf-8"))
    cargo = plan["cargo"]
    results: dict[str, dict[str, Any]] = {}
    for name in plan["tests"]:
        binary = REAL_TESTS[name]
        command = [cargo, "test", "--locked", "--test", binary, name, "--", "--ignored", "--exact"]
        print("Focused command: " + " ".join(command), flush=True)
        try:
            process = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, check=False)
        except OSError as error:
            results[name] = {"status": "not-executed", "error": str(error)}
            print(f"could not execute {name}: {error}", file=sys.stderr, flush=True)
            continue
        output = process.stdout + process.stderr
        print(output, end="" if output.endswith("\n") else "\n", flush=True)
        observed = next((TEST_LINE.match(line.strip()) for line in output.splitlines() if TEST_LINE.match(line.strip())), None)
        if observed is None:
            results[name] = {
                "status": "not-executed",
                "error": f"cargo exit={process.returncode}; test harness emitted no result line",
            }
        elif observed.group(2) == "ok" and process.returncode == 0:
            results[name] = {"status": "pass"}
        else:
            results[name] = {"status": "fail", "error": "ignored real-tool test failed; see focused output"}
    Path(plan["output"]).write_text(json.dumps(results, sort_keys=True) + "\n", encoding="utf-8")
    return 0


def main() -> int:
    if len(sys.argv) == 3 and sys.argv[1] == "--run-plan":
        return run_plan(Path(sys.argv[2]))

    try:
        tests = ignored_tests()
    except (OSError, ValueError) as error:
        print(f"real-tool-test configuration error: {error}", file=sys.stderr)
        return 2

    versions: dict[str, str] = {}
    input_errors: dict[str, str] = {}
    executables: dict[str, Path] = {}
    for variable, fallback, version_args in [
        ("FILAMENT_Z3", "z3", ["-version"]),
        ("FILAMENT_CVC5", "cvc5", ["--version"]),
    ]:
        executable, error = configured_executable(variable, fallback)
        if error:
            input_errors[variable] = error
            versions[variable] = "unavailable"
            continue
        assert executable is not None
        version, error = executable_version(executable, version_args)
        if error:
            input_errors[variable] = error
            versions[variable] = "unavailable"
        else:
            executables[variable] = executable
            versions[variable] = version or "unknown"

    cargo, cargo_error = configured_executable(
        "CARGO", "cargo", require_absolute=False, canonicalize=False
    )
    cargo_version = "unavailable"
    if cargo_error:
        input_errors["cargo"] = cargo_error
    else:
        cargo_version, cargo_error = executable_version(cargo, ["--version"])
        if cargo_error:
            input_errors["cargo"] = cargo_error
            cargo_version = "unavailable"
    quoin, quoin_error = configured_executable("QUOIN", "quoin", require_absolute=False)
    quoin_version = "unavailable"
    if quoin_error:
        print(f"cannot record SUITE-004: {quoin_error}", file=sys.stderr)
        return 2
    quoin_version, quoin_error = executable_version(quoin, ["--version"])
    if quoin_error:
        print(f"cannot record SUITE-004: {quoin_error}", file=sys.stderr)
        return 2

    locked_build_value = os.environ.get("LOCKED_BUILD", "locked-build.sh")
    locked_build = Path(locked_build_value).expanduser()
    if locked_build.is_file() and os.access(locked_build, os.X_OK):
        locked_build = locked_build.resolve()
    else:
        resolved = shutil.which(locked_build_value)
        if resolved:
            locked_build = Path(resolved).resolve()
        elif os.environ.get("LOCKED_BUILD_DEFAULT") == "1":
            fallback = Path.home() / "dev/worktrees/locked-build.sh"
            if fallback.is_file() and os.access(fallback, os.X_OK):
                locked_build = fallback.resolve()
            else:
                input_errors["locked_build"] = f"locked-build wrapper not found: {locked_build_value}"
        else:
            input_errors["locked_build"] = f"LOCKED_BUILD does not resolve to executable wrapper: {locked_build_value}"

    producer, producer_error = configured_executable("FILAMENT_ENGINEERING_ASSURANCE", "engineering-assurance")
    producer_version = "unavailable"
    if producer_error:
        input_errors["FILAMENT_ENGINEERING_ASSURANCE"] = producer_error
    else:
        producer_version, producer_error = executable_version(producer, ["--version"])
        if producer_error:
            input_errors["FILAMENT_ENGINEERING_ASSURANCE"] = producer_error

    plan: list[str] = []
    for name in tests:
        required = {"FILAMENT_Z3", "FILAMENT_CVC5"}
        if name not in SOLVER_TESTS:
            required.add("FILAMENT_ENGINEERING_ASSURANCE")
        if cargo_error:
            required.add("cargo")
        if input_errors.get("locked_build"):
            required.add("locked_build")
        if not (required & input_errors.keys()):
            plan.append(name)

    outcomes: dict[str, dict[str, Any]] = {}
    if plan:
        with tempfile.TemporaryDirectory(prefix="filament-real-tool-") as temp_dir:
            temp = Path(temp_dir)
            path_bin = temp / "bin"
            path_bin.mkdir()
            for variable, name in [("FILAMENT_Z3", "z3"), ("FILAMENT_CVC5", "cvc5")]:
                if variable in executables:
                    (path_bin / name).symlink_to(executables[variable])
            if producer is not None:
                (path_bin / "engineering-assurance").symlink_to(producer)
            output_path = temp / "outcomes.json"
            plan_path = temp / "plan.json"
            plan_path.write_text(json.dumps({"cargo": str(cargo), "tests": plan, "output": str(output_path)}), encoding="utf-8")
            env = os.environ.copy()
            env["FILAMENT_Z3"] = str(executables.get("FILAMENT_Z3", ""))
            env["FILAMENT_CVC5"] = str(executables.get("FILAMENT_CVC5", ""))
            env["PATH"] = str(path_bin) + os.pathsep + env.get("PATH", "")
            command = [str(locked_build), str(ROOT / "target"), sys.executable, str(Path(__file__).resolve()), "--run-plan", str(plan_path)]
            print("Focused command through locked-build: " + " ".join(command), flush=True)
            process = subprocess.run(command, cwd=ROOT, env=env, check=False)
            if output_path.exists():
                outcomes = json.loads(output_path.read_text(encoding="utf-8"))
            elif process.returncode != 0:
                for name in plan:
                    outcomes[name] = {"status": "not-executed", "error": f"locked-build exited {process.returncode} before the runner wrote outcomes"}

    failed = False
    entries: list[dict[str, Any]] = []
    for name, test in tests.items():
        source = test["source"]
        config: dict[str, str] = {
            "test_source": source,
            "source_trace_ids": ",".join(test["sourceTraceIds"]),
            "command": f"cargo test --locked --test {test['binary']} {name} -- --ignored --exact",
            "cargo_version": cargo_version or "unavailable",
            "quoin_version": quoin_version or "unavailable",
            "FILAMENT_Z3": str(executables.get("FILAMENT_Z3", os.environ.get("FILAMENT_Z3", "unset"))),
            "FILAMENT_Z3_version": versions["FILAMENT_Z3"],
            "FILAMENT_CVC5": str(executables.get("FILAMENT_CVC5", os.environ.get("FILAMENT_CVC5", "unset"))),
            "FILAMENT_CVC5_version": versions["FILAMENT_CVC5"],
            "solver_limits": "AdapterLimits::default()" if name != "real_solver_timeout_on_a_quantified_query_is_inconclusive" else "wall_time_ms=1; cleanup_time_ms=1000; graceful_cleanup_ms=20; monitor_interval_ms=1",
            "test_inputs": "fixtures constructed by the ignored test body",
        }
        if name not in SOLVER_TESTS:
            config["FILAMENT_ENGINEERING_ASSURANCE"] = str(producer or os.environ.get("FILAMENT_ENGINEERING_ASSURANCE", "unset"))
            config["FILAMENT_ENGINEERING_ASSURANCE_version"] = producer_version or "unavailable"
        required = {"FILAMENT_Z3", "FILAMENT_CVC5"}
        if name not in SOLVER_TESTS:
            required.add("FILAMENT_ENGINEERING_ASSURANCE")
        if cargo_error:
            required.add("cargo")
        if input_errors.get("locked_build"):
            required.add("locked_build")
        missing = {key: input_errors[key] for key in required if key in input_errors}
        if missing:
            result = {"status": "not-executed", "error": "; ".join(f"{key}: {value}" for key, value in sorted(missing.items()))}
        else:
            result = outcomes.get(name, {"status": "not-executed", "error": "no result was produced"})
        status = result["status"]
        if status == "not-executed":
            config["execution_state"] = "not-executed"
            config["error"] = str(result.get("error", "required input unavailable"))
            outcome = "error"
            failed = True
        elif status == "fail":
            config["execution_state"] = "executed"
            config["error"] = str(result.get("error", "test failed"))
            outcome = "fail"
            failed = True
        else:
            config["execution_state"] = "executed"
            outcome = "pass"
        entries.append(entry(test, outcome, config))

    results_file = tempfile.NamedTemporaryFile(prefix="filament-real-tool-", suffix=".json", delete=False, mode="w", encoding="utf-8")
    with results_file:
        json.dump({"entries": entries}, results_file, sort_keys=True)
        results_file.write("\n")
    tool = f"filament-analyze real-tool-test (cargo {cargo_version}; quoin {quoin_version})"
    recorded = subprocess.run(
        [str(quoin), "evidence", "record", "--suite", SUITE, "--tool", tool, "--adapter", "entries", "--kind", "Integration", "--repo", str(ROOT), "--results", results_file.name, "--timestamp", utc_timestamp()],
        cwd=ROOT,
        check=False,
    )
    if recorded.returncode != 0:
        return recorded.returncode
    if failed:
        print("real-tool-test recorded error/not-executed results; no passing real-tool run is claimed", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
