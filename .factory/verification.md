# Verification report — FAIL

Work order: `sqlite-workload-lab-verify-1`  
Verified candidate: `fe3a3205cb0f03fa8092442949f295c0fda03d12`  
Live URL: <https://sqlite-workload-lab.sociobot.in/>  
Date: 2026-08-28

## Decision

**FAIL.** The CLI safely rejects mutating measured SQL at `run` time, but its documented preflight command, `check`, incorrectly reports the same invalid manifest as valid. This violates the stated invalid-input/SQL-shape validation contract and makes a CI preflight unreliable. No product source was modified during verification.

## Release gates

| Check | Result | Evidence |
| --- | --- | --- |
| Clean candidate | PASS | Checkout was clean and `HEAD` was exactly `fe3a3205cb0f03fa8092442949f295c0fda03d12` before verifier-document changes. |
| Install | PASS | `npm ci`: 24 packages installed; audit reported 0 vulnerabilities. |
| Unit/integration/browser suite | PASS | `npm test`: 1 Rust unit + 4 Rust CLI integration tests passed; 2 static-site tests passed; Playwright: 7 passed, 1 expected mobile-only skip. |
| Production build | PASS | `npm run build` completed: locked release CLI plus `dist/site/`. |
| Lint | PASS | After adding the missing pinned-toolchain clippy component, `cargo clippy --locked --all-targets -- -D warnings` passed. |
| Publish artifact | PASS | `cargo package --locked` packaged and verified 36 files, 239.6 KiB / 110.9 KiB compressed. |
| Clean consumer install | PASS | Installed the `.crate` into a fresh temporary Cargo root; the installed `sqlite-workload-lab 0.1.0` printed help and completed init/check/run. |
| Live deployment identity | PASS | Candidate and live SHA-256 values matched for `index.html`, `index-DNY6hJnh.js`, `index-CUpl0W1U.css`, and `lab-landscape.webp`. |

## CLI end-to-end evidence

From the clean consumer install:

- `init lab.toml`, `check lab.toml`, and `--json run lab.toml --profile host --out baseline` succeeded, producing JSON and Markdown reports with SQLite build data, PRAGMAs, plan, CPU flags, timing samples, and an honest evidence label.
- `matrix` on this AVX/AVX2 host stopped at the generated `emulated-x86-v2` profile with exit code 1 and the explicit forbidden-feature message. `matrix --allow-profile-mismatch` produced all three declared profile reports and preserved mismatch evidence.
- A candidate report with a deliberately doubled median caused `compare --threshold 15` to exit 2 and print `Gate: FAIL` (+105.56% in this run).
- A boundary manifest with `repetitions = 2` exited 1 with machine-readable `{"error":"lab.repetitions must be at least 3","ok":false}`.
- A real QEMU execution could not be performed because `qemu-x86_64` is not installed in this verification environment. The tool's mismatch path was exercised instead; no claim of actual emulated-CPU execution is made here.

## Defects

### Medium — `check` accepts an invalid mutating workload

Reproduction from the clean consumer artifact:

1. Run `sqlite-workload-lab init lab.toml`.
2. Change the generated query SQL to `DELETE FROM docs` without changing the pinned fixture.
3. Run `sqlite-workload-lab --json check mutating.toml`.

Observed: exit code 0 and `{"ok":true,"action":"check",...}`.  
Expected: non-zero validation failure before a workload is accepted, because measured queries must be read-only and the CLI's `check` description promises SQL-shape validation.  
Safety recovery: `sqlite-workload-lab --json run mutating.toml --profile host` did exit 1 with `query release-evidence mutates the database; measured queries must be read-only`; no mutation occurs because execution uses an isolated temporary database. The late rejection does not make the preflight truthful.

### Low — service-worker cache name is not versioned

`site/public/sw.js` hard-codes `sqlite-workload-lab-v1`; the update cleanup branch therefore cannot distinguish a future release cache. The deployed worker currently activates and `registration.update()` is clean, but successive releases will retain old runtime-cached hashed assets in the same cache. Use a release-derived cache version before the next PWA release.

## Web, privacy, accessibility, and performance evidence

- Fresh live Chromium checks on desktop and 390×844 mobile: HTTP 200, no console/page errors, no third-party outbound requests, no horizontal overflow, visible 3 px keyboard focus ring, and arrow-key tab activation selected Context.
- Live axe scan: 0 serious/critical violations on both desktop and mobile. The document has `lang=en`, one `h1`, and a `main` landmark. Reduced motion is implemented by the source media query; its animation/transition durations are reduced to `.01ms`.
- Service worker: a fresh controlled page registered `/sw.js`, `registration.update()` found no pending worker, and an offline reload rendered the h1 plus the Offline mode status with no console errors.
- Live response policy: HTTPS 200 with HSTS, `X-Content-Type-Options: nosniff`, strict-origin referrer policy, and a self-only CSP (`default-src`, `script-src`, `style-src`, `font-src`, `connect-src`). Hashed assets and the hero have `Cache-Control: public, max-age=31536000, immutable`; HTML and SW revalidate at 30 seconds.
- No networking/telemetry code was found in the Rust sources; browser request capture found only the product origin. No remote fonts/scripts are used.
- Built budgets: JS 2,074 B; CSS 14,331 B; self-hosted fonts 88,660 B combined; hero WebP 58,562 B. All meet the stated budgets.
- Lighthouse mobile against live production: Performance 100, Accessibility 100, Best Practices 100, SEO 100; FCP 1.2 s, LCP 1.5 s, TBT 90 ms, CLS 0.

## Required remediation and re-verification

Make `check` reject non-read-only SQL consistently with `run` (without executing the measured query), add a regression test for the `DELETE FROM docs` case, then rerun the clean install, `npm test`, `npm run build`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo package --locked`, and this consumer scenario. Version the service-worker cache as part of the next web release.
