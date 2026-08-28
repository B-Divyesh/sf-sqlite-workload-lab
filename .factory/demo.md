# Demo sandbox

## One-click web demo

Open <https://sqlite-workload-lab.sociobot.in/#demo> or choose **Try it with sample data** on the first screen. The recorded workload, context, and CI-diff tabs use fixed sample evidence. The page does not read, upload, or store a real workload.

## CLI demo

Run `sqlite-workload-lab demo`. It creates a uniquely named temporary directory, writes the bundled FTS5 fixture and pinned starter manifest there, executes the `host` sample profile, then prints the paths to its JSON and Markdown reports. Nothing is written to an existing project.

Use `sqlite-workload-lab demo --out sample-run` to keep the artifacts in a specific new directory. Delete that directory to reset the CLI demo. The command refuses to reuse or overwrite an existing directory.
