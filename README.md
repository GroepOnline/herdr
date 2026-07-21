# herdr

<p align="center">
  <img src="assets/logo.png" alt="Herdr" width="100" />
</p>

<p align="center">
  <a href="https://herdr.chefgroep.nl">Website</a> ·
  <a href="https://herdr.chefgroep.nl/docs/quick-start/">Quick start</a> ·
  <a href="https://herdr.chefgroep.nl/docs/integrations/">Integrations</a> ·
  <a href="https://herdr.chefgroep.nl/docs/configuration/">Configuration</a> ·
  <a href="https://herdr.chefgroep.nl/docs/socket-api/">Socket API</a> ·
  <a href="https://github.com/OnlineChefGroep/herdr/releases/latest">Latest release</a>
</p>

<p align="center">
  <a href="https://github.com/OnlineChefGroep/herdr/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/OnlineChefGroep/herdr?display_name=tag&sort=semver"></a>
  <a href="https://github.com/OnlineChefGroep/herdr/actions"><img alt="Build status" src="https://img.shields.io/github/actions/workflow/status/OnlineChefGroep/herdr/ci.yml?branch=main"></a>
  <a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/license-AGPL--3.0--or--later-blue"></a>
</p>

**A terminal-native multiplexer and control surface for AI coding agents.**

Herdr gives you persistent workspaces, tabs and real terminal panes, with agent-aware states such as blocked, working, done and idle. Detach and reattach without killing the running agents. There is no Electron shell, wrapped agent UI or macOS-only native application.

The OnlineChefGroep distribution tracks upstream Herdr and adds the CHEF release, integration and deployment layer. The current stable line is **v0.7.5**; `main` can contain validated post-release fixes before the next tag.

## Install

Linux and macOS direct install:

```bash
curl -fsSL https://herdr.chefgroep.nl/install.sh | sh
```

Windows preview beta:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://herdr.chefgroep.nl/install.ps1 | iex"
```

Homebrew:

```bash
brew install herdr
```

mise:

```bash
mise use -g herdr
```

When an older mise version cannot find the registry entry, update mise or temporarily use:

```bash
mise use -g github:OnlineChefGroep/herdr
```

Stable Linux/macOS binaries and preview artifacts are available from [GitHub Releases](https://github.com/OnlineChefGroep/herdr/releases). Native Windows binaries remain preview-only.

## Quick start

Start Herdr in the directory containing your work:

```bash
herdr
```

Herdr starts or attaches to a background session server and opens a workspace when needed.

Common controls:

| Key | Action |
|---|---|
| `ctrl+b`, then `shift+n` | New workspace |
| `ctrl+b`, then `c` | New tab |
| `ctrl+b`, then `v` or `-` | Split pane |
| `ctrl+b`, then `w` | Switch workspace |
| `ctrl+b`, then `z` | Zoom pane |
| `ctrl+b`, then `[` | Copy mode |
| `ctrl+b`, then `q` | Detach client |

Detaching closes only the client. The server and pane processes continue running. Run `herdr` again to reattach.

## Core model

- **Server and client:** one background server owns the session; clients can detach and reattach.
- **Workspaces, tabs and panes:** workspaces are project-level containers, tabs group panes, and panes are real terminal processes.
- **Named sessions:** use `herdr session attach <name>`, `herdr session stop <name>` and `herdr session list` for isolated runtime namespaces.
- **Agent awareness:** foreground processes, terminal output and direct integrations drive blocked, working, done and idle state.
- **Persistence and restore:** pane processes survive client detach; supported agent integrations can restore native agent sessions after a full restart.
- **Remote operation:** Herdr works over SSH and supports direct remote attach.

## Update

For direct installs:

```bash
herdr update
```

A running server continues using its current binary until it is stopped or handed off. For the default session:

```bash
herdr server stop
herdr
```

For a named session:

```bash
herdr session stop <name>
herdr session attach <name>
```

`herdr update --handoff` is experimental and attempts to transfer supported live panes to the replacement server.

Package-manager installs update through their package manager:

```bash
brew upgrade herdr
mise upgrade herdr
```

Direct Linux/macOS installs use the stable channel by default. Preview builds can be selected with:

```bash
herdr channel set preview
```

Return to stable with:

```bash
herdr channel set stable
```

Windows beta installs currently use the preview channel.

## Agent support

Automatic process and terminal-output detection works without hooks. Current built-in coverage includes:

- pi
- Claude Code
- Codex
- Droid
- Amp
- OpenCode
- Grok CLI
- Hermes Agent
- Kilo Code CLI
- Devin CLI
- Cursor Agent CLI
- Antigravity CLI
- Kimi Code CLI
- GitHub Copilot CLI
- Qoder CLI
- Kiro CLI

Gemini CLI and Cline are detected but are not yet considered fully validated.

Official integrations can provide native session identity, semantic state reporting or both:

```bash
herdr integration install pi
herdr integration install omp
herdr integration install claude
herdr integration install codex
herdr integration install copilot
herdr integration install devin
herdr integration install droid
herdr integration install kimi
herdr integration install opencode
herdr integration install kilo
herdr integration install hermes
herdr integration install mastracode
herdr integration install qodercli
herdr integration install cursor
```

See the [integration documentation](https://herdr.chefgroep.nl/docs/integrations/) for exact capabilities and setup requirements.

## Agent automation

Herdr exposes a local socket API that lets agents create workspaces, split or zoom panes, spawn helpers, inspect output and wait for state changes.

Install the reusable skill globally:

```bash
npx skills add OnlineChefGroep/herdr --skill herdr -g
```

Start with:

- [Agent skill documentation](https://herdr.chefgroep.nl/docs/agent-skill/)
- [`SKILL.md`](./SKILL.md)
- [Socket API documentation](https://herdr.chefgroep.nl/docs/socket-api/)

## Remote and direct attach

Run Herdr normally on a remote host:

```bash
ssh you@yourserver
herdr
```

Or attach from the local terminal:

```bash
herdr --remote workbox
herdr --remote ssh://you@yourserver:2222
```

Directly attach to a server-owned agent or terminal:

```bash
herdr agent attach <target>
herdr terminal attach <terminal_id>
```

See [persistence and remote documentation](https://herdr.chefgroep.nl/docs/persistence-remote/) for SSH behavior, named sessions and handoff details.

## Configuration

The configuration file is located at:

```text
~/.config/herdr/config.toml
```

Print the complete default configuration:

```bash
herdr --default-config
```

Herdr writes logs under `~/.config/herdr/`. In persistent-session mode, `herdr-client.log` and `herdr-server.log` are usually the relevant files.

## Documentation

- [Quick start](https://herdr.chefgroep.nl/docs/quick-start/)
- [Install and update](https://herdr.chefgroep.nl/docs/install/)
- [Session state and restore](https://herdr.chefgroep.nl/docs/session-state/)
- [Configuration](https://herdr.chefgroep.nl/docs/configuration/)
- [Integrations](https://herdr.chefgroep.nl/docs/integrations/)
- [Socket API](https://herdr.chefgroep.nl/docs/socket-api/)
- [`SKILL.md`](./SKILL.md)

## Development

Read [`AGENTS.md`](./AGENTS.md) and [`CONTRIBUTING.md`](./CONTRIBUTING.md) before changing the repository.

```bash
git clone https://github.com/OnlineChefGroep/herdr.git
cd herdr
cargo build --release
./target/release/herdr

just test
just check
```

The release metadata is currently aligned as follows:

- Rust crate: `0.7.5`
- CHEF distribution package: `0.7.5-chef`
- Stable Git tag/release line: `v0.7.5`

## License

Herdr is dual-licensed:

1. GNU Affero General Public License v3.0 or later (`AGPL-3.0-or-later`).
2. Commercial licenses for organizations that cannot comply with the AGPL.

Commercial and partnership contact: `hey@herdr.chefgroep.nl`.