# Cursor MCP servers

`mcp.json` configures project-scoped MCP servers. Commands are resolved through
`PATH` so the config stays machine-independent; the Stitch API key never enters
git.

## `stitch` (Google Stitch)

Requires the `chefgroep-stitch-mcp` launcher on your `PATH` (for example in
`~/.local/bin`). The launcher is the only place the Stitch API key lives; it is
not part of this repository, so ask a maintainer for the install steps and
credential.

If the launcher is not installed, the `stitch` server simply fails to start and
nothing else in the workspace is affected.
