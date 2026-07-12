# onlinechefgroep-herdr

Herdr - fast terminal multiplexer with AI agent detection.

OnlineChefGroep distribution of [ogulcancelik/herdr](https://github.com/ogulcancelik/herdr).

## Install

```bash
npm install -g onlinechefgroep-herdr
```

Or with bun:

```bash
bun add -g onlinechefgroep-herdr
```

## Quick start

```bash
herdr              # start multiplexer
herdr --version    # show version
herdr config       # show config
```

## Features

- AI agent detection (Claude Code, Copilot, Cursor, Devin, OpenCode, +20 more)
- Split panes, tabs, workspaces
- Remote pairing via SSH
- Kitty graphics protocol
- Workspace persistence and snapshots
- Copy mode with vim/emacs bindings

## Build from source

```bash
git clone https://github.com/OnlineChefGroep/herdr.git
cd herdr
cargo build --release
```

## License

MIT - see [LICENSE](https://github.com/OnlineChefGroep/herdr/blob/master/LICENSE)
RMEOF
echo "README.md OK" && cd ~/herdr-private && git add npm/ && git status --short npm/
