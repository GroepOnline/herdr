from pathlib import Path

path = Path(".github/pr79_structural_fixes.py")
text = path.read_text()
old = '''replace(
    "src/ui/shell/mod.rs",
    "        app.sidebar_width,",
    "        app.sidebar_width\\n"
    "            .clamp(app.sidebar_min_width, app.sidebar_max_width),",
    1,
)
'''
new = old.replace("    1,\n)", "    2,\n)")
if text.count(old) != 1:
    raise SystemExit("sidebar width patch anchor not found exactly once")
path.write_text(text.replace(old, new))
