# herdr

Terminal based agent runtime for coding agents.

## Scope and Audience

These instructions are layered.

- Unless a section explicitly says it is maintainer-only, local-machine-only, or
  external-contributor-only, treat it as universal project guidance.
- Universal project rules apply to every agent working on Herdr, including forks.
- Maintainer workflow applies only when the acting GitHub account is
  `OnlineChefGroep` or Can explicitly says this is maintainer work. If the account
  is not `OnlineChefGroep`, skip maintainer workflow and follow the external
  contributor guardrail instead.
- Local Can machine workflow applies only on Can's own workstation or Windows
  VM setup, for example when `/home/can/Projects/herdr`, `HERDR_ENV=1`, or the
  `windows-wirt` SSH alias exists. If those facts are not true, skip local
  machine workflow.
- External contributor guardrail applies whenever the acting GitHub account is
  not `OnlineChefGroep`, the work is happening in a fork, or the account cannot be
  determined.

## Universal Project Rules

### Principles

- **State is separated from runtime.** `AppState` is pure data, testable without PTYs or async. `PaneState` is separate from `PaneRuntime`. Workspace logic doesn't need real terminals.
- **Render is pure.** `compute_view()` handles geometry and mutations. `render()` takes `&AppState` and only draws. Never mutate state during render.
- **No god objects.** If a module is doing too many things, split it. `app/` is already split into state, actions, and input. Keep it that way.
- **Platform code is isolated.** OS-specific behavior lives in the matching `src/platform/<os>.rs` file, with only shared traits, types, wrappers, and testable contracts in `src/platform/mod.rs`. Core modules don't have `#[cfg(target_os)]`.
- **Detection is decoupled.** The detector reads a screen snapshot, never touches the parser or viewport state.
- **Screen detection is evidence-based.** When changing `src/detect/manifests/`, first capture the relevant bottom-buffer state with `herdr agent read <pane> --source detection --format text` and, when styling or alternate screen behavior matters, `--format ansi`. Decide which visible controls are invariant, which are alternatives, and encode them as explicit AND/OR gates. Do not match whole-pane incidental text, and do not use the user-visible viewport for agent status because users can scroll it.
- **UI patterns should be reused.** Herdr is a mouse-first TUI. New dialogs, onboarding, settings, and post-update flows should follow the existing UI/UX language and interaction patterns instead of inventing one-off screens. Prefer reusing existing modal/screen structure, affordances, and close actions so the app feels consistent.

### Runtime/client boundary guardrail

Herdr is migrating toward a server-owned runtime protocol with the TUI as one client. New work should not deepen the current server/TUI coupling.

Before adding state, API fields, events, commands, or socket messages, classify the feature:

- Shared runtime/session fact: belongs in server state and should be exposed through the JSON API/event path when practical.
- TUI presentation state: belongs only in the TUI/client layer.

Do not add new shared behavior that only works through the private TUI client socket. Use neutral server/API names, not UI-surface names like sidebar, row, card, or widget.

Examples:

- Pane/agent metadata, process state, terminal state, events: server/runtime.
- Sidebar layout, token placement, colors, selection, modals, mouse/viewport state: TUI/client.
- Workspace/tab/pane remain shared session organization for now, but avoid making them mandatory identity for unrelated runtime features.

## Maintainer Workflow

This section applies only when the acting GitHub account is `OnlineChefGroep` or
Can explicitly says this is maintainer work. If the acting account is not
`OnlineChefGroep`, skip this section and follow the external contributor guardrail.

### Multi-agent isolation

Read-only investigation can happen in the shared checkout.

Small changes or small tasks are fine in the default main worktree. If you find unrelated implementation changes already in progress in the main worktree, use a dedicated worktree instead. Use a dedicated worktree for bigger features too.

Use this layout:

- shared integration checkout: `../herdr`
- task worktrees: `../herdr-worktrees/<task-slug>`
- task branches: `issue/<id>-<slug>` when an issue exists

Do all code edits, tests, and validation inside the task worktree.

Commit on the task branch in that worktree.

When the change is ready, fast-forward the shared checkout at `../herdr` to the task branch commit, then push `origin/main` from `../herdr`. Do not treat the task branch as the final landing branch.

If the current session is already inside an isolated task worktree, keep using it. Do not create nested worktrees.

Before committing, propose the commit message and get alignment.

After the change is integrated, remove the task worktree and delete the task branch locally and remotely.

## Testing

Use `just` recipes by default instead of invoking cargo or scripts directly.

```bash
just test               # cargo nextest + maintenance script tests
just check              # formatting check + cargo nextest + maintenance script tests
```

Run `just check` before committing unless Can explicitly accepts narrower validation. Do not bypass failing checks; fix the failure or explain exactly why a narrower check is enough.

### Quality CI

Quality CI is the required parallel gate for PRs. The single required check is `CI / Quality gate` (aggregates Lint, Test, Maintenance, Windows lint, Release metadata, and Release smoke). Spec: `.github/quality-ci.md`.

Mechanical failures are auto-committed by `quality-autofix.yml` (`ci: autofix mechanical quality`). Non-mechanical failures get one sticky `<!-- herdr-quality-remediation -->` PR comment, the `quality-remediation` label, and a `repository_dispatch` event `herdr-quality-remediation` for Cursor Automations / cloud agents. Agents should inspect `gh run view <run_id> --log-failed`, fix the real failure, push to the PR branch, and validate with `gh pr checks <pr> --watch`. This Cloud VM forbids local Cargo builds; use GitHub Actions as the source of truth. Opt out with PR label `ci-autofix-disabled`.

Agent entrypoints:

- Skill: `.cursor/skills/herdr-quality-ci-remediation/` (Codex mirror: `.codex/skills/herdr-quality-ci-remediation/`)
- Subagent (fix): `.cursor/agents/herdr-quality-ci-remediator.md`
- Subagent (read-only triage): `.cursor/agents/herdr-quality-ci-diagnoser.md`

Unit tests live next to the code (`#[cfg(test)] mod tests`). New `AppState` or `Workspace` behavior should be testable with `AppState::test_new()` and `Workspace::test_new()` without PTYs.

For broad refactors or release-risk regressions, classify the risk before editing. Treat changes as refactor-risk when they touch two or more core surfaces, persisted state, protocol/API IDs, workspace/tab/pane identity, restore/handoff, agent detection authority, or UI/input state projection. Before moving code, identify the protected behavior and add or name characterization tests. Identity/state refactors should use the test-only invariants `AppState::assert_invariants_for_test()` or `Workspace::assert_invariants_for_test()` with adversarial state from `AppState::test_with_adversarial_identity_state()` or `Workspace::test_adversarial_identity_state()`. Run a roundtable for broad refactors and release-risk regressions, not for routine local fixes.

When testing a new Herdr build from inside an existing Herdr session, use
`cargo run -- ...` and clear inherited Herdr socket overrides so the debug
binary talks to the debug `herdr.chefgroep.nl` server instead of the installed stable
server:

```bash
env -u HERDR_SOCKET_PATH -u HERDR_CLIENT_SOCKET_PATH cargo run -- <command>
```

## Local Can Machine Workflow

This section applies only on Can's workstation or Windows VM setup. If the
acting GitHub account is not `OnlineChefGroep`, skip this section and follow the
external contributor guardrail.

### Windows VM validation

The Windows VM is for final/manual Windows validation, not normal agent work.
Connect to it with the `windows-wirt` SSH alias.

Use the single reusable checkout at `C:\work\repo`. Do not create additional
persistent Herdr clones or worktrees on the VM. The Windows account is already
named `herdr`, so avoid paths like `C:\Users\herdr\herdr`.

Before validating a fix on Windows, sync or apply the Linux worktree changes
into `C:\work\repo`, then run the needed Windows build or test commands there.
Reuse the shared Rust caches under `C:\Users\herdr\.cargo` and
`C:\Users\herdr\.rustup`. Do not use WSL on the VM. The VM may have a newer
Zig on `PATH`; Herdr currently requires Zig 0.15.2, so set
`$env:ZIG = "C:\Users\herdr\zig-0.15.2\zig.exe"` before running Cargo commands
that build the vendored libghostty-vt.

After validation, leave `C:\work\repo` clean. Remove temporary files and delete
`C:\work\repo\target` when disk space is tight, but keep the shared Cargo and
Rustup caches. Unless Can explicitly asks to keep the patched tree for more
manual testing, reset `C:\work\repo` back to a clean checkout before finishing.

## Agent Detection Updates

Agent detection changes should use the manifest hot-reload loop. Can drives the real agent UI into the target state, then you read the pane with `herdr agent read <pane> --source detection --format text` and inspect matching with `herdr agent explain <pane> --json`. Update the bundled manifest in `src/detect/manifests/<agent>.toml`, copy that manifest to the local override path at `~/.config/herdr/agent-detection/<agent>.toml`, then run `herdr server reload-agent-manifests`. Can verifies the live pane state, and once the rule is correct, remove the local override so the committed bundled manifest remains the source of truth.

Do not add large agent-specific full-screen fixture suites for routine manifest tuning. Keep Rust tests focused on manifest parsing, rule semantics, skip-state semantics, source precedence, cache reload behavior, and update flow. Use live pane reads for agent-specific screen evidence.

## Vendored libghostty-vt

`vendor/libghostty-vt.vendor.json` records the upstream source commit currently vendored.

Local patches on top of the vendored source must be tracked in `vendor/libghostty-vt.patches.md` and stored as patch files under `vendor/patches/libghostty-vt/`. Each entry should say why the patch exists, the Herdr issue, upstream PR/discussion, vendored base commit, touched files, verification, and the exact removal condition.

When updating libghostty-vt, check every active patch in `vendor/libghostty-vt.patches.md`. If the new upstream commit contains the fix, remove the local patch and index entry, then rerun the listed verification. If not, reapply the patch on top of the new vendored source.

`just check` runs maintenance tests that verify local libghostty-vt patch files are listed in the index and reverse-apply cleanly against the vendored tree. Do not leave a patch file untracked or an indexed patch unapplied.

## Docs

Stable public docs live in `website/src/content/docs/`. They are the currently released herdr.chefgroep.nl docs. Do not document unreleased behavior there during normal feature or fix work.

Unreleased docs live in `docs/next/website/src/content/docs/`. Update those when a user-facing change needs docs before the next release. `docs/next/README.md` and `docs/next/CHANGELOG.md` stage root README and changelog changes.

The website build runs `website/scripts/prepare-docs.mjs`. It keeps stable docs at `/docs/` and generates preview docs at `/docs/preview/` from `docs/next/website/src/content/docs/`. Do not edit generated `website/src/content/docs/preview/`.

During release review, copy approved next docs into the stable docs and run `just release-docs-check`. Normal feature/fix work should not edit root `README.md`, root `CHANGELOG.md`, or `website/latest.json` unless explicitly requested.

Put local PRDs, planning notes, and exploratory specs under `.local/prd/`; `.local/` is ignored and locally controlled.

## Commit Style

Use lowercase conventional commits, no emojis, and no AI co-author lines. Commit subjects feed preview release notes, so keep them descriptive.

Before committing, propose the commit message and get alignment.

When a normal feature or fix commit relates to a GitHub issue, add a commit body line `refs #<issue-number>` after the subject:

```text
fix: handle pane focus

refs #82
```

Do not use GitHub closing keywords like `fixes #<issue-number>`, `closes #<issue-number>`, or `resolves #<issue-number>` in normal commits. `main` contains unreleased work; release CI closes referenced issues after the GitHub Release is created.

## Code Conventions

- Rust: no `unwrap()` in production code. Use `tracing` for logging. Use `#[allow]` only with a comment explaining why.
- Rust platform-specific code must be compile-gated. Put OS APIs and substantial OS behavior in `src/platform/`; when platform checks are needed elsewhere, use `#[cfg(windows)]`, `#[cfg(unix)]`, or target-specific `#[cfg(...)]` on imports, fields, functions, impls, and match arms so Windows-only code does not compile into Unix builds and Unix-only code does not compile into Windows builds. Use `cfg!(...)` only for pure cross-platform policy constants whose branches both compile on every target.
- Don't add dependencies without a reason. Check whether existing dependencies cover the need first.
- Integration asset versions (`HERDR_INTEGRATION_VERSION` markers and matching `*_INTEGRATION_VERSION` constants) are migration versions relative to the latest released tag, not per-commit counters on `main`. If an integration asset changes multiple times between releases, bump it once from the version in the latest release.
- When changing the server/client wire protocol, compare `src/protocol/wire.rs::PROTOCOL_VERSION` against the latest released tag. Bump it only if the current source protocol is not already greater than the latest released protocol. Update hardcoded protocol expectations and manual protocol fixtures in tests.

## Release Channels

This section is maintainer-only for release actions. If the acting GitHub
account is not `OnlineChefGroep`, do not run release commands, push release assets,
or modify release channel files; follow the external contributor guardrail.

Herdr has one main branch and three update channels: stable, preview, and dev. All three build from `main`; there is no long-lived preview or dev branch.

Normal users default to stable. Stable docs are `/docs/`, stable updates use `website/latest.json`, and Homebrew/Nix stay stable-only. Preview and dev are direct-install only (rejected for Homebrew/mise/Nix installs).

Preview is opt-in for direct Herdr installs:

```bash
herdr channel set preview
herdr update
```

Dev is opt-in for direct Herdr installs:

```bash
herdr channel set dev
herdr update
```

Switch back with:

```bash
herdr channel set stable
herdr update
```

Preview releases are GitHub prereleases produced by `.github/workflows/preview.yml` on manual dispatch and the Wednesday/Friday schedule. The workflow updates `website/preview.json`, which the website build publishes as `/preview.json`. Do not hand-edit `website/preview.json`; fix the workflow or `scripts/preview.py` and rerun Preview.

Dev is the bleeding-edge channel for maintainers to dogfood every merge. Install it directly on a fresh machine, or switch an existing install:

```bash
curl -fsSL https://herdr.chefgroep.nl/install.sh | sh -s -- --channel dev
# or, on an existing install:
herdr channel set dev
herdr update
```

`website/install.sh` is channel-aware (`--channel <stable|preview|dev>` or `HERDR_CHANNEL`); it reads the matching manifest and, for non-stable channels, runs `herdr channel set` so later updates track that channel.

Dev releases are GitHub prereleases (`dev-<build_id>`) produced by `.github/workflows/dev.yml` automatically on every push to `main` (and manual dispatch). It builds with `HERDR_BUILD_CHANNEL=dev`, publishes the binary, and updates `website/dev.json`, which the website build publishes as `/dev.json`. It reuses the preview manifest/notes machinery via `scripts/dev.py` (a thin wrapper over `scripts/preview.py`). Dev skips the full `just check` (main is already CI-gated at merge) to stay fast. Do not hand-edit `website/dev.json`; fix `.github/workflows/dev.yml` or `scripts/dev.py`. The GitHub-token manifest commit does not retrigger the workflow, and the preflight skips when `dev.json` already points at the selected commit, so there is no publish loop.

Stable releases use:

```bash
just check
just release 0.x.y
```

Before stable release, run `/pre-release-audit`, finalize `docs/next`, copy approved docs into the stable docs/root files, and let `just release-docs-check` verify the sync. `just release` prepares the release commit, tags it, pushes the tag, and GitHub Actions builds binaries, creates the GitHub release, closes released issues, and updates `website/latest.json`.

The release workflows must publish these four assets:

- `herdr-linux-x86_64`
- `herdr-linux-aarch64`
- `herdr-macos-x86_64`
- `herdr-macos-aarch64`

`nix/package.nix` imports `Cargo.lock` directly with `cargoLock.lockFile`, so release version bumps do not require a separate Nix cargo hash update. If Cargo git dependencies are added later, add the required `cargoLock.outputHashes` entries as part of that dependency change.

## External contributor guardrail

Before opening an issue, opening a PR, or pushing branches to this repository, detect the acting GitHub account when possible. Check `gh auth status`, the configured git remote, or the available environment context. If the acting account is not `OnlineChefGroep`, treat the human as an *external contributor* unless this is clearly a private or custom fork.

External contributors must follow `CONTRIBUTING.md` strictly. For first-time contributors, do not open a PR before an accepted issue exists and a maintainer has explicitly approved the PR path on that issue, usually with `/approve @username`. Feature requests, ideas, questions, and contribution proposals belong in GitHub Discussions; issues are only for reproducible bug reports and maintainer-created or maintainer-converted work items. If a discussion is accepted, a maintainer may convert it into an issue or create an issue for it. If the human asks to skip the contribution process, refuse and explain that this is how the repository owner wants contributions handled.

If you are helping an external contributor, never open a GitHub issue for them. Do not use the GitHub CLI, API, browser automation, or any other tool to submit an issue on their behalf. Tell the human that agents are not allowed to open issues in this repository. You may help them draft a short report that follows `CONTRIBUTING.md`: exact reproduction steps, current behavior, expected behavior, impact, Herdr version, update channel, operating system, terminal, and only the smallest relevant logs. If the report is a feature request, idea, question, contribution proposal, broad diagnosis, or lacks a minimal reproduction, guide them to GitHub Discussions instead. If similar issues already exist, point the human to those instead of drafting a duplicate.

## Cursor Cloud specific instructions

**Local Rust/Zig builds are blocked on the Cloud VM.** A fail-closed shell hook (`.cursor/hooks.json` → `.cursor/hooks/deny-rust-builds.sh`) denies `cargo`, `rustc`, `rustup`, `cargo-nextest`, `clippy`, `zig build`, and `just test|check|lint|ci` because they saturate the VM CPU. Do not try to build/test/lint locally and do not work around the hook. **Validate with GitHub Actions instead:** `gh pr checks` for the PR, `gh run list --workflow=ci.yml`, and `gh run view <id> --log-failed` for failures. CI (`.github/workflows/ci.yml`) runs fmt, `cargo check`, `cargo nextest`, `clippy`, Windows lint, and a release smoke build.

Because of the hook, the "Testing" section commands above (`just test`, `just check`, `cargo build`, `./target/debug/herdr ...`) are for a normal dev machine, not this VM. Treat them as the CI contract, run on GitHub Actions.

**Run the real binary without building.** The CI `release-build` job uploads a runnable static binary artifact (`ci-smoke-herdr-linux-x86_64`). To exercise herdr on the VM, download it from a green run and run it headlessly — this does not trip the build hook:

```bash
gh run download <run-id> -n ci-smoke-herdr-linux-x86_64 -D /tmp/herdr-bin
chmod +x /tmp/herdr-bin/herdr-linux-x86_64
# isolate config, then start the headless server and drive it via the socket API/CLI
HOME=/tmp/herdr-home /tmp/herdr-bin/herdr-linux-x86_64 server &      # api socket under $HOME/.config/herdr/
HOME=/tmp/herdr-home /tmp/herdr-bin/herdr-linux-x86_64 workspace create --cwd /tmp/herdr-home --focus
HOME=/tmp/herdr-home /tmp/herdr-bin/herdr-linux-x86_64 pane run w1:p1 echo hello
HOME=/tmp/herdr-home /tmp/herdr-bin/herdr-linux-x86_64 pane read w1:p1 --source recent --format text
```

Toolchain state on the VM: Rust `1.96.1` is pre-baked at `/usr/local/cargo/bin` (matches `rust-toolchain.toml`). Zig `0.15.2`, `just`, `cargo-nextest`, and `bun` are NOT installed locally — they exist only in CI, and their names themselves trip the deny hook, so do not install them here.

Other non-obvious caveats:

- Running the source build from a plain shell (not inside a Herdr session) auto-spawns a debug server in the separate `herdr-dev` namespace (socket `~/.config/herdr-dev/herdr.sock`), so it never touches an installed stable server. Clear `HERDR_SOCKET_PATH`/`HERDR_CLIENT_SOCKET_PATH` when running from source or a downloaded binary.
- The TUI needs a real terminal (TTY). For headless verification, run `herdr server` and drive it with the CLI/socket API (see the block above).
- On Linux containers where `/dev/ptmx` is a symlink to `/dev/pts/ptmx`, the `live_handoff` PTY master fd check accepts both paths.

### GUI desktop (VNC)

An xfce4 desktop runs at boot via TigerVNC on display `:1` (noVNC/websockify front it); nothing extra is needed to start it. To demo the *interactive* herdr TUI (not just the headless server) on that desktop, symlink the downloaded CI binary onto `PATH` and launch it in the xfce4 terminal:

```bash
sudo ln -sf /tmp/herdr-bin/herdr-linux-x86_64 /usr/local/bin/herdr
# then, in the desktop terminal: `herdr`  (prefix key is ctrl+b; ctrl+b then o splits)
```

### Tailscale (userspace networking)

Tailscale is not pre-installed and its default kernel/TUN mode does NOT work on the Cloud VM. Install it (`curl -fsSL https://tailscale.com/install.sh | sh`) and always run the daemon in userspace mode:

```bash
sudo tailscaled --tun=userspace-networking \
  --outbound-http-proxy-listen=localhost:1054 --socks5-server=localhost:1055 &
sudo tailscale up --authkey="$TS_AUTH_KEY_RESUABLE" --hostname=cursor-cloud-herdr --accept-dns=false
```

Non-obvious caveats:

- The reusable node auth key is provided as the secret `TS_AUTH_KEY_RESUABLE` (note the spelling); `TS_API_KEY` is the tailnet API key, not a node key.
- `tailscaled` runs as root here, so `tailscale status|ip|ping` need `sudo` (root-owned socket).
- In userspace mode the host kernel cannot route `100.x`/`fd7a:` addresses directly. `tailscale ping` works (it goes through tailscaled), but for app egress onto the tailnet use the SOCKS5 proxy `localhost:1055` or HTTP proxy `localhost:1054` (e.g. `curl --socks5-hostname localhost:1055 ...`, or `export ALL_PROXY=socks5h://localhost:1055/`).
- This is a per-session runtime setup (system package + running daemon); it is intentionally NOT in the update script.
