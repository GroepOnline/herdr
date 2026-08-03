---
name: create-subagent
description: >-
  Project endpoint for /create-subagent. Scaffold a focused Cursor subagent at
  .cursor/agents/<name>.md with YAML frontmatter and a strict return contract.
  Use when a skill needs context-isolated lane work, parallel specialists, or
  independent verification. Pair with /create-skill for the twin chain.
disable-model-invocation: true
---

# Create subagent (project endpoint)

Create one specialized subagent file. Subagents get a clean context; the parent
must pass all inputs in the launch prompt.

## Twin chain

| Side | Path | Owns |
| --- | --- | --- |
| Skill | `.cursor/skills/<name>/SKILL.md` | When/why, sequencing, merge/report |
| Subagent | `.cursor/agents/<name>.md` | One focused execution lane |

If no skill exists yet, run `/create-skill` next and cross-link both files.

## Steps

1. **Inventory** — read the generated metadata catalog `.cursor/agents/.index.yaml`
   (name/path/summary only) or `.cursor/INDEX.md`. Reuse on name/role clash; open a
   peer agent prompt only when you are extending that named agent. Do not
   duplicate the same agent under `.claude/agents` or `.codex/agents`.
2. **Decide skill vs subagent** — use a subagent when you need context isolation, parallel lanes, or multi-step specialized execution. Use a skill alone for single-shot guidance.
3. **Write** `.cursor/agents/<name>.md`:

   ```markdown
   ---
   name: <kebab-name>
   description: >-
     Role + when the parent should delegate (include "use proactively" if
     auto-delegation is desired).
   model: inherit
   readonly: false
   ---

   You are …

   ## Inputs you expect
   ## Hard rules
   ## Procedure
   ## Return format (mandatory)
   ```

4. **Frontmatter fields**

   | Field | Notes |
   | --- | --- |
   | `name` | Optional; defaults from filename; kebab-case |
   | `description` | Delegation hint — invest here |
   | `model` | `inherit` or explicit model id |
   | `readonly` | `true` for audit/verify-only agents |
   | `is_background` | `true` only for long non-blocking lanes |

5. **Prompt quality**
   - One responsibility
   - Explicit input contract
   - Hard stop / out-of-scope rules
   - Mandatory concise return format for the parent
6. **Wire the skill** — add a Twin chain / Handoff section pointing at this agent.
7. **Verify** — description is specific; no generic "helper"; return format present.
8. **Report** — path, invoke hints (`/<name>`, "use the \<name> subagent to…"), paired skill.

## Guardrails

- Do not create dozens of vague agents; start with 1–3 focused roles.
- Do not duplicate built-ins (Explore / Bash / Browser) without a domain reason.
- Prefer `readonly: true` for reviewers/verifiers.

## Example twins (this repo)

| Twin | Skill | Subagent | Split |
| --- | --- | --- | --- |
| PR gate | `.cursor/skills/pr-gate/SKILL.md` | `.cursor/agents/pr-gate.md` | Skill = gate contract; agent = execute one PR |
| CI fixer | `.cursor/skills/ci-fixer/SKILL.md` | `.cursor/agents/ci-fixer.md` | Skill = fix policy; agent = triage logs + patch |
| Parallel lane | `.cursor/skills/parallel-lanes/SKILL.md` | `.cursor/agents/parallel-lane.md` | Skill = ~4-lane dispatch; agent = one scoped lane |

Cross-link both files: skill gets a **Twin chain** table; agent gets **Related skills** pointing back.
