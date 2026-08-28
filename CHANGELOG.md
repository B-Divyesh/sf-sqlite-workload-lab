# Changelog

All notable changes follow Keep a Changelog. This project uses semantic versioning.

## [0.1.0] - 2026-08-28

### Added

- Declarative SQLite workload runner with pinned fixtures and SQLite version checks.
- Hardware, container, virtualized, and emulator evidence labels with CPU feature validation.
- JSON and Markdown reports containing build, PRAGMA, plan, and repeated timing evidence.
- CI comparator with a configurable regression threshold and exit code 2.
- Static documentation site with an interactive recorded report walkthrough.

### Fixed

- Made `check` reject mutating measured SQL with the same SQLite read-only classification used by `run`.
- Versioned service-worker caches from the package version and site-content digest so updates can retire stale caches.
