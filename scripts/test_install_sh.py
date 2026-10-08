from __future__ import annotations

import hashlib
import json
import os
import stat
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
INSTALLER = ROOT / "website/install.sh"


class InstallScriptTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.fake_bin = self.root / "fake-bin"
        self.install_dir = self.root / "install-bin"
        self.fake_bin.mkdir()
        self.install_dir.mkdir()

        self.binary = self.root / "release-herdr"
        self.binary.write_bytes(b"verified herdr release\n")
        self.manifest = self.root / "latest.json"
        self.write_manifest(hashlib.sha256(self.binary.read_bytes()).hexdigest())
        self.write_fake_commands()

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def write_manifest(self, checksum: str) -> None:
        self.manifest.write_text(
            json.dumps(
                {
                    "version": "1.2.3",
                    "assets": {
                        "linux-x86_64": {
                            "url": "https://downloads.example/herdr-linux-x86_64",
                            "sha256": checksum,
                        }
                    },
                },
                indent=2,
            )
            + "\n",
            encoding="utf-8",
        )

    def write_executable(self, name: str, body: str) -> None:
        path = self.fake_bin / name
        path.write_text("#!/bin/sh\nset -eu\n" + body, encoding="utf-8")
        path.chmod(0o755)

    def write_fake_commands(self) -> None:
        self.write_executable(
            "uname",
            """case "${1:-}" in
  -s) printf '%s\\n' Linux ;;
  -m) printf '%s\\n' x86_64 ;;
  *) exit 2 ;;
esac
""",
        )
        self.write_executable(
            "curl",
            """output=""
url=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    -o) shift; output="$1" ;;
    -*) ;;
    *) url="$1" ;;
  esac
  shift
done
case "$url" in
  */latest.json)
    if [ -n "$output" ]; then cp "$TEST_MANIFEST" "$output"; else cat "$TEST_MANIFEST"; fi
    ;;
  https://downloads.example/herdr-linux-x86_64)
    [ -n "$output" ] || exit 2
    cp "$TEST_BINARY" "$output"
    ;;
  *)
    printf 'unexpected URL: %s\\n' "$url" >&2
    exit 22
    ;;
esac
""",
        )

    def environment(self) -> dict[str, str]:
        env = os.environ.copy()
        env.update(
            {
                "PATH": f"{self.fake_bin}:{self.install_dir}:{env['PATH']}",
                "HOME": str(self.root / "home"),
                "HERDR_BASE_URL": "https://manifest.example",
                "HERDR_INSTALL_DIR": str(self.install_dir),
                "TEST_MANIFEST": str(self.manifest),
                "TEST_BINARY": str(self.binary),
            }
        )
        return env

    def run_installer(self) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["sh", str(INSTALLER)],
            cwd=ROOT,
            env=self.environment(),
            text=True,
            capture_output=True,
            check=False,
        )

    def test_shell_syntax(self) -> None:
        subprocess.run(["sh", "-n", str(INSTALLER)], check=True)

    def test_verified_binary_is_installed_atomically(self) -> None:
        result = self.run_installer()

        self.assertEqual(result.returncode, 0, result.stderr)
        installed = self.install_dir / "herdr"
        self.assertEqual(installed.read_bytes(), self.binary.read_bytes())
        self.assertTrue(installed.stat().st_mode & stat.S_IXUSR)
        self.assertIn("verified SHA-256", result.stdout)
        self.assertEqual(list(self.install_dir.glob(".herdr-install.*")), [])

    def test_checksum_mismatch_preserves_existing_binary(self) -> None:
        installed = self.install_dir / "herdr"
        installed.write_bytes(b"existing trusted binary\n")
        installed.chmod(0o755)
        self.write_manifest("0" * 64)

        result = self.run_installer()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("checksum mismatch", result.stderr)
        self.assertEqual(installed.read_bytes(), b"existing trusted binary\n")
        self.assertEqual(list(self.install_dir.glob(".herdr-install.*")), [])

    def test_missing_checksum_is_rejected(self) -> None:
        data = json.loads(self.manifest.read_text(encoding="utf-8"))
        del data["assets"]["linux-x86_64"]["sha256"]
        self.manifest.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")

        result = self.run_installer()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("invalid SHA-256", result.stderr)
        self.assertFalse((self.install_dir / "herdr").exists())


if __name__ == "__main__":
    unittest.main()
