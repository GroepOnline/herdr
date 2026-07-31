#!/usr/bin/env python3
"""One-shot CI patch: disable fragile Zig cache reuse on Windows lint."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PATH = ROOT / ".github/workflows/ci.yml"

text = PATH.read_text(encoding="utf-8")
old = '''      - name: Install Zig
        uses: mlugg/setup-zig@d1434d08867e3ee9daa34448df10607b98908d29 # v2.2.1
        with:
          version: ${{ env.ZIG_VERSION }}

      - name: Cargo clippy on Windows
        shell: bash
        run: cargo clippy --bin herdr --locked -- -D warnings
'''
new = '''      - name: Install Zig
        uses: mlugg/setup-zig@d1434d08867e3ee9daa34448df10607b98908d29 # v2.2.1
        with:
          version: ${{ env.ZIG_VERSION }}
          # Restored Windows Zig caches have produced missing generated files.
          # Keep this lane deterministic; Rust's Cargo cache remains enabled.
          use-cache: false

      - name: Remove Zig build outputs
        shell: bash
        run: rm -rf .zig-cache vendor/libghostty-vt/.zig-cache vendor/libghostty-vt/zig-out

      - name: Cargo clippy on Windows
        shell: bash
        run: cargo clippy --bin herdr --locked -- -D warnings
'''
if text.count(old) != 1:
    raise SystemExit(f"expected one Windows Zig block, found {text.count(old)}")
PATH.write_text(text.replace(old, new, 1), encoding="utf-8")
print("Windows Zig cache patch applied")
