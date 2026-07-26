#!/usr/bin/env bash
# Restore canvas markdown onto EXISTING Herdr panes (look-first).
# Usage: restore-canvas.sh --audit-dir /tmp/<repo>-audit-<date>/canvas
# Discovers labeled panes where-live|map|findings|connections in focused workspace.
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

mapfile -t ROWS < <(herdr pane list | python3 -c '
import json,sys
d=json.load(sys.stdin)["result"]["panes"]
# prefer focused workspace
focused=None
for p in d:
    if p.get("focused"):
        focused=p["workspace_id"]; break
if not focused and d:
    focused=d[0]["workspace_id"]
want={"where-live":None,"map":None,"findings":None,"connections":None}
for p in d:
    if p.get("workspace_id")!=focused: continue
    lab=(p.get("label") or "").strip().lower()
    if lab in want and want[lab] is None:
        want[lab]=p["pane_id"]
for k,v in want.items():
    print("%s\t%s" % (k, v or ""))
')

declare -A PANE
for row in "${ROWS[@]}"; do
  key="${row%%$'\t'*}"
  val="${row#*$'\t'}"
  PANE["$key"]="$val"
done

render() {
  local pane="$1" file="$2"
  [[ -n "$pane" && -f "$file" ]] || return 0
  herdr pane run "$pane" /usr/bin/less -R "$file"
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

echo "restored onto panes: where-live=${PANE[where-live]:-none} map=${PANE[map]:-none} findings=${PANE[findings]:-none} connections=${PANE[connections]:-none}"
