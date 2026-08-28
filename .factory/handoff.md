# SQLite Workload Lab — build handoff

Work order: `sqlite-workload-lab-build-1`

Version: `0.1.0`

Completed: 2026-08-28

## What shipped

- A Rust single-binary CLI with `init`, `check`, `run`, `matrix`, and `compare` commands plus global `--json` output.
- A strict TOML workload format that pins SQLite version and fixture SHA-256, requires at least three CPU profiles, rejects unsafe PRAGMA names and mutating measured queries, and validates required/forbidden CPU features.
- Isolated SQLite runs using bundled SQLite/FTS5. Reports capture source ID, compile options, observed PRAGMAs, query plans, CPU flags, architecture/OS, row counts, result digests, warmups, and every measured timing sample.
- Deterministic JSON and diff-friendly Markdown evidence. Reports call out hardware, virtualized, container, or emulator evidence and include the repeated-run/statistical-claim caveat.
- A comparator that refuses unlike profile/build/fixture/workload contexts and changed query results by default. A median regression above the threshold exits with code 2 for CI.
- An accessible static documentation site at `dist/site/`, including a keyboard-operated report walkthrough, 390 px layout, useful offline state/service worker, no analytics, and no third-party runtime requests.
- A product-specific “compatibility observatory” visual system and original AI-generated hero. The final 58 KB WebP is `site/public/lab-landscape.webp`; its prompt/deployment metadata is in `.factory/design.md` and `.factory/lab-landscape.prompt.json`.

## Run and verify

```sh
npm install
npm test
npm run build
cargo package --locked
```

- `npm test`: passed. Rust: 1 unit + 4 CLI integration tests. Site: 2 budget/semantics tests. Playwright: 7 passed, 1 intentional desktop skip for the mobile-only overflow assertion. The browser suite covers axe serious/critical violations, console errors, keyboard tabs, 390 px overflow, and an offline reload.
- `npm run build`: passed. It builds the optimized 3.3 MB CLI and Vite site with `index.html` at `dist/site/index.html`.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo package --locked --allow-dirty`: passed and verified a 110 KB crate at `target/package/sqlite-workload-lab-0.1.0.crate`. Use `cargo package --locked` from the committed tree; registry publication remains with the factory.
- `npm audit --audit-level=high`: 0 vulnerabilities.
- Factory URL verification against the production preview: HTTP 200, no console/page errors, `lang=en`, one `h1`, a `main` landmark, no missing image alt, and no unlabeled buttons.

## Lighthouse and budgets

Mobile Lighthouse against the production Vite preview:

| Category / metric | Result |
| --- | ---: |
| Performance | 99 |
| Accessibility | 100 |
| Best practices | 100 |
| SEO | 100 |
| First contentful paint | 1.5 s |
| Largest contentful paint | 1.8 s |
| Total blocking time | 0 ms |
| Cumulative layout shift | 0 |
| Speed index | 1.5 s |

Initial payloads: 2.1 KB JS, 14.3 KB CSS, 88.7 KB self-hosted fonts, and 58.6 KB hero WebP. These are all enforced by tests where applicable. The complete static output is 212 KB.

## Deployment

Deploy `dist/site/` after `npm run build`; no DNS, infrastructure, billing, or registry state was changed. `staticwebapp.config.json` provides security headers and immutable caching for hashed assets. The service worker caches the documentation shell and cleans old cache versions.

## Known gaps and next steps

- The v1 binary runs *inside* the selected hardware, container, or QEMU environment; it does not provision Docker or QEMU itself. README examples show both launch patterns. A later version could add opt-in runner adapters without changing report format.
- CPU features come from `/proc/cpuinfo`; non-Linux builds still report architecture/OS but may have an empty feature list, so strict feature profiles will fail safely there.
- Measurements are descriptive medians/p95 values, not significance claims. Reviewers should keep repetitions high and run on controlled hosts.
- The factory still needs to publish platform release archives and deploy `dist/site/`.
