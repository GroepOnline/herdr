---
name: herdr-settings-ui-redesign
description: Plan and execute the bold Herdr settings modal redesign (section IA, config surfacing, animations, CI-only validation). Use when redesigning settings UI, expanding SettingsSection, or surfacing config.toml/CLI options in the TUI.
disable-model-invocation: true
---

# Herdr settings UI redesign

## Pre-flight

1. Read [references/design-spec.md](references/design-spec.md)
2. Read [references/architecture-map.md](references/architecture-map.md)
3. Read [references/ci-validation.md](references/ci-validation.md)

## Workflow

1. **Explore (grok-4.5):** dispatch `settings-explorer` for current vs target section map and risks.
2. **Implement (composer-2.5, max 3 parallel):** dispatch `settings-implementer` per independent seam (shell/layout, appearance/layout/notifications, input/terminal/updates/agents/advanced).
3. **Review (grok-4.5):** `settings-reviewer` after each pushable slice.
4. **CI (grok-4.5):** on red checks, `settings-diagnose` — never local cargo/just.

## Constraints

- Pure render; shared hit-test geometry; mouse-first modal language.
- Surface config/settings CLI options; leave pure runtime actions on CLI.
- Commit style: lowercase conventional commits; `refs #<n>` when applicable.
