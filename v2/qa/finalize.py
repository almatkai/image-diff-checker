#!/usr/bin/env python3
"""Audit scope and record final evidence/inventory without touching Git's index."""
from pathlib import Path
import hashlib,json,subprocess
r=Path(__file__).resolve().parents[1];clone=r.parent;q=r/'.qa'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def load(p):return json.loads(p.read_text())
def git(*args):return subprocess.check_output(['git',*args],cwd=clone)
assert git('rev-parse','--show-toplevel').decode().strip()==str(clone)
assert git('branch','--show-current').decode().strip()=='feat/native-v2-rust'
head=git('rev-parse','HEAD').decode().strip();assert head=='a1703be718311980934c98a23bcb530b8330f755'
assert git('diff','--cached','--name-only')==b''
assert git('diff','--name-only').decode().splitlines()==['README.md']
tracked=git('ls-files','-z').decode().split('\0');v1={}
for name in tracked:
 if not name:continue
 old=git('show','HEAD:'+name)
 if name=='README.md':assert (clone/name).read_bytes()==old.rstrip()+b'\n\n[Rust V2 development implementation](v2/README.md).\n'
 else:assert(clone/name).read_bytes()==old,name
 v1[name]=hashlib.sha256(old).hexdigest()
fresh=load(q/'fresh-build.json');work=r/fresh['work'];source=work/'source';product={}
for p in [r/'Cargo.toml',r/'Cargo.lock',r/'extension.json',*sorted((r/'src').rglob('*.rs'))]:
 assert p.read_bytes()==(source/p.relative_to(r)).read_bytes(),p
 product[str(p.relative_to(r))]=sha(p)
package=load(q/'package.json');process=load(q/'process-report.json')
assert process['sha256']==package['members']['bin/image-diff-checker']==fresh['sha256']
assert process['packageDir']==str(Path(package['binary']).parent.parent)
assert sha(r/package['archive'])==package['sha256']
assert (r/'resources/image-diff-checker-icon.png').read_bytes()==(clone/'assets/image-diff-checker-icon.png').read_bytes()
matrix=load(r/'qa/validation-matrix.json')
for row in matrix:
 p=q/(row['name']+'.log')
 if p.exists():row['logSha256']=sha(p)
(r/'qa/validation-matrix.json').write_text(json.dumps(matrix,indent=2)+'\n')
pins=load(r/'qa/pins.json');pins['fixtureSha256']={str(p.relative_to(r)):sha(p)for p in sorted((r/'qa/fixtures').rglob('*'))if p.is_file()};(r/'qa/pins.json').write_text(json.dumps(pins,indent=2)+'\n')
codec=load(q/'codec-report.json');source_oracle=load(q/'source-oracle.json');renderer=load(q/'renderer.log');schema=load(q/'schema-report.json')
assert source_oracle['unexpectedProducers']==0 and source_oracle['executedSubmitCallbacks']==4
assert source_oracle['images']==406 and source_oracle['pairs']==7573 and source_oracle['raw']==702 and source_oracle['jpegBoundaries']==720
assert len(codec['decode'])==406 and all(x['differentChannels']==0 and x['differentPixels']==0 and x['maxChannelDelta']==0 for x in codec['decode'])
assert codec['pixelmatchRawExact']==702 and codec['pixelmatchSourcePairsExact']==7573
assert codec['nativeRepresentationDifferences']==[]
assert process['jpegExactSubmits']==18 and process['jpegABADrafts']==9 and process['jpegInputsUnchanged']
(r/'qa/jpeg-parity.json').write_text(json.dumps(codec,indent=2)+'\n')
pins['jpegDecoder']={'package':'jpeg-js','version':'0.4.4','sourceSha256':sha(r/'qa/fixtures/jpeg-js/decoder.js'),'headerSha256':sha(r/'resources/JPEG-DECODER-HEADER'),'licenses':['Apache-2.0','BSD-3-Clause'],'native':'src/jpeg.rs','historicalEvidenceSha256':sha(r/'qa/jpeg-implementation-evidence.json'),'historicalObservationsSha256':sha(r/'qa/jpeg-observations.json')}
(r/'qa/pins.json').write_text(json.dumps(pins,indent=2)+'\n')
assert renderer['assertions']==41 and renderer['nativeCallsInert']['clipboard']==0
assert all(row['exitCode']==({'v1-typecheck':2,'windows-link':101}.get(row['name'],0))for row in matrix)
evidence={'head':head,'branch':'feat/native-v2-rust','indexEmpty':True,'scope':['v2/','README discovery link'],'unchangedV1FileSha256':v1,'finalProductSources':product,'icon':pins['icon'],'sourceOracle':source_oracle,'jpegScopeNative':load(q/'jpeg-scope-native.json'),'pixelmatch':{'rawExact':702,'sourcePairsExact':7573,'jpegPairsExact':7210,'pngGifDecodedExact':209,'jpegDecodedExact':197,'finiteJpegParityFile':'qa/jpeg-parity.json','historicalJpegObservationFile':'qa/jpeg-observations.json','changedNativeResults':0,'changedNativeCounts':0,'universalJpegParity':False},'renderer':renderer,'schema':schema,'process':process,'freshBuild':fresh,'package':package,'cargoNotices':load(r/'qa/dependency-notices.json'),'knownFailures':{'v1Typecheck':'Exact-lock baseline lacks vitest for two original .spec.ts files; two implicit-any errors also occur.','windowsRelease':'Installed GNU target checks; release attempt lacks x86_64-w64-mingw32-dlltool.'},'notRun':['native GUI and native picker','browser image decoding','full host build/GUI','Windows native/MSVC','app installation/release'],'xerj':pins['xerj'],'qaBreachObserved':False,'published':False}
(r/'qa/evidence.json').write_text(json.dumps(evidence,indent=2)+'\n')
visible=[x for x in git('ls-files','--cached','--others','--exclude-standard','-z').decode().split('\0')if x]
assert all(x in tracked or x.startswith('v2/')for x in visible)
assert not any(any(part in {'.qa','target','node_modules'}for part in Path(x).parts)or x.endswith('.zip')for x in visible)
inventory=sorted(set(visible)|{'v2/INDEX.sha256'})
(r/'INDEX.sha256').write_text(''.join(sha(clone/p)+'  '+p+'\n'for p in inventory if p!='v2/INDEX.sha256'))
print(json.dumps({'gitVisibleFiles':len(inventory),'indexedHashes':len(inventory)-1,'v1UnchangedExceptReadme':True,'head':head,'binary':fresh['sha256'],'archive':package['sha256'],'notices':len(evidence['cargoNotices']),'indexEmpty':True}))
