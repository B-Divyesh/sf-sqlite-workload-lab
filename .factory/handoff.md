# SQLite Workload Lab — verification handoff

## **FAIL** — candidate `fe3a3205cb0f03fa8092442949f295c0fda03d12`

Verified on 2026-08-28 against <https://sqlite-workload-lab.sociobot.in/>. The live site's HTML, JS, CSS, and hero asset hashes exactly match the candidate's fresh production build.

The candidate passes clean install, all repository tests, production build, clippy, package verification, a clean-consumer CLI install/exercise, live desktop/mobile accessibility checks, privacy/outbound-request checks, offline reload, headers/caching checks, and Lighthouse (100/100/100/100; LCP 1.5 s).

It is nevertheless **not releasable**: `sqlite-workload-lab check` returns success for a manifest whose measured query is `DELETE FROM docs`, while `run` correctly rejects that same manifest as mutating. This makes the documented/preflight SQL validation untrustworthy. `run` remains safe because it uses an isolated temporary database and rejects before execution.

Full evidence, exact commands/results, severity, and required remediation are in [.factory/verification.md](verification.md). No product code was modified by verification.

## Re-verify after remediation

```sh
npm ci
npm test
npm run build
cargo clippy --locked --all-targets -- -D warnings
cargo package --locked
```

Install `target/package/sqlite-workload-lab-0.1.0.crate` into a clean Cargo root and confirm that a pinned manifest with `DELETE FROM docs` fails at `check`, not only at `run`. Also version the service-worker cache before the next web release.
