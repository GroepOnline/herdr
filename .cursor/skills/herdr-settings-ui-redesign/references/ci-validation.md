# CI-only validation

```bash
git push -u origin HEAD
gh pr checks --json name,bucket,state,workflow,link
gh run view <run-id> --log-failed
```

Primary gate: `.github/workflows/ci.yml` (`Check & Test`, `Release smoke build`).

Forbidden locally: `cargo *`, `just test|check|lint|ci|build|fmt-check|windows-lint`.
