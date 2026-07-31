from pathlib import Path

path = Path("src/ui/shell/integration_tests.rs")
text = path.read_text()
old = """    #[test]
    fn full_pipeline_renders_real_terminal_surface() {
"""
new = """    #[tokio::test]
    async fn full_pipeline_renders_real_terminal_surface() {
"""
if text.count(old) != 1:
    raise SystemExit(f"terminal integration test anchor count: {text.count(old)}")
path.write_text(text.replace(old, new))
