# Verification report — FAIL

Work order: `sqlite-workload-lab-verify-2`  
Candidate: `dc125b00b97651b83826406cdf4137e380af77fe`  
Live URL: <https://sqlite-workload-lab.sociobot.in/>  
Verified: 2026-08-28

## Decision

**FAIL.** Fresh package-consumer testing reproduces a material CLI contract defect: `check` claims success for a manifest whose measured SQL is `DELETE FROM docs`, although `run` rejects that same input as a mutation. `check` is documented as validating “SQL shape,” so it cannot safely serve as the intended CI preflight. No product source was modified during this verification.

## Clean-checkout gates

| Check | Result | Evidence |
| --- | --- | --- |
| Candidate identity and clean checkout | PASS | Detached clone at exactly `dc125b00b97651b83826406cdf4137e380af77fe`; clean before generated build artifacts. |
| Install | PASS | `npm ci`: 24 packages, audit: 0 vulnerabilities. |
| Repository tests | PASS | `npm test`: 1 Rust unit, 4 Rust CLI integration, 2 built-site unit tests, and Playwright 7 passed / 1 expected mobile-only skip. |
| Exact production build | PASS | `npm run build`: locked release binary plus `dist/site/`. |
| Static analysis | PASS | `cargo clippy --locked --all-targets -- -D warnings` passed (installed the missing pinned-toolchain clippy component first). |
| Publish artifact | PASS | `cargo package --locked --allow-dirty` packaged and verified 37 files: 243.0 KiB / 112.5 KiB compressed. No publishing was attempted. |
| Clean consumer | PASS with defect below | Installed the `.crate` into a new Cargo root; `--help`, `init`, `check`, `run`, JSON output, report generation, CPU mismatch handling, and `compare` exercised. |
| Live deployment match | PASS | SHA-256 matched fresh `dist/site/` for HTML, JS, CSS, and hero WebP (values below). |

## CLI evidence

From the packaged artifact installed into `/tmp/sqlite-workload-consumer-HHUQSN/install`:

- `init lab.toml`, `check lab.toml`, and `--json run lab.toml --profile host --out reports` succeeded. The report contains SQLite version, 52 compile options, two observed PRAGMAs, an EXPLAIN plan, 92 CPU flags, timing samples, `hardware` evidence, and the explicit descriptive-only statistical-claim disclaimer.
- A deliberately doubled candidate median caused `compare reports/host.json candidate.json --threshold 15` to exit **2**, with `Gate: FAIL` and a +105.26% regression.
- Boundary validation worked: `repetitions = 2` caused `--json check` to exit **1** with `lab.repetitions must be at least 3`.
- CPU-compatibility recovery worked: the local AVX/AVX2 host was rejected for `emulated-x86-v2` (exit **1**, forbidden AVX and AVX2); `--allow-profile-mismatch` produced an `emulator`-labelled report with `profile_match: false` and both mismatches retained. QEMU itself is not installed, so no real QEMU execution claim is made.

## Defects

### Medium — `check` accepts mutating measured SQL

Reproduction against the clean installed package:

1. `sqlite-workload-lab init lab.toml`
2. Change the query line to `sql = "DELETE FROM docs"` while preserving the fixture pin.
3. Run `sqlite-workload-lab --json check mutating.toml`.

Observed: exit **0** and:

```json
{"ok":true,"action":"check","message":"Valid: 1 queries across 3 CPU profiles; SQLite 3.50.2."}
```

Expected: `check` must fail before reporting the manifest valid, because measured queries must be read-only and its command help promises SQL-shape validation. As a safety recovery, `sqlite-workload-lab --json run mutating.toml --profile host` exited **1** with `query release-evidence mutates the database; measured queries must be read-only`; no report directory was created. The late rejection does not make a successful preflight truthful.

### Low — service-worker cache version is fixed

`site/public/sw.js` uses the fixed cache key `sqlite-workload-lab-v1`. The deployed worker currently controls the page, `registration.update()` has no waiting worker, and offline reload works, but future releases cannot cleanly distinguish cache generations. Use a release-derived cache version before the next web release.

## Live web, privacy, security, and performance evidence

- Fresh Chromium desktop (1440px) and mobile (390×844) runs: HTTP 200, no console/page errors, no non-product-origin requests, no horizontal overflow, `lang=en`, exactly one `h1`, and a `main` landmark. Axe found **0 serious/critical** violations on each viewport.
- Keyboard-only checks: the skip link received a visible `rgb(97, 231, 205) solid 3px` focus outline; ArrowRight changed the selected report tab from Run to Context. Reduced-motion computed animation and transition duration as `0.00001s`.
- PWA: service worker is controlling, has no waiting update after `registration.update()`, and a fresh offline reload retained the H1 and the “Offline mode” status with no errors.
- Privacy: Rust/site source scan found no telemetry/network client; browser capture observed only the product origin. The site has no remote fonts/scripts or analytics. The external GitHub links were not followed during load.
- Security response policy: HTTPS HTML returned HSTS, `nosniff`, `strict-origin-when-cross-origin`, and CSP `default-src 'self'` with self-only script/style/font/connect directives. HTML and `/sw.js` revalidate at 30 seconds; hashed JS/CSS and hero are `public, max-age=31536000, immutable`.
- Fresh Lighthouse mobile JSON (2026-08-28 07:10 UTC): Performance **99**, Accessibility **100**, Best Practices **100**, SEO **100**; FCP 1.4 s, LCP 1.7 s, TBT 110 ms, CLS 0. Lighthouse wrote results but its browser tab crashed during cleanup; the completed result JSON was preserved.
- Built assets: JS 2,074 B; CSS 14,331 B; fonts 88,660 B combined; hero 58,562 B. All are within the stated budgets.
- No server-side product API exists on this static deployment, therefore rate-limit testing and a `429` threshold are not applicable. There is no sign-in.

### Candidate/live SHA-256 matches

| Asset | SHA-256 |
| --- | --- |
| `index.html` | `bd52301bfe3f0909f626615e266f2c33f41ed50b3bb1fa749256992bea0f4fed` |
| `assets/index-DNY6hJnh.js` | `c008f3b395f464f33224d7de89a552d8c5362e03c28202a590e99edf8e8f421d` |
| `assets/index-CUpl0W1U.css` | `e301d6c7495a8153f882d1b09e13a4198ad447eb7dc6d45eaf54be9215e40e95` |
| `lab-landscape.webp` | `28fb23959f50e0a57fed2e9242f97eef562f78ce0dde5d4705e7c590a8e3e92f` |

## Required remediation

Make `Manifest::validate` (and thus `check`) reject non-read-only measured SQL without executing it; add the `DELETE FROM docs` regression test. Version the service-worker cache. Then rerun the clean install, `npm test`, `npm run build`, clippy, `cargo package --locked`, the clean consumer flow, and live deployment identity comparison.
