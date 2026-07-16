# CHEF Regression Test Plan

## Scope

Regression tests for downstream modifications. Must pass before any
merge to master. Run with: `cargo nextest run --features chef-tests`

## Test Categories

### 1. Rendering (render_ansi.rs)

| Test | Validates |
|---|---|
| test_wide_cell_at_row_end_no_overflow | #8: wide cell at last column does not carry to_skip into next row |
| test_invalidated_clamped_to_row | #8: invalidated counter does not cross row boundary |
| test_zero_width_cell_returns_one | #10: control chars get width 1, not 0 |
| test_cjk_wide_cell_counted_as_two | #10: CJK characters correctly report width 2 |
| test_border_title_clears_previous_style | #11: title area style is reset before writing |
| test_terminal_ansi_is_default_encoding | Render encoding defaults to TerminalAnsi |
| test_ime_anchor_never_repeats | repeat_ime_anchor_after_sync() always false |
| test_graphics_wrapped_in_save_restore | Kitty graphics wrapped in ESC 7/8 within sync block |

### 2. Update Isolation (update.rs)

| Test | Validates |
|---|---|
| test_update_url_uses_env_override | HERDR_UPDATE_BASE_URL overrides default herdr.chefgroep.nl URL |
| test_update_url_falls_back_to_const | Without env, uses STABLE_UPDATE_MANIFEST_URL |
| test_no_runtime_download_from_herdr_dev | Network calls only go to configured base URL |

### 3. Config Profile (config/chef.toml)

| Test | Validates |
|---|---|
| test_chef_profile_imports_correctly | chef.toml imports without error |
| test_chef_profile_sets_prefix_ctrl_a | Prefix key is ctrl+a |
| test_chef_profile_disables_focus_redraw | redraw_on_focus_gained = false |

### 4. Agent Detection (detect/)

| Test | Validates |
|---|---|
| test_freebuff_manifest_loads | freebuff.toml parses and registers |
| test_junie_manifest_loads | junie.toml parses and registers |
| test_openclaude_manifest_loads | openclaude.toml parses and registers |
| test_custom_manifests_min_engine_v2 | All custom manifests declare min_engine_version >= 2 |

### 5. SSH / Remote Runtime (integration)

| Test | Validates |
|---|---|
| test_ssh_pty_renders_ansi | SSH PTY renders TerminalAnsi frames correctly |
| test_ssh_resize_triggers_full_redraw | Terminal resize forces full frame, not diff |
| test_fish_shell_detected | fish shell is detected and terminal config applied |

### 6. Windows Terminal Client (client/mod.rs)

| Test | Validates |
|---|---|
| test_cell_pixel_metrics_zero_fallback | cell_width_px=0 triggers TerminalAnsi fallback |
| test_windows_terminal_env_detected | Windows Terminal env vars detected correctly |
| test_no_semantic_frame_over_ssh | SemanticFrame not sent over SSH PTY |

## Running

```bash
# All regression tests
cargo nextest run -p herdr --features chef-tests

# Just rendering
cargo nextest run -p herdr render

# Just update isolation
cargo nextest run -p herdr update
```

## CI Integration

Tests run on every push to master and on every PR. The CI workflow
must pin Zig 0.15.2 + Rust 1.96.1 and install musl target.

