<p align="center">
  <img src="assets/herdr-lockup.svg" alt="Herdr by ChefGroep" width="560" />
</p>

<p align="center">
  <strong>Terminal-native workspace runtime for coding agents.</strong><br />
  Real terminals. Persistent sessions. Agent-aware orchestration.
</p>

<p align="center">
  <a href="https://herdr.chefgroep.nl">Website</a> ·
  <a href="https://herdr.chefgroep.nl/docs/quick-start/">Quick start</a> ·
  <a href="https://herdr.chefgroep.nl/docs/integrations/">Integrations</a> ·
  <a href="https://herdr.chefgroep.nl/docs/socket-api/">Socket API</a> ·
  <a href="./SKILL.md">Agent skill</a> ·
  <a href="https://github.com/GroepOnline/herdr/releases">Releases</a>
</p>

---

Herdr is a terminal workspace manager built for running coding agents without turning them into tabs in somebody else's GUI.

It gives you persistent workspaces, tabs, panes, agent state, remote attach, plugins, and an automation surface while keeping the actual agent process in a real terminal. Detach the client and the work keeps running. Reattach later from the same machine or another host.

Herdr is not an IDE, not an Electron wrapper, and not a web dashboard. It is a native terminal application with a background session server and a CLI/socket control surface that humans and agents can both use.

<video src="https://herdr.chefgroep.nl/assets/demo-v2.mp4" controls width="100%" muted loop playsinline></video>

## Why Herdr

Coding-agent workflows have a coordination problem before they have a model problem.

Once several agents, shells, test processes, dev servers, reviews, and remote machines are active at the same time, a normal terminal stops giving you enough structure. Traditional multiplexers provide persistence and panes, but they do not understand agent lifecycle. GUI agent managers understand lifecycle, but usually replace the terminal instead of managing it.

Herdr sits between those two approaches:

- **real terminal processes** — agents run in their own native terminal UI
- **persistent sessions** — detach without stopping panes or background work
- **workspaces, tabs, and panes** — organize multiple projects without opening another IDE
- **agent awareness** — `working`, `blocked`, `done`, `idle`, and `unknown` are first-class states
- **automation** — the CLI and local socket can create layout, start agents, send work, read output, and wait for state changes
- **remote operation** — attach over SSH and keep the same workspace model on servers
- **plugins** — add workflow and fleet behavior without replacing the core terminal model

## Install

### Linux and macOS

```bash
curl -fsSL https://herdr.chefgroep.nl/install.sh | sh
```

The installer verifies release metadata and SHA-256 checksums before atomically replacing the binary.

### Homebrew

```bash
brew tap GroepOnline/tap
brew install GroepOnline/tap/groeponline-herdr
```

### npm or Bun

```bash
npm install --global groeponline-herdr
# or
bun add --global groeponline-herdr
```

The package installs the native binary and verifies the release checksum metadata.

### mise

```bash
mise use -g herdr
```

If an older mise version does not know the Herdr registry entry yet:

```bash
mise use -g github:GroepOnline/herdr
```

### Windows

Windows support is currently beta:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://herdr.chefgroep.nl/install.ps1 | iex"
```

You can also download release binaries directly from [GitHub Releases](https://github.com/GroepOnline/herdr/releases).

## Quick start

Start Herdr in the project directory:

```bash
cd ~/projects/my-project
herdr
```

Herdr starts or attaches to the default background session. If the session is empty, it creates the first workspace automatically.

Useful defaults:

| Action | Key |
| --- | --- |
| New tab | `prefix+c` |
| Next / previous tab | `prefix+n` / `prefix+p` |
| Workspace navigator | `prefix+w` |
| New workspace | `prefix+shift+n` |
| New worktree | `prefix+shift+g` |
| Split pane | `prefix+v` / `prefix+minus` |
| Move between panes | `prefix+h/j/k/l` |
| Zoom pane | `prefix+z` |
| Toggle sidebar | `prefix+b` |
| Detach | `prefix+q` |

The default prefix is `ctrl+b`.

Detach with `ctrl+b q`. The Herdr client exits, but the server and pane processes remain alive. Run `herdr` again to reattach.

## Mental model

Herdr has four core runtime concepts.

### Session

A session is one Herdr server namespace. The default `herdr` command attaches to the default session. Named sessions let you keep completely separate runtime state:

```bash
herdr session list
herdr session attach work
herdr session stop work
```

### Workspace

A workspace is the project-level container. It normally maps to a repository or working directory and contains tabs and panes.

### Tab

Tabs group related panes inside a workspace. A tab may contain an agent, test process, server, shell, logs, or any other terminal process.

### Pane

A pane is a real PTY-backed terminal process. Herdr does not redraw an agent into a proprietary chat UI; it manages the terminal the agent already owns.

## Agent awareness

Herdr continuously projects each recognized agent into one of five lifecycle states:

| State | Meaning |
| --- | --- |
| `working` | The agent is actively doing work. |
| `blocked` | Herdr recognized an approval, question, or other user-input gate. |
| `done` | Work settled while the tab was in the background and has not been seen yet. |
| `idle` | The agent is ready for input and the result has been seen. |
| `unknown` | An agent is present, but Herdr does not have enough evidence to claim another state. |

`unknown` is deliberately not treated as `idle`.

Herdr combines foreground-process detection, terminal-output manifests, and optional native integrations. Integrations can contribute semantic lifecycle state, native session identity for restore, or both.

Inspect the evidence behind a live state with:

```bash
herdr agent list
herdr agent get <agent-or-pane>
herdr agent explain <agent-or-pane>
herdr agent read <agent-or-pane> --source recent-unwrapped --lines 120
```

## Supported agents

Herdr currently ships detection manifests for:

- Aider
- Amp
- Antigravity CLI
- Claude Code
- Cline
- Codex
- Command Code
- Cursor Agent CLI
- Devin CLI
- Droid
- Freebuff
- Gemini CLI
- GitHub Copilot CLI
- Grok CLI
- Hermes Agent
- Junie
- Kilo Code CLI
- Kimi Code CLI
- Kiro CLI
- Maki
- Muse
- OpenClaude
- OpenCode, including `opencode2` / `open-code-2` aliases
- Pi
- Qoder CLI
- Qwen Code

Detection coverage varies by agent and version. Some agents provide full idle/working/blocked recognition; others use a smaller screen-detection surface. The built-in manifests are versioned so detection can evolve independently from the core UI.

For agents outside the built-in set, Herdr still works as a normal terminal workspace manager. Custom integrations can report metadata and lifecycle state through the socket API.

See the [integration docs](https://herdr.chefgroep.nl/docs/integrations/) for the current installation matrix.

## Start and coordinate agents from the CLI

Herdr is also controllable from inside a managed pane.

Create a sibling pane without stealing focus:

```bash
herdr pane split --current --direction right --cwd "$PWD" --no-focus
```

Start a recognized coding agent in that pane:

```bash
herdr agent start reviewer --kind codex --pane <pane-id>
```

Send work and wait for a settled lifecycle state:

```bash
herdr agent prompt reviewer \
  "Review the current diff and report only actionable findings." \
  --wait --timeout 120000
```

Read its terminal output:

```bash
herdr agent read reviewer --source recent-unwrapped --lines 120
```

Wait for a specific condition when needed:

```bash
herdr agent wait reviewer --until blocked --timeout 120000
```

The CLI returns structured responses for control operations. Do not guess pane or workspace IDs; use the IDs returned by Herdr.

## Agent skill

The repository includes an agent-facing skill that teaches coding agents how to control Herdr safely and how to coordinate neighboring panes without taking over the user's focus.

Install it globally with:

```bash
npx skills add GroepOnline/herdr --skill herdr -g
```

Or read it directly:

- [`SKILL.md`](./SKILL.md)
- [Agent skill documentation](https://herdr.chefgroep.nl/docs/agent-skill/)
- [Socket API](https://herdr.chefgroep.nl/docs/socket-api/)

The skill covers caller context, opaque IDs, safe pane creation, agent startup, prompting, waiting, output reads, plugin actions, and coordination rules.

## Plugins and fleet operations

Plugins add workflow-level behavior without changing Herdr's core terminal model.

```bash
herdr plugin list
herdr plugin install <owner>/<repo>[/subdir...] [--ref REF]
herdr plugin enable <plugin-id>
herdr plugin disable <plugin-id>
herdr plugin action list
herdr plugin action invoke <action-id> --plugin <plugin-id>
```

Plugin actions can be exposed through the action palette, keybindings, chains, the CLI, or the socket API.

The GroepOnline/ChefGroep distribution also uses plugins for operational context such as fleet health, GitHub status, Linear context, Cloudflare tunnels, session parking, UDO metrics, and other internal workflows. Those integrations sit on top of the same public plugin surface; they are not required to use Herdr.

## Remote operation

Herdr works normally over SSH:

```bash
ssh you@yourserver
herdr
```

You can also attach directly from your local terminal:

```bash
herdr --remote workbox
herdr --remote ssh://you@yourserver:2222
```

Remote attach can add connection reuse and fallback SSH keepalives while preserving your normal SSH configuration. Disable Herdr's SSH config management with:

```toml
[remote]
manage_ssh_config = false
```

Direct terminal attachment is also available:

```bash
herdr agent attach <target>
herdr terminal attach <terminal-id>
```

See [persistence and remote](https://herdr.chefgroep.nl/docs/persistence-remote/) for the full remote model.

## Persistence, restart, and restore

Client detach is cheap: pane processes continue running behind the Herdr server.

A full server stop is different. Stopping the server ends its pane processes:

```bash
herdr server stop
```

After updating the binary, start Herdr again to run the new server version.

Herdr also supports native agent-session restore for integrations that expose stable session identity, and an experimental live handoff path:

```bash
herdr update --handoff
```

Treat handoff as experimental. For production workflows, prefer explicit session restore unless you have verified live handoff for the foreground processes you care about.

## Updates and channels

Update the install that is first on your `PATH`:

```bash
herdr update
```

Check which binary and install type you are running:

```bash
herdr --version
```

Direct installs support channels:

```bash
herdr channel set stable
herdr channel set preview
herdr channel set dev
herdr update
```

- `stable` — published stable releases
- `preview` — release-candidate / preview builds
- `dev` — builds following merges to `main`

Homebrew, npm, mise, and Nix installations update through their package managers rather than the direct-install channel mechanism.

## Configuration

Default configuration lives at:

```text
~/.config/herdr/config.toml
```

Print the full current default configuration:

```bash
herdr --default-config
```

Herdr includes configuration for layout, keybindings, themes, sound, notifications, agent presentation, headless terminal size, integrations, plugins, remote behavior, and update settings.

The UI includes a settings surface, but the file remains the authoritative portable representation.

See [configuration](https://herdr.chefgroep.nl/docs/configuration/).

## Copy and terminal behavior

Herdr keeps terminal interaction native.

- drag-select text inside panes with the mouse
- double-click tokens or words
- use `prefix+[` for keyboard copy mode
- use `h/j/k/l`, `w/b/e`, and `{` / `}` for movement
- start selection with `v` or Space
- copy with `y` or Enter
- exit with `q` or Esc

Some SSH terminals intercept selection themselves. In PuTTY-like environments, hold `Shift` while dragging to use the host terminal's selection behavior.

## Herdr vs. terminal multiplexers and GUI agent managers

| Capability | tmux-style multiplexer | GUI agent manager | Herdr |
| --- | :---: | :---: | :---: |
| Persistent terminal processes | yes | varies | yes |
| Detach / reattach | yes | varies | yes |
| Real agent terminal UI | yes | often wrapped | yes |
| Workspaces, tabs, panes | panes/windows | usually | yes |
| Agent lifecycle awareness | no | usually | yes |
| `blocked` / `done` semantics | no | varies | yes |
| Agent-readable CLI / socket | not agent-specific | varies | yes |
| Remote SSH workflow | yes | varies | yes |
| Plugin workflow layer | no | varies | yes |

Herdr is not trying to replace tmux for every shell workflow or replace a full IDE. Its focus is the overlap: persistent terminal workspaces where multiple coding agents need to remain visible, controllable, and automatable.

## Architecture

At a high level:

```text
terminal client
     │
     ▼
Herdr session server
     │
     ├── workspace
     │    ├── tab
     │    │    ├── pane / PTY / shell
     │    │    └── pane / PTY / coding agent
     │    └── tab ...
     │
     ├── agent detection + lifecycle projection
     ├── session state + restore metadata
     ├── plugin runtime
     └── local socket API / CLI control
```

The server owns persistent runtime state. Clients render and interact with it. Agents and external tooling can use the CLI/socket surface without needing to reproduce Herdr's internal state model.

## Documentation

- [Quick start](https://herdr.chefgroep.nl/docs/quick-start/)
- [Install and updates](https://herdr.chefgroep.nl/docs/install/)
- [Concepts](https://herdr.chefgroep.nl/docs/concepts/)
- [Session state and restore](https://herdr.chefgroep.nl/docs/session-state/)
- [Persistence and remote](https://herdr.chefgroep.nl/docs/persistence-remote/)
- [Agent automation](https://herdr.chefgroep.nl/docs/agent-automation/)
- [Agent skill](https://herdr.chefgroep.nl/docs/agent-skill/)
- [Integrations](https://herdr.chefgroep.nl/docs/integrations/)
- [Plugins](https://herdr.chefgroep.nl/docs/plugins/)
- [Configuration](https://herdr.chefgroep.nl/docs/configuration/)
- [Socket API](https://herdr.chefgroep.nl/docs/socket-api/)
- [CLI reference](https://herdr.chefgroep.nl/docs/cli-reference/)

## Development

If you are an AI coding agent working on the repository, read [`AGENTS.md`](./AGENTS.md) first. Contributors should also read [`CONTRIBUTING.md`](./CONTRIBUTING.md).

```bash
git clone https://github.com/GroepOnline/herdr
cd herdr
cargo build --release
./target/release/herdr
```

Common maintenance commands:

```bash
just test
just check
just website-build
```

The repository includes release checks for synchronized changelogs, detection manifests, generated configuration references, versioned documentation snapshots, and release metadata.

## Project

Herdr is developed in the open by GroepOnline / ChefGroep and is used as part of the broader ChefGroep and UDO agent/operations stack.

Website: [herdr.chefgroep.nl](https://herdr.chefgroep.nl)

Enterprise and partnership contact: `hey@herdr.chefgroep.nl`

## Sponsor

Development is funded directly rather than through an advertising or hosted lock-in model.

[Become a GitHub Sponsor](https://github.com/sponsors/GroepOnline) · see [`SPONSORS.md`](./SPONSORS.md) for sponsorship information.

## License

Herdr is dual-licensed:

1. **Open source:** GNU Affero General Public License v3.0 or later (`AGPL-3.0-or-later`).
2. **Commercial:** commercial licenses are available for organizations that cannot comply with the AGPL.

Commercial licensing: `hey@herdr.chefgroep.nl`
