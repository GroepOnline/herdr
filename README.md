# herdr

<p align="center">
  <img src="assets/logo.png" alt="herdr" width="100" />
</p>

<p align="center">
  <a href="https://herdr.chefgroep.nl">website</a> · <a href="#install">install</a> · <a href="#quick-start">quick start</a> · <a href="#supported-agents">agents</a> · <a href="https://herdr.chefgroep.nl/docs/integrations/">integrations</a> · <a href="https://herdr.chefgroep.nl/docs/configuration/">config</a> · <a href="https://herdr.chefgroep.nl/docs/socket-api/">socket api</a> · <a href="#sponsors">sponsor</a>
</p>

---

<video src="https://herdr.chefgroep.nl/assets/demo-v2.mp4" controls width="100%" autoplay muted loop playsinline></video>

**agent multiplexer that lives in your terminal.**

workspaces, tabs, panes. mouse-native: click, drag, split. every agent at a glance — blocked, working, done. detach and reattach, agents keep running. no gui app, no electron, no mac-only wrapper. you see the agent's own terminal, not someone's interpretation of it.

---

## install

**one-liner (Linux / macOS):**

```bash
curl -fsSL https://herdr.chefgroep.nl/install.sh | sh
```

the installer verifies the manifest SHA-256 before atomically replacing the binary.

**windows (preview beta):**

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://herdr.chefgroep.nl/install.ps1 | iex"
```

**homebrew:**

```bash
brew tap GroepOnline/tap
brew install GroepOnline/tap/groeponline-herdr
```

**npm / bun:**

```bash
npm install --global groeponline-herdr
# or: bun add --global groeponline-herdr
```

the npm postinstall verifies `SHA256SUMS` before installing the native binary.

**mise:**

```bash
mise use -g herdr
```

**nix:**

```nix
# flake.nix — herdr is available as a flake
```

**direct download:** stable Linux/macOS binaries from [releases](https://github.com/GroepOnline/herdr/releases). native Windows binaries are beta; preview and dev channels publish Linux (musl) binaries only.

the [GroepOnline tap](https://github.com/GroepOnline/homebrew-tap) and [herdr.chefgroep.nl/latest.json](https://herdr.chefgroep.nl/latest.json) are the canonical distribution sources.

## quick start

```bash
herdr
```

start in the directory where the work lives. herdr starts or attaches to one background session server and opens a workspace if none exists. run an agent in the root pane, then split:

| action | key |
|--------|-----|
| new workspace | `prefix+shift+n` |
| split pane (right) | `prefix+v` |
| split pane (down) | `prefix+minus` |
| new tab | `prefix+c` |
| switch workspace | `prefix+w` |
| detach | `prefix+q` |

detach closes the client only — the server and all pane processes keep running. open another terminal and run `herdr` again to reattach.

## core concepts

**server and client.** `herdr` attaches to a background server by default. detaching closes only the client. `herdr server stop` stops the default server and kills its panes. named sessions are separate server namespaces: `herdr session attach work`, `herdr session stop work`, `herdr session list`.

**workspaces, tabs, panes.** a workspace is the project-level container. tabs group panes inside a workspace. panes are real terminal processes, not rewritten agent views.

**copy.** herdr copies pane text, not the sidebar. drag-select inside a pane, double-click a word or token, or press `prefix+[` for keyboard copy mode. in copy mode: `h/j/k/l` to move, `w/b/e` for words, `{`/`}` for paragraphs, `v` for visual, `y` to yank.

**keybindings.** herdr uses explicit keybinding strings. `prefix+n` means press the configured prefix, then `n`. `ctrl+alt+n`, `cmd+k`, `alt+1`, and function-key chords are direct terminal-mode shortcuts — no prefix needed. plain printable keys like `n` steal normal typing, so use `prefix+n` unless you intentionally want a modifier-gated direct binding.

**agent awareness.** the sidebar and navigator use one explicit projection: `working`, `blocked`, `done`, `idle`, or `unknown`. `unknown` means insufficient evidence, not idle. detection works with process names and terminal output by default; official integrations can add native session identity for restore, semantic state reports, or both.

## update

```bash
herdr update
```

`herdr update` updates the binary that is first on `PATH`. direct installs download from the configured channel. homebrew, npm, and mise installs run their package-manager upgrade. a leftover `~/.local/bin/herdr` that shadows a later homebrew, npm, or mise install is renamed to `herdr.direct.bak` after that upgrade succeeds; `herdr update --force-direct` keeps updating the leftover. `herdr --version` prints the binary path and install kind on stderr, and `herdr channel` prints the configured channel.

**channels:**

| channel | command | purpose |
|---------|---------|---------|
| stable (default) | `herdr channel set stable` | production |
| preview | `herdr channel set preview` | test before next stable |
| dev | `herdr channel set dev` | smoke-test merges to `main` |

preview and dev are direct-install only (homebrew, mise, and nix stay on stable). see [install docs](https://herdr.chefgroep.nl/docs/install/) and [session state docs](https://herdr.chefgroep.nl/docs/session-state/) for the full update, restart, restore, and handoff matrix.

## how it compares

|                          | tmux | gui managers | herdr |
|--------------------------|------|--------------|-------|
| persistent sessions       | ✓    | —            | ✓     |
| detach / reattach        | ✓    | —            | ✓     |
| panes, tabs, workspaces  | ✓    | ✓            | ✓     |
| agent awareness          | —    | ✓            | ✓     |
| lives in your terminal   | ✓    | —            | ✓     |
| real terminal views      | ✓    | —            | ✓     |
| mouse-native             | —    | ✓            | ✓     |
| lightweight binary       | ✓    | —            | ✓     |
| agents can orchestrate   | ?    | ?            | ✓     |

tmux gives you persistence and panes, but it was built before agents existed. gui managers show agent state, but they make you leave your terminal and use their wrapped view. herdr is persistence and awareness in one tool that stays out of your way.

## remote and attach

herdr works over normal SSH. run it on the remote host, detach, and reattach later:

```
ssh you@yourserver
herdr
```

attach from your local terminal without opening a shell first:

```bash
herdr --remote workbox
herdr --remote ssh://you@yourserver:2222
```

remote attach adds fallback SSH keepalives and connection reuse by default while preserving your own SSH config. set `[remote].manage_ssh_config = false` for plain `ssh`.

direct attach connects your current terminal to one server-owned terminal:

```bash
herdr agent attach <target>
herdr terminal attach <terminal_id>
```

see [persistence and remote docs](https://herdr.chefgroep.nl/docs/persistence-remote/) for remote keybinding, named-session, and handoff details.

## agent awareness

the sidebar shows which agents are blocked, working, done, idle, or unknown. workspaces roll up to their most urgent state so you can scan the full list at a glance. `unknown` is intentionally visible instead of being presented as idle; use `herdr agent explain <target>` to inspect the evidence and authority behind a live state.

**states:**

- 🔴 **blocked** — agent needs input or approval
- 🟡 **working** — agent is actively running
- 🔵 **done** — work finished, you have not looked at it yet
- 🟢 **idle** — done and seen
- ⚪ **unknown** — herdr does not have enough evidence to claim another state

detection works by reading foreground process and terminal output. zero config, no hooks required. official integrations provide session restore identity (claude code, codex, github copilot cli, devin, droid, qodercli, cursor agent cli) or both semantic state and session identity (pi, omp, kimi code cli, opencode, kilo code cli, hermes, mastracode).

## lives in your terminal

not a gui window, not a web dashboard, not electron. herdr runs inside whatever terminal you already use. single rust binary, no dependencies. works inside tmux as the outer terminal environment.

## what you get

- **workspaces** — organized around git repos or folder names, each with its own tabs and panes
- **tabs** — first-class in the socket api and cli
- **copy-friendly** — drag-select pane text, double-click tokens, or keyboard copy mode
- **notifications** — sounds and toasts for background events; tab-aware suppression
- **agent-aware presentation** — configurable sidebar rows, semantic state icons/text, per-agent layouts, terminal titles, git context, and plugin-reported metadata
- **motion you can tune** — large spinner catalog, dots or symbols for status indicators, live preview in settings
- **18 built-in themes** — catppuccin, terminal, tokyo night, gruvbox, one, solarized, kanagawa, rosé pine, vesper, and light variants
- **session persistence** — pane processes survive client detach; sessions restore panes after full restart, with opt-in recent screen history

## agents can use herdr too

the local Unix socket lets agents create workspaces, split or zoom panes, spawn helpers, read output, and wait for state changes. install the reusable skill:

```bash
npx skills add GroepOnline/herdr --skill herdr -g
```

start with the [agent skill docs](https://herdr.chefgroep.nl/docs/agent-skill/), [socket API docs](https://herdr.chefgroep.nl/docs/socket-api/), and [`SKILL.md`](./SKILL.md).

## supported agents

automatic detection works out of the box — process name matching plus terminal output heuristics.

| agent | idle / done | working | blocked |
|-------|-------------|---------|---------|
| [pi](https://pi.dev) | ✓ | ✓ | partial |
| [claude code](https://docs.anthropic.com/en/docs/claude-code) | ✓ | ✓ | ✓ |
| [codex](https://github.com/openai/codex) | ✓ | ✓ | ✓ |
| [droid](https://factory.ai) | ✓ | ✓ | ✓ |
| [amp](https://ampcode.com) | ✓ | ✓ | ✓ |
| [opencode](https://github.com/anomalyco/opencode) | ✓ | ✓ | ✓ |
| [grok cli](https://x.ai/grok) | ✓ | ✓ | ✓ |
| [hermes agent](https://github.com/NousResearch/hermes-agent) | ✓ | ✓ | ✓ |
| [kilo code cli](https://kilo.ai/) | ✓ | ✓ | ✓ |
| [devin cli](https://docs.devin.ai/cli) | ✓ | ✓ | ✓ |
| cursor agent | ✓ | ✓ | ✓ |
| antigravity cli | ✓ | ✓ | ✓ |
| kimi code cli | ✓ | ✓ | ✓ |
| [github copilot cli](https://github.com/features/copilot) | ✓ | ✓ | ✓ |
| [qodercli](https://qoder.com/cli) | ✓ | ✓ | ✓ |
| [kiro cli](https://kiro.dev/docs/cli/) | ✓ | ✓ | — |

detected but not fully tested: gemini cli, cline.

for agents outside the built-in list, herdr still works as a terminal multiplexer with workspaces, panes, and tiling. custom integrations can report agent labels over the socket api — see the [socket api docs](https://herdr.chefgroep.nl/docs/socket-api/).

### direct integrations

official integrations have two roles. claude code, codex, github copilot cli, devin, droid, qodercli, and cursor agent cli report session identity for native restore, while their state still comes from screen detection. pi, omp, kimi code cli, opencode, kilo code cli, hermes, and mastracode report both semantic state and session identity. install with:

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

see the [integrations docs](https://herdr.chefgroep.nl/docs/integrations/) for setup details.

## keybindings

press `ctrl+b` to enter prefix mode. default actions are prefix-first and tmux-like:

| key | action |
|-----|--------|
| `prefix+c` | new tab |
| `prefix+n` / `prefix+p` | next / previous tab |
| `prefix+1..9` | switch tab |
| `prefix+w` | workspace navigation |
| `prefix+g` | session navigator |
| `prefix+shift+n` | new workspace |
| `prefix+shift+g` | new worktree |
| `prefix+shift+w` | rename workspace |
| `prefix+shift+d` | close workspace |
| `prefix+h/j/k/l` | focus pane |
| `prefix+shift+h/j/k/l` | swap pane |
| `prefix+v` / `prefix+minus` | split pane |
| `prefix+x` | close pane |
| `prefix+b` | toggle sidebar |
| `prefix+z` | zoom pane |
| `prefix+r` | resize mode |
| `prefix+q` | detach |

mouse is supported throughout. resize mode uses `h`/`l` for width, `j`/`k` for height, and `esc` to exit. full syntax, optional actions, indexed bindings, and custom command bindings live in the [configuration docs](https://herdr.chefgroep.nl/docs/configuration/).

## configuration

config file: `~/.config/herdr/config.toml`

```bash
herdr --default-config   # print full default config
```

in-app settings cover theme, sound, and toast preferences. herdr writes logs under `~/.config/herdr/`; in persistent session mode, `herdr-client.log` and `herdr-server.log` are usually the useful files. full configuration and logging details live in the [configuration docs](https://herdr.chefgroep.nl/docs/configuration/).

## docs

- [quick start](https://herdr.chefgroep.nl/docs/quick-start/) — first session, panes, copy, named sessions
- [install](https://herdr.chefgroep.nl/docs/install/) — install, update, homebrew, mise, nix
- [session state](https://herdr.chefgroep.nl/docs/session-state/) — detach, restart restore, agent restore, live handoff
- [configuration](https://herdr.chefgroep.nl/docs/configuration/) — keybindings, themes, notifications, environment variables
- [integrations](https://herdr.chefgroep.nl/docs/integrations/) — pi, omp, claude code, codex, cursor agent cli, github copilot cli, droid, kimi code cli, opencode, kilo code cli, hermes, mastracode, qodercli
- [`SKILL.md`](./SKILL.md) — reusable agent skill
- [socket api](https://herdr.chefgroep.nl/docs/socket-api/) — socket protocol and cli reference

## agent instructions

if you are an ai agent helping with this repository, read [`AGENTS.md`](./AGENTS.md) before making changes and [`CONTRIBUTING.md`](./CONTRIBUTING.md) before opening issues or PRs.

## development

```bash
git clone https://github.com/GroepOnline/herdr
cd herdr
cargo build --release
./target/release/herdr

just test        # unit tests
just check       # formatting, tests, and maintenance checks
```

## sponsors

herdr is built full-time, in the open, with no revenue behind it. sponsoring directly funds development, stability, and the path to a real agent runtime.

[**→ become a sponsor**](https://github.com/sponsors/GroepOnline) · enterprise / partnership: hey@herdr.chefgroep.nl · see [SPONSORS.md](./SPONSORS.md) for tiers. thank you 🐑

## license

herdr is dual-licensed:

1. **open source** — GNU Affero General Public License v3.0 or later (AGPL-3.0-or-later)
2. **commercial** — commercial licenses available for organizations that cannot comply with AGPL

contact: hey@herdr.chefgroep.nl
