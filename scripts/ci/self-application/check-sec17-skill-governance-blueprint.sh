#!/usr/bin/env bash
# SEC-17 self-application: keep the skill governance blueprint concrete.
set -euo pipefail

REPO_DIR="${1:-$(cd "$(dirname "$0")/../../.." && pwd)}"
SECURITY_RULES="${REPO_DIR}/rules/claude-rules/common/security.md"

python3 - <<'PY' "${SECURITY_RULES}"
import re
import sys
from pathlib import Path

path = Path(sys.argv[1])
errors: list[str] = []

if not path.exists():
    errors.append("missing rules/claude-rules/common/security.md")
else:
    text = path.read_text(encoding="utf-8")
    match = re.search(r"^## SEC-17:.*?(?=^## SEC-18:)", text, re.M | re.S)
    if match is None:
        errors.append("missing SEC-17 block")
        block = ""
    else:
        block = match.group(0)

    required_terms = {
        "SkillGuard source": "arXiv 2606.03024",
        "PCAA source": "arXiv 2606.04104",
        "manifest": "manifest",
        "default-deny": "default-deny",
        "runtime access control": "runtime access control",
        "context influence plane": "context influence",
        "action side effects plane": "action side effects",
        "capability inference": "inferred capabilities",
        "multi-skill composition": "sensitive chains",
        "action certificate": "action certificate",
        "boundary facts": "boundary facts",
        "runtime receipt": "runtime receipt",
    }
    lower_block = block.lower()
    for label, needle in required_terms.items():
        if needle.lower() not in lower_block:
            errors.append(f"SEC-17 missing {label}: {needle}")

if errors:
    print("FAIL: SEC-17 skill governance blueprint check failed")
    for error in errors:
        print(error)
    raise SystemExit(1)

print("OK: SEC-17 keeps manifest, default-deny, dual-plane, and action-certificate guidance")
PY
