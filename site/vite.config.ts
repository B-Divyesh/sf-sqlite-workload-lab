import { defineConfig } from "vite";
import { readFileSync } from "node:fs";
import { fileURLToPath, URL } from "node:url";
import { releaseId } from "./release-id.js";

const serviceWorker = readFileSync(new URL("./sw.js", import.meta.url), "utf8")
  .replace("__RELEASE_ID__", releaseId);

export default defineConfig({
  root: fileURLToPath(new URL(".", import.meta.url)),
  publicDir: "public",
  plugins: [{
    name: "release-versioned-service-worker",
    generateBundle() {
      this.emitFile({ type: "asset", fileName: "sw.js", source: serviceWorker });
    },
  }],
  build: {
    outDir: "../dist/site",
    emptyOutDir: true,
    target: "es2022",
    sourcemap: false,
  },
  preview: {
    port: 4173,
    strictPort: true,
  },
});
