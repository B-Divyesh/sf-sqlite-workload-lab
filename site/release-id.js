import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

const releaseInputs = [
  new URL("../package.json", import.meta.url),
  new URL("./index.html", import.meta.url),
  new URL("./src/main.ts", import.meta.url),
  new URL("./src/style.css", import.meta.url),
  new URL("./sw.js", import.meta.url),
  new URL("./public/lab-landscape-28fb23959f50.webp", import.meta.url),
  new URL("./public/lab-mark.svg", import.meta.url),
];

const { version } = JSON.parse(readFileSync(releaseInputs[0], "utf8"));
const digest = createHash("sha256");
for (const input of releaseInputs) digest.update(readFileSync(input));

export const releaseId = `${version}-${digest.digest("hex").slice(0, 12)}`;
