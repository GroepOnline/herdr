# Output Panel

`com.chefgroep.output-panel` — a universal output/log browser inside Herdr.

It answers "what is this machine printing right now" without leaving the
session: Herdr panes, Herdr's own server and client logs, plugin invocations
and plugin state, Cursor MCP server logs, live network probes, systemd
journals, agent session transcripts, and anything you add yourself.

```bash
herdr plugin link /home/joep/Documents/herdr/plugins/output-panel
herdr plugin pane open --plugin com.chefgroep.output-panel --entrypoint output \
  --placement overlay --focus
```

## Architecture

One module per source, registered in `src/providers/index.js`. A provider is
`{ id, group, list(ctx), read(channel, ctx, opts) }`; everything else — sidebar
grouping, filtering, follow mode, search, export, the headless CLI — is derived
from that contract. Adding a source is one file plus one line in the registry.

| File | Purpose |
|---|---|
| `herdr-plugin.toml` | Manifest: 4 actions + the `output` pane entrypoint |
| `channels.example.json` | Copy-paste template for user-defined channels |
| `src/index.js` | Headless CLI: `list-channels`, `groups`, `read-channel`, `open-pane` |
| `src/tui.js` | Interactive panel: terminal setup, key dispatch, side effects, headless JSON fallback |
| `src/state.js` | Panel state and pure transforms (rows, selection, viewport, matches) |
| `src/render.js` | Builds one frame string per repaint (no clear-and-redraw) |
| `src/keys.js` | Keymap tables; drives dispatch, footer hints and the help overlay |
| `src/format.js` | ANSI-aware clipping, log-level colour, match highlighting |
| `src/lib/channel.js` | `channel()` / `result()` / `failure()` constructors and group order |
| `src/lib/exec.js` | Async spawn with a hard timeout; never rejects |
| `src/lib/files.js` | Backward tail reads, bounded recursive scans, size/age formatting |
| `src/lib/herdr.js` | `herdr` CLI wrappers and newline-JSON socket RPC |
| `src/lib/paths.js` | Path resolution and the provider context |
| `src/lib/probe.js` | TCP/HTTP probes and the known-local-service table |
| `src/lib/redact.js` | Secret redaction (see below) |
| `src/providers/panes.js` | Live Herdr panes |
| `src/providers/herdr-logs.js` | Herdr server/client log, config, status |
| `src/providers/plugin-logs.js` | Plugin invocation logs, all and per plugin |
| `src/providers/fleet-ops.js` | `fleet_ops.json` and sibling plugin state files |
| `src/providers/mcp.js` | Cursor MCP logs, `mcp.json`, wrapper logs, gateway health |
| `src/providers/network.js` | Sockets, connections, interfaces, probes, Tailscale |
| `src/providers/system.js` | systemd user units and journals |
| `src/providers/agents.js` | pi and Cursor agent session transcripts |
| `src/providers/custom.js` | User-defined channels from `channels.json` |

## Channel catalogue

Channel ids are namespaced and stable; the panel selects by id, so refreshing
never moves your selection.

### Panes

| Channel | Source |
|---|---|
| `pane:<pane_id>` | `herdr pane read <pane_id> --source recent\|visible\|recent-unwrapped\|detection --lines N --format text\|ansi` |

`a` cycles the snapshot source and `A` toggles ANSI passthrough while a pane
channel is open.

### Herdr

| Channel | Source |
|---|---|
| `herdr:server-log` | `~/.config/herdr/herdr-server.log` |
| `herdr:client-log` | `~/.config/herdr/herdr-client.log` |
| `herdr:config` | `~/.config/herdr/config.toml` |
| `herdr:status` | `herdr status` |
| `herdr:status-server` | `herdr status server` |

### Plugins

| Channel | Source |
|---|---|
| `plugin:logs:all` | `herdr plugin log list --limit N` |
| `plugin:logs:<plugin_id>` | `herdr plugin log list --plugin <id> --limit N` |
| `state:<plugin_id>:<file>.json` | `~/.local/state/herdr/plugins/<plugin_id>/<file>.json` |

Per-invocation `stdout` and `stderr` are expanded, prefixed `out|` and `err|`.

### MCP

| Channel | Source |
|---|---|
| `mcp:config` | `~/.cursor/mcp.json`, structure preserved, values masked |
| `mcp:sessions` | index of `~/.config/Cursor/logs/<stamp>/` directories |
| `mcp:server:<name>` | `~/.config/Cursor/logs/<newest>/mcp-server-<name>.log`, prefixed with the matching `mcp.json` entry |
| `mcp:process` | `~/.config/Cursor/logs/<newest>/mcpprocess.log` |
| `mcp:exthost:<window>` | `<newest>/window<N>/exthost/anysphere.cursor-mcp/MCP Logs.*.log` |
| `mcp:wrapper:<name>` | stdio wrapper logs in `/tmp` matching `*mcp*.log` / `*cdp*.log` (e.g. `/tmp/brave-origin-beta-cdp.log`) |
| `mcp:gateways` | probes `http://127.0.0.1:9090/`, `:9091/health`, `:9222/json/version` |

Configured servers with no log file yet still get a channel showing their
(redacted) config plus the exact path that was checked.

### Network

| Channel | Source |
|---|---|
| `net:listening` | `ss -ltnp` |
| `net:established` | `ss -tnp state established` |
| `net:udp` | `ss -lunp` |
| `net:interfaces` | `ip -brief address` |
| `net:probes` | TCP + read-only HTTP GET against kater `:9090`/`:9091`, brave CDP `:9222`, opencodex `:10100`, joep-ops `:10101`, vault `:8080`, vault-api `:8321`, vaultwarden `:8222`, webtop `:3000` |
| `net:tailscale` | `tailscale status`, falling back to `sudo -n tailscale status`; if sudo would prompt it reports "unavailable (needs sudo)" instead of hanging |

### System

| Channel | Source |
|---|---|
| `sys:units` | `systemctl --user list-units --type=service,timer --all` |
| `sys:failed` | `systemctl --user --failed` |
| `sys:journal` | `journalctl --user -n N --no-pager --output short-iso` |
| `sys:journal-errors` | same with `-p err` |
| `sys:unit:<unit>` | `journalctl --user -u <unit> -n N`, for pipewire, wireplumber and any `cursor*`/`kater*`/`ocx*`/`herdr*`/`codex*`/`chef*`/`joep-ops*`/`moshi*`/`cg-share*` user unit |

### Agents

| Channel | Source |
|---|---|
| `agent:pi:<session>` | `~/.pi/agent/sessions/**/<session>.jsonl` |
| `agent:cursor:<uuid>` | `~/.cursor/projects/*/agent-transcripts/**/<uuid>.jsonl` |

The 12 newest sessions per source are listed. Raw JSONL is decoded into one
line per record: time, record type, role, `stopReason`, `errorMessage` and a
text preview.

### Custom

Anything in `channels.json` (below).

## `channels.json`

Optional user config at
`$HERDR_PLUGIN_CONFIG_DIR/channels.json`, by default
`~/.config/herdr/plugins/config/com.chefgroep.output-panel/channels.json`.
`channels.example.json` in this directory is a ready template.

```json
{
  "channels": [
    {
      "id": "syslog",
      "title": "system log",
      "kind": "file",
      "path": "/var/log/syslog",
      "group": "Custom",
      "lines": 300
    },
    {
      "id": "docker-ps",
      "title": "docker containers",
      "kind": "command",
      "argv": ["docker", "ps", "--all"],
      "group": "Custom",
      "timeout_ms": 5000
    }
  ]
}
```

| Field | Meaning |
|---|---|
| `id` | appended to `custom:` to form the channel id |
| `title` | sidebar label |
| `kind` | `file` or `command` |
| `path` | file channels: absolute path, tailed |
| `argv` | command channels: argv array, no shell |
| `group` | sidebar group, default `Custom` |
| `lines` | file channels: tail length |
| `timeout_ms` | command channels: hard timeout, default 3000 |

A bare JSON array is accepted instead of `{ "channels": [...] }`. Invalid JSON
becomes a channel that shows the parse error rather than disappearing.

## Keys

### Channel list

| Key | Action |
|---|---|
| `j` `k` `↑` `↓` | move selection |
| `pgup` `pgdn` `ctrl+u` `ctrl+d` | page |
| `g` `G` `home` `end` | first / last row |
| `enter` `l` `→` | read the selected channel |
| `space` | collapse / expand the group |
| `[` `]` | collapse all / expand all |
| `/` | filter channels (id, title, subtitle, group, source) |
| `esc` | clear filter |
| `t` | open the channel in follow mode |
| `f` | focus that pane in Herdr (`pane.focus` over the socket) |
| `o` | open the channel's cwd with `xdg-open` |
| `e` | open the channel's cwd in `$VISUAL`/`$EDITOR` |
| `s` | open the agent session file in `$PAGER` |
| `R` `r` | rebuild the channel list |
| `?` | help overlay |
| `q` `ctrl+c` | quit |

### Output view

| Key | Action |
|---|---|
| `j` `k` `↑` `↓` | scroll a line |
| `pgup` `pgdn` `ctrl+u` `ctrl+d` | scroll a page |
| `g` `G` `home` `end` | top / bottom |
| `r` | re-read the channel |
| `t` | toggle follow (2s poll, FOLLOW shown in the header) |
| `/` | search within the buffer (incremental, highlighted) |
| `n` `N` | next / previous match |
| `esc` | clear the search, or go back when no search is active |
| `a` | cycle pane snapshot source |
| `A` | toggle ANSI passthrough for pane channels |
| `w` | write the buffer to a temp file |
| `E` | export and open in `$VISUAL`/`$EDITOR` |
| `P` | export and open in `$PAGER` |
| `y` | copy the export path (`wl-copy`, `xclip`, `xsel`) |
| `b` `h` `←` | back to the channel list |
| `?` | help overlay |
| `q` `ctrl+c` | quit |

Exports go to `$TMPDIR/herdr-output-panel-<channel>-<ts>.log` with mode `0600`,
ANSI stripped and a provenance header.

## Headless use

Every channel is scriptable with the same registry the UI uses.

```bash
node src/index.js list-channels                 # all channels as JSON
node src/index.js list-channels --group MCP     # one category
node src/index.js groups                        # ids grouped by category
node src/index.js read-channel herdr:server-log --lines 200
node src/index.js read-channel pane:wV:p1 --source detection --format ansi
node src/index.js read-channel mcp:config --json
node src/index.js open-pane --placement split --direction down
```

Bare pane ids still work: `read-channel wV:p1` resolves to `pane:wV:p1`.

`node src/tui.js` with a non-TTY stdin dumps the channel catalogue as JSON and
exits, which is what smoke tests use.

## Safety rules

- **Secrets are masked before display.** Redaction runs on every byte a
  provider emits — file tails, command output, JSON, exports, headless stdout.
  Rules: keys matching `password`/`secret`/`token`/`api_key`/`access_key`/
  `authorization`/`auth_token`/`credential`/`bearer`/`private_key`/
  `client_secret`/`cookie`/`salt`/`signature`, whole subtrees under `env`,
  `headers`, `secrets` and `credentials`, credentials in URLs
  (`scheme://user:pass@host`) and query strings, `Bearer`/`Basic` header
  values, and literal token shapes (GitHub, GitLab, `sk-`, Slack, Linear,
  Notion, Resend, Tailscale, AWS, Google, JWT).
- **Read-only.** The panel never writes to a source, never restarts a service,
  and never touches browser profiles. The Brave probe only fetches
  `/json/version`.
- **Never blocks.** Everything that touches the network, a CLI or
  `journalctl` goes through an async spawn with a hard timeout (3s default, 6s
  for `journalctl`, 1.5s per probe).
- **Bounded memory.** The output buffer keeps the last 5000 lines; file tails
  read backwards in 64 KB chunks and stop at 4 MB.
- **Terminal is always restored.** Alternate screen, cursor and raw mode are
  reset on quit, `SIGINT`, `SIGTERM` and `SIGHUP`, and the follow interval is
  cleared on exit.

## Environment

| Variable | Effect |
|---|---|
| `HERDR_BIN_PATH` | path to the `herdr` binary (default `herdr`) |
| `HERDR_SOCKET_PATH` | API socket (default `~/.config/herdr/herdr.sock`) |
| `HERDR_PLUGIN_CONFIG_DIR` | where `channels.json` is read from |
| `HERDR_PLUGIN_STATE_DIR` | plugin state directory |
| `OUTPUT_PANEL_LINES` | default tail length per read (default 400) |
| `OUTPUT_PANEL_TIMEOUT_MS` | default command timeout (default 3000) |
| `NO_COLOR` | disable all colour |
