import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

test("@claim:cli-local the CLI demo uses only local code and files", async () => {
  const cargo = await readFile("Cargo.toml", "utf8");
  const sources = await Promise.all([
    "src/main.rs",
    "src/lib.rs",
    "src/manifest.rs",
    "src/runner.rs",
    "src/report.rs",
    "src/compare.rs",
  ].map((path) => readFile(path, "utf8")));
  assert.doesNotMatch(cargo, /\b(reqwest|ureq|hyper|tonic|telemetry|analytics)\b/i);
  assert.doesNotMatch(sources.join("\n"), /\b(TcpStream|UdpSocket|http::|https?:\/\/)\b/);

  const parent = await mkdtemp(join(tmpdir(), "sqlite-workload-local-"));
  const output = join(parent, "demo");
  try {
    execFileSync("target/debug/sqlite-workload-lab", ["demo", "--out", output], {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    });
    const report = JSON.parse(await readFile(join(output, "reports/host.json"), "utf8"));
    assert.equal(report.environment.evidence_kind, "hardware");
  } finally {
    await rm(parent, { recursive: true, force: true });
  }
});
