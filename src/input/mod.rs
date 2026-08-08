mod encode;
mod model;
mod parse;

// Some key encoders are only referenced on platform-specific input paths.
#[allow(unused_imports)]
pub use encode::{
    encode_cursor_key, encode_key, encode_mouse_button, encode_mouse_scroll, encode_terminal_key,
};
pub use model::{
    host_modify_other_keys_mode, KeyboardProtocol, MouseProtocolEncoding, MouseProtocolMode,
    TerminalKey,
};
pub use parse::parse_terminal_key_sequence;
