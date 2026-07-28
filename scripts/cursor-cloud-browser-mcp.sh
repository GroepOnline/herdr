#!/usr/bin/env bash
# ChefGroep browser MCP launcher for Cursor Cloud (Lightpanda, not Chrome).
# Installed to ~/.local/bin/chefgroep-agent-browser-mcp by cursor-cloud-install.sh.
set -euo pipefail

PROFILE="${CHEFGROEP_BROWSER_PROFILE:-${HOME}/.config/chefgroep/agent-browser-chrome}"
STATE="${CHEFGROEP_BROWSER_STATE:-${HOME}/.local/share/chefgroep/agent-browser-state}"
SESSION="${CHEFGROEP_BROWSER_SESSION:-chefgroep-default}"
ENGINE="${CHEFGROEP_BROWSER_ENGINE:-lightpanda}"

export AGENT_BROWSER_SESSION="${SESSION}"
export AGENT_BROWSER_PROFILE_DIR="${PROFILE}"
export AGENT_BROWSER_PROFILE="${PROFILE}"
export AGENT_BROWSER_STATE_DIR="${STATE}"
export AGENT_BROWSER_ENGINE="${ENGINE}"
export AGENT_BROWSER_HEADLESS="${AGENT_BROWSER_HEADLESS:-1}"

if [[ "${ENGINE}" == "lightpanda" ]] && ! command -v lightpanda >/dev/null 2>&1; then
  echo "missing lightpanda on PATH" >&2
  exit 1
fi
if ! command -v agent-browser >/dev/null 2>&1; then
  echo "missing agent-browser on PATH" >&2
  exit 1
fi

mkdir -p "${PROFILE}" "${STATE}"
exec agent-browser \
  --session "${SESSION}" \
  --profile "${PROFILE}" \
  --engine "${ENGINE}" \
  --restore \
  --restore-save auto \
  mcp --tools core,state,tabs
