# Landing-page copy audit

Audited from `site/index.html` on 2026-08-28. Counts treat hyphenated terms and numbers as one word. Interface labels and fragments are listed separately after the sentences. No sentence exceeds 22 words and no banned word appears.

| Words | Sentence |
| ---: | --- |
| 8 | Performance evidence that travels with your SQLite release. |
| 13 | For SQLite maintainers, pin fixtures and CPU profiles to catch regressions before release. |
| 10 | A fast query and a portable binary are different claims. |
| 14 | The lab keeps the workload identical while making the execution environment impossible to miss. |
| 17 | Pin SQLite, fixture bytes, PRAGMAs, repetitions, queries, and at least three CPU profiles in one TOML file. |
| 12 | Run the same binary on hardware, in a container, or under QEMU. |
| 6 | Every report keeps its evidence label. |
| 3 | Compare matching contexts. |
| 5 | Changed results stop the comparison. |
| 7 | Median regressions above your threshold stop CI. |
| 10 | This static walkthrough mirrors fields emitted by the real CLI. |
| 6 | No workload or report is uploaded. |
| 3 | Descriptive medians only. |
| 7 | The lab does not claim statistical significance. |
| 5 | Three environments. Three honest labels. |
| 11 | Best evidence for actual latency and throughput on that named machine. |
| 12 | Reproduces libraries and build context; it does not change the host CPU. |
| 12 | Exercises an older CPU model and exposes leaked AVX flags before release. |
| 3 | One Rust binary. |
| 4 | A checked-in TOML manifest. |
| 9 | No account, token, collector, hosted runner, or automatic tuning. |
| 6 | Your database never enters this page. |
| 7 | The CLI has no networking or telemetry. |
| 17 | The documentation site stores no personal data and uses no analytics, cookies, third-party scripts, or remote fonts. |
| 4 | Open source under MIT. |
| 7 | Built for claims that can be checked. |

Fragments are concise labels: “Try it with sample data”, “Run your first profile”, “JSON + Markdown”, “> 15% fails CI”, “Telemetry — None”, evidence-field names, environment names, and navigation labels. None contains a banned word.

## Terminology

| Concept | Term used |
| --- | --- |
| Declarative test definition | workload |
| Recorded command output | report |
| Execution target and requirements | CPU profile |
| Reference measurement | baseline |
| New measurement | candidate |
| Runtime proof category | evidence label |
| Allowed slowdown | threshold |
| Included try-out | sample data |
