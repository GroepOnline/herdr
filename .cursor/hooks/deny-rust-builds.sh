#!/usr/bin/env bash
# Deny local Rust/Zig builds for herdr (CPU saturation).
# Validate via GitHub Actions instead: gh pr checks / gh run view --log-failed
set -euo pipefail

input=$(cat)

DENY_HOOK_INPUT="$input" python3 - <<'PY'
import json
import os
import re
import sys

deny_message = (
    "Local Rust/Zig builds are forbidden on this machine "
    "(cargo/rustc/zig saturate CPU). Do not run cargo, rustc, rustup, "
    "cargo-nextest, clippy, zig build, or just test|check|lint|ci|build|"
    "fmt-check|windows-lint. "
    "Validate with GitHub Actions: `gh pr checks` and "
    "`gh run view <id> --log-failed`."
)

raw = os.environ.get("DENY_HOOK_INPUT", "")
try:
    data = json.loads(raw) if raw.strip() else {}
except json.JSONDecodeError:
    # failClosed: invalid input should not allow a build through
    print(json.dumps({
        "permission": "deny",
        "user_message": deny_message,
        "agent_message": "deny-rust-builds hook received invalid JSON on stdin.",
    }))
    raise SystemExit(0)

command = data.get("command") or ""

# Explicit allowlist for CI inspection / non-build tooling
allow_patterns = [
    r"(?:^|[\s;&|`($])gh\s+pr\s+checks(?:\s|$|[;&|)`])",
    r"(?:^|[\s;&|`($])gh\s+run\s+view(?:\s|$|[;&|)`])",
    r"(?:^|[\s;&|`($])gh\s+workflow\s+view(?:\s|$|[;&|)`])",
]

if any(re.search(p, command) for p in allow_patterns):
    print(json.dumps({"permission": "allow"}))
    raise SystemExit(0)

patterns = [
    # cargo / rustc / rustup / cargo-nextest / clippy / rustfmt as a command token
    r"(?:^|[\s;&|`($])(?:sudo\s+)?(?:cargo|rustc|rustup|cargo-nextest|clippy|rustfmt)(?:\s|$|[;&|)`])",
    # zig build (not all zig invocations)
    r"(?:^|[\s;&|`($])(?:sudo\s+)?zig\s+build(?:\s|$|[;&|)`])",
    # just recipes that compile/test/lint Rust
    r"(?:^|[\s;&|`($])(?:sudo\s+)?just\s+(?:test|check|lint|ci|build|fmt-check|windows-lint|build-libghostty-vt)(?:\s|$|[;&|)`])",
]

if any(re.search(p, command) for p in patterns):
    print(json.dumps({
        "permission": "deny",
        "user_message": deny_message,
        "agent_message": deny_message,
    }))
else:
    print(json.dumps({"permission": "allow"}))
PY
