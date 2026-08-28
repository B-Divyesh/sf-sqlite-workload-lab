# Visual thesis: the compatibility observatory

SQLite Workload Lab uses a **luminous glass data landscape**: a dark, instrument-like field where workload traces travel through translucent SQLite pages and split across CPU strata. It should feel like evidence collected in a quiet lab, not a generic developer dashboard. The glow always carries meaning—cyan is a measured workload, lime is a compatible result, and coral is a regression or unsupported instruction set.

## Palette

The site is deliberately single-mode because the hero is an emitted-light observatory scene. Every surface is painted explicitly.

| Token | Value | Role |
| --- | --- | --- |
| night-950 | `#07100f` | page background |
| night-900 | `#0b1715` | raised field |
| glass | `rgba(20, 43, 39, .76)` | translucent instrument surface |
| glass-edge | `#47796c` | borders and controls (3:1+ against the background) |
| paper | `#f2f7ed` | primary text |
| mist | `#a9bdb4` | secondary text (7.4:1 on night-950) |
| aqua | `#61e7cd` | workload/action accent |
| ink-on-aqua | `#06231d` | accent contrast |
| lime | `#c9f27b` | pass/portable result |
| amber | `#ffc86b` | caution/emulated evidence |
| coral | `#ff8c79` | regression/error |

The palette comes from SQLite’s page cache and the silicon layers beneath it: smoked green glass, phosphor traces, and warm warning lamps. Status is always paired with a word or icon, never color alone.

## Type

- Interface and narrative: `InterVariable`, self-hosted WOFF2, with system sans fallback. Open counters stay readable at small sizes.
- Evidence and commands: `JetBrainsMono`, self-hosted WOFF2, with ui-monospace fallback. Tabular figures make run comparisons line up.
- Scale: 14 / 16 / 20 / 28 / clamp(44–72) px; body text never below 16 px. Reading measure is capped at 70 characters.

## Space and shape

An 8 px base rhythm with 4 px for fine alignment: 8, 12, 16, 24, 32, 48, 72, 112. Corners are clipped rather than bubbly: 2 px on code, 10 px on compact controls, 20 px on major glass fields. Hairlines and small coordinate labels make the page feel calibrated. Cards appear only for independent evidence records.

## Interaction grammar

- Primary actions glow slightly and move 1 px toward the pointer when pressed.
- Segmented demo controls switch one evidence plane at a time; arrow keys move between tabs.
- Copy actions immediately change their label to “Copied” and announce through a live region.
- The phone layout keeps the command and result, but removes decorative axis labels and stacks the profile readout.
- Offline is an expected state: a compact status strip explains that docs remain available and no run data leaves the machine.

## Motion policy

Entrance uses one 420 ms upward fade for the instrument cluster; evidence traces reveal once in 600 ms. UI state changes run 160–220 ms and animate only opacity/transform. Nothing loops. Under `prefers-reduced-motion: reduce`, all transforms and reveals become immediate opacity states; smooth scrolling is disabled.

## Original asset plan and provenance

- `site/public/lab-landscape-28fb23959f50.webp`: AI-generated raster hero showing a glass SQLite page monolith, query traces, and three CPU strata. The filename contains the first 12 characters of its SHA-256 digest so immutable caching is safe. It explains the core relationship—one workload, multiple evidence environments—and contains no text, logos, people, or decorative stock imagery.
- Prompt: “Wide editorial 3D illustration for a developer tool landing page: a luminous translucent SQLite database page monolith suspended over three distinct dark silicon strata, one cyan query trace entering the glass and splitting into three measured paths, tiny amber and lime instrument lights, smoked teal glass, black-green laboratory void, precise etched grid, realistic refraction, elegant technical observatory mood, large quiet negative space on the left, no words, no letters, no logos, no UI screenshot, no people, no watermark.”
- Generated with the factory `factory-image` deployment via `/opt/fleet/lib/gen-image.sh`, 1536×1024 high quality, 2026-08-28. The generated image is project-original and distributed under the repository MIT license. The PNG source is not shipped; the optimized WebP is capped at 300 KB.
- Icons and the small profile diagram are original inline SVG/code-native geometry authored for this product.

## Accessibility and performance constraints

Focus uses a 3 px aqua outline with 3 px offset. Text contrast is at least 4.5:1; borders and focus indicators are at least 3:1. The hero has explicit dimensions and descriptive alt text. Initial JS stays below 200 KB, CSS below 50 KB, two self-hosted font files below 120 KB combined, and the WebP hero below 300 KB.
