#!/usr/bin/env bash
# Cursor Cloud start for herdr — join ChefGroep Tailscale fleet (same class as
# cursor-cloud-herdr / cursor-cloud-cursor already on the tailnet), wire MCP,
# prove Vault Serve + fleet peers. Idempotent.
#
# Secrets (Cloud Agents dashboard) — never commit values:
#   TS_AUTH_KEY_RESUABLE              reusable node key (spelling intentional)
#   CF_ACCESS_CLIENT_ID/SECRET        remote Kater SSE (optional)
#   KATER_SSE_URL                     default https://kater.chefgroep.online/sse
#   CHEF_VAULT_ADMIN_TOKEN            for remote vault MCP on joep (optional)
#   CHEF_VAULT_API_TOKEN              dashboard/API bearer via Serve (optional)
#
# After first green boot: save workspace snapshot in the Cloud Agents UI
# (environment.json has agentCanUpdateSnapshot=true).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CACHE="${HERDR_CLOUD_SKILLS_CACHE:-${HOME}/.cache/chefgroep/cloud-agent}"
TS_STATE="${HOME}/.local/state/chefgroep/tailscale"
TS_SOCK="${TS_STATE}/tailscaled.sock"
# Existing fleet hostname pattern (already seen on the tailnet).
TS_HOSTNAME="${HERDR_CLOUD_TS_HOSTNAME:-cursor-cloud-herdr}"
# Vault dashboard via Tailscale Serve on joep (tailnet-only HTTPS).
VAULT_SERVE_URL="${CHEF_VAULT_SERVE_URL:-https://joep.tail86a8f2.ts.net}"

mkdir -p \
  "${TS_STATE}" \
  "${HOME}/.cursor" \
  "${HOME}/.local/bin" \
  "${HOME}/.config/chefgroep"

log() { printf '+ %s\n' "$*"; }
warn() { printf 'WARN: %s\n' "$*" >&2; }

is_live_cursor_plane() {
  local origin=""
  [[ -d "${HOME}/.cursor/.git" ]] || return 1
  origin="$(git -C "${HOME}/.cursor" remote get-url origin 2>/dev/null || true)"
  [[ "${origin}" == *OnlineChefGroep/cursor* ]]
}

ensure_path() {
  case ":${PATH}:" in
    *":${HOME}/.local/bin:"*) ;;
    *) export PATH="${HOME}/.local/bin:${PATH}" ;;
  esac
}

ts() {
  if [[ -n "${TS_SOCKET:-}" && -S "${TS_SOCKET}" ]]; then
    sudo tailscale --socket="${TS_SOCKET}" "$@"
  else
    sudo tailscale "$@"
  fi
}

start_tailscale_fleet() {
  local key="${TS_AUTH_KEY_RESUABLE:-}"

  if [[ -z "${key}" ]]; then
    warn "TS_AUTH_KEY_RESUABLE unset — cannot join fleet (vault/kater/fleet MCP stay offline)"
    return 1
  fi
  if ! command -v tailscaled >/dev/null 2>&1; then
    warn "tailscaled missing from image"
    return 1
  fi

  if ! pgrep -x tailscaled >/dev/null 2>&1; then
    log "start tailscaled userspace (fleet node ${TS_HOSTNAME})"
    sudo tailscaled \
      --tun=userspace-networking \
      --socks5-server=localhost:1055 \
      --outbound-http-proxy-listen=localhost:1054 \
      --state="${TS_STATE}/tailscaled.state" \
      --socket="${TS_SOCK}" \
      >/tmp/herdr-tailscaled.log 2>&1 &
    local i
    for i in 1 2 3 4 5 6 7 8; do
      [[ -S "${TS_SOCK}" ]] && break
      sleep 1
    done
  else
    log "tailscaled already running"
  fi

  if [[ -S "${TS_SOCK}" ]]; then
    export TS_SOCKET="${TS_SOCK}"
  fi

  log "tailscale up → fleet hostname=${TS_HOSTNAME}"
  ts up \
    --authkey="${key}" \
    --hostname="${TS_HOSTNAME}" \
    --accept-dns=false \
    || {
      warn "tailscale up failed (see /tmp/herdr-tailscaled.log)"
      return 1
    }

  cat > "${HOME}/.config/chefgroep/cloud-tailscale-env.sh" <<EOF
# Source for Tailscale userspace egress (kernel cannot route 100.x directly)
export ALL_PROXY=socks5h://localhost:1055/
export HTTP_PROXY=http://localhost:1054/
export HTTPS_PROXY=http://localhost:1054/
export NO_PROXY=localhost,127.0.0.1,::1
export TS_SOCKET=${TS_SOCKET:-/var/run/tailscale/tailscaled.sock}
export CHEF_VAULT_SERVE_URL=${VAULT_SERVE_URL}
EOF

  # shellcheck disable=SC1090
  source "${HOME}/.config/chefgroep/cloud-tailscale-env.sh"

  log "tailscale status (fleet peers)"
  ts status 2>/dev/null | head -n 16 || true

  # Prove fleet membership the way existing cursor-cloud-* nodes do.
  local peer
  for peer in joep bc-scan-2 bc-scan-arm; do
    if ts ping --c 1 --timeout 3s "${peer}" >/dev/null 2>&1; then
      log "ping ok: ${peer}"
    else
      warn "ping fail: ${peer} (ACL/offline?)"
    fi
  done
}

write_vault_config() {
  # Dashboard/login URL is Tailscale Serve; MCP talks Vaultwarden on joep loopback via SSH.
  python3 - "${VAULT_SERVE_URL}" <<'PY'
import json, os, sys
from pathlib import Path
serve = sys.argv[1].rstrip("/")
path = Path.home() / ".config/chefgroep/vault.json"
cfg = {}
if path.exists():
    try:
        cfg = json.loads(path.read_text())
    except Exception:
        cfg = {}
cfg["serveUrl"] = serve
cfg["vaultUrl"] = os.environ.get("CHEF_VAULT_URL", "http://127.0.0.1:8222")
cfg["copyqContainer"] = os.environ.get("CHEF_VAULT_COPYQ_CONTAINER", "chefgroep-desktop")
cfg["desktopContainer"] = os.environ.get("CHEF_VAULT_DESKTOP_CONTAINER", "chefgroep-desktop")
# Never write tokens into vault.json from cloud env dumps; MCP gets them via env.
path.parent.mkdir(parents=True, exist_ok=True)
path.write_text(json.dumps(cfg, indent=2) + "\n")
print(f"wrote {path} serveUrl={serve}")
PY
}

probe_vault_serve() {
  # Userspace: use SOCKS5 for MagicDNS/HTTPS Serve.
  if ! command -v curl >/dev/null 2>&1; then
    return 0
  fi
  local code
  code="$(curl -sS -o /dev/null -w '%{http_code}' --max-time 8 \
    --socks5-hostname localhost:1055 \
    "${VAULT_SERVE_URL}/" 2>/dev/null || echo 000)"
  if [[ "${code}" == "200" ]]; then
    log "vault Serve OK ${VAULT_SERVE_URL}/ → ${code}"
  else
    warn "vault Serve ${VAULT_SERVE_URL}/ → HTTP ${code} (is joep online + serve up?)"
  fi
}

install_vault_mcp_launcher() {
  install -m 0755 \
    "${ROOT}/scripts/cursor-cloud-vault-mcp.sh" \
    "${HOME}/.local/bin/chefgroep-vault-mcp"
}

write_mcp_config() {
  local kater_url="${KATER_SSE_URL:-https://kater.chefgroep.online/sse}"
  local kater_profile="${KATER_PROFILE:-core}"
  local browser_mcp="${HOME}/.local/bin/chefgroep-agent-browser-mcp"
  local vault_mcp="${HOME}/.local/bin/chefgroep-vault-mcp"
  local mcp_path="${HOME}/.cursor/mcp.json"

  HERDR_CLOUD_BROWSER_MCP="${browser_mcp}" \
  HERDR_CLOUD_VAULT_MCP="${vault_mcp}" \
  KATER_SSE_URL="${kater_url}" \
  KATER_PROFILE="${kater_profile}" \
  CHEF_VAULT_SERVE_URL="${VAULT_SERVE_URL}" \
  python3 - "${mcp_path}" <<'PY'
import json, os, sys

path = sys.argv[1]
servers = {}

kater_url = os.environ.get("KATER_SSE_URL", "https://kater.chefgroep.online/sse")
kater = {
    "type": "sse",
    "url": kater_url,
    "env": {"KATER_PROFILE": os.environ.get("KATER_PROFILE", "core")},
}
cid = os.environ.get("CF_ACCESS_CLIENT_ID", "").strip()
csec = os.environ.get("CF_ACCESS_CLIENT_SECRET", "").strip()
if cid and csec:
    kater["headers"] = {
        "CF-Access-Client-Id": cid,
        "CF-Access-Client-Secret": csec,
    }
servers["kater"] = kater

browser = os.environ.get("HERDR_CLOUD_BROWSER_MCP", "").strip()
if browser and os.path.isfile(browser) and os.access(browser, os.X_OK):
    servers["chefgroep-browser"] = {"command": browser}

# Fleet stdio MCPs over Tailscale SSH (same pattern as laptop mcp.json).
servers["chefgroep-browser-fleet"] = {
    "command": "ssh",
    "args": [
        "-o", "BatchMode=yes",
        "-o", "ConnectTimeout=8",
        "ubuntu@bc-scan-2",
        "/var/lib/agent-browser-mcp/run",
    ],
}
servers["joep-brain"] = {
    "command": "ssh",
    "args": [
        "-o", "BatchMode=yes",
        "-o", "ConnectTimeout=8",
        "ubuntu@bc-scan-arm",
        "/var/lib/joep-brain/mcp/run",
    ],
}
servers["upcloud"] = {
    "command": "ssh",
    "args": [
        "-o", "BatchMode=yes",
        "-o", "ConnectTimeout=8",
        "ubuntu@bc-scan-2",
        "/var/lib/upcloud-mcp/run",
    ],
}

# Vault: remote MCP on joep (has Docker + TS Serve). Auth token via env if set.
vault_mcp = os.environ.get("HERDR_CLOUD_VAULT_MCP", "").strip()
if vault_mcp and os.path.isfile(vault_mcp) and os.access(vault_mcp, os.X_OK):
    env = {
        "CHEF_VAULT_SSH_TARGET": os.environ.get("CHEF_VAULT_SSH_TARGET", "joep"),
        "CHEF_VAULT_URL": "http://127.0.0.1:8222",
        "CHEF_VAULT_COPYQ_CONTAINER": "chefgroep-desktop",
        "CHEF_VAULT_DESKTOP_CONTAINER": "chefgroep-desktop",
    }
    # Pass through secrets when present (Cursor Secrets → process env).
    for key in ("CHEF_VAULT_ADMIN_TOKEN", "TS_SOCKET"):
        val = os.environ.get(key, "").strip()
        if val:
            env[key] = val
    servers["chefgroep-vault"] = {"command": vault_mcp, "env": env}

os.makedirs(os.path.dirname(path), exist_ok=True)
with open(path, "w", encoding="utf-8") as f:
    json.dump({"mcpServers": servers}, f, indent=2)
    f.write("\n")
print(f"wrote {path} servers={','.join(sorted(servers))}")
print(f"vault_dashboard={os.environ.get('CHEF_VAULT_SERVE_URL', '')}")
PY
}

ensure_desktop_hint() {
  cat > "${HOME}/.config/chefgroep/cloud-desktop.txt" <<EOF
Herdr Cursor Cloud — fleet node ${TS_HOSTNAME}
- Tailscale userspace: source ~/.config/chefgroep/cloud-tailscale-env.sh
- Vault dashboard (Serve): ${VAULT_SERVE_URL}/  (login / webtop flows via joep)
- Vault MCP: chefgroep-vault → Tailscale SSH to joep → local chefvault-mcp
- Fleet browser: chefgroep-browser-fleet → bc-scan-2
- After first green boot: Cloud Agents UI → save/update workspace snapshot
  (agentCanUpdateSnapshot=true in .cursor/environment.json)
- Do not wipe ~/.config/chefgroep/agent-browser-chrome
EOF
}

ensure_joep_chefvault_mcp_link() {
  # Best-effort: ensure laptop has ~/.local/bin/chefvault-mcp for the SSH launcher.
  local src="/home/joep/Documents/Github/OnlineChefGroep/chefgroep-vault/packages/mcp/dist/index.js"
  local dest="/home/joep/.local/bin/chefvault-mcp"
  if [[ -f "${src}" && ! -e "${dest}" ]]; then
    ln -sfn "${src}" "${dest}" 2>/dev/null || true
  fi
}

main() {
  log "herdr Cursor Cloud start (cwd=${ROOT})"
  ensure_path

  if is_live_cursor_plane && [[ "${HERDR_CLOUD_FORCE_START:-0}" != "1" ]]; then
    log "skip start mutations on live laptop ~/.cursor plane"
    ensure_joep_chefvault_mcp_link
    return 0
  fi

  install_vault_mcp_launcher
  write_vault_config

  if start_tailscale_fleet; then
    probe_vault_serve
  fi

  write_mcp_config
  ensure_desktop_hint

  log "tools: gh=$(command -v gh || echo missing) herdr=$(command -v herdr || echo missing) agent-browser=$(command -v agent-browser || echo missing)"
  log "fleet node=${TS_HOSTNAME} vault_serve=${VAULT_SERVE_URL}"
  log "start complete — save Cloud workspace snapshot when this looks green"
}

main "$@"
