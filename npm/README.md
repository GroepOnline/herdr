# groeponline-herdr

GroepOnline's Herdr distribution: a terminal-native multiplexer and control surface for AI coding agents.

## Install

```bash
npm install --global groeponline-herdr
# or
bun add --global groeponline-herdr
```

The package supports Linux and macOS on x64 and ARM64. During postinstall it downloads the matching binary from the same GitHub release as the package version, validates it against `SHA256SUMS`, and atomically installs it inside the package. Native Windows binaries remain preview-only and are not installed by this package.

## Update

```bash
npm install --global groeponline-herdr@latest
```

`herdr update` detects npm-managed binaries and directs them back to npm instead of overwriting files inside `node_modules`.

## Quick start

```bash
herdr
herdr --version
herdr config
```

## Build from source

```bash
git clone https://github.com/GroepOnline/herdr.git
cd herdr
cargo build --release --locked
```

## License

AGPL-3.0-or-later — see [LICENSE](https://github.com/GroepOnline/herdr/blob/main/LICENSE).
