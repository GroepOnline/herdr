from pathlib import Path

path = Path("src/app/input/mouse.rs")
text = path.read_text()

old = """    fn new_shell_new_tab_click_requests_new_tab() {
        let mut app = app_for_new_shell_test();
        let layout = app.new_shell_layout.clone().unwrap();
"""
new = """    fn new_shell_new_tab_click_requests_new_tab() {
        let mut app = app_for_new_shell_test();
        app.prompt_new_tab_name = false;
        let layout = app.new_shell_layout.clone().unwrap();
"""
if text.count(old) != 1:
    raise SystemExit(f"new-tab test anchor count: {text.count(old)}")
text = text.replace(old, new)

old = """        assert!(result.is_none());
        assert_eq!(app.mode, Mode::Terminal);
    }
}
"""
new = """        assert!(result.is_none());
        assert_eq!(app.mode, Mode::Navigate);
    }
}
"""
if text.count(old) != 1:
    raise SystemExit(f"dead-space test anchor count: {text.count(old)}")
path.write_text(text.replace(old, new))
