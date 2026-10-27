#!/usr/bin/env python3
"""Check immutable root-cause history and actual locked decoder provenance."""
from pathlib import Path
import hashlib,json
r=Path(__file__).resolve().parents[1]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
history=r/'qa/jpeg-implementation-evidence.json'
assert sha(history)=="13d9ec13ee403deb84a7d1e6261de59628a7d5b4b1771576b3083c9924388977", 'Historical evidence must not be refreshed'
actual=r/'.qa/v1/node_modules/jpeg-js/lib/decoder.js'
assert sha(actual)=='a3f175fd6f62d142aad94d3bd90f3a30be4e076baf9b6a6fa31c8e84d9d4aa9f'
assert actual.read_bytes()==(r/'qa/fixtures/jpeg-js/decoder.js').read_bytes()
(r/'.qa/jpeg-source-check.json').write_text(json.dumps({'lockedDecoderSha256':sha(actual),'historicalEvidenceSha256':sha(history),'historicalEvidenceUnchanged':True,'nativeReplacement':'src/jpeg.rs'},indent=2)+'\n')
print('Locked jpeg-js source and unchanged historical root-cause evidence verified')
