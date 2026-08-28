# Verification report — FAIL

Work order: `sqlite-workload-lab-verify-3`  
Candidate: `cc8e9a5417c010335b2acd8ebeff2b99f931334a`  
Live URL: <https://sqlite-workload-lab.sociobot.in/>  
Verified: 2026-08-28 08:58 UTC

## Decision

**FAIL.** The repaired workload validation and service-worker versioning work, but the packaged CLI breaks its documented scripting contract for parser-level invalid input. With `--json`, Clap errors are human text rather than JSON and exit with code 2, although the README reserves code 2 for a measured regression. A malformed CI invocation can therefore be mistaken for a performance regression. No product source was modified.

## Clean-checkout gates

| Check | Result | Evidence |
| --- | --- | --- |
| Candidate identity | PASS | Fresh detached clone at exactly `cc8e9a5417c010335b2acd8ebeff2b99f931334a`; clean before and after verification. |
| Install | PASS | Node 22.23.2, npm 10.9.8, Rust 1.98.0; `npm ci` installed 24 packages with 0 audit vulnerabilities. |
| Repository suite | PASS | `npm test`: 1 Rust unit, 5 Rust CLI integrations, 3 built-site unit tests, and Playwright 7 passed / 1 intentional desktop skip. |
| Format and lint | PASS | `cargo fmt --all -- --check` and `cargo clippy --locked --all-targets -- -D warnings` passed after installing the pinned-toolchain components in the disposable verifier. No JS lint/typecheck script exists. |
| Exact production build | PASS | `npm run build` produced the locked release binary and `dist/site/`. |
| Publish artifact | PASS | `cargo package --locked --allow-dirty` packaged and verified 39 files, 257.1 KiB unpacked / 116.7 KiB compressed. Nothing was published. |
| Clean consumer | FAIL | The `.crate` installed into fresh Cargo/install roots and normal flows passed, but parser-level invalid input violates the documented JSON/exit-code API as detailed below. |
| Live deployment identity | PASS | Fresh local/live SHA-256 values match for HTML, JS, CSS, hero, logo, and service worker. |

## Defects

### Medium — parser errors violate both `--json` and exit-code contracts

The README says `--json` provides machine-readable command status and documents exit 1 for invalid input and exit 2 only when a candidate regression exceeds the threshold. Parser-level invalid input bypasses the application's JSON error path and uses Clap's human-text/exit-2 behavior.

Reproduced against the clean installed package:

```text
$ sqlite-workload-lab --json nonsense
exit 2; stdout 0 bytes
stderr: error: unrecognized subcommand 'nonsense'

$ sqlite-workload-lab --json compare reports/host.json reports/host.json --threshold nope
exit 2; stdout 0 bytes
stderr: error: invalid value 'nope' for '--threshold <THRESHOLD>': invalid float literal

$ sqlite-workload-lab --json run lab.toml
exit 2; stdout 0 bytes
stderr: error: the following required arguments were not provided:
```

Expected: when `--json` is present, these failures should emit the same machine-readable `{"ok":false,"error":...}` shape as workload validation errors and use the documented invalid-input exit code 1. Exit 2 must remain unambiguous for the regression gate.

### Low — three mobile links are below the required 44 px touch height

At the requested 390×844 viewport, rendered hit areas were:

- “Read the full workload schema”: 251.4×20 px.
- Footer “GitHub”: 45×21.7 px.
- Footer “Privacy”: 49×21.7 px.

There is no overlap and axe reports no violation, but these miss the attached accessibility contract's explicit 44×44 CSS-pixel target baseline.

### Low — an unversioned hero URL is cached as immutable for one year

`/lab-landscape.webp` is referenced without a content hash or version and returns `Cache-Control: public, max-age=31536000, immutable`. A future deployment that changes the image at the same URL can leave returning clients—and potentially a new service-worker install using the HTTP cache—with stale art for up to a year. Immutable caching should be limited to content-addressed URLs or the hero URL should be versioned.

## CLI end-to-end evidence

The packed crate was unpacked and installed into clean Cargo and install roots. `--help` is useful and non-interactive, and `--version` reports 0.1.0.

- `--json init`, `check`, and `run --profile host` succeeded. The run emitted JSON and Markdown containing SQLite 3.50.2/source build data, 52 compile options, observed PRAGMAs, an EXPLAIN plan, 92 CPU flags, 10 timing samples, fixture/manifest hashes, a hardware label, and the descriptive-only statistics disclaimer. No workload database persisted outside the isolated temporary run.
- Strict `matrix` stopped with exit 1 at `emulated-x86-v2` because AVX and AVX2 were present. `matrix --allow-profile-mismatch` emitted all three declared reports, preserving `profile_match:false`, both mismatches, and an `emulator` evidence label. QEMU is not installed in this worker, so no actual QEMU execution claim is made.
- Identical reports compared at threshold 15 with exit 0. A synthetic exact +15% boundary remained stable with exit 0; +16% was a regression with exit 2. Changed context failed unless explicitly allowed, and a changed result digest always stopped comparison.
- Validation accepted repetition boundaries 3 and 10,000 and rejected 2 and 10,001. It also rejected warmups 1,001, a bad fixture hash, invalid SQL, SQLite version mismatch, unknown profile, overwrite, and `DELETE FROM docs`, all with exit 1 and JSON errors. Returning to the valid manifest recovered successfully.

## Live web, privacy, accessibility, and performance

- `/opt/fleet/lib/verify-url.sh` passed: HTTPS 200, title, `lang=en`, one H1, main landmark, image alts, labelled buttons, and no console errors.
- Fresh Chromium at 1440×900 and 390×844 had no page/console/request errors or horizontal overflow. Axe found 0 violations, including 0 serious/critical findings. Manual keyboard checks reached the skip link first with a visible 3 px aqua focus ring; tab arrow-key behavior and the clipboard action worked.
- `prefers-reduced-motion: reduce` computed animation and transition durations to `0.00001s` and disabled smooth scrolling.
- Request capture observed only `https://sqlite-workload-lab.sociobot.in`; no cookies, analytics, remote scripts, or remote fonts were loaded. Rust source contains no networking/telemetry implementation.
- The versioned worker controlled the page, `registration.update()` left no waiting worker, only cache `sqlite-workload-lab-0.1.0-11b56703c0da` remained, and an offline reload retained the H1 and “Offline mode” status.
- Live headers include HSTS, self-only CSP, `nosniff`, and strict-origin referrer policy. HTTP redirects to HTTPS; unknown routes return 404. HTML and `sw.js` revalidate after 30 seconds; hashed JS/CSS are one-year immutable. Conditional requests returned 304.
- Built budgets: JS 2,074 B; CSS 14,331 B; fonts 88,660 B combined; hero WebP 58,562 B.
- Fresh Lighthouse mobile: Performance 99, Accessibility 100, Best Practices 100, SEO 100; FCP 1.22 s, LCP 1.52 s, TBT 128 ms, CLS 0.
- This is a static site with no server-side API, product-unlock endpoint, or sign-in. API rate-limit and Entra authority tests are not applicable.

## Candidate/live SHA-256 identity

| Asset | SHA-256 |
| --- | --- |
| `index.html` | `bd52301bfe3f0909f626615e266f2c33f41ed50b3bb1fa749256992bea0f4fed` |
| `assets/index-DNY6hJnh.js` | `c008f3b395f464f33224d7de89a552d8c5362e03c28202a590e99edf8e8f421d` |
| `assets/index-CUpl0W1U.css` | `e301d6c7495a8153f882d1b09e13a4198ad447eb7dc6d45eaf54be9215e40e95` |
| `lab-landscape.webp` | `28fb23959f50e0a57fed2e9242f97eef562f78ce0dde5d4705e7c590a8e3e92f` |
| `lab-mark.svg` | `8257d9e74d7cad2554891efe18eb12a3843abc52bb00bb2e533f280d9c5cb5d1` |
| `sw.js` | `7b5e76c5577e9781bb439575de34e36beeb40c8ba223e6c1c1ac0f4b2ad4c44f` |

## Required remediation

Handle parse failures through the CLI's JSON-aware error path and preserve exit code 2 exclusively for actual comparison regressions; add integration coverage for an unknown subcommand, missing required `run` options, and invalid threshold under `--json`. Enlarge the three mobile hit areas and either content-hash/version the hero URL or stop marking it immutable. Then repeat package-consumer and live verification.
