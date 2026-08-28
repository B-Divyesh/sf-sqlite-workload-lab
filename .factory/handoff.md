# SQLite Workload Lab — verification handoff

## **FAIL** — candidate `dc125b00b97651b83826406cdf4137e380af77fe`

Freshly verified on 2026-08-28 against <https://sqlite-workload-lab.sociobot.in/>. The live HTML, JS, CSS, and hero WebP SHA-256 values exactly match this candidate’s fresh production build.

The clean checkout passed `npm ci`, `npm test`, `npm run build`, clippy, package verification, clean-consumer install/exercise, live desktop/mobile and keyboard accessibility, axe (0 serious/critical), privacy/outbound request checks, PWA offline/update checks, response headers/caching, and Lighthouse (99/100/100/100; LCP 1.7 s). No product code was changed during verification.

It is **not releasable**: `sqlite-workload-lab --json check` returns success for a pinned workload containing `DELETE FROM docs`, even though `run` correctly rejects that SQL before execution. This contradicts the documented SQL-shape/preflight contract and makes CI validation unreliable. The full evidence, exact consumer reproduction, severity, and remediation are in [.factory/verification-2.md](verification-2.md).

## Re-verify after remediation

```sh
npm ci
npm test
npm run build
cargo clippy --locked --all-targets -- -D warnings
cargo package --locked --allow-dirty
```

Install the resulting crate in a clean Cargo root, then prove that the mutating manifest fails at `check` as well as `run`. Version the service-worker cache before the next web release.
