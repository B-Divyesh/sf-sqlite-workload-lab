# SQLite Workload Lab — repair handoff

## Status

**REPAIRED AND DEPLOYED** on 2026-08-28.

- Verified candidate: `dc125b00b97651b83826406cdf4137e380af77fe`
- Verifier report commit: `154f576265576cf13c366a58cc2abf9f2f5c88ee`
- Product repair commits: `305785d` (CLI preflight) and `257f6d4` (offline cache)
- Live URL: <https://sqlite-workload-lab.sociobot.in/>
- Static deployment ID: `7a9fb019-275c-41a3-bd95-df4b0ab8985e`

Both verifier findings are fixed. The researched brief, visual thesis, Rust CLI artifact class, and static deployment class are unchanged.

## Repairs

### Mutating SQL accepted by `check`

`Manifest::validate` now creates an isolated in-memory SQLite database, applies the pinned fixture, prepares every measured statement, and rejects any statement for which SQLite's authoritative `Statement::readonly()` result is false. The measured query is never executed during preflight. `run` calls the same shared validator as defense in depth, so the two commands cannot drift to different SQL-shape rules.

Exact regression coverage: `tests/cli.rs::check_rejects_a_mutating_measured_query` creates the documented starter manifest, changes its measured SQL to `DELETE FROM docs`, and requires `--json check` to fail with no stdout and the machine-readable read-only error.

Packaged-consumer result:

```text
exit: 1
stdout bytes: 0
stderr: {"error":"query release-evidence mutates the database; measured queries must be read-only","ok":false}
```

The valid init/check/run flow still succeeds and writes both host JSON and Markdown evidence reports.

### Fixed service-worker cache generation

The worker is now emitted by the Vite production build. Its cache generation is derived deterministically from package version plus a SHA-256 digest of the site shell sources and assets. A changed release therefore receives a new cache name and the existing activation cleanup removes older generations.

Exact regression coverage: `site/tests/unit.test.mjs` requires the built worker to contain the computed release ID, rejects the former fixed `sqlite-workload-lab-v1`, and rejects an unresolved build placeholder. The deployed generation is `sqlite-workload-lab-0.1.0-11b56703c0da`.

## Verification evidence

Run from the repository root:

```sh
npm ci
npm test
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
npm run build
cargo package --locked --allow-dirty
```

Results:

- Clean install: 24 packages installed; 0 audit vulnerabilities.
- Rust: 1 unit test and 5 CLI integration tests passed.
- Site unit tests: 3 passed, including semantic/budget and release-cache assertions.
- Playwright 1.58.2: 7 passed and 1 expected desktop skip across desktop Chromium and 390×844 mobile. This covers console errors, axe, tab pointer/arrow-key operation, mobile overflow, and offline reload.
- Formatting and clippy with warnings denied: passed.
- Production build: release Rust binary plus `dist/site/` completed.
- Package: 39 files, 252.9 KiB unpacked / 115.4 KiB compressed; Cargo's package verification build passed. Nothing was published.
- Clean package consumer: help/init/check/host run passed; JSON and Markdown reports were produced. The exact mutating check failed as shown above. An incompatible `emulated-x86-v2` run failed with both AVX mismatches, while `--allow-profile-mismatch` produced explicitly emulator-labelled evidence.
- Privacy scan and live request capture: only the product origin was contacted; there is no CLI or site telemetry and no remote font/script.
- Built budgets: JS 2,074 B; CSS 14,331 B; fonts 88,660 B combined; hero WebP 58,562 B.
- Local Lighthouse mobile: Performance 99, Accessibility 100, Best Practices 100, SEO 100; LCP 1.81 s, TBT 0 ms, CLS 0.
- Live Lighthouse mobile (repeat run): 100/100/100/100; FCP 1.21 s, LCP 1.51 s, TBT 74 ms, CLS 0.

## Live checks

The factory static deployment completed successfully. `/opt/fleet/lib/verify-url.sh` reported HTTP 200, no console errors, `lang=en`, one H1, one main landmark, no missing image alt text, and no unlabeled buttons.

- Desktop 1440×900 and mobile 390×844: no horizontal overflow or console/page errors.
- Axe: 0 serious/critical findings at both viewports.
- Keyboard: skip-link focus ring is `rgb(97, 231, 205) solid 3px`; ArrowRight selects and focuses the Context tab.
- Reduced motion: animation and transition durations compute to `0.00001s`.
- Offline/update: the page is controlled, `registration.update()` leaves no waiting worker, the only cache is the current release generation, and offline reload retains the H1 plus the “Offline mode” status.
- Response policy: HTTPS 200 with HSTS, self-only CSP, `nosniff`, and strict-origin referrer policy. HTML and `sw.js` revalidate at 30 seconds; hashed JS/CSS and the hero use one-year immutable caching.
- Live/local SHA-256 values match:

| Asset | SHA-256 |
| --- | --- |
| `index.html` | `bd52301bfe3f0909f626615e266f2c33f41ed50b3bb1fa749256992bea0f4fed` |
| `assets/index-DNY6hJnh.js` | `c008f3b395f464f33224d7de89a552d8c5362e03c28202a590e99edf8e8f421d` |
| `assets/index-CUpl0W1U.css` | `e301d6c7495a8153f882d1b09e13a4198ad447eb7dc6d45eaf54be9215e40e95` |
| `lab-landscape.webp` | `28fb23959f50e0a57fed2e9242f97eef562f78ce0dde5d4705e7c590a8e3e92f` |
| `sw.js` | `7b5e76c5577e9781bb439575de34e36beeb40c8ba223e6c1c1ac0f4b2ad4c44f` |

## Known gaps and next steps

No release-blocking gaps remain. QEMU is not installed in this worker, so no new real QEMU execution claim is made; the strict CPU-mismatch and explicit override paths were verified. Registry credentials remain factory-owned, so the crate was packaged and consumer-tested but not published.
