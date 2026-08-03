---
name: audit
description: Report-only parallel thermos audit with Herdr canvas (no fixes)
---

Run skill `herdr-thermos-audit` now.

- Mode: **report-only** (niet handelen). Do not spawn `review-fixer`.
- Lanes: thermos A+B in parallel; optional `audit-scope-mapper` as C.
- Visualize in neighbor Herdr pane/tabs when `HERDR_ENV=1`.
- Ingest summary to joep-brain when finished.
