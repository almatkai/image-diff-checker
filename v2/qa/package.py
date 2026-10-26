#!/usr/bin/env python3
"""Closed development ZIP only; artifacts deliberately remain empty."""
from pathlib import Path
import hashlib,json,zipfile,sys,shutil,tempfile
r=Path(__file__).resolve().parents[1];binary=Path(sys.argv[1]) if len(sys.argv)>1 else r/'.qa/target/release/image-diff-checker'
files={'bin/image-diff-checker':binary,'LICENSE':r/'LICENSE','NOTICE':r/'NOTICES.md'}
for p in sorted((r/'resources').rglob('*')):
 if p.is_file():files[str(p.relative_to(r))]=p
m=json.loads((r/'extension.json').read_text());assert m['artifacts']=={}
m['files']={name:hashlib.sha256(p.read_bytes()).hexdigest()for name,p in files.items()}
archive=r/'image-diff-checker-aarch64-apple-darwin-dev.zip'
with zipfile.ZipFile(archive,'w',compression=zipfile.ZIP_DEFLATED)as z:
 for name,p in files.items():
  info=zipfile.ZipInfo(name);info.create_system=3;info.external_attr=((0o100755 if name.startswith('bin/') else 0o100644)<<16);info.compress_type=zipfile.ZIP_DEFLATED;z.writestr(info,p.read_bytes())
 z.writestr('extension.json',json.dumps(m,indent=2)+'\n')
parent=r/'.qa/extracted';parent.mkdir(exist_ok=True);extracted=Path(tempfile.mkdtemp(prefix='arm64-',dir=parent))
with zipfile.ZipFile(archive)as z:
 assert set(z.namelist())==set(files)|{'extension.json'}
 z.extractall(extracted)
for name,sha in m['files'].items():assert hashlib.sha256((extracted/name).read_bytes()).hexdigest()==sha
(extracted/'bin/image-diff-checker').chmod(0o755)
report={'archive':str(archive.relative_to(r)),'sha256':hashlib.sha256(archive.read_bytes()).hexdigest(),'bytes':archive.stat().st_size,'members':m['files'],'extracted':str(extracted.relative_to(r)),'binary':str(extracted/'bin/image-diff-checker'),'developmentOnly':True}
(r/'.qa/package.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items()if k!='members'}))
