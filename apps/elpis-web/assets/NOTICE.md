# Bundled font

`FiraCode-Regular.ttf` is **Fira Code**, © The Fira Code Project Authors,
licensed under the **SIL Open Font License 1.1** (OFL-1.1). It is bundled and
embedded into the web demo's wasm artifact because browsers do not expose system
fonts to the WebGPU pipeline, so Blinc needs the raw TTF bytes to shape glyphs.

See https://github.com/tonsky/FiraCode and
https://openfontlicense.org/ for the license text. Redistribution of the font
under OFL-1.1 is permitted.
