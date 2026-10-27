# Notices

Original port code and original extension: MIT; see [LICENSE](LICENSE) and untouched
[V1 LICENSE](../LICENSE). Source repository is `almatkai/image-diff-checker`,
baseline `a1703be718311980934c98a23bcb530b8330f755`. Original Raycast code/icon:
`raycast/extensions@6734d85992290066fd31a7f5bee3cca3d3545a1a`,
`extensions/image-diff-checker`.

Original 153,226-byte icon SHA256:
`05c952bb2706629c1f0ee55433e7033be83463cd5a5ac30c7e8072db0a5ddb2c`.
Downloaded upstream bytes, own `assets/image-diff-checker-icon.png` and V2 resource
are byte equal. No icon was generated, resized or reencoded.

pixelmatch 7.1.0 is copyright 2025 Mapbox, ISC:
[authentic license](resources/PIXELMATCH-LICENSE). The direct algorithm translation
is in `src/pixelmatch.rs`; original locked source is retained in
`qa/fixtures/pixelmatch/index.js`. Its matching options are exactly those used by
the own source helper. This is not the older Jimp-internal pixelmatch 5.3.0.

The SDK 0.1.1 is Apache-2.0, pinned to
`e955911f0bf5b558f9d3eb5183df0adc95c22af2`;
[authentic license](resources/SDK-LICENSE). All Cargo dependency notices, including
dev/platform dependencies, are copied unmodified into `resources/licenses` and
listed with exact versions/hashes in [qa/dependency-notices.json](qa/dependency-notices.json).

Generic descriptor-safe input reads, lifecycle/revision patterns, and host/renderer
contract QA adapt public MIT-licensed `almatkai/image-base64` V2 at immutable
`51e302cacdaeea11038704ef2825893cb714e1f3`. Its
[license](resources/IMAGE-BASE64-LICENSE) is retained. No clone or dependency on
that feature branch is created. The original own V1 source is not changed.

Host excerpts/fixtures come only from exact Git-show bytes at
`ea100cee3f69711de12485ba061950c5fc65258f`, with the original
[host license](qa/fixtures/host/LICENSE). All proposed changes remain in V2.

The native JPEG decoder in `src/jpeg.rs` derives from the actual locked
jpeg-js 0.4.4 `lib/decoder.js`, copyright 2011 notmasteryet, **Apache-2.0**.
Its authentic [source header](resources/JPEG-DECODER-HEADER), full
[Apache-2.0 license](resources/JPEG-DECODER-APACHE-2.0), and the source's
Loeffler/Ligtenberg/Moschytz IDCT attribution are retained. The source package
also carries Eugene Ware's **BSD-3-Clause** [license](resources/JPEG-JS-LICENSE).
The Cargo license aggregate includes MIT, ISC, Apache-2.0 and BSD-3-Clause.
This file is a modified bounded Rust translation, not the original JavaScript.
The exact unmodified decoder is a QA provenance fixture, never a runtime/helper.
`jpeg-decoder` is no longer a product or development dependency.

QA uses real exact-lock Jimp/jpeg-js/pngjs/omggif/pixelmatch.
JS dependencies and renderer tools are not shipped in the product ZIP. Synthetic
images are generated locally from numerical patterns, without user imagery or
third-party artwork. Their hashes and generator are Git-visible. Deliberately
invalid tiny ZIPs are owned parser fixtures.
