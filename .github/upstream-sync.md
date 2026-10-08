# Upstream sync — trunk + overlay

Process for bringing `GroepOnline/herdr` back in step with upstream
`herdrdev/herdr` (the former `ogulcancelik/herdr` path redirects there) and for
keeping it in step afterwards.

## Why this process exists

Downstream shares no usable git ancestry with upstream today:

- merge base with upstream `master`: `fb0c9717` (2026-04-01)
- **0** of the last 40 upstream commits are ancestors of `main`
- upstream work was replayed as new commits; ~627 upstream commits since the
  fork are absent (2026-07: 95, 2026-08: 202, 2026-09: 289, 2026-10: 41)
- 1912 paths exist on both sides; **882 differ in content** (vendor 493,
  docs/versions 75, src/integration 34, src/app 33, src/detect 19, src/api 19,
  src/cli 13, src/server 13, src/ui 10)

Replay-based syncing is the root cause of the drift. This process replaces it.

## Model: upstream is the trunk, downstream is an overlay

```
upstream v0.9.3 ── upstream master ── sync/upstream-v0.9.3 (real ancestry)
                                            │  + overlay commits (one per concern)
                                            ▼
                                          main ── tag (next release)

weekly drift gate compares the ledger against upstream and fails on a stale pin
```

- **Trunk** commits are real upstream commits, so `git rebase --onto` works and
  the upstream delta is always exactly one `git log` away.
- **Overlay** commits are ours: one commit per concern, each with a ledger entry.
- Never mix the two in one commit. Never re-play upstream by hand when the
  trunk can carry it.

## The ledger (source of truth)

| File | Role |
| --- | --- |
| `sync/upstream.json` | pinned upstream base: repo, tag, commit, generated_at, summary counters |
| `sync/overlay.tsv` | overlay inventory: `path_glob`, `concern`, `owner`, `rationale`, `tests`, `removal_condition` |
| `sync/ledger.json` | generated snapshot: path classification and the unported upstream commit list |

- `python3 scripts/upstream_sync_ledger.py --check` runs offline in
  `just maintenance` and fails when the pin, the overlay inventory or the
  snapshot is inconsistent.
- `python3 scripts/upstream_sync_ledger.py --generate` needs a blobless local
  upstream clone and refreshes `sync/ledger.json`.
- `.github/workflows/upstream-drift.yml` (planned, wave G) refreshes the
  comparison weekly and fails (or opens an issue) when the pin is older than
  the cadence or a critical-path area moved.

Critical paths for drift alerts: `src/integration`, `src/detect`, `src/protocol`,
`vendor`, `crates`, `distribution/agent-detection`.

## Waves and exit gates

Each wave is its own PR onto the trunk branch with its own gate; waves may not
be merged on a red gate.

| Wave | Scope | Exit gate |
| --- | --- | --- |
| A — trunk and build base | fork point on upstream `v0.9.3`, `crates/ghostty-vt`, re-vendor to the upstream pin, patch index merge, workspace/Cargo, `PROTOCOL_VERSION` 18 → 22 | `CI / Quality gate` green incl. vendor patch tests |
| B — detection and manifests | union of bundled manifests plus `distribution/agent-detection`; keep downstream-only agents; respect `STAGED_WEBSITE_MANIFESTS` | manifest check, bun asset tests, live pane reads |
| C — integrations and hooks | upstream integration commits, CST-based config editing, per-target asset version bumps | non-destructive hook test with foreign (Moshi) entries plus host E2E |
| D — TUI, client, input, terminal | input pipeline, kitty graphics, terminal effects, handoff and suspend behaviour | Quality gate plus TTY validation |
| E — overlay reinstatement | website, npm, workers, plugins, config, fleet/gateway, settings UI, mobile UI | overlay ledger has no unclassified paths |
| F — CI, release, packaging | downstream release trust chain on the new tree, open PR rebase, version policy | `just release-metadata`, `just maintenance`, `just release-verify` |
| G — verification and landing | full verification protocol, single landing PR, tag, docs | protocol below fully green |

## Runbook

```bash
# freeze
git tag pre-sync-0.8.8 <current-main-sha>          # rollback point
git fetch https://github.com/herdrdev/herdr refs/heads/master:refs/remotes/upstream/master \
  'refs/tags/v0.9.3:refs/tags/v0.9.3'

# trunk
git worktree add ../herdr-worktrees/upstream-sync-trunk -b sync/upstream-v0.9.3 v0.9.3

# overlay commit (one concern per commit)
git checkout -b sync/overlay-<concern> sync/upstream-v0.9.3
# ... bring the concern over, no upstream replay ...
git commit -m "chore(sync): reapply <concern> overlay"

# ledger
python3 scripts/upstream_sync_ledger.py --generate --upstream-url https://github.com/herdrdev/herdr
python3 scripts/upstream_sync_ledger.py --check
```

## Non-negotiables

- No `git merge -X theirs`, no bulk replay/cherry-pick scripts, no `ours`-style
  conflict shortcuts. Conflicts are resolved by hand against upstream intent.
- Never weaken, skip or delete a test to make a gate pass. A gate that cannot
  run is reported as not run.
- Rust/Zig builds and `just test|check|lint` are CI-owned. Local validation
  here means: `gh pr checks`, `gh run view <id> --log-failed`, and the
  downloaded smoke binary.
- Vendored patches follow `vendor/*.patches.md`: every applied patch is indexed,
  every indexed patch reverse-applies; patches whose fix landed upstream are
  removed together with their index entry.
- `PROTOCOL_VERSION` is only bumped when the source protocol is not already
  greater than the latest released protocol.
- Integration asset versions are migration versions relative to the latest
  released tag, not per-commit counters.
- Stable docs (`website/src/content/docs/`) only describe released behaviour;
  unreleased behaviour goes to `docs/next/`.
- Do not port Hermes distribution changes (see `DOWNSTREAM.md`); the Hermes
  integration target may remain in code as a documented overlay item.

## Verification protocol

| # | Check | Owner |
| --- | --- | --- |
| 1 | `CI / Quality gate` (lint, tests, maintenance, Windows lint, release metadata, release smoke) | CI |
| 2 | Vendor patch index reverse-applies | CI (maintenance) |
| 3 | Protocol negotiation: old client against new server yields the upgrade path | CI + headless run |
| 4 | Smoke binary: server → workspace → pane run/read | agent on the Cloud VM |
| 5 | TUI behaviour (mouse, modals, settings, resize) | maintainer on the desktop |
| 6 | Manifest check, bun asset tests, `herdr agent explain` on live panes | CI + maintainer |
| 7 | Hook config is non-destructive: foreign hook entries byte-identical after install/uninstall/update | agent on a host + fixture test |
| 8 | Mobile client (Moshi) push on a blocked agent | maintainer on device |
| 9 | Gateway/fleet `GET /v1/ops/context` returns 401 unauthenticated and rejects foreign origins | agent, headless |
| 10 | Website and docs build, preview docs generated | CI + dev-server preview |
| 11 | `just release-verify <version>` (release, checksums, live manifest, asset URLs) | CI |

## Drift prevention

- Rebase PR within 7 days of every upstream stable tag.
- `upstream-drift.yml` fails when the pin is stale or a critical path changed.
- A wave is not "done" until its overlay entries carry a test name.

## M1 gate reconciliation (trunk `v0.9.3` plus overlay)

Measured on `sync/upstream-v0.9.3` after `chore(sync): reapply downstream CI and
tooling overlay` and `chore(sync): reapply downstream non-Rust distribution
overlay`, using the gates that can run without Rust/Zig on the Cloud VM. Rust
lanes only report through CI, which needs a pull request against `main`.

| Gate | Result | Cause | Wave |
| --- | --- | --- | --- |
| `scripts.test_upstream_sync_ledger` | pass | ledger consistent on the trunk | — |
| `scripts.test_ci_changed_paths`, `scripts.test_ci_quality` | pass | lane contract intact | — |
| bun integration asset tests | pass (18) | assets unchanged | — |
| `scripts.test_vendor_portable_pty` (2 tests) | fail | the downstream patch contract (`0001-force-system-conpty`) does not match the upstream vendored tree (`0001-control-conpty-loading`), plus `0002`/`0003` | A |
| `scripts.test_agent_detection_manifest_check` (aider, staged website manifest) | fail | bundled manifests moved to upstream 2026.09.18.x while the website manifests and the `STAGED_WEBSITE_MANIFESTS` entry still assume the downstream baseline; downstream-only manifests are not overlaid yet | B |
| `scripts.test_config_reference_check` (`ui.tab_bar_right`) | fail | `docs/next/.../config-reference.json` still allows `datetime`, `text`, `command`; the trunk config model allows `zoom`, `hostname` | B/D |
| `scripts/ci_quality.py check-release-metadata` | fail | `src/bin/herdr-gateway.rs` (downstream gateway) is not overlaid yet | E |
| `scripts.test_vendor_portable_pty::cargo metadata` | error | needs `cargo`; CI-owned on this VM | CI |
| fmt, check, nextest, clippy, Windows lint, release smoke | not run | Rust lanes are CI-owned; they run once the sync PR exists | CI |

Rules for the waves: a wave may not "fix" a gate by weakening it. Wave A resolves
the vendor patch entries, wave B the manifest and config-reference drift, wave E
the gateway overlay, and CI settles the Rust lanes.

### Overlay lesson: tooling must be selective

The first tooling overlay copied `scripts/` and `justfile` wholesale from the
downstream branch, which overwrote upstream-owned scripts. Nine of them exist on
both sides (`config_reference_check.py`, `preview.py`, `test_preview.py`,
`test_changelog.py`, `test_agent_detection_manifest_check.py`,
`test_config_reference_check.py`, `test_vendor_libghostty_vt.py`,
`test_vendor_portable_pty.py`, `windows_smoke_conpty_path.ps1`).

Rule from here on: a script or workflow that upstream owns stays upstream unless
the downstream version carries a documented need. The remaining 24 scripts are
downstream-only additions (release, docs, fleet and control-plane tooling) and
are overlay by definition. Every wholesale directory copy is a red flag; overlay
commits list the files they overwrite.

### M1 wave progress (2026-10-08)

| Item | Before | Now | How |
| --- | --- | --- | --- |
| Vendor patch contract tests | fail | pass | restored upstream's `test_vendor_portable_pty.py`, `test_vendor_libghostty_vt.py` and `windows_smoke_conpty_path.ps1`; upstream's `0001-control-conpty-loading` (absolute-path loading, hash verification, no bare DLL search) supersedes the downstream `force-system-conpty` patch |
| Config reference | fail | pass | restored upstream's `config_reference_check.py` and its test; the trunk config model already carries `ui.tab_bar_right` |
| Agent manifests | fail | pass | brought the five downstream manifests into `src/detect/manifests/`, aligned `website/agent-detection/` with the bundled manifests, added `letta`, retired the now-dead grok staging entry |
| Overlay footprint | not checked | checked | the ledger now records `overlay_scope` and `validate_scope` fails when a path changed against the pinned base is not declared in `sync/overlay.tsv` (543 paths, 0 undeclared) |
| Release metadata | fail | fail | `src/bin/herdr-gateway.rs` is not overlaid yet (wave E) |
| Vendored cargo test, Rust lanes | not run | not run | need `cargo`; CI-owned and only reachable once the sync PR exists |

Catalog ownership is now explicit: upstream publishes from
`distribution/agent-detection/`, downstream publishes from
`website/agent-detection/` (tracked as an overlay concern with its removal
condition).

### Wave C baseline: the hook guarantee is already in the trunk

The mobile-client (Moshi) complaint traces to the downstream integration code
that predates upstream's rewrite. The trunk already carries the fixed design plus
tests, so wave C does not need to invent it:

- `src/integration/config_file.rs` — "Protected writes for user-owned integration
  configuration": write-target checks, hard-link rejection, and refusal to
  clobber a config that another writer owns.
- `src/integration/claude_settings.rs` — CST-based edits that keep user
  formatting and unrelated hooks, with tests such as
  `install_preserves_untouched_formatting_and_complete_trailing_suffix`,
  `install_migrates_wildcard_session_start_and_preserves_user_hook` and
  `uninstall_preserves_unrelated_hook_text`.
- `src/integration/tests.rs` — per-target preservation coverage:
  `install_claude_removes_deprecated_completion_hooks_and_preserves_user_hooks`,
  `uninstall_claude_removes_herdr_hooks_and_preserves_others` and the equivalent
  copilot, devin, droid and kimi cases.
- `src/integration/opencode_config.rs` — JSONC-preserving plugin registration
  (`add_and_remove_tui_plugin_preserves_jsonc_config`,
  `cli_registration_preserves_options_and_other_preferences`).

Wave C therefore only has to: keep the five downstream-only integration targets
as overlay (commandcode, freebuff, plus their assets), re-derive the downstream
`*_INTEGRATION_VERSION` values once, and add one explicit end-to-end fixture that
installs a hook set containing foreign (mobile-client) entries, runs
install/uninstall/update, and asserts the foreign entries are byte-identical.
