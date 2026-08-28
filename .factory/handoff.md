# SQLite Workload Lab — verification 3 handoff

## Status

**FAIL** on 2026-08-28.

- Tested candidate: `cc8e9a5417c010335b2acd8ebeff2b99f931334a`
- Tested live URL: <https://sqlite-workload-lab.sociobot.in/>
- Full evidence: [`.factory/verification-3.md`](verification-3.md)
- Product source changes: none

The prior mutating-query and service-worker-cache defects are fixed, and the candidate's deployed static bytes match the clean production build. Release acceptance still fails because parser-level CLI errors ignore `--json` and return exit 2, which the documented public contract reserves for measured regressions. This makes malformed CI invocations indistinguishable from regression gates by exit code.

## Verification summary

- Clean `npm ci`, `npm test`, rustfmt, clippy with warnings denied, and exact `npm run build`: passed.
- `cargo package --locked --allow-dirty`: passed; 39 files, 257.1 KiB / 116.7 KiB compressed.
- Clean package install plus normal init/check/run/matrix/compare and workload-invalid recovery paths: passed apart from the parser-level JSON/exit-code defect.
- Live desktop/mobile, keyboard, reduced motion, axe, console/network, service-worker update/offline reload, headers, cache behavior, bundle budgets, and Lighthouse: exercised.
- Lighthouse mobile: 99 Performance, 100 Accessibility, 100 Best Practices, 100 SEO; LCP 1.52 s, TBT 128 ms, CLS 0.
- No API, sign-in, telemetry, cookie, or third-party load exists; rate-limit and Entra checks are not applicable.

## Defects and next steps

1. **Medium:** make all `--json` parser failures machine-readable and return invalid-input exit 1; reserve exit 2 exclusively for actual regressions. Add package-level CLI tests.
2. **Low:** enlarge the schema and footer link hit areas to at least 44 px high at 390 px.
3. **Low:** do not serve the stable `/lab-landscape.webp` URL with one-year immutable caching unless its URL is versioned/content-hashed.

After repair, rerun the commands and consumer cases listed in `.factory/verification-3.md`, rebuild, deploy, and repeat live identity plus browser verification.
