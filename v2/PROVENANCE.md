# Immutable sources

- Own fresh HTTPS clone/fetch: `https://github.com/almatkai/image-diff-checker`,
  `origin/main`/unchanged HEAD `a1703be718311980934c98a23bcb530b8330f755`.
  Independent connected GitHub ref read confirmed that SHA before checkout.
- Upstream icon: exact URL and SHA in [qa/pins.json](qa/pins.json). The own fresh
  manifest confirms the upstream path, commit, icon, command, platform declarations
  and own alias `com.tezbar.image-diff-checker`.
- SDK API/protocol/host contracts were read at immutable HTTPS revision
  `e955911f0bf5b558f9d3eb5183df0adc95c22af2`; exact fixtures are under `qa/fixtures/sdk`.
- Main app was accessed only with `git show <pin>:<explicit-path>`. Its dirty
  working files, build/profile state, and Git metadata were not modified. No host
  `ls-tree`, checkout, stash, dependency installation, build or GUI was used.
- Public Image Base64 generic patterns were fetched by immutable raw URLs into
  ignored reference storage. Adapted source/tests/patch generators and notices are
  retained here. No previous temporary clone was recreated or read.
- Fresh own source was read fully: component, actual hook, validators, diff helper,
  tests and manifest/lock. Codec behavior was verified against the installed exact
  locked modules, including PNG normalization, GIF first-frame blitting, Jimp EXIF
  transforms and pixelmatch 7.1.0. The original component and hook submit callbacks
  and extracted actual locked useForm implementation execute in source QA.
- XERJ canonical namespace: `ax-8e1af70e7dd3`. Index attempt received HTTP 429/RSS
  circuit-breaker rejection with 0/15 indexed files. No schema/prefix workaround,
  global restart, or false completeness claim. Full reads, `rg`, exact Git bytes
  and hashes supply coverage. Clone-owned log: `.qa/xerj/index.log`.

Every mutable QA home/cache/temp/build/image/archive path is clone-owned. Source
oracle producers are gated before callbacks: read/write paths, descriptor calls,
callbacks/promises/streams, os home/temp and identity, subprocesses, network,
browser and clipboard. The allowed codec reads/writes use only owned fixtures.
Representative forbidden producers are exercised explicitly; unexpected attempts
must fail the run. The synthetic DOM has no external resource loader.

No staging, commit, push, PR, release, app installation, clone deletion, ledger
edit, sibling/SDK edit, or subdelegation is part of this implementation.

JPEG review-round correction: `src/jpeg.rs` is a direct bounded Rust translation
of locked jpeg-js **0.4.4** `lib/decoder.js`, not an approximate RGB correction.
The original decoder SHA256 is
`a3f175fd6f62d142aad94d3bd90f3a30be4e076baf9b6a6fa31c8e84d9d4aa9f`;
the exact file is `qa/fixtures/jpeg-js/decoder.js`. Source acquisition is the
unchanged own V1 lock's npm tarball (integrity remains in that lock). The Apache
header applies to this decoder independently of the package's BSD-3-Clause label;
both notices are shipped. Canonical Huffman lookup replaces the JS tree, while
scan coefficient semantics, row/column IDCT stages, ToInt32 shifts/typed-array
writes, coordinate truncation, and two-stage color truncation are preserved.
Checked parsing, strict scan history, bounded memory/work and malformed rejection
are deliberate safety adaptations. Prior jpeg-decoder implementation evidence and
nonzero observations remain unchanged as historical artifacts; current exact
assertions and `qa/jpeg-parity.json` supersede their final-result interpretation.
