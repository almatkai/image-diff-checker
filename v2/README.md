# Image Diff Checker — Rust V2

Development-only external executable, schema 3 / protocol 1, for
`com.tezbar.native.image-diff-checker`. The original **Image Diff Check** command
supports macOS and Windows. Source `extension.json` has `artifacts: {}`; no
release, installation, or catalog entry is supplied.

```sh
cargo build --locked --release --manifest-path v2/Cargo.toml
```

MSRV is Rust 1.85. The independent SDK 0.1.1 uses HTTPS Git at immutable
`e955911f0bf5b558f9d3eb5183df0adc95c22af2`. The product has no JS engine, Node,
Tauri, host dependency, installed helper, or network/image-upload operation.

The Form has explicit single-file **Actual** and **Expected** pickers, Threshold
0, 0.1, …, 1 (initially 0), and **Run**. Successful comparison displays the diff
PNG as image-only Markdown. **Back** returns to the retained draft. The source
does not display counts, percentages, separate actual/expected previews, or offer
copy/open/reveal/export actions; this port preserves that scope. Unequal dimensions
and decoding/read failures produce the original generic comparison error.

The pixelmatch 7.1.0 algorithm is translated directly, including its unusual
coordinate-dependent alpha background, YIQ constants, antialias neighbor traversal,
yellow/red colors, grayscale background, truncation, and equality shortcut.
PNG/GIF/JPEG decoded RGBA and generated diff pixels match the executed source
corpus. JPEG uses a bounded Rust translation of the actual locked jpeg-js 0.4.4
component decoder: baseline/extended 8-bit DCT and progressive scans, restart
intervals, integer IDCT, nearest chroma samples and JS-compatible color truncation.
Grayscale, Adobe CMYK/YCCK, 4:4:4/4:2:2/4:2:0 and all eight EXIF orientations are
covered. Exact parity is measured on 197 owned JPEGs and 7,573 complete comparisons;
finite tests do not establish equality for every arbitrary JPEG.

The JPEG path accepts 8-bit Huffman DCT frames with 1, 3 or Adobe-marked 4
components. Two-component, higher-precision, lossless, arithmetic and multi-frame
JPEGs are rejected. Locked Jimp also rejects the two-component/unmarked-CMYK,
lossless and arithmetic cases checked in QA; it accepts the owned 12-bit-declared
SOF1 probe that this bounded port rejects. Source tolerance for damaged streams is
not reproduced. Details and historical root-cause evidence are in [QA.md](QA.md).

Bounds and ownership:

- Selected files: absolute UTF-8 ordinary paths ≤4096 bytes; filename ≤512 bytes;
  ≤32 MiB/file; ≤8192/side and ≤16,000,000 decoded pixels. PNG, JPEG/JPG, GIF are
  detected from bytes after source extension validation. Both selections are read
  into frozen byte buffers for each deliberate Run. Inputs are never modified.
- Symlinks (including directory components), FIFOs/devices, and Windows reparse
  paths are rejected. Unix traverses components with descriptor-relative
  no-follow opens and rechecks file identity, size and modification metadata.
  Windows has compile coverage only; native path-race acceptance is unverified.
- Full diff PNG stays in bounded session memory. PNGs ≤7,500,000 bytes use their
  full image in the renderer's native data-URI contract. Larger outputs use an
  explicitly announced preview of at most 1,500,000 pixels; the full diff stays
  unchanged in memory. No filename/path enters Markdown. No temporary image,
  export file, history, preferences, journal, or background task is created.
- At most 16 sessions and 128 MiB of retained PNG/Markdown resources per process.
  JPEG coefficient planes, sample planes and RGBA together are preflighted to
  384 MiB per decode, with fallible bulk reservations, ≤256 scans, ≤16,000,000
  entropy block visits and bounded coefficient magnitudes. Both encoded inputs
  (≤64 MiB together) and a previous decoded input (≤64 MB) may coexist with these
  working buffers; the 128 MiB budget applies to retained results. Other decoder
  and working buffers are additionally bounded. Session cleanup and
  Back drop owned resources. Invalid drafts survive failures. Tokens/revisions
  fence old requests; every deliberate submit is consumed, even on failure.
  Checkpoints are outside SDK staged state. No crash durability is claimed.

The pinned host requires the **unapplied** generic
[integration patch](integration/host-prerequisites.patch) for native pickers,
all-field revisions, late-response fencing, Detail callbacks, exact verified
replacement aliases, and active-version icons separate from canonical preferences.
It has not been applied to Tezbar. Actual pinned Rust excerpts and React components
are exercised with synthetic profiles and inert OS callbacks. Real native dialogs,
browser image decoding, full-host build/GUI, and Windows native execution are unrun.

See [QA.md](QA.md), [NOTICES.md](NOTICES.md), [PROVENANCE.md](PROVENANCE.md),
[qa/evidence.json](qa/evidence.json), and the complete [INDEX.sha256](INDEX.sha256).
