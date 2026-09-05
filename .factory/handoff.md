# SQLite Workload Lab — repair 3 handoff

> Verification update, 2026-09-05: **FAIL** at implementation `699737e2d31e66c829d3348ba1166d2ce6bad37b` and documentation commit `7df5dc9548d57563f38db0e9cc584e9b73efb6d2`. The repaired CLI contracts, claimed tests, package consumer flow, live identity, accessibility smoke checks, privacy, and offline path pass. The live site still has five release-blocking verification findings: non-plain first-screen copy, incomplete one-click demo sandbox, missing real Privacy/Terms/product-404 routes, six untested public claims, and incomplete metadata. See `.factory/verification-4.md` for commands, evidence, and repairs needed. No product code was changed by this verifier.

## Status

**REPAIRED, PUSHED, AND DEPLOYED** on 2026-08-28.

- Work order: `sqlite-workload-lab-repair-3`
- Failed candidate: `cc8e9a5417c010335b2acd8ebeff2b99f931334a`
- Verifier report commit: `3c96ab88a20e819a19ca32bec2d82621f9d8b564`
- Product repair commit: `699737e2d31e66c829d3348ba1166d2ce6bad37b`
- Live URL: <https://sqlite-workload-lab.sociobot.in/>
- Azure Static Web Apps deployment: `0029f0ab-ebec-457b-9487-30cfab98786d`

The Rust CLI artifact and static deployment class are unchanged. The researched brief and luminous-glass visual thesis are preserved.

## Repairs

### JSON parser and exit-code contract

The CLI now parses with `Cli::try_parse_from`. Help and version still print normally with exit 0. Every parser-level invalid invocation returns exit 1; if `--json` is present, stderr is one valid `{"ok":false,"error":...}` document and stdout stays empty. Exit 2 remains exclusive to a completed comparison with a measured regression.

Exact integration coverage in `tests/cli.rs` parses stderr as JSON and asserts exit 1 for:

- unknown subcommand: `--json nonsense`;
- missing required option: `--json run lab.toml`;
- invalid numeric option: `--json compare baseline.json candidate.json --threshold nope`.

The existing regression comparison still asserts exit 2 and `Gate: FAIL`.

### Mobile touch targets

The schema link and footer links are now inline flex targets with a 44 px minimum height. The 390 px Playwright regression measures the exact three reported links. Live rendered sizes are:

| Link | Width | Height |
| --- | ---: | ---: |
| Read the full workload schema | 247.4 px | 44 px |
| GitHub | 45 px | 44 px |
| Privacy | 49 px | 44 px |

### Immutable hero cache

The hero is now served as `/lab-landscape-28fb23959f50.webp`, where `28fb23959f50` is the first 12 characters of the file's SHA-256 digest. HTML, service-worker precache input, release digest input, deployment cache policy, visual provenance, and budget tests all use that URL. The regression test requires the hashed URL everywhere, requires the immutable cache rule, and rejects the former unversioned build output.

Live evidence: the content-addressed hero returns `Cache-Control: public, max-age=31536000, immutable`; a conditional request returns 304; the old `/lab-landscape.webp` returns 404.

## Demo and claims

`sqlite-workload-lab demo` now runs the bundled FTS5 sample in a uniquely named temporary directory and prints every artifact path. `--out <new-directory>` keeps it at a chosen location and refuses overwrite. The first screen links directly to the recorded sample report. `.factory/demo.md`, `.factory/claims.json`, and `.factory/copy-audit.md` document the sandbox, claim tests, and plain-language audit. Each listed claim has one matching `@claim:<id>` test.

## Verification evidence

Commands run from the repository root:

```sh
npm ci
npm test
npm run typecheck
npm run lint
npm run build
cargo package --locked --allow-dirty
```

Results:

- Clean install: 26 packages; npm audit found 0 vulnerabilities.
- Rust: 1 unit test and 10 CLI integration tests passed.
- Site/claim unit tests: 5 passed.
- Playwright 1.58.2: 10 passed and 2 expected viewport skips across desktop Chromium and 390×844 mobile.
- TypeScript strict typecheck, rustfmt, and clippy with warnings denied: passed.
- Production build: locked release binary and `dist/site/` completed.
- Crate package: 46 files; 279.1 KiB unpacked and 123.5 KiB compressed; Cargo's package verification build passed. Nothing was published.
- Built payloads: JS 2,074 B; CSS 14,441 B; fonts 88,660 B combined; hero WebP 58,562 B.

### Clean package consumer

The packed crate was installed into fresh Cargo and install roots. The installed `0.1.0` binary passed help/version, demo, init, check, host run, strict matrix mismatch, override matrix, and regression comparison flows.

- All three reported malformed `--json` invocations: exit 1, empty stdout, parseable `ok:false` stderr.
- Synthetic regression above 15%: exit 2 with `Gate: FAIL`.
- Strict emulated profile on this AVX/AVX2 host: exit 1 with both forbidden-feature mismatches.
- Explicit mismatch override: three report pairs; emulated report retained `profile_match:false`, both mismatches, and `evidence_kind:"emulator"`.

### Browser, accessibility, privacy, and offline

`/opt/fleet/lib/verify-url.sh` passed locally and live: HTTP 200, title, `lang=en`, one H1, main landmark, image alt text, labelled buttons, and no console errors.

Fresh local and live Chromium checks at 1440×900 and 390×844 found:

- zero axe violations;
- zero console or page errors and zero horizontal overflow;
- only the product origin requested and no cookies;
- skip link first in tab order with `rgb(97, 231, 205) solid 3px` focus;
- ArrowRight focuses and selects the Context tab;
- reduced-motion animation and transition durations of `0.00001s`;
- service worker controlling, no waiting worker, and only `sqlite-workload-lab-0.1.0-aced19b000d3` cached;
- offline reload retained the H1 and displayed the Offline mode status.

Live Lighthouse mobile: Performance 100, Accessibility 100, Best Practices 100, SEO 100; FCP 1.20 s, LCP 1.50 s, TBT 0 ms, CLS 0.

### Live response policy and identity

HTTPS HTML returns HSTS, self-only CSP, `nosniff`, and strict-origin referrer policy. HTTP redirects to HTTPS. HTML and `sw.js` revalidate after 30 seconds. Hashed JS/CSS and the content-addressed hero are one-year immutable. Unknown routes return 404.

Fresh local/live SHA-256 matches:

| Asset | SHA-256 |
| --- | --- |
| `index.html` | `4e8d3eb50d51c7bab33abc12949530e0b8ab3d75d4010d1e7f9902bb7022d539` |
| `sw.js` | `3f83b8d4bb229ae3b340c3840db092e32fa4f84cf2240831387ca66e00759cdb` |
| `lab-landscape-28fb23959f50.webp` | `28fb23959f50e0a57fed2e9242f97eef562f78ce0dde5d4705e7c590a8e3e92f` |
| `lab-mark.svg` | `8257d9e74d7cad2554891efe18eb12a3843abc52bb00bb2e533f280d9c5cb5d1` |
| `assets/index-CNvyn9SR.js` | `c008f3b395f464f33224d7de89a552d8c5362e03c28202a590e99edf8e8f421d` |
| `assets/index-DiShM6xl.css` | `d07f0f4fed90e19daf5ed884dde0c2fc1ce18029682454bf9256001053403ec5` |

## Known gaps

No release-blocking gap remains. QEMU is not installed in this worker, so no new claim of real QEMU execution is made; strict CPU mismatch and the explicit investigative override were verified. This static product has no server API, sign-in, payment, or remote data store, so API rate-limit and identity-provider checks are not applicable.
