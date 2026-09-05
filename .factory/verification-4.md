# Verify SQLite workload evidence before release — FAIL

Work order: `sqlite-workload-lab-verify-4`  
Implementation reviewed: `699737e2d31e66c829d3348ba1166d2ce6bad37b`  
Documentation/report commit: `7df5dc9548d57563f38db0e9cc584e9b73efb6d2`  
Live URL: <https://sqlite-workload-lab.sociobot.in/>  
Verified: 2026-09-05

## Job, audience, and first action

The job is to create comparable SQLite workload evidence before a release. It is for SQLite maintainers who need to check performance and CPU compatibility. On both fresh desktop and 390×844 phone pages, the first action is **Try it with sample data**. It links to `#demo` and, after the smooth scroll completes, shows a populated recorded CLI report.

## Verdict

**FAIL — 5 findings and 6 untested public claims.**

The implementation fixes all earlier verifier findings. The CLI and its declared claims pass. The live site still does not meet the required plain-language, demo-sandbox, claims, route, and metadata contracts. No product source was changed during verification.

## Prior findings

| Earlier finding | Current disposition | Evidence |
| --- | --- | --- |
| `check` accepted mutating measured SQL | Fixed | Clean installed package: `--json check` on `DELETE FROM docs` exits 1 with `ok:false`; the exact `@claim:read-only-preflight` command passed. |
| Service-worker cache name was fixed | Fixed | Live controlled page used only `sqlite-workload-lab-0.1.0-aced19b000d3`; no waiting worker; offline reload passed. |
| Parser errors used exit 2 and non-JSON stderr | Fixed | Clean installed package: `--json nonsense` exits 1, keeps stdout empty, and emits one `ok:false` JSON document. Exact parser regression tests passed in `npm test`. |
| Three phone links were under 44 px high | Fixed | The mobile Playwright touch-target test passed; the repair tests cover the schema, GitHub, and Privacy links. |
| Immutable hero URL was unversioned | Fixed | Live and local use `/lab-landscape-28fb23959f50.webp`; the bytes match SHA-256 `28fb23959f50…`, and the response is immutable. |

## Findings

### Medium — the first screen does not state the job in plain words

Live `<title>` and the only `<h1>` are `SQLite Workload Lab — performance evidence that travels` and `Performance evidence that travels with your SQLite release.` The latter is a metaphor, not a plain-language job headline, and does not say that the tool runs and compares pinned SQLite workloads. The demo heading, `Follow the evidence plane by plane.`, is also a mood heading. This fails the plain-words first-screen and title requirements despite the useful audience sentence and primary action.

### Medium — the one-click sample is not a complete demo sandbox

The primary action only changes the fragment to `#demo` and scrolls to an HTML walkthrough. The fresh desktop and phone pages contain neither a persistent `Demo — sample data, nothing is saved` label nor a Reset demo action. The walkthrough is labelled `Recorded walkthrough`, not sample data, and is not a self-hosted terminal recording/asciinema/SVG of the binary as required for a CLI. The installed `sqlite-workload-lab demo` command itself is safe: it creates a new directory and refuses overwrite. That does not make the landing action a complete, labelled, resettable one-click sandbox.

### Medium — required product routes and product 404 are absent

`/privacy` and `/terms` both return HTTP 404. `/404` and an arbitrary unknown route also return the Azure Static Web Apps generic page, which loads Azure-hosted resources and provides no product-styled way back. The footer has only a `#privacy` fragment, no Terms link, no Param Factory attribution, and no version/build id. `sitemap.xml` lists only `/`. This fails the required real-URL, legal-page, footer, sitemap, and designed-404 structure.

### Medium — six public claims are outside `.factory/claims.json`

The claims file has one tagged test for each of its eight entries, and all eight declared commands pass. It does not list or test these visitor-facing claims:

1. Reports are always deterministic JSON plus Markdown (`README`).
2. Every result is labelled hardware, virtualized, container, or emulated evidence (`README`).
3. An emulated run is never presented as a hardware claim (`README`).
4. The documented command works under QEMU (`README`).
5. The lab does not claim statistical significance (`README` and live demo).
6. There is no account, token, collector, hosted runner, or automatic tuning (`live install section`).

The existing evidence test checks one hardware report and the override path checks one emulator report. It does not prove the broader category, determinism, QEMU, statistics, or no-account/hosted-runner claims. The claims contract requires each public claim to have an entry and a corresponding sandbox test, or for the copy to be removed.

### Low — required discoverability metadata is incomplete

The live and built pages omit a canonical URL, Open Graph metadata, Twitter card metadata, an Apple touch icon, and the required 1200×630 product image. The title is under 60 characters but is not plain words. The test suite does not cover these omissions.

## Passed checks

| Check | Result | Evidence |
| --- | --- | --- |
| Clean setup | PASS | `npm ci`: 26 packages; 0 audit vulnerabilities. |
| Main suite | PASS | `npm test`: 1 Rust unit, 10 CLI integration, 5 Node site tests, and Playwright 10 passed / 2 expected viewport skips. |
| Declared claims | PASS | All eight exact commands from `.factory/claims.json` passed. |
| Typecheck and lint | PASS | `npm run typecheck`; `npm run lint` after installing the pinned toolchain's missing `rustfmt` and `clippy` components. |
| Production build and package | PASS | `npm run build`; `cargo package --locked --allow-dirty` (46 files; 283.8 KiB unpacked, 124.9 KiB compressed). |
| Clean package consumer | PASS | Fresh Cargo home and install root installed the packaged crate. Help, version, demo, no-overwrite recovery, init, check, JSON host run, parser error, mutation rejection, repetition boundary, strict matrix failure, mismatch override, and regression exit 2 behaved as documented. |
| Candidate/live identity | PASS | Fresh SHA-256 values match for HTML, service worker, hero, mark, JS, and CSS. |
| Live desktop and phone | PASS except findings above | Fresh 1440×900 and 390×844 contexts had zero console/page errors, zero axe violations, no overflow, same-origin requests only, no cookies, visible skip-link focus, and working arrow-key tabs. |
| Motion, offline, privacy, headers | PASS | Reduced motion computes `0.00001s` durations and `scroll-behavior:auto`; the controlled live worker reloads offline with the H1 and Offline mode status; HTTPS has HSTS, self-only CSP, nosniff, and strict-origin referrer policy. |

`/opt/fleet/lib/verify-url.sh https://sqlite-workload-lab.sociobot.in/ /work/.evidence/verify-live-4` passed. Its fresh browser report found a title, `lang=en`, one H1, main landmark, image alt text, labelled buttons, and no console errors. Playwright axe was used because `@axe-core/cli` could not locate a system Chrome binary in this container.

## Required repair and re-verification

Replace the title, H1, and demo heading with plain job-based language. Make the landing demo an actual labelled CLI recording and provide the required sample/reset semantics, or adjust the product to a direct, testable CLI demo entry. Add real `/privacy` and `/terms` pages, a product 404 page, sitemap entries, complete footer, and required metadata. Inventory every public claim, add one tagged sandbox test per retained claim, and remove unsupported claims. Then redeploy and repeat the exact claim commands, clean package-consumer flow, and fresh live desktop/phone checks.
