---
name: herdr-local-verify
description: >-
  CI-only validation for Herdr — no local cargo/rust/zig/just on Cloud VM.
  Use gh pr checks and gh run view --log-failed instead of just test/check.
  Optional download of ci-smoke-herdr-linux-x86_64 artifact for headless CLI.
---

# Herdr local verify (CI-only)

Cloud VM and hooked shells **deny** local Rust/Zig builds. Treat GitHub Actions as the compile/test source of truth.

## Forbidden locally

- `cargo *`, `rustc`, `rustup`, `cargo-nextest`, `clippy`, `rustfmt`
- `zig build`
- `just test|check|lint|ci|build|fmt-check|windows-lint|build-libghostty-vt`

Hook: `.cursor/hooks/deny-rust-builds.sh` via `beforeShellExecution`.

## Required validation loop

1. Commit and push the branch
2. `gh pr checks --json name,bucket,state,workflow,link`
3. On failure, derive the run id from the check `link` (`gh pr checks` has no `id` field), then read the log:

   ```bash
   run_id="$(gh pr checks --json bucket,link \
     --jq 'first(.[] | select(.bucket == "fail") | .link | capture("/runs/(?<id>[0-9]+)") | .id)')"
   gh run view "$run_id" --log-failed
   ```

4. Fix one actionable failure; push again

Quality gate: `CI / Quality gate` (see `.github/quality-ci.md`).

## Headless runtime (no compile)

Download CI smoke binary from a green run:

```bash
run_id=<green-run-id>
gh run download "$run_id" -n ci-smoke-herdr-linux-x86_64 -D /tmp/herdr-bin
chmod +x /tmp/herdr-bin/herdr-linux-x86_64

export HOME=/tmp/herdr-home
/tmp/herdr-bin/herdr-linux-x86_64 server &
server_pid=$!
trap 'kill "$server_pid" 2>/dev/null || true' EXIT

# The socket appears only once the server is up; never issue client commands before it exists.
for _ in $(seq 50); do
  [[ -S "$HOME/.config/herdr/herdr.sock" ]] && break
  sleep 0.2
done

/tmp/herdr-bin/herdr-linux-x86_64 workspace create --cwd "$HOME" --focus
```

Clear `HERDR_SOCKET_PATH` / `HERDR_CLIENT_SOCKET_PATH` when using debug or downloaded binaries.

## Delegate

| Task | Agent / skill |
| --- | --- |
| CI triage (read-only) | `.cursor/agents/herdr-quality-ci-diagnoser.md` |
| CI fix loop | `.cursor/agents/herdr-quality-ci-remediator.md` |
| Quality CI playbook | `.cursor/skills/herdr-quality-ci-remediation/` |

## Stop conditions

- Do not bypass the deny hook or install just/cargo-nextest locally on Cloud VM.
- Do not claim `just check` passed without CI evidence.
- TUI verification needs a real TTY; use socket API/CLI headlessly in cloud.
