import assert from "node:assert/strict";
import { readFile, stat } from "node:fs/promises";
import { test } from "node:test";
import { releaseId } from "../release-id.js";

test("built document contains required semantics", async () => {
  const html = await readFile("dist/site/index.html", "utf8");
  assert.match(html, /<html lang="en">/);
  assert.equal((html.match(/<h1[ >]/g) ?? []).length, 1);
  assert.match(html, /<main id="main">/);
  assert.match(html, /<title>SQLite Workload Lab/);
  assert.doesNotMatch(html, /https:\/\/(fonts|cdn)\./);
});

test("static assets stay inside product budgets", async () => {
  const hero = await stat("dist/site/lab-landscape-28fb23959f50.webp");
  assert.ok(hero.size <= 300 * 1024, `hero is ${hero.size} bytes`);
  const index = await readFile("dist/site/index.html", "utf8");
  const scripts = [...index.matchAll(/<script[^>]+src="([^"]+)"/g)].map((match) => `dist/site${match[1]}`);
  const styles = [...index.matchAll(/<link[^>]+href="([^"]+\.css)"/g)].map((match) => `dist/site${match[1]}`);
  const jsBytes = (await Promise.all(scripts.map((path) => stat(path)))).reduce((sum, value) => sum + value.size, 0);
  const cssBytes = (await Promise.all(styles.map((path) => stat(path)))).reduce((sum, value) => sum + value.size, 0);
  const builtFiles = await Array.fromAsync((await import("node:fs/promises")).glob("dist/site/assets/*.woff2"));
  const fontBytes = (await Promise.all(builtFiles.map((path) => stat(path)))).reduce((sum, value) => sum + value.size, 0);
  assert.ok(jsBytes <= 200 * 1024, `initial JS is ${jsBytes} bytes`);
  assert.ok(cssBytes <= 50 * 1024, `initial CSS is ${cssBytes} bytes`);
  assert.ok(fontBytes <= 120 * 1024, `self-hosted fonts are ${fontBytes} bytes`);
});

test("service-worker cache is versioned for this release", async () => {
  const worker = await readFile("dist/site/sw.js", "utf8");
  assert.match(worker, new RegExp(`const CACHE = "sqlite-workload-lab-${releaseId}"`));
  assert.doesNotMatch(worker, /sqlite-workload-lab-v1/);
  assert.doesNotMatch(worker, /__RELEASE_ID__/);
});

test("immutable hero asset uses its content hash in the URL", async () => {
  const html = await readFile("dist/site/index.html", "utf8");
  const worker = await readFile("dist/site/sw.js", "utf8");
  const policy = await readFile("dist/site/staticwebapp.config.json", "utf8");
  const heroUrl = "/lab-landscape-28fb23959f50.webp";
  assert.match(html, new RegExp(`src="${heroUrl}"`));
  assert.match(worker, new RegExp(`"${heroUrl}"`));
  assert.equal(JSON.parse(policy).routes.find((route) => route.route === heroUrl).headers["Cache-Control"], "public, max-age=31536000, immutable");
  await stat(`dist/site${heroUrl}`);
  await assert.rejects(stat("dist/site/lab-landscape.webp"));
});
