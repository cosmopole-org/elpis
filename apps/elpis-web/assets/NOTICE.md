# Bundled fonts

Browsers do not expose system fonts to the WebGPU pipeline, so Blinc needs the
raw TTF bytes to shape glyphs. These fonts are bundled and embedded into the web
demo's wasm artifact.

`DejaVuSans.ttf` / `DejaVuSans-Bold.ttf` are **DejaVu Sans**, a public-domain /
permissively-licensed font derived from Bitstream Vera. They are the demo's
default **sans-serif** family: Blinc's generic `sans-serif` resolution falls
back to the name "DejaVu Sans", so without a registered sans-serif face every
default text element fails to shape and renders as nothing. See
https://dejavu-fonts.github.io/ for the license (Bitstream Vera / Public Domain).

`FiraCode-Regular.ttf` is **Fira Code**, © The Fira Code Project Authors,
licensed under the **SIL Open Font License 1.1** (OFL-1.1). It provides the
**monospace** family. See https://github.com/tonsky/FiraCode and
https://openfontlicense.org/ for the license text. Redistribution of the font
under OFL-1.1 is permitted.
