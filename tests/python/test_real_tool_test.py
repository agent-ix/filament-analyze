# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX

import json
import sys
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scripts"))
import real_tool_test


class ResultClassificationTests(unittest.TestCase):
    def test_pass_line_is_a_pass(self):
        self.assertEqual(
            real_tool_test.classified_result(0, "test example ... ok\n"),
            {"status": "pass"},
        )

    def test_failed_test_line_is_a_failure(self):
        self.assertEqual(
            real_tool_test.classified_result(101, "test example ... FAILED\n"),
            {"status": "fail", "error": "ignored real-tool test failed; see focused output"},
        )

    def test_missing_result_line_is_not_executed(self):
        self.assertEqual(
            real_tool_test.classified_result(101, "error: could not compile\n"),
            {
                "status": "not-executed",
                "error": "cargo exit=101; test harness emitted no result line",
            },
        )

    def test_not_executed_outcome_is_error_and_retains_reason(self):
        item, failed = real_tool_test.result_entry(
            {"symbol": "module::test", "traceIds": ["FR-001-AC-1"]},
            {"status": "not-executed", "error": "solver missing"},
            {},
        )
        self.assertTrue(failed)
        self.assertEqual(item["outcome"], "error")
        self.assertEqual(item["config"]["execution_state"], "not-executed")
        self.assertEqual(item["config"]["error"], "solver missing")

    def test_pass_and_failure_outcomes_are_distinct(self):
        test = {"symbol": "module::test", "traceIds": ["FR-001-AC-1"]}
        passed, failed_pass = real_tool_test.result_entry(test, {"status": "pass"}, {})
        failed, failed_test = real_tool_test.result_entry(
            test, {"status": "fail", "error": "assertion"}, {}
        )
        self.assertFalse(failed_pass)
        self.assertTrue(failed_test)
        self.assertEqual(passed["outcome"], "pass")
        self.assertEqual(passed["config"]["execution_state"], "executed")
        self.assertEqual(failed["outcome"], "fail")
        self.assertEqual(failed["config"]["execution_state"], "executed")


class QuoinRecordingTests(unittest.TestCase):
    def test_records_entries_and_returns_quoin_success(self):
        captured = {}

        def fake_run(command, **kwargs):
            captured["command"] = command
            captured["kwargs"] = kwargs
            result_path = Path(command[command.index("--results") + 1])
            captured["payload"] = json.loads(result_path.read_text(encoding="utf-8"))
            captured["result_path"] = result_path
            return type("Completed", (), {"returncode": 0})()

        with patch.object(real_tool_test.subprocess, "run", side_effect=fake_run):
            exit_code = real_tool_test.record_results(
                Path("/bin/quoin"), [{"symbol": "module::test", "outcome": "pass"}], "cargo 1.94.1", "quoin 0.36.10", Path("/repo")
            )
        self.addCleanup(captured["result_path"].unlink, missing_ok=True)
        self.assertEqual(exit_code, 0)
        self.assertIn("cargo 1.94.1", " ".join(captured["command"]))
        self.assertEqual(captured["payload"]["entries"][0]["outcome"], "pass")
        self.assertEqual(captured["kwargs"]["cwd"], Path("/repo"))

    def test_propagates_quoin_recording_failure(self):
        with patch.object(
            real_tool_test.subprocess, "run", return_value=type("Completed", (), {"returncode": 23})()
        ) as run:
            exit_code = real_tool_test.record_results(
                Path("/bin/quoin"), [], "cargo 1.94.1", "quoin 0.36.10"
            )
        results_path = Path(run.call_args.args[0][run.call_args.args[0].index("--results") + 1])
        self.addCleanup(results_path.unlink, missing_ok=True)
        self.assertEqual(exit_code, 23)


if __name__ == "__main__":
    unittest.main()
