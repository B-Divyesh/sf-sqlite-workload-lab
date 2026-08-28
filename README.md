# SQLite Workload Lab

SQLite Workload Lab is a zero-telemetry CLI for maintainers who need reviewable evidence before publishing a SQLite performance claim or binary. It runs a declarative, pinned workload; records SQLite build details, PRAGMAs, query plans, CPU features, and repeated timings; then produces stable JSON and Markdown reports that CI can compare.

The tool labels every result as hardware, virtualized, container, or emulated evidence. It does not turn an emulated run into a hardware claim, and it does not call a small timing difference statistically significant.

## Install

Build the single binary with the pinned Rust toolchain:

```sh
cargo install --path .
sqlite-workload-lab --help
```

Release archives are produced by the factory; this repository does not publish itself.

## Usage

Create a documented starter workload:

```sh
sqlite-workload-lab init lab.toml
```

Validate pins and declared CPU profiles without executing SQL:

```sh
sqlite-workload-lab check lab.toml
```

Run one profile in the environment that represents it. The same command works inside a container or under QEMU; `environment = "container"` or `"emulator"` keeps that evidence visibly separate in the report.

```sh
sqlite-workload-lab run lab.toml --profile x86-64-v2 --out reports/candidate
```

Run every profile whose runner is `native` in the current environment:

```sh
sqlite-workload-lab matrix lab.toml --out reports/candidate
```

Compare a candidate with a committed baseline. A regression over 15% exits with code 2, which makes the command a useful CI gate.

```sh
sqlite-workload-lab compare \
  reports/baseline/x86-64-v2.json \
  reports/candidate/x86-64-v2.json \
  --threshold 15 \
  --markdown reports/candidate/x86-64-v2-diff.md
```

Add `--json` before a subcommand for machine-readable command status. Reports are always deterministic JSON plus a human-readable Markdown companion unless `--format` selects one.

### Workload format

```toml
schema_version = 1

[lab]
name = "search-release"
database = "tmp/search.db"
fixture = "fixtures/search.sql"
fixture_sha256 = "<sha256>"
sqlite_version = "3.50.4"
warmups = 2
repetitions = 12

[[profiles]]
id = "x86-64-v2"
environment = "emulator" # hardware | virtualized | container | emulator
runner = "native"        # execute this profile in its declared environment
required_cpu_features = ["sse4_2", "popcnt"]
forbidden_cpu_features = ["avx", "avx2"]

[[profiles]]
id = "x86-64-v3"
environment = "container"
runner = "native"
required_cpu_features = ["avx", "avx2"]

[[profiles]]
id = "arm64"
environment = "hardware"
runner = "native"
required_cpu_features = ["asimd"]

[[pragmas]]
name = "journal_mode"
value = "WAL"

[[queries]]
name = "phrase-search"
sql = "SELECT rowid FROM docs WHERE docs MATCH 'sqlite NEAR workload' LIMIT 20"
capture_plan = true
```

The fixture hash and SQLite version are mandatory pins. `check` fails on a missing fixture, a mismatched SHA-256, duplicate names, unsafe PRAGMA names, or fewer than three profiles. `run` fails when required CPU features are absent or forbidden features are present; use `--allow-profile-mismatch` only to investigate, and the report will carry the mismatch.

## Exit codes

| Code | Meaning |
| ---: | --- |
| 0 | Command succeeded; comparison is within threshold |
| 1 | Invalid workload, missing dependency/file, SQLite error, or incompatible profile |
| 2 | Candidate regression exceeded the configured threshold |

## Development

Requirements: Rust 1.85+ and Node 22+.

```sh
npm install
npm test
npm run build
```

`npm test` runs Rust unit/integration tests and site tests. `npm run build` creates the release CLI and the static docs site at `dist/site/` (with `index.html` at that root). For local docs development use `npm run dev`; for a production preview use `npm run preview`.

Ready-to-publish checks:

```sh
cargo package --allow-dirty
npm run build:site
```

## CI pattern

Run the same manifest on three explicit runners (for example QEMU x86-64-v2, an x86-64-v3 container host, and arm64 hardware), upload each JSON report, then gate each result with `compare`. Do not merge reports from unlike profiles: the comparator rejects profile, SQLite version, fixture, and workload mismatches by default.

## Privacy and scope

All workload execution and reports stay local. The CLI has no networking, analytics, crash reporting, or automatic upload. The docs site also has no tracking and works offline after its first visit. This is a reproducibility harness, not an ORM, hosted benchmark farm, or automatic query tuner.

## License

MIT. See [LICENSE](LICENSE).
