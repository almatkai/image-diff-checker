#!/usr/bin/env python3
"""Save final owned boundary fixture snapshots after confined source producers exit."""
from pathlib import Path
import json,shutil,hashlib
r=Path(__file__).resolve().parents[1]
for image in json.loads((r/'qa/oracle.json').read_text())['images']:
 if image.get('generatedBoundary'):
  name=image['path'];assert name.startswith('boundary-jpeg-') and Path(name).name==name
  shutil.copyfile(r/'.qa/oracle'/name,r/'qa/fixtures/images'/name)

out=r/'qa/fixtures/images'
manifest=[{'path':p.name,'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}for p in sorted(out.iterdir())if p.is_file()]
(r/'qa/fixtures/images.json').write_text(json.dumps(manifest,indent=2)+'\n')

scope=r/'qa/fixtures/jpeg-scope';scope.mkdir(exist_ok=True)
for name in ['scope-12bit.jpg','scope-twoComponents.jpg','scope-unmarkedCMYK.jpg','scope-losslessMarker.jpg','scope-arithmeticMarker.jpg']:
 shutil.copyfile(r/'.qa/oracle'/name,scope/name)
