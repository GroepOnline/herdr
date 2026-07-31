from pathlib import Path

path = Path("src/app/input/mouse.rs")
text = path.read_text()
start = text.index("    fn new_shell_sidebar_miss_returns_none() {")
end = text.index("\n    #[test]", start)
section = text[start:end]
old = "        assert_eq!(app.mode, Mode::Terminal);"
new = "        assert_eq!(app.mode, Mode::Navigate);"
if section.count(old) != 1:
    raise SystemExit(f"sidebar miss expectation count: {section.count(old)}")
path.write_text(text[:start] + section.replace(old, new) + text[end:])
