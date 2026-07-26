---
name: find-skills
description: Route skill discovery for Herdr/ChefGroep agents — npx skills find, skills.sh quality checks, and local skill roots.
---

# Find skills (Herdr router)

Use when the user asks how to find, install, or evaluate agent skills.

## Discovery order

1. **Repo-local (Herdr):** `.cursor/skills/`, `.codex/skills/`, `.cursor/agents/`
2. **ChefGroep laptop roots:** `~/.cursor/skills/`, `~/.agents/skills/`, `~/.claude/skills/`
3. **Public catalog:** `npx skills find <query>` (skills.sh ecosystem)
4. **Capability router (Cursor):** `~/.cursor/skills/capability-router/SKILL.md` when installed

## Quality checks before adopting external skills

- Prefer skills with clear `name` + `description` frontmatter and a single responsibility.
- Skim for secret handling, destructive shell patterns, and local cargo/build requirements incompatible with Herdr (`AGENTS.md` denies local Rust on cloud VMs).
- Cross-check leaderboard / repo activity on [skills.sh](https://skills.sh) when choosing between duplicates.
- For Herdr CI work, prefer `fix-ci` and `herdr-quality-ci-remediation` over generic CI skills.

## Install pattern

```bash
npx skills find "<query>"
# follow upstream install instructions; mirror into .cursor/skills/ when repo-specific
```

Do not commit secrets or vendor entire skill marketplaces into herdr — keep thin routers and repo-specific playbooks only.
