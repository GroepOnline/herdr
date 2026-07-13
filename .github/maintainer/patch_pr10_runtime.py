from __future__ import annotations

from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one match, found {count}: {old[:100]!r}")
    file.write_text(text.replace(old, new, 1), encoding="utf-8")


PATH = "src/client/mod.rs"

replace_once(
    PATH,
    "use std::time::Duration;\n",
    "use std::time::Duration;\n#[cfg(unix)]\nuse std::time::Instant;\n",
)

replace_once(
    PATH,
    "static RECEIVED_KITTY_GRAPHICS_IDS: OnceLock<Mutex<HashSet<u32>>> = OnceLock::new();\n",
    "static RECEIVED_KITTY_GRAPHICS_IDS: OnceLock<Mutex<HashSet<u32>>> = OnceLock::new();\n\n#[cfg(unix)]\nconst ATTACH_PREFIX_TIMEOUT: Duration = Duration::from_millis(75);\n",
)

replace_once(
    PATH,
    '''struct AttachEscapeState {
    prefix: Vec<u8>,
    pending_prefix: bool,
    buffered_input: Vec<u8>,
}''',
    '''struct AttachEscapeState {
    prefix: Vec<u8>,
    pending_prefix: bool,
    buffered_input: Vec<u8>,
    partial_since: Option<Instant>,
}''',
)

replace_once(
    PATH,
    '''        Ok(Self {
            prefix,
            pending_prefix: false,
            buffered_input: Vec::new(),
        })''',
    '''        Ok(Self {
            prefix,
            pending_prefix: false,
            buffered_input: Vec::new(),
            partial_since: None,
        })''',
)

replace_once(
    PATH,
    '''    #[cfg(unix)]
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
    }''',
    '''    #[cfg(unix)]
    fn filter_input(
        &mut self,
        data: Vec<u8>,
        viewport_rows: u16,
        mouse_scroll_lines: usize,
    ) -> AttachInputAction {
        self.buffered_input.extend_from_slice(&data);
        self.process_buffer(
            Instant::now(),
            false,
            viewport_rows,
            mouse_scroll_lines,
        )
    }

    #[cfg(unix)]
    fn flush_expired(
        &mut self,
        now: Instant,
        viewport_rows: u16,
        mouse_scroll_lines: usize,
    ) -> AttachInputAction {
        let Some(partial_since) = self.partial_since else {
            return AttachInputAction::None;
        };
        if now.saturating_duration_since(partial_since) < ATTACH_PREFIX_TIMEOUT {
            return AttachInputAction::None;
        }
        self.process_buffer(now, true, viewport_rows, mouse_scroll_lines)
    }

    #[cfg(unix)]
    fn process_buffer(
        &mut self,
        now: Instant,
        flush_partial: bool,
        viewport_rows: u16,
        mouse_scroll_lines: usize,
    ) -> AttachInputAction {
        let mut output = Vec::with_capacity(self.buffered_input.len() + self.prefix.len());
        let mut cursor = 0usize;
        let mut waiting_on_partial = false;

        while cursor < self.buffered_input.len() {
            let remaining_len = self.buffered_input.len() - cursor;

            if self.pending_prefix {
                if self.buffered_input[cursor] == b'q' {
                    self.buffered_input.clear();
                    self.pending_prefix = false;
                    self.partial_since = None;
                    return AttachInputAction::Detach;
                }

                if self.buffered_input[cursor..].starts_with(&self.prefix) {
                    cursor += self.prefix.len();
                    output.extend_from_slice(&self.prefix);
                    self.pending_prefix = false;
                    continue;
                }

                if remaining_len < self.prefix.len()
                    && self.prefix.starts_with(&self.buffered_input[cursor..])
                {
                    if flush_partial {
                        output.extend_from_slice(&self.prefix);
                        output.extend_from_slice(&self.buffered_input[cursor..]);
                        cursor = self.buffered_input.len();
                        self.pending_prefix = false;
                    } else {
                        waiting_on_partial = true;
                    }
                    break;
                }

                output.extend_from_slice(&self.prefix);
                output.push(self.buffered_input[cursor]);
                cursor += 1;
                self.pending_prefix = false;
                continue;
            }

            if self.buffered_input[cursor..].starts_with(&self.prefix) {
                cursor += self.prefix.len();
                self.pending_prefix = true;
                continue;
            }

            if remaining_len < self.prefix.len()
                && self.prefix.starts_with(&self.buffered_input[cursor..])
            {
                if flush_partial {
                    output.extend_from_slice(&self.buffered_input[cursor..]);
                    cursor = self.buffered_input.len();
                } else {
                    waiting_on_partial = true;
                }
                break;
            }

            output.push(self.buffered_input[cursor]);
            cursor += 1;
        }

        if cursor > 0 {
            self.buffered_input.drain(..cursor);
        }
        self.partial_since = waiting_on_partial.then_some(now);

        if output.is_empty() {
            AttachInputAction::None
        } else if let Some(action) =
            attach_scroll_action(&output, viewport_rows, mouse_scroll_lines)
        {
            action
        } else {
            AttachInputAction::Forward(output)
        }
    }''',
)

replace_once(
    PATH,
    "            ClientLoopEvent::Timer => {}\n",
    '''            ClientLoopEvent::Timer => {
                #[cfg(unix)]
                {
                    let viewport_rows = state.reported_size.1;
                    let mouse_scroll_lines = state.mouse_scroll_lines;
                    if let Some(attach_escape) = &mut state.attach_escape {
                        match attach_escape.flush_expired(
                            Instant::now(),
                            viewport_rows,
                            mouse_scroll_lines,
                        ) {
                            AttachInputAction::Forward(data) => {
                                if !data.is_empty() {
                                    let msg = ClientMessage::Input { data };
                                    if let Err(e) = write_to_server(&mut write_stream, &msg) {
                                        return Err(ClientError::ConnectionLost(e));
                                    }
                                }
                            }
                            AttachInputAction::Scroll {
                                source,
                                direction,
                                lines,
                                column,
                                row,
                                modifiers,
                            } => {
                                let msg = ClientMessage::AttachScroll {
                                    source,
                                    direction,
                                    lines,
                                    column,
                                    row,
                                    modifiers,
                                };
                                if let Err(e) = write_to_server(&mut write_stream, &msg) {
                                    return Err(ClientError::ConnectionLost(e));
                                }
                            }
                            AttachInputAction::Detach => {
                                let _ = write_to_server(&mut write_stream, &ClientMessage::Detach);
                                return Ok(());
                            }
                            AttachInputAction::None => {}
                        }
                    }
                }
            }
''',
)

new_tests = r'''
    #[cfg(unix)]
    #[test]
    fn attach_escape_flushes_a_standalone_escape_after_multibyte_prefix_timeout() {
        let mut escape = AttachEscapeState::new(b"\x1b[24~".to_vec()).unwrap();
        assert!(matches!(
            escape.filter_input(vec![0x1b], 24, 3),
            AttachInputAction::None
        ));

        let expired = Instant::now()
            .checked_sub(ATTACH_PREFIX_TIMEOUT + Duration::from_millis(1))
            .unwrap();
        escape.partial_since = Some(expired);
        match escape.flush_expired(Instant::now(), 24, 3) {
            AttachInputAction::Forward(bytes) => assert_eq!(bytes, vec![0x1b]),
            other => panic!("expected timed-out Escape to be forwarded, got {other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn attach_escape_processes_large_pastes_without_front_removal() {
        let mut escape = AttachEscapeState::new(vec![0x01]).unwrap();
        let paste = vec![b'x'; 64 * 1024];
        match escape.filter_input(paste.clone(), 24, 3) {
            AttachInputAction::Forward(bytes) => assert_eq!(bytes, paste),
            other => panic!("expected paste bytes to be forwarded, got {other:?}"),
        }
        assert!(escape.buffered_input.is_empty());
    }

'''
replace_once(
    PATH,
    "    #[cfg(unix)]\n    #[test]\n    fn attach_escape_turns_wheel_into_scroll_action()",
    new_tests
    + "    #[cfg(unix)]\n    #[test]\n    fn attach_escape_turns_wheel_into_scroll_action()",
)

text = Path(PATH).read_text(encoding="utf-8")
if ".remove(0)" in text:
    raise SystemExit("src/client/mod.rs: front-removal remains after patch")
if "flush_expired" not in text or "ATTACH_PREFIX_TIMEOUT" not in text:
    raise SystemExit("src/client/mod.rs: timeout handling was not installed")
