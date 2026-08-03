#!/usr/bin/env bash
# Restore canvas markdown onto EXISTING Herdr panes (look-first).
# Usage: restore-canvas.sh --audit-dir /tmp/<repo>-audit-<date>/canvas
# Discovers where-live|map|findings|connections via pane label, then tab label.
set -euo pipefail

AUDIT=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --audit-dir) AUDIT="${2:-}"; shift 2 ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

[[ -n "${AUDIT}" && -d "${AUDIT}" ]] || { echo "need --audit-dir existing canvas dir" >&2; exit 2; }
[[ "${HERDR_ENV:-}" == "1" ]] || { echo "HERDR_ENV!=1" >&2; exit 1; }

mapfile -t ROWS < <(python3 -c '
import json, subprocess, sys

def load(args):
    out = subprocess.check_output(["herdr", *args], text=True)
    return json.loads(out)["result"]

panes = load(["pane", "list"])["panes"]
tabs = load(["tab", "list"])["tabs"]
tab_by_id = {t["tab_id"]: t for t in tabs}

focused = None
for p in panes:
    if p.get("focused"):
        focused = p["workspace_id"]
        break
if not focused:
    for t in tabs:
        if t.get("focused"):
            focused = t["workspace_id"]
            break
if not focused and panes:
    focused = panes[0]["workspace_id"]

want = {"where-live": None, "map": None, "findings": None, "connections": None}
for p in panes:
    if p.get("workspace_id") != focused:
        continue
    pane_lab = (p.get("label") or "").strip().lower()
    tab = tab_by_id.get(p.get("tab_id") or "", {})
    tab_lab = (tab.get("label") or "").strip().lower()
    # Pane rename wins (where-live); tab create --label covers map/findings/connections.
    lab = pane_lab or tab_lab
    if lab in want and want[lab] is None:
        want[lab] = p["pane_id"]

for key, value in want.items():
    print("%s\t%s" % (key, value or ""))
')

declare -A PANE
for row in "${ROWS[@]}"; do
  key="${row%%$'\t'*}"
  val="${row#*$'\t'}"
  PANE["$key"]="$val"
done

# Canvas contract: record the live ids after every restore, before rendering,
# so a /tmp wipe can be recovered from WHERE-LIVE.md.
{
  echo "# Where live"
  echo
  echo "Restored $(date -u '+%Y-%m-%dT%H:%M:%SZ') by restore-canvas.sh."
  echo
  echo "| Label | Pane |"
  echo "| --- | --- |"
  for key in where-live map findings connections; do
    echo "| ${key} | ${PANE[$key]:-none} |"
  done
} >"$AUDIT/WHERE-LIVE.md"

RESTORED=0

render() {
  local pane="$1" file="$2"
  [[ -n "$pane" && -f "$file" ]] || return 0
  herdr pane run "$pane" /usr/bin/less -R "$file"
  RESTORED=$((RESTORED + 1))
}

render "${PANE[where-live]:-}" "$AUDIT/WHERE-LIVE.md"
render "${PANE[map]:-}" "$AUDIT/00-map.md"
# Prefer FINAL on findings if present
if [[ -f "$AUDIT/03-FINAL.md" ]]; then
  render "${PANE[findings]:-}" "$AUDIT/03-FINAL.md"
else
  render "${PANE[findings]:-}" "$AUDIT/01-findings.md"
fi
render "${PANE[connections]:-}" "$AUDIT/02-connections.md"

if [[ "${RESTORED}" -eq 0 ]]; then
  echo "no labeled canvas pane restored — bootstrap the canvas first (see references/herdr-canvas.md)" >&2
  exit 1
fi

echo "restored onto panes: where-live=${PANE[where-live]:-none} map=${PANE[map]:-none} findings=${PANE[findings]:-none} connections=${PANE[connections]:-none}"
