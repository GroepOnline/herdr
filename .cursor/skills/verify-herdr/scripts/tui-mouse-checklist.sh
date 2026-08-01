#!/usr/bin/env bash
# Print (and optionally verify) the mouse-first TUI drive checklist.
# Agents / computerUse follow this; headless CI skips unless artifacts exist.
set -euo pipefail

ARTIFACT_DIR="${HERDR_MOUSE_ARTIFACT_DIR:-/opt/cursor/artifacts}"
SHOT_DIR="${ARTIFACT_DIR}/screenshots"
VERIFY_ONLY=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --verify) VERIFY_ONLY=1; shift ;;
    *) shift ;;
  esac
done

cat <<'EOF'
verify-herdr · mouse-first TUI checklist
=======================================
1. Isolated HOME + cleared sockets; dark terminal preferred
2. Open Settings via sidebar menu → settings (mouse); fallback Ctrl+b s
3. Click ≥2 settings sections (appearance, layout, plugins, agents, terminal, …)
4. Dismiss settings (close / Esc / outside)
5. Right-click pane → Split right + Split down (≥3 panes ideal)
6. Click panes to focus; run markers (echo / herdr --version / pwd)
7. Optional: open keybinds / another modal via menu
8. Save screenshots + short mp4 under /opt/cursor/artifacts/

CLI cousins (headless, no mouse):
  scripts/e2e.sh
  scripts/cli.sh config check
  scripts/cli.sh pane split …

Sandbox: see features/sandbox.md (HERDR_VERIFY_SANDBOX stub)
EOF

required=(
  "$ARTIFACT_DIR/herdr-mouse-settings-demo.mp4"
  "$SHOT_DIR/herdr-mouse-02-settings-open.webp"
  "$SHOT_DIR/herdr-mouse-08-commands.webp"
)

if [[ "$VERIFY_ONLY" -eq 1 ]]; then
  missing=0
  for f in "${required[@]}"; do
    if [[ -f "$f" ]]; then
      printf '  ✓ %s\n' "$f"
    else
      printf '  ✗ missing %s\n' "$f" >&2
      missing=1
    fi
  done
  if [[ "$missing" -eq 1 ]]; then
    echo "MOUSE PROOF FAIL: required artifacts missing" >&2
    exit 1
  fi
  echo "MOUSE PROOF PASS: required artifacts present"
  exit 0
fi

echo
echo "Tip: after a computerUse demo, re-run with --verify"
echo "  $0 --verify"
