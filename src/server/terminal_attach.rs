pub(crate) fn paste_payload_for_runtime(
    runtime: &crate::terminal::TerminalRuntime,
    text: &str,
) -> String {
    let bracketed = runtime
        .input_state()
        .map(|state| state.bracketed_paste)
        .unwrap_or(false);
    crate::raw_input::encode_paste_for_mode(text, bracketed)
}

/// Rewrite host input for a direct terminal-attach session.
///
/// Complete host bracketed-paste sequences are stripped and only re-wrapped when
/// the attached pane has enabled DECSET 2004. Other input is forwarded unchanged.
pub(crate) fn rewrite_attach_input_bytes<'a>(
    runtime: &crate::terminal::TerminalRuntime,
    data: &'a [u8],
) -> std::borrow::Cow<'a, [u8]> {
    let bracketed = runtime
        .input_state()
        .map(|state| state.bracketed_paste)
        .unwrap_or(false);
    crate::raw_input::rewrite_host_paste_bytes(data, bracketed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn rewrite_attach_input_strips_host_paste_when_pane_unset() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime");
        let _runtime_guard = rt.enter();
        let (runtime, _rx) = crate::terminal::TerminalRuntime::test_with_channel(80, 24);
        let rewritten =
            rewrite_attach_input_bytes(&runtime, b"\x1b[200~npm run lint:check\x1b[201~");
        assert_eq!(rewritten.as_ref(), b"npm run lint:check");
        drop(runtime);
        drop(_runtime_guard);
        rt.shutdown_timeout(Duration::from_millis(100));
    }

    #[test]
    fn rewrite_attach_input_keeps_host_paste_when_pane_enabled() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime");
        let _runtime_guard = rt.enter();
        let (runtime, _rx) = crate::terminal::TerminalRuntime::test_with_channel(80, 24);
        runtime.test_process_pty_bytes(b"\x1b[?2004h");
        let host = b"\x1b[200~hello\x1b[201~";
        let rewritten = rewrite_attach_input_bytes(&runtime, host);
        assert_eq!(rewritten.as_ref(), host);
        drop(runtime);
        drop(_runtime_guard);
        rt.shutdown_timeout(Duration::from_millis(100));
    }
}
