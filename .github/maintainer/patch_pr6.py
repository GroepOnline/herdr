from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one occurrence, found {count}: {old[:100]!r}")
    file.write_text(text.replace(old, new, 1))


client_path = Path("src/client/mod.rs")
client = client_path.read_text()
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

#[derive(Debug)]
#[cfg(unix)]
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
client_path.write_text(client)

replace_once(
    "src/client/mod.rs",
    '''pub fn run_terminal_attach(terminal_id: String, takeover: bool) -> io::Result<()> {
    run_client_with_mode(
        RenderEncoding::TerminalAnsi,
        Some((terminal_id, takeover)),
        Some(AttachEscapeState::default()),
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

client = client_path.read_text()
default_count = client.count("AttachEscapeState::default()")
if default_count != 6:
    raise SystemExit(
        f"src/client/mod.rs: expected six attach tests using default state, found {default_count}"
    )
client_path.write_text(
    client.replace(
        "AttachEscapeState::default()",
        "AttachEscapeState::new(vec![0x02]).unwrap()",
    )
)

extra_tests = r'''
    #[cfg(unix)]
    #[test]
    fn attach_prefix_bytes_preserves_configured_terminal_sequences() {
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

'''
replace_once(
    "src/client/mod.rs",
    "    #[cfg(unix)]\n    #[test]\n    fn attach_escape_turns_wheel_into_scroll_action()",
    extra_tests
    + "    #[cfg(unix)]\n    #[test]\n    fn attach_escape_turns_wheel_into_scroll_action()",
)

app_path = Path("src/app/mod.rs")
app = app_path.read_text()
old_call = "        app.route_client_input(vec![0x02, b'\\t']);"
if app.count(old_call) != 2:
    raise SystemExit(
        f"src/app/mod.rs: expected two hardcoded prefix-tab calls, found {app.count(old_call)}"
    )
first_call = '''        let mut prefix_tab = crate::input::encode_terminal_key(
            crate::input::TerminalKey::new(app.state.prefix_code, app.state.prefix_mods),
            crate::input::KeyboardProtocol::Legacy,
        );
        prefix_tab.push(b'\t');
        app.route_client_input(prefix_tab.clone());'''
app = app.replace(old_call, first_call, 1)
app = app.replace(old_call, "        app.route_client_input(prefix_tab);", 1)
app_path.write_text(app)

replace_once(
    "src/app/actions.rs",
    "use tracing::{info, warn};",
    "use tracing::{debug, info, warn};",
)
replace_once(
    "src/app/actions.rs",
    '            warn!(pane = pane_id.raw(), "PaneDied for unknown pane");',
    '            debug!(pane = pane_id.raw(), "ignoring stale PaneDied event for unknown pane");',
)

ci_path = Path(".github/workflows/ci.yml")
ci = ci_path.read_text()
skip = " --skip route_client_input_prefix_tab_dispatches_global_last_pane"
if ci.count(skip) != 1:
    raise SystemExit(
        f".github/workflows/ci.yml: expected one temporary test skip, found {ci.count(skip)}"
    )
ci_path.write_text(ci.replace(skip, "", 1))
