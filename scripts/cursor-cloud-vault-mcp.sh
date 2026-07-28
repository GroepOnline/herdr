#!/usr/bin/env bash
# Cloud-side launcher: run chefgroep-vault MCP on joep over Tailscale SSH.
# Vault Docker + Vaultwarden stay on joep; secrets stay in joep's vault.json.
#
# Requires Cloud Agent on the ChefGroep tailnet (start script) and Tailscale SSH
# to CHEF_VAULT_SSH_TARGET (default: joep).
set -euo pipefail

TARGET="${CHEF_VAULT_SSH_TARGET:-joep}"
REMOTE_MCP="${CHEF_VAULT_REMOTE_MCP:-/home/joep/.local/bin/chefvault-mcp}"
REMOTE_FALLBACK="${CHEF_VAULT_REMOTE_MCP_FALLBACK:-/home/joep/Documents/Github/OnlineChefGroep/chefgroep-vault/packages/mcp/dist/index.js}"

remote_cmd=$(
  cat <<EOF
set -euo pipefail
export CHEF_VAULT_URL="\${CHEF_VAULT_URL:-http://127.0.0.1:8222}"
export CHEF_VAULT_COPYQ_CONTAINER="\${CHEF_VAULT_COPYQ_CONTAINER:-chefgroep-desktop}"
export CHEF_VAULT_DESKTOP_CONTAINER="\${CHEF_VAULT_DESKTOP_CONTAINER:-chefgroep-desktop}"
export CHEF_VAULT_REPO_ROOT="\${CHEF_VAULT_REPO_ROOT:-/home/joep/Documents/Github/OnlineChefGroep/chefgroep-vault}"
# Admin token comes from joep ~/.config/chefgroep/vault.json via loadConfig — do not pass on argv.
if [ -x '${REMOTE_MCP}' ]; then
  exec '${REMOTE_MCP}'
elif [ -f '${REMOTE_FALLBACK}' ]; then
  exec node '${REMOTE_FALLBACK}'
fi
echo "chefvault-mcp missing on ${TARGET}" >&2
exit 1
EOF
)

sock_args=()
if [[ -n "${TS_SOCKET:-}" && -S "${TS_SOCKET}" ]]; then
  sock_args=(--socket="${TS_SOCKET}")
fi

if command -v tailscale >/dev/null 2>&1 && sudo tailscale "${sock_args[@]}" status >/dev/null 2>&1; then
  exec sudo tailscale "${sock_args[@]}" ssh "${TARGET}" -- bash -lc "${remote_cmd}"
fi

exec ssh \
  -o BatchMode=yes \
  -o ConnectTimeout=8 \
  "${TARGET}" \
  "bash -lc $(printf '%q' "${remote_cmd}")"
