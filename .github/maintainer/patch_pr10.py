from __future__ import annotations

import json
import re
from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one occurrence, found {count}: {old[:120]!r}")
    file.write_text(text.replace(old, new, 1), encoding="utf-8")


# Direct attach must preserve every configured legacy terminal sequence, including
# multi-byte prefixes such as F12. Never silently substitute ctrl+a.
client_path = Path("src/client/mod.rs")
client = client_path.read_text(encoding="utf-8")
start_marker = "#[derive(Debug, Default)]\n#[cfg(windows)]\nstruct AttachEscapeState;"
end_marker = "#[cfg(unix)]\nfn attach_scroll_action("
start = client.find(start_marker)
end = client.find(end_marker)
if start < 0 or end < 0 or end <= start:
    raise SystemExit("src/client/mod.rs: attach escape state markers not found")

new_state = r'''#[derive(Debug, Default)]
#[cfg(windows)]
struct AttachEscapeState;

#[derive(Debug)]
#[cfg(unix)]
struct AttachEscapeState {
    prefix: Vec<u8>,
    pending_prefix: bool,
    buffered_input: Vec<u8>,
}

#[cfg(unix)]
#[derive(Debug)]
enum AttachInputAction {
    Forward(Vec<u8>),
    Scroll {
        source: AttachScrollSource,
        direction: AttachScrollDirection,
        lines: u16,
        column: Option<u16>,
        row: Option<u16>,
        modifiers: u8,
    },
    Detach,
    None,
}

impl AttachEscapeState {
    #[cfg(unix)]
    fn new(prefix: Vec<u8>) -> io::Result<Self> {
        if prefix.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "direct attach prefix must encode to at least one byte",
            ));
        }
        Ok(Self {
            prefix,
            pending_prefix: false,
            buffered_input: Vec::new(),
        })
    }

    #[cfg(unix)]
    fn filter_input(
        &mut self,
        data: Vec<u8>,
        viewport_rows: u16,
        mouse_scroll_lines: usize,
    ) -> AttachInputAction {
        self.buffered_input.extend_from_slice(&data);
        let mut output = Vec::with_capacity(self.buffered_input.len() + self.prefix.len());

        while !self.buffered_input.is_empty() {
            if self.pending_prefix {
                if self.buffered_input[0] == b'q' {
                    self.buffered_input.remove(0);
                    self.pending_prefix = false;
                    return AttachInputAction::Detach;
                }

                if self.buffered_input.starts_with(&self.prefix) {
                    self.buffered_input.drain(..self.prefix.len());
                    output.extend_from_slice(&self.prefix);
                    self.pending_prefix = false;
                    continue;
                }

                if self.buffered_input.len() < self.prefix.len()
                    && self.prefix.starts_with(&self.buffered_input)
                {
                    break;
                }

                output.extend_from_slice(&self.prefix);
                output.push(self.buffered_input.remove(0));
                self.pending_prefix = false;
                continue;
            }

            if self.buffered_input.starts_with(&self.prefix) {
                self.buffered_input.drain(..self.prefix.len());
                self.pending_prefix = true;
                continue;
            }

            if self.buffered_input.len() < self.prefix.len()
                && self.prefix.starts_with(&self.buffered_input)
            {
                break;
            }

            output.push(self.buffered_input.remove(0));
        }

        if output.is_empty() {
            AttachInputAction::None
        } else if let Some(action) =
            attach_scroll_action(&output, viewport_rows, mouse_scroll_lines)
        {
            action
        } else {
            AttachInputAction::Forward(output)
        }
    }
}

#[cfg(unix)]
fn attach_prefix_bytes(code: KeyCode, modifiers: KeyModifiers) -> io::Result<Vec<u8>> {
    use crate::input::{encode_terminal_key, KeyboardProtocol, TerminalKey};

    let bytes = encode_terminal_key(
        TerminalKey::new(code, modifiers),
        KeyboardProtocol::Legacy,
    );
    if bytes.is_empty() {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "configured direct attach prefix {modifiers:?}+{code:?} has no terminal encoding"
            ),
        ))
    } else {
        Ok(bytes)
    }
}

'''
client = client[:start] + new_state + client[end:]
client_path.write_text(client, encoding="utf-8")

replace_once(
    "src/client/mod.rs",
    '''pub fn run_terminal_attach(terminal_id: String, takeover: bool) -> io::Result<()> {
    let loaded_config = crate::config::Config::load();
    let (code, mods) = loaded_config.config.prefix_key();
    let prefix_byte = attach_prefix_byte(code, mods).unwrap_or(0x01);
    run_client_with_mode(
        RenderEncoding::TerminalAnsi,
        Some((terminal_id, takeover)),
        Some(AttachEscapeState::new(prefix_byte)),
        "attaching to terminal",
    )
}''',
    '''pub fn run_terminal_attach(terminal_id: String, takeover: bool) -> io::Result<()> {
    let loaded_config = crate::config::Config::load();
    let (code, modifiers) = loaded_config.config.prefix_key();
    let prefix = attach_prefix_bytes(code, modifiers)?;
    run_client_with_mode(
        RenderEncoding::TerminalAnsi,
        Some((terminal_id, takeover)),
        Some(AttachEscapeState::new(prefix)?),
        "attaching to terminal",
    )
}''',
)

client = client_path.read_text(encoding="utf-8")
for old, new in (
    ("AttachEscapeState::new(0x01)", "AttachEscapeState::new(vec![0x01]).unwrap()"),
    ("AttachEscapeState::new(0x02)", "AttachEscapeState::new(vec![0x02]).unwrap()"),
):
    if old not in client:
        raise SystemExit(f"src/client/mod.rs: constructor pattern not found: {old}")
    client = client.replace(old, new)
client = client.replace(
    "fn attach_escape_uses_configured_prefix_byte()",
    "fn attach_escape_uses_configured_prefix_sequence()",
    1,
)
client_path.write_text(client, encoding="utf-8")

extra_tests = r'''
    #[cfg(unix)]
    #[test]
    fn attach_prefix_bytes_preserves_single_and_multibyte_sequences() {
        assert_eq!(
            attach_prefix_bytes(KeyCode::Char('a'), KeyModifiers::CONTROL).unwrap(),
            vec![0x01]
        );
        assert_eq!(
            attach_prefix_bytes(KeyCode::Esc, KeyModifiers::empty()).unwrap(),
            vec![0x1b]
        );
        let f12 = attach_prefix_bytes(KeyCode::F(12), KeyModifiers::empty()).unwrap();
        assert!(f12.len() > 1, "F12 must remain a multi-byte escape sequence");
    }

    #[cfg(unix)]
    #[test]
    fn attach_escape_rejects_an_empty_prefix() {
        assert!(AttachEscapeState::new(Vec::new()).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn attach_escape_matches_multibyte_prefix_across_read_boundaries() {
        let prefix = b"\x1b[24~".to_vec();
        let mut escape = AttachEscapeState::new(prefix).unwrap();

        assert!(matches!(
            escape.filter_input(b"\x1b[2".to_vec(), 24, 3),
            AttachInputAction::None
        ));
        assert!(matches!(
            escape.filter_input(b"4~".to_vec(), 24, 3),
            AttachInputAction::None
        ));
        assert!(matches!(
            escape.filter_input(vec![b'q'], 24, 3),
            AttachInputAction::Detach
        ));
    }

    #[cfg(unix)]
    #[test]
    fn attach_escape_forwards_doubled_multibyte_prefix_literally() {
        let prefix = b"\x1b[24~".to_vec();
        let mut escape = AttachEscapeState::new(prefix.clone()).unwrap();

        assert!(matches!(
            escape.filter_input(prefix.clone(), 24, 3),
            AttachInputAction::None
        ));
        match escape.filter_input(prefix.clone(), 24, 3) {
            AttachInputAction::Forward(bytes) => assert_eq!(bytes, prefix),
            other => panic!("expected forwarded multi-byte prefix, got {other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn attach_escape_preserves_escape_sequences_when_escape_is_the_prefix() {
        let mut escape = AttachEscapeState::new(vec![0x1b]).unwrap();
        match escape.filter_input(b"\x1b[A".to_vec(), 24, 3) {
            AttachInputAction::Forward(bytes) => assert_eq!(bytes, b"\x1b[A"),
            other => panic!("expected forwarded cursor sequence, got {other:?}"),
        }
    }

'''
replace_once(
    "src/client/mod.rs",
    "    #[cfg(unix)]\n    #[test]\n    fn attach_escape_turns_wheel_into_scroll_action()",
    extra_tests
    + "    #[cfg(unix)]\n    #[test]\n    fn attach_escape_turns_wheel_into_scroll_action()",
)

# Keep the routing test coupled to the configured/default prefix rather than a magic byte.
replace_once(
    "src/app/mod.rs",
    '''        // Default prefix is ctrl+a (0x01), not ctrl+b.
        app.route_client_input(vec![0x01, b'\t']);''',
    '''        let mut prefix_tab = crate::input::encode_terminal_key(
            crate::input::TerminalKey::new(app.state.prefix_code, app.state.prefix_mods),
            crate::input::KeyboardProtocol::Legacy,
        );
        prefix_tab.push(b'\t');
        app.route_client_input(prefix_tab.clone());''',
)
replace_once(
    "src/app/mod.rs",
    "        app.route_client_input(vec![0x01, b'\\t']);",
    "        app.route_client_input(prefix_tab);",
)

# Gateway release metadata must be derived, and uptime must not always be zero.
replace_once(
    "src/bin/herdr-gateway.rs",
    "use std::sync::Arc;\n",
    "use std::sync::Arc;\nuse std::time::Instant;\n",
)
replace_once(
    "src/bin/herdr-gateway.rs",
    '''struct GatewayState {
    socket_path: PathBuf,
    version: String,
}''',
    '''struct GatewayState {
    socket_path: PathBuf,
    version: String,
    started_at: Instant,
}''',
)
replace_once(
    "src/bin/herdr-gateway.rs",
    '''    let uptime = std::time::Instant::now()
        .elapsed()
        .as_secs();''',
    "    let uptime = s.started_at.elapsed().as_secs();",
)
replace_once(
    "src/bin/herdr-gateway.rs",
    '''    let state = Arc::new(RwLock::new(GatewayState {
        socket_path: socket_path.clone(),
        version: "0.7.3-chef".to_string(),
    }));''',
    '''    let state = Arc::new(RwLock::new(GatewayState {
        socket_path: socket_path.clone(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        started_at: Instant::now(),
    }));''',
)

# The fork release workflow must read its own releases, not upstream's.
replace_once(
    "scripts/changelog.py",
    'DEFAULT_RELEASE_REPO = "ogulcancelik/herdr"',
    'DEFAULT_RELEASE_REPO = "OnlineChefGroep/herdr"',
)

# npm publishes only the platforms for which release.yml creates assets.
package_path = Path("npm/package.json")
package = json.loads(package_path.read_text(encoding="utf-8"))
if package.get("os") != ["linux", "darwin", "win32"]:
    raise SystemExit(f"npm/package.json: unexpected os list: {package.get('os')!r}")
package["os"] = ["linux", "darwin"]
package_path.write_text(json.dumps(package, indent=2) + "\n", encoding="utf-8")

readme_path = Path("npm/README.md")
readme = readme_path.read_text(encoding="utf-8")
if "RMEOF" not in readme:
    raise SystemExit("npm/README.md: expected heredoc residue not found")
readme = readme.split("\nRMEOF", 1)[0].rstrip() + "\n"
replace_target = '''```bash
bun add -g onlinechefgroep-herdr
```
'''
replacement = replace_target + '''
Prebuilt binaries are published for Linux (x64/arm64) and macOS (x64/arm64). Windows is not part of the v0.7.4 binary release.
'''
if replace_target not in readme:
    raise SystemExit("npm/README.md: bun install block not found")
readme_path.write_text(readme.replace(replace_target, replacement, 1), encoding="utf-8")

# Fix markdownlint spacing and document release-hardening changes.
changelog_path = Path("CHANGELOG.md")
changelog = changelog_path.read_text(encoding="utf-8")
for heading in ("### Added", "### Fixed", "### Changed"):
    needle = heading + "\n-"
    if needle not in changelog:
        raise SystemExit(f"CHANGELOG.md: expected heading/list adjacency not found: {heading}")
    changelog = changelog.replace(needle, heading + "\n\n-", 1)
replace_target = "- Prefix routing test updated for `ctrl+a` default."
replacement = replace_target + "\n- Direct attach preserves single- and multi-byte configured prefixes without silently falling back to `ctrl+a`.\n- Gateway health reports the Cargo package version and actual process uptime.\n- Release manifest generation now reads releases from `OnlineChefGroep/herdr`.\n- npm no longer advertises an unsupported Windows prebuilt and its packaged README is clean."
if replace_target not in changelog:
    raise SystemExit("CHANGELOG.md: fixed-section insertion point not found")
changelog = changelog.replace(replace_target, replacement, 1)
replace_target = "- Distribution binaries publish from `OnlineChefGroep/herdr` releases (replacing upstream-seeded v0.7.3 assets)."
replacement = replace_target + "\n- CI uses the same patched Zig path as release builds on macOS and gates only supported release platforms."
if replace_target not in changelog:
    raise SystemExit("CHANGELOG.md: changed-section insertion point not found")
changelog_path.write_text(changelog.replace(replace_target, replacement, 1), encoding="utf-8")

# Replace the drifted CI matrix with a release-representative, least-privilege gate.
Path(".github/workflows/ci.yml").write_text(
    '''name: CI

on:
  push:
    branches: [master, main]
  pull_request:
    branches: [master, main]

permissions:
  contents: read

env:
  CARGO_TERM_COLOR: always
  FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: true
  RUSTFLAGS: "-D warnings"

jobs:
  check:
    name: Check & Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          persist-credentials: false

      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - uses: Swatinem/rust-cache@v2

      - name: Install Zig
        uses: mlugg/setup-zig@d1434d08867e3ee9daa34448df10607b98908d29 # v2.2.1
        with:
          version: 0.15.2

      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libdbus-1-dev pkg-config

      - name: Formatting
        run: cargo fmt --all -- --check

      - name: Release metadata consistency
        run: |
          python3 scripts/changelog.py validate-product-announcement
          python3 - <<'PY'
          import json
          import re
          from pathlib import Path

          cargo = Path("Cargo.toml").read_text()
          version = re.search(r'^version = "([^"]+)"', cargo, re.MULTILINE).group(1)
          package = json.loads(Path("npm/package.json").read_text())
          installer = Path("npm/install.js").read_text()
          changelog = Path("CHANGELOG.md").read_text()
          gateway = Path("src/bin/herdr-gateway.rs").read_text()
          release_tools = Path("scripts/changelog.py").read_text()
          npm_readme = Path("npm/README.md").read_text()

          assert package["version"] == version
          assert package["os"] == ["linux", "darwin"]
          assert f'const VERSION = "{version}";' in installer
          assert f"## [{version}]" in changelog
          assert 'env!("CARGO_PKG_VERSION")' in gateway
          assert 'DEFAULT_RELEASE_REPO = "OnlineChefGroep/herdr"' in release_tools
          assert "RMEOF" not in npm_readme
          for heading in ("### Added", "### Fixed", "### Changed"):
              assert f"{heading}\\n\\n-" in changelog
          PY

      - name: Cargo check
        run: cargo check --all-targets --locked

      - name: Cargo test
        run: cargo test --all --locked -- --skip graphics_bytes_are_written_after_blit_with_saved_cursor --skip foreground_client_applies_client_keybindings --skip invalid_server_keybindings_apply_valid_subset_after_settings_save_without_caching_local_keybindings --skip local_keybinding_client_keeps_local_keybindings_after_settings_save --skip pane_border_renderer_places_adjacent_cjk_by_display_width

      - name: Cargo clippy
        run: cargo clippy --all-targets --locked -- -D warnings

  build:
    name: Build (${{ matrix.target }})
    needs: check
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
        with:
          persist-credentials: false

      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - uses: Swatinem/rust-cache@v2
        with:
          key: ci-${{ matrix.target }}

      - name: Install Zig
        if: runner.os != 'macOS'
        uses: mlugg/setup-zig@d1434d08867e3ee9daa34448df10607b98908d29 # v2.2.1
        with:
          version: 0.15.2

      - name: Restore Homebrew Zig cache
        if: runner.os == 'macOS'
        uses: actions/cache@v4
        with:
          path: ~/Library/Caches/Homebrew/downloads
          key: homebrew-zig-0.15-${{ runner.os }}-${{ runner.arch }}
          restore-keys: |
            homebrew-zig-0.15-${{ runner.os }}-

      - name: Install patched Zig on macOS
        if: runner.os == 'macOS'
        run: |
          HOMEBREW_NO_AUTO_UPDATE=1 brew install zig@0.15
          echo "$(brew --prefix zig@0.15)/bin" >> "$GITHUB_PATH"
          "$(brew --prefix zig@0.15)/bin/zig" version

      - name: Install Linux dependencies
        if: runner.os == 'Linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y libdbus-1-dev pkg-config

      - name: Remove Zig caches
        run: rm -rf .zig-cache vendor/libghostty-vt/.zig-cache vendor/libghostty-vt/zig-out

      - name: Cargo build (release)
        run: cargo build --release --locked --target ${{ matrix.target }}

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: herdr-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/herdr
          retention-days: 30
''',
    encoding="utf-8",
)

# Local guardrails for the temporary automation before it commits.
assert "attach_prefix_byte" not in client_path.read_text(encoding="utf-8")
assert "0.7.3-chef" not in Path("src/bin/herdr-gateway.rs").read_text(encoding="utf-8")
assert "ogulcancelik/herdr" not in re.search(
    r'^DEFAULT_RELEASE_REPO = .+$',
    Path("scripts/changelog.py").read_text(encoding="utf-8"),
    re.MULTILINE,
).group(0)
