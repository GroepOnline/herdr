---
name: settings-redesign
description: Start or continue the bold Herdr settings UI redesign with parallel workers and CI-only validation.
---

Use skill `herdr-settings-ui-redesign`.

1. If no plan exists, dispatch `settings-explorer` and write `.local/prd/settings-ui-redesign-plan.md`.
2. Execute via parallel workers:
   - Implementers: `settings-implementer` (composer-2.5), max 3
   - Reviewers: `settings-reviewer` (grok-4.5)
   - CI failures: `settings-diagnose` (grok-4.5)
3. Never run cargo/just locally — validate with `gh pr checks` only.
