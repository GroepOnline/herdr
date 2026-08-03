---
name: create-skill
description: >-
  Project endpoint for /create-skill. Scaffold a Cursor Agent Skill under
  .cursor/skills/<name>/SKILL.md with valid frontmatter, twin-chain hookup to a
  subagent when needed, and a short verify pass. Use when adding or upgrading a
  reusable workflow skill.
disable-model-invocation: true
---

# Create skill (project endpoint)

Scaffold a project skill the next agent can load cold. Prefer extending an
existing skill over inventing a near-duplicate. SSOT is `.cursor/` only — do not
copy the same skill into `.agents` / `.claude` / `.codex`.

## Twin chain

If the workflow needs isolated multi-step execution, also run `/create-subagent`
and link both sides (skill owns sequence; subagent owns one lane).

| Side | Path |
| --- | --- |
| Skill | `.cursor/skills/<name>/SKILL.md` |
| Subagent (optional) | `.cursor/agents/<name>.md` |

## Steps

1. **Inventory** — read the generated metadata catalog `.cursor/skills/.index.yaml` (name/path/summary only) or `.cursor/INDEX.md`. Reuse if coverage exists. Do not read peer `SKILL.md` bodies to browse; open one only when you are extending that named skill.
2. **Name** — kebab-case; folder name **must** equal frontmatter `name`.
3. **Interview the repo** — derive triggers, pre-reads, stop conditions, and evidence from real paths/commands. Ask the user only for what you cannot observe.
4. **Write `SKILL.md`** with YAML frontmatter:

   ```yaml
   ---
   name: <kebab-name>
   description: >-
     What it does and when to use it (include slash/intent triggers).
   # disable-model-invocation: true   # only for explicit /slash skills
   ---
   ```

5. **Body sections** (keep under ~500 lines; put detail in `references/`):
   - When to use
   - Pre-reads
   - Ordered steps
   - Stop conditions
   - Evidence contract
   - Handoff (subagent / next skill)
6. **Optional helpers** — `scripts/` must be executable; document exact invocation.
7. **Verify**
   - Frontmatter `name` matches folder
   - Forward-slash paths only
   - No unresolved placeholders (`TODO`, `<fill>`, `...`)
   - Description states triggers clearly
8. **Report** — paths created, how to invoke (`/<name>` or natural language), twin subagent if any.

## Guardrails

- Do not create a skill for a one-shot task better done inline.
- Do not put always-on policy here; use `.cursor/rules/*.mdc` for that.
- Prefer `disable-model-invocation: true` for meta/scaffolding skills (this one).

## Example twins (this repo)

| Twin | Skill | Subagent | Split |
| --- | --- | --- | --- |
| PR gate | `.cursor/skills/pr-gate/SKILL.md` | `.cursor/agents/pr-gate.md` | Skill = merge-ready contract; agent = one PR lane |
| CI fixer | `.cursor/skills/ci-fixer/SKILL.md` | `.cursor/agents/ci-fixer.md` | Skill = when/how to fix; agent = log triage + minimal diff |
| Parallel lanes | `.cursor/skills/parallel-lanes/SKILL.md` | `.cursor/agents/parallel-lane.md` | Skill = dispatch + integrate; agent = one disjoint file set |

Umbrella (no 1:1 twin): `local-verify` maps to multiple subagents (`kater-verify`, `ci-fixer`, `pr-gate`).
Deep slice without twin: `kater-gateway` (hand off to `kater-verify` / `ci-fixer`).
