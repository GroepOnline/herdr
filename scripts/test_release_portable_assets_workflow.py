from __future__ import annotations

import os
import re
import stat
import subprocess
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
WORKFLOW_PATH = REPO_ROOT / ".github/workflows/release-portable-assets.yml"

LINUX_BUILD_TOOLS_STEP = "Install build tools on Linux"

# Matches each `cat > "$toolchain/<name>" <<'EOF' ... EOF` heredoc block and
# captures its body, dedented by the indentation shared by the `cat` line, the
# body, and the closing `EOF`.
HEREDOC_RE = re.compile(
    r'^(?P<indent>[ \t]*)cat > "\$toolchain/(?P<name>\w+)" <<\'EOF\'\n'
    r'(?P<body>(?:.*\n)*?)'
    r'(?P=indent)EOF\n',
    re.MULTILINE,
)

# A workflow step starting at `- name: ...` continues until the next sibling
# list item at the same indent (another `- `), not nested children.
STEP_RE = re.compile(
    r'^(?P<indent>[ \t]*)- name: (?P<name>.+)\n'
    r'(?P<body>(?:(?!\1- ).*\n)*)',
    re.MULTILINE,
)


def _dedent_heredoc(indent: str, body: str) -> str:
    lines = []
    for line in body.split("\n"):
        if line.startswith(indent):
            lines.append(line[len(indent) :])
        else:
            lines.append(line)
    return "\n".join(lines)


def load_workflow_text() -> str:
    return WORKFLOW_PATH.read_text(encoding="utf-8")


def extract_named_step(workflow_text: str, step_name: str) -> str:
    """Return the full text of the first workflow step with the given name."""
    for match in STEP_RE.finditer(workflow_text):
        if match.group("name").strip() == step_name:
            return match.group(0)
    raise AssertionError(f"workflow step not found: {step_name!r}")


def extract_toolchain_wrapper_scripts(step_text: str) -> dict[str, str]:
    return {
        m.group("name"): _dedent_heredoc(m.group("indent"), m.group("body"))
        for m in HEREDOC_RE.finditer(step_text)
    }


class ReleasePortableAssetsWorkflowTests(unittest.TestCase):
    """Exercises the aarch64-musl Zig cc/cxx wrapper scripts and RUSTFLAGS
    override in the Linux build-tools step of release-portable-assets.yml."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow_text = load_workflow_text()
        cls.linux_build_tools_step = extract_named_step(
            cls.workflow_text, LINUX_BUILD_TOOLS_STEP
        )
        cls.wrappers = extract_toolchain_wrapper_scripts(cls.linux_build_tools_step)

    def test_cc_and_cxx_wrappers_are_present(self) -> None:
        self.assertIn("cc", self.wrappers)
        self.assertIn("cxx", self.wrappers)

    def test_cc_wrapper_uses_bash_with_strict_mode(self) -> None:
        body = self.wrappers["cc"]
        self.assertTrue(body.startswith("#!/usr/bin/env bash\n"))
        self.assertIn("set -euo pipefail", body)

    def test_cxx_wrapper_uses_bash_with_strict_mode(self) -> None:
        body = self.wrappers["cxx"]
        self.assertTrue(body.startswith("#!/usr/bin/env bash\n"))
        self.assertIn("set -euo pipefail", body)

    @staticmethod
    def _write_executable(path: Path, content: str) -> None:
        path.write_text(content, encoding="utf-8")
        mode = path.stat().st_mode
        path.chmod(mode | stat.S_IEXEC | stat.S_IXGRP | stat.S_IXOTH)

    def _run_wrapper(self, wrapper_name: str, args: list[str]) -> tuple[list[str], int]:
        """Runs the extracted wrapper script with the given args, against a
        fake `zig` on PATH that records its own argv to a log file. Returns
        the recorded argv (one entry per line) and the wrapper's exit code.
        """
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)

            wrapper_path = tmp_path / wrapper_name
            self._write_executable(wrapper_path, self.wrappers[wrapper_name])

            fake_bin = tmp_path / "bin"
            fake_bin.mkdir()
            log_path = tmp_path / "zig-invocation.log"
            self._write_executable(
                fake_bin / "zig",
                "#!/usr/bin/env bash\n"
                f'printf \'%s\\n\' "$@" > "{log_path}"\n'
                "exit 0\n",
            )

            env = dict(os.environ)
            env["PATH"] = f"{fake_bin}:{env['PATH']}"
            result = subprocess.run(
                [str(wrapper_path), *args],
                env=env,
                capture_output=True,
                text=True,
            )
            recorded = (
                log_path.read_text(encoding="utf-8").splitlines()
                if log_path.exists()
                else []
            )
            return recorded, result.returncode

    def test_cc_wrapper_strips_duplicate_rust_target_triple(self) -> None:
        recorded, code = self._run_wrapper(
            "cc",
            ["--target=aarch64-unknown-linux-musl", "-O2", "-c", "foo.c"],
        )
        self.assertEqual(code, 0)
        self.assertEqual(recorded, ["cc", "-target", "aarch64-linux-musl", "-O2", "-c", "foo.c"])

    def test_cxx_wrapper_strips_duplicate_rust_target_triple(self) -> None:
        recorded, code = self._run_wrapper(
            "cxx",
            ["--target=aarch64-unknown-linux-musl", "-O2", "-c", "foo.cpp"],
        )
        self.assertEqual(code, 0)
        self.assertEqual(recorded, ["c++", "-target", "aarch64-linux-musl", "-O2", "-c", "foo.cpp"])

    def test_cc_wrapper_preserves_other_args_when_duplicate_target_absent(self) -> None:
        recorded, code = self._run_wrapper("cc", ["-O2", "-c", "foo.c"])
        self.assertEqual(code, 0)
        self.assertEqual(recorded, ["cc", "-target", "aarch64-linux-musl", "-O2", "-c", "foo.c"])

    def test_cc_wrapper_with_no_args(self) -> None:
        recorded, code = self._run_wrapper("cc", [])
        self.assertEqual(code, 0)
        self.assertEqual(recorded, ["cc", "-target", "aarch64-linux-musl"])

    def test_cc_wrapper_strips_every_occurrence_of_duplicate_target(self) -> None:
        recorded, code = self._run_wrapper(
            "cc",
            [
                "--target=aarch64-unknown-linux-musl",
                "-O2",
                "--target=aarch64-unknown-linux-musl",
            ],
        )
        self.assertEqual(code, 0)
        self.assertEqual(recorded, ["cc", "-target", "aarch64-linux-musl", "-O2"])

    def test_cc_wrapper_preserves_argument_containing_a_space(self) -> None:
        recorded, code = self._run_wrapper("cc", ["-DFOO=bar baz", "-c"])
        self.assertEqual(code, 0)
        self.assertEqual(
            recorded, ["cc", "-target", "aarch64-linux-musl", "-DFOO=bar baz", "-c"]
        )

    def test_cc_wrapper_only_strips_exact_target_flag(self) -> None:
        # An argument that merely shares a prefix with the legacy Rust triple
        # flag must not be treated as a match and stripped.
        recorded, code = self._run_wrapper(
            "cc",
            ["--target=aarch64-unknown-linux-musl-extra", "-c"],
        )
        self.assertEqual(code, 0)
        self.assertEqual(
            recorded,
            [
                "cc",
                "-target",
                "aarch64-linux-musl",
                "--target=aarch64-unknown-linux-musl-extra",
                "-c",
            ],
        )

    def test_cxx_wrapper_only_strips_exact_target_flag(self) -> None:
        recorded, code = self._run_wrapper(
            "cxx",
            ["--target=aarch64-unknown-linux-musl-extra", "-c"],
        )
        self.assertEqual(code, 0)
        self.assertEqual(
            recorded,
            [
                "c++",
                "-target",
                "aarch64-linux-musl",
                "--target=aarch64-unknown-linux-musl-extra",
                "-c",
            ],
        )

    def test_cc_wrapper_propagates_zig_failure_exit_code(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)

            wrapper_path = tmp_path / "cc"
            self._write_executable(wrapper_path, self.wrappers["cc"])

            fake_bin = tmp_path / "bin"
            fake_bin.mkdir()
            self._write_executable(fake_bin / "zig", "#!/usr/bin/env bash\nexit 7\n")

            env = dict(os.environ)
            env["PATH"] = f"{fake_bin}:{env['PATH']}"
            result = subprocess.run(
                [str(wrapper_path), "-c", "foo.c"],
                env=env,
                capture_output=True,
                text=True,
            )
            self.assertEqual(result.returncode, 7)

    def test_cc_wrapper_fails_closed_when_zig_is_missing(self) -> None:
        # `set -euo pipefail` plus the unresolved `exec zig` must produce a
        # non-zero exit rather than silently doing nothing.
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)

            wrapper_path = tmp_path / "cc"
            self._write_executable(wrapper_path, self.wrappers["cc"])

            env = dict(os.environ)
            env["PATH"] = "/nonexistent"
            result = subprocess.run(
                [str(wrapper_path), "-c", "foo.c"],
                env=env,
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(result.returncode, 0)

    def test_rustflags_disable_self_contained_linking_for_aarch64_musl_target(self) -> None:
        self.assertIn(
            'echo "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS=-C link-self-contained=no"',
            self.linux_build_tools_step,
        )

    def test_rustflags_env_line_is_written_immediately_after_linker_env_line(self) -> None:
        lines = self.linux_build_tools_step.splitlines()
        linker_idx = next(
            i
            for i, line in enumerate(lines)
            if "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=" in line
        )
        self.assertIn(
            "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS=-C link-self-contained=no",
            lines[linker_idx + 1],
        )

    def test_extraction_ignores_wrappers_outside_linux_build_tools_step(self) -> None:
        # A same-named wrapper elsewhere in the workflow must not be selected.
        polluted = (
            self.workflow_text
            + "\n"
            + "      - name: Decoy step\n"
            + "        run: |\n"
            + "          cat > \"$toolchain/cc\" <<'EOF'\n"
            + "          #!/usr/bin/env bash\n"
            + "          exec false\n"
            + "          EOF\n"
        )
        step = extract_named_step(polluted, LINUX_BUILD_TOOLS_STEP)
        wrappers = extract_toolchain_wrapper_scripts(step)
        self.assertIn("set -euo pipefail", wrappers["cc"])
        self.assertNotIn("exec false", wrappers["cc"])


if __name__ == "__main__":
    unittest.main()
