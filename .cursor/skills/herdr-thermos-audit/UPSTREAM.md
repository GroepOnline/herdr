# Upstream / ownership

- **Operating SSOT (global Cursor install):** `OnlineChefGroep/herdr-ops` → `skills/herdr-thermos-audit` via `./install.sh`
- **This copy:** repo-local so Herdr product workspaces discover `/audit` without a separate ops clone
- **Personal live tree on Joep:** `~/.cursor/skills/herdr-thermos-audit/` (filled by herdr-ops install)

When editing the skill, prefer changing herdr-ops first, then sync into this path (or run `herdr-ops` install `--link`).
