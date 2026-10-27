# Validation and known differences

This is an uncommitted development port. No release, app installation, GUI,
clipboard, real picker, user image, Downloads directory or external upload was used.
All mutable QA paths are under `.qa/` in this clone. The main app was read only via
exact `git show` at `ea100cee3f69711de12485ba061950c5fc65258f`.

The final command matrix is [qa/validation-matrix.json](qa/validation-matrix.json);
durable results are [qa/evidence.json](qa/evidence.json). Ignored full logs and
transcripts remain in `.qa/` for parent review. To reproduce with the already
installed Rust stable/1.85 toolchains, GNU target and Pillow:

```sh
python3 -B v2/qa/run.py python3 -B v2/qa/verify.py
```

The runner copies V1 into `.qa/v1`, uses its untouched exact npm lock, and installs
only disposable QA npm dependencies with ignored scripts. The renderer's separate
lock is Git-visible. No target, system tool, app or extension is installed. Fresh
SDK acquisition uses ordinary HTTPS Cargo Git without local/path overrides.

Executed checks:

- Stable and Rust 1.85: all-target tests (12 tests), fmt check, strict all-target
  Clippy. The locked QA-only zopfli is pinned to MSRV-compatible 0.8.2.
- Windows GNU: all-target compile check passes. Release build attempt fails at
  missing `x86_64-w64-mingw32-dlltool`. No tools/targets were installed. MSVC,
  native Windows execution and real OS path-race behavior remain unrun.
- 406 owned synthetic images reconstruct exactly: 197 JPEGs and 209 PNG/GIF
  (including 180 source-decoded JPEG boundary PNGs). All original 38 fixtures and
  605 comparisons remain. New gradients and random chroma/block neighborhoods use
  1×1, 7×9, 17×13 and 33×19 dimensions, qualities 10/73/100, baseline/progressive,
  4:4:4/4:2:2/4:2:0, restart intervals 1/3, grayscale, CMYK, synthetic YCCK sample
  planes and eight oriented progressive subsampled variants.
- All 7,573 source comparisons match exact native masks/counts and encoded-then-
  decoded PNG pixels; 7,210 involve JPEG. No format exemptions remain. 720 pairs
  deliberately alter one source-decoded JPEG channel and test threshold 0 and
  nearby 0.00001/0.001/0.1. All 702 raw RGBA cases remain exact, including actual
  metric-boundary sides, equality, alpha coordinates, AA and random neighborhoods.
  AA-yellow pixels are required to occur; PNG compression bytes are not compared.
- A standalone minimal T.81 grayscale stream decodes all 16 MP exactly. Every
  truncation and every single-byte inversion of four baseline/progressive/CMYK/
  restart fixtures returns an error or a dimension-consistent raster without a
  panic. Explicit invalid Huffman classes/trees/categories, AC runs, spectral
  ranges, successive approximation, selectors and sampling are rejected.
- The actual own TSX component, `useImagesForm`, validators and helper execute
  with real locked Jimp/pixelmatch and exact extracted locked `useForm` functions.
  Four real submit callbacks exercise required validation, success and preserved
  previous Markdown on decode/size errors. Source file-picker onChange callbacks
  and the actual SubmitForm action are invoked. A malicious filename never enters
  generated Markdown. All UI/OS effects are inert. Producer gates precede callbacks;
  six explicit denial probes pass and unexpected producers are zero.
- Real SDK JSONL pipes through the independently built, ZIP-extracted ARM64
  executable: 277 requests; normal run, exact returned diff RGBA, invalid/decode/
  unequal-size errors, all-field draft revisions, A/B/A with an invalid intermediate,
  Back, repeated identical submits with fresh IDs, stale action/revision, refresh,
  16-session capacity, 140 canceled opens, a large frame, in-flight compare cancel,
  disposal and EOF. Eighteen fresh JPEG submits also verify exact returned PNG
  pixels for JPEG/PNG, JPEG/JPEG, restart, YCCK, odd/oriented and one-channel cases;
  nine JPEG A/B/A draft cycles include invalid intermediates, stale revisions and
  Back. Input hashes are unchanged. Max observed frame: 4,758,187 bytes, below 16 MiB. No reverse
  host effects; data/temp directories stay empty. Input hashes stay unchanged.
- After real adapter entry, another 140 captured SDK scopes are canceled using
  TestContext drop; retained checkpoints are pruned and the next open succeeds.
  No background tasks, durable state or filesystem image resources are created.
- Actual native picker Rust producer body executes with mocked dialog/OS callbacks.
  Actual pinned installer/registry/native route excerpts run before/after the
  unapplied patch, with closed ZIP inventory, wrong identity/protocol/capability,
  command/platform, ambiguity, tampering, traversal/link/name negatives and separate
  canonical preferences versus active-version resources. No full-host build.
- Actual React Form/Detail/Markdown/Surface/Runtime components execute in JSDOM:
  41 assertions cover before/after all-fields, native absolute selection, both
  product pickers, pending/unmount protection, A/B/A, stale responses, Detail event
  wrapping, actual process PNG data URI, PNG/JPEG/GIF src contracts and inert
  malicious text. JSDOM verifies DOM/resource URLs; browser image decoding and real
  GUI/picker behavior remain unrun. Network/clipboard effects are zero.
- Full draft-2020-12 package schema passes, including URI format validation and
  33 negatives. Source artifacts are empty. No invented release URL is present.
- Real 32 MiB file boundary, symlink/FIFO/directory rejection, 8192-side/16 MP
  bounds, a full 4000×4000 decode, oversized header rejection, and large preview
  reduction with unchanged full diff bytes are exercised.
- V1 exact-lock smoke test passes (1 test). Original typecheck fails unchanged:
  two `.spec.ts` files import absent `vitest`, also causing two implicit-any
  diagnostics. The root package/lock/source/assets/license/metadata remain byte
  identical to HEAD. Only the root README discovery link is changed.
- A fresh independent Cargo home built copied final product sources over HTTPS.
  The closed dev ZIP is extracted, every member hash checked, and its actual
  production binary runs `--protocol-info`, `--self-test` and the process scenarios.

JPEG reconstruction correction:

The production decoder now translates the locked jpeg-js 0.4.4 integer IDCT,
coefficient scans, nearest component sampling and color arithmetic in Rust.
All 197 supported JPEG fixtures match dimensions and RGBA exactly; all supported
corpus masks/counts/decoded output PNGs match. `tests/oracle.rs` asserts equality
before producing a report; a changed sample/count fails, rather than refreshing
expected differences. Current zero-difference evidence is
[qa/jpeg-parity.json](qa/jpeg-parity.json).

[qa/jpeg-implementation-evidence.json](qa/jpeg-implementation-evidence.json) and
[qa/jpeg-observations.json](qa/jpeg-observations.json) are **unchanged historical
round-one evidence**: the superseded jpeg-decoder implementation differed on all
17 old JPEGs, 319 output comparisons and 22 counts. Their interpolation/IDCT/color
root cause led to replacing that dependency; they no longer describe the final
implementation. No difficult case was removed.

Accepted scope is 8-bit Huffman DCT SOF0/SOF1/SOF2, one/three or Adobe-marked four
components, sample factors 1–4, restart and progressive scans. Declared tested
sampling modes are 4:4:4/4:2:2/4:2:0. No universal arbitrary-JPEG equality is claimed.
The owned 12-bit-declared SOF1 probe is accepted by locked Jimp at 17×13 and rejected
by native; this is an explicit higher-precision exclusion, not an ordinary JPEG
parity exemption. Locked-source rejection observations for two components,
unmarked CMYK, lossless and arithmetic markers are recorded by source QA. The
native decoder also rejects multi-frame and malformed/out-of-sequence/truncated
streams rather than reproducing source tolerant repair.

JPEG bulk allocations are fallible and their combined coefficient/sample/RGBA
sizes are checked against 384 MiB before allocation. Dimensions cap the products
used for indexing. Input ≤32 MiB/file, ≤8192/side and ≤16 MP limits remain; the
128 MiB retained result budget is separate from transient decoder memory. At most
256 scans and 16,000,000 entropy block visits are processed, Huffman codes use at
most 16 bits, AC runs are range checked, and coefficient magnitude is capped at
1,048,576. IDCT locals use exact bounded integer products plus explicit ToInt32
shifts/stores. Full 16 MP JPEG and PNG decoding pass; work limits may reject an
unusual image with many individually valid scans. No JS/OS/external decoder runs
in the product.

Intentional bounded adaptations: 32 MiB inputs, 16 MP decoded images, 16 sessions,
128 MiB retained result resources; no symlink/reparse/FIFO/device inputs; explicit
preview-only reduction above 7,500,000 PNG bytes; explicit Back to the retained
Form. Source Jimp can also sniff BMP/TIFF renamed with an allowed suffix; this port
accepts only declared PNG/JPEG/GIF bytes. Unsupported/oversized/unsafe inputs fail
without source writes or partial result publication. Source has no export/copy or
separate input-preview actions; none are added. There is no crash-persistent journal,
exactly-once OS effect claim, full-host or native platform acceptance claim.
