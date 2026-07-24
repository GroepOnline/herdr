# Quality CI reference (skill copy)

Source of truth for architecture and maintainer wiring: `.github/quality-ci.md`.

Keep this file aligned when the gate job names or remediation dispatch payload change.

## Required check

`CI / Quality gate`

## Parallel jobs

- `CI / Lint`
- `CI / Test`
- `CI / Maintenance`
- `CI / Windows lint` (native `windows-latest`)
- `CI / Release metadata`
- `CI / Release smoke build (x86_64-unknown-linux-musl)`

## Autofix vs remediation

| Workflow | Role | Pushes? |
|---|---|---|
| `quality-autofix.yml` | Same-repo mechanical fmt + npm VERSION sync | Yes (`ci: autofix mechanical quality`) |
| `quality-remediation.yml` | Sticky brief + `quality-remediation` label + `repository_dispatch` | No |

Escape hatch PR label: `ci-autofix-disabled`.

## Dispatch payload

```json
{
  "event_type": "herdr-quality-remediation",
  "client_payload": {
    "pr": 123,
    "run_id": 987654321,
    "head_sha": "0123456789abcdef0123456789abcdef01234567",
    "branch": "cursor/example-branch"
  }
}
```

## Agent commands

```bash
gh run view <run_id> --repo OnlineChefGroep/herdr --log-failed
gh pr checks <pr> --repo OnlineChefGroep/herdr --watch
gh pr view <pr> --repo OnlineChefGroep/herdr --json files,labels,headRefName,headRefOid
```
