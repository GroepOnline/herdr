# Cursor project control plane

Committed under `.cursor/`:

| Path | Role |
| --- | --- |
| `skills/` | Agent skills (`herdr`, `verify-herdr`, …) |
| `agents/` | Focused subagents |
| `rules/` | Always-on / requestable rules |
| `commands/` | Slash commands |
| `hooks/` | Shell + catalog hooks (`deny-rust-builds`, `fetch-cursor-artifacts`) |
| `scripts/cloud-install.sh` | Cloud env update: herdr binary + npm + Cursor standards |
| `environment.json` | Snapshot + install hook for Cloud Agents |
| `INDEX.md` | Auto-generated catalog (regenerate; do not hand-edit) |
| `mcp.json` | Project MCP servers |

## Refresh standards + binary

```bash
bash .cursor/scripts/cloud-install.sh
# or just the catalog:
.cursor/hooks/fetch-cursor-artifacts.sh --write-cache
python3 scripts/generate_cursor_index.py --allow-org-leak
```

## MCP: `stitch` (Google Stitch)

Requires the `chefgroep-stitch-mcp` launcher on your `PATH` (for example in
`~/.local/bin`). The launcher is the only place the Stitch API key lives; it is
not part of this repository, so ask a maintainer for the install steps and
credential.

If the launcher is not installed, the `stitch` server simply fails to start and
nothing else in the workspace is affected.
