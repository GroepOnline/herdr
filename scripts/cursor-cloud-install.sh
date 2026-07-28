#!/usr/bin/env bash
# Cursor Cloud install for herdr (.cursor/environment.json).
#
# Idempotent bootstrap after checkout. Does NOT compile Rust/Zig (CI-only).
# Syncs skills/control-plane, installs gh/herdr/browser tooling, prepares Vault
# MCP build when secrets will be present at start.
#
# Safe for public herdr: private dependency clones fail soft when the generated
# GitHub token lacks scope. Laptop live ~/.cursor plane is never overlaid.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CACHE="${HERDR_CLOUD_SKILLS_CACHE:-${HOME}/.cache/chefgroep/cloud-agent}"
BIN="${HOME}/.local/bin"
mkdir -p \
  "${CACHE}" \
  "${BIN}" \
  "${HOME}/.cursor/skills" \
  "${HOME}/.cursor/agents" \
  "${HOME}/.cursor/commands" \
  "${HOME}/.cursor/rules" \
  "${HOME}/.cursor/hooks" \
  "${HOME}/.agents/skills" \
  "${HOME}/.config/chefgroep/agent-browser-chrome" \
  "${HOME}/.local/share/chefgroep/agent-browser-state"

log() { printf '+ %s\n' "$*"; }
warn() { printf 'WARN: %s\n' "$*" >&2; }

ensure_path() {
  case ":${PATH}:" in
    *":${BIN}:"*) ;;
    *) export PATH="${BIN}:${PATH}" ;;
  esac
}

sync_repo() {
  local slug="$1"
  local name="${2:-${slug##*/}}"
  local dest="${CACHE}/${name}"

  if [[ -d "${dest}/.git" ]]; then
    log "update ${slug}"
    if git -C "${dest}" fetch --depth 1 origin HEAD \
      && git -C "${dest}" reset --hard FETCH_HEAD; then
      return 0
    fi
    warn "could not update ${slug}; keeping existing checkout"
    return 0
  fi

  log "clone ${slug}"
  mkdir -p "${CACHE}"
  if command -v gh >/dev/null 2>&1; then
    if gh repo clone "${slug}" "${dest}" -- --depth 1; then
      return 0
    fi
  fi
  if git clone --depth 1 "https://github.com/${slug}.git" "${dest}"; then
    return 0
  fi

  warn "could not clone ${slug} (token missing private scope?); skipping"
  rm -rf "${dest}"
  return 1
}

is_live_cursor_plane() {
  local origin=""
  [[ -d "${HOME}/.cursor/.git" ]] || return 1
  origin="$(git -C "${HOME}/.cursor" remote get-url origin 2>/dev/null || true)"
  [[ "${origin}" == *OnlineChefGroep/cursor* ]]
}

link_children() {
  local src="$1"
  local dest="$2"
  local item name

  [[ -d "${src}" ]] || return 0
  mkdir -p "${dest}"
  shopt -s nullglob
  for item in "${src}"/*; do
    [[ -e "${item}" || -L "${item}" ]] || continue
    name="$(basename "${item}")"
    if [[ -e "${dest}/${name}" && ! -L "${dest}/${name}" && -d "${dest}/${name}" ]]; then
      if [[ "${HERDR_CLOUD_REPLACE_SKILL_DIRS:-0}" != "1" ]]; then
        warn "keep existing dir ${dest}/${name}"
        continue
      fi
    fi
    rm -rf "${dest}/${name}"
    ln -sfn "${item}" "${dest}/${name}"
  done
  shopt -u nullglob
}

copy_indexes() {
  local src="$1"
  local dest="$2"
  local f
  mkdir -p "${dest}"
  for f in INDEX.md .index.yaml; do
    if [[ -f "${src}/${f}" ]]; then
      cp -a "${src}/${f}" "${dest}/${f}"
    fi
  done
}

install_cursor_plane() {
  local plane="${CACHE}/cursor"
  sync_repo "OnlineChefGroep/cursor" "cursor" || return 0

  log "link OnlineChefGroep/cursor → ~/.cursor + ~/.agents"
  for kind in skills agents commands rules; do
    if [[ -d "${plane}/${kind}" ]]; then
      link_children "${plane}/${kind}" "${HOME}/.cursor/${kind}"
      copy_indexes "${plane}/${kind}" "${HOME}/.cursor/${kind}"
    fi
  done
  if [[ -d "${plane}/skills" ]]; then
    link_children "${plane}/skills" "${HOME}/.agents/skills"
    copy_indexes "${plane}/skills" "${HOME}/.agents/skills"
  fi
  [[ -f "${plane}/AGENTS.md" ]] && ln -sfn "${plane}/AGENTS.md" "${HOME}/.cursor/AGENTS.md"
  [[ -f "${plane}/INDEX.md" ]] && ln -sfn "${plane}/INDEX.md" "${HOME}/.cursor/INDEX.md"
}

install_herdr_ops() {
  local ops="${CACHE}/herdr-ops"
  sync_repo "OnlineChefGroep/herdr-ops" "herdr-ops" || return 0
  if [[ -f "${ops}/install.sh" ]]; then
    log "herdr-ops install.sh --link"
    bash "${ops}/install.sh" --link
  else
    link_children "${ops}/skills" "${HOME}/.cursor/skills"
    link_children "${ops}/skills" "${HOME}/.agents/skills"
  fi
}

install_chefgroep_skills() {
  local bundle="${CACHE}/ChefGroep-Skills"
  sync_repo "OnlineChefGroep/ChefGroep-Skills" "ChefGroep-Skills" || return 0
  if [[ -d "${bundle}/skills" ]]; then
    link_children "${bundle}/skills" "${HOME}/.cursor/skills"
    link_children "${bundle}/skills" "${HOME}/.agents/skills"
  fi
  if [[ -d "${bundle}/agents" ]]; then
    link_children "${bundle}/agents" "${HOME}/.cursor/agents"
  fi
}

mirror_workspace_skills() {
  local src="${ROOT}/.cursor/skills"
  [[ -d "${src}" ]] || return 0
  log "mirror workspace .cursor/skills → ~/.cursor + ~/.agents"
  link_children "${src}" "${HOME}/.cursor/skills"
  link_children "${src}" "${HOME}/.agents/skills"
}

install_browser_tools() {
  log "install Lightpanda + agent-browser"
  if [[ ! -x "${BIN}/lightpanda" ]]; then
    curl -fsSL -o "${BIN}/lightpanda" \
      "https://github.com/lightpanda-io/browser/releases/download/nightly/lightpanda-x86_64-linux" \
      || warn "lightpanda download failed"
    chmod +x "${BIN}/lightpanda" 2>/dev/null || true
  fi

  if ! command -v agent-browser >/dev/null 2>&1; then
    npm install -g agent-browser@0.33.0 \
      || npm install -g agent-browser \
      || warn "npm install agent-browser failed"
  fi

  install -m 0755 \
    "${ROOT}/scripts/cursor-cloud-browser-mcp.sh" \
    "${BIN}/chefgroep-agent-browser-mcp"
}

install_herdr_cli() {
  if command -v herdr >/dev/null 2>&1; then
    log "herdr already on PATH: $(command -v herdr)"
    return 0
  fi
  log "install herdr (dev channel binary — no local cargo)"
  if curl -fsSL https://herdr.chefgroep.nl/install.sh | sh -s -- --channel dev; then
    log "herdr install.sh ok"
  else
    warn "herdr install.sh failed; agent can still download CI smoke binary via gh"
  fi
}

install_vault_mcp() {
  local vault="${CACHE}/chefgroep-vault"
  sync_repo "OnlineChefGroep/chefgroep-vault" "chefgroep-vault" || return 0

  if [[ -f "${vault}/packages/mcp/dist/index.js" ]]; then
    log "vault MCP dist already built"
    return 0
  fi

  if [[ ! -f "${vault}/packages/mcp/package.json" ]]; then
    warn "chefgroep-vault MCP package missing"
    return 0
  fi

  log "build chefgroep-vault MCP (npm)"
  (
    cd "${vault}/packages/mcp"
    if [[ -f package-lock.json ]]; then
      npm ci --ignore-scripts 2>/dev/null || npm install --ignore-scripts
    else
      npm install --ignore-scripts
    fi
    npm run build 2>/dev/null || npx tsc -p tsconfig.json 2>/dev/null || true
  ) || warn "vault MCP build failed; vault MCP skipped until CHEF_VAULT_* secrets + rebuild"
}

write_cloud_readme() {
  cat > "${HOME}/.config/chefgroep/cloud-agent-bootstrap.txt" <<'EOF'
Herdr Cursor Cloud bootstrap
- Skills: ~/.cursor/skills + ~/.agents/skills (from OnlineChefGroep/cursor, ChefGroep-Skills, herdr-ops)
- Browser: Lightpanda via chefgroep-agent-browser-mcp (profile ~/.config/chefgroep/agent-browser-chrome)
- Desktop login: Cursor computer-use / noVNC; do not wipe the browser profile
- Fleet MCP: needs TS_AUTH_KEY_RESUABLE + SSH keys for bc-scan-2 / bc-scan-arm
- Kater: KATER_SSE_URL + CF_ACCESS_CLIENT_ID/SECRET in Cursor Secrets
- Vault: CHEF_VAULT_URL + CHEF_VAULT_ADMIN_TOKEN (Tailscale-reachable Vaultwarden)
- Validate Rust via gh pr checks — never cargo/just check on this VM
EOF
}

count_skills() {
  local dir="$1"
  local n=0
  if [[ -d "${dir}" ]]; then
    n="$(find "${dir}" -mindepth 1 -maxdepth 1 \( -type d -o -type l \) 2>/dev/null | wc -l | tr -d ' ')"
  fi
  printf '%s' "${n}"
}

main() {
  log "herdr Cursor Cloud install (cwd=${ROOT})"
  log "skills cache: ${CACHE}"
  ensure_path

  # Always refresh durable CLIs (safe on laptop too).
  install_browser_tools
  install_herdr_cli

  if is_live_cursor_plane && [[ "${HERDR_CLOUD_FORCE_SKILL_SYNC:-0}" != "1" ]]; then
    log "skip skill overlay: ~/.cursor is the live OnlineChefGroep/cursor checkout"
    sync_repo "OnlineChefGroep/cursor" "cursor" || true
    sync_repo "OnlineChefGroep/herdr-ops" "herdr-ops" || true
    sync_repo "OnlineChefGroep/ChefGroep-Skills" "ChefGroep-Skills" || true
    sync_repo "OnlineChefGroep/chefgroep-vault" "chefgroep-vault" || true
    write_cloud_readme
    log "cache refreshed under ${CACHE}"
    return 0
  fi

  export HERDR_CLOUD_REPLACE_SKILL_DIRS="${HERDR_CLOUD_REPLACE_SKILL_DIRS:-1}"
  install_cursor_plane
  install_chefgroep_skills
  install_herdr_ops
  mirror_workspace_skills
  install_vault_mcp
  write_cloud_readme

  log "done: ~/.cursor/skills=$(count_skills "${HOME}/.cursor/skills") ~/.agents/skills=$(count_skills "${HOME}/.agents/skills")"
  log "bins: gh=$(command -v gh || echo missing) herdr=$(command -v herdr || echo missing) lightpanda=$(command -v lightpanda || echo missing) agent-browser=$(command -v agent-browser || echo missing)"
}

main "$@"
