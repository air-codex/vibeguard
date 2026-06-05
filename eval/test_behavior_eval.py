#!/usr/bin/env python3
"""Tests for behavior-level eval reporting."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

import run_behavior_eval


class BehaviorEvalTest(unittest.TestCase):
    def test_dataset_loader_validates_required_fields(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "behavior.jsonl"
            path.write_text('{"id": "missing"}\n', encoding="utf-8")

            with self.assertRaises(run_behavior_eval.BehaviorDatasetError):
                run_behavior_eval.load_jsonl(path)

    def test_json_expectations_check_nested_paths(self) -> None:
        stdout = json.dumps({
            "hookSpecificOutput": {
                "permissionDecision": "deny",
                "permissionDecisionReason": "blocked",
            }
        })

        checks = run_behavior_eval.evaluate_expectations(
            {
                "exit_code": 0,
                "json": [{"path": "hookSpecificOutput.permissionDecision", "equals": "deny"}],
                "stdout_contains": ["blocked"],
            },
            0,
            stdout,
        )

        self.assertTrue(all(check["passed"] for check in checks))

    def test_timeout_stream_text_decodes_bytes(self) -> None:
        self.assertEqual(run_behavior_eval.timeout_stream_text(b"partial\n"), "partial\n")
        self.assertEqual(run_behavior_eval.timeout_stream_text(None), "")

    def test_missing_required_coverage_reduces_score_and_fails(self) -> None:
        samples = [
            {
                "id": "covered",
                "rule": "L7",
                "hook": "pre-bash-guard",
                "profile": "default",
                "severity": "critical",
                "platform": "claude",
            }
        ]
        results = [
            {
                "id": "covered",
                "rule": "L7",
                "hook": "pre-bash-guard",
                "profile": "default",
                "severity": "critical",
                "platform": "claude",
                "passed": True,
            }
        ]
        requirements = [
            {"platform": "claude", "hook": "pre-bash-guard"},
            {"platform": "codex", "hook": "pre-bash-guard"},
        ]

        report = run_behavior_eval.build_report(
            samples,
            results,
            requirements,
            {
                "min_pass_rate": 100.0,
                "min_coverage_rate": 100.0,
                "slice_min_pass_rate": 100.0,
            },
            metadata={},
        )

        self.assertEqual(report["verdict"], "fail")
        self.assertEqual(report["coverage"]["coverage_rate"], 50.0)
        self.assertEqual(report["score"], 50.0)
        self.assertEqual(report["coverage"]["missing"], [{"platform": "codex", "hook": "pre-bash-guard"}])

    def test_behavior_summary_contains_required_observability_fields(self) -> None:
        report = {
            "metadata": {
                "commit": "abc123",
                "dataset_source": "/repo/eval/behavior/datasets/v1.jsonl",
                "sample_digest": "digest123",
                "sample_count": 2,
                "scorer_version": "behavior-e2e-v1",
            },
            "verdict": "fail",
            "pass_rate": 50.0,
            "total": 2,
            "coverage": {"coverage_rate": 75.0},
            "slice_failures": [{"dimension": "rule", "value": "L1"}],
            "failures": ["one failure"],
        }

        summary = run_behavior_eval.build_behavior_summary(
            report,
            Path("/tmp/eval/runs/20260101T000000Z-abc123/results.json"),
        )

        self.assertEqual(summary["kind"], "behavior")
        self.assertEqual(summary["score_type"], "deterministic")
        self.assertEqual(summary["commit"], "abc123")
        self.assertEqual(summary["dataset_digest"], "digest123")
        self.assertEqual(summary["pass_rate"], 50.0)
        self.assertEqual(summary["coverage_rate"], 75.0)
        self.assertEqual(summary["slice_failures"], [{"dimension": "rule", "value": "L1"}])


if __name__ == "__main__":
    unittest.main()
