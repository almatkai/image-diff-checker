#!/usr/bin/env python3
"""Expand only owned tiny synthetic inputs. Stage generation under clone .qa."""
from pathlib import Path
from PIL import Image
import random,hashlib,json,shutil
r=Path(__file__).resolve().parents[1];stage=r/'.qa/jpeg-fixtures';stage.mkdir(exist_ok=True)
added=[]
def save(im,name,**kw):
 p=stage/name;im.save(p,**kw);added.append(name);return p
for pattern in ['gradient','blocks']:
 for w,h in [(1,1),(7,9),(17,13),(33,19)]:
  rand=random.Random(718+w*100+h)
  a=Image.new('RGB',(w,h))
  a.putdata([((x*7+y*11)%256,(x*3+y*19)%256,(x*29+y*5)%256) if pattern=='gradient' else tuple(rand.randrange(256) if x%8==0 or y%8==0 else ((x//4+y//3+c)%2)*255 for c in range(3)) for y in range(h)for x in range(w)])
  prefix=f'jpeg-{pattern}-{w}x{h}'
  save(a,prefix+'.png')
  for sub in [0,1,2]:
   for quality in [10,73,100]:
    for progressive in [False,True]:
     save(a,f'{prefix}-q{quality}-s{sub}-p{int(progressive)}.jpg',quality=quality,subsampling=sub,progressive=progressive)
  if (w,h)==(33,19):
   for progressive in [False,True]:
    for interval in [1,3]:
     p=save(a,f'{prefix}-restart{interval}-p{int(progressive)}.jpg',quality=83,subsampling=2,progressive=progressive,restart_marker_blocks=interval)
     assert b'\xff\xdd' in p.read_bytes() and b'\xff\xd0' in p.read_bytes()
   for mode in ['L','CMYK']:
    for progressive in [False,True]:
     p=save(a.convert(mode),f'{prefix}-{mode}-p{int(progressive)}.jpg',quality=83,progressive=progressive)
     if mode=='CMYK':
      # Four synthetic DCT component planes also exercise the Adobe YCCK transform.
      b=bytearray(p.read_bytes());i=b.index(b'Adobe\0');assert b[i+11]==0;b[i+11]=2
      name=f'{prefix}-YCCK-p{int(progressive)}.jpg';(stage/name).write_bytes(b);added.append(name)
   for orientation in range(1,9):
    exif=Image.Exif();exif[274]=orientation
    save(a,f'{prefix}-exif{orientation}.jpg',quality=73,subsampling=2,progressive=True,exif=exif)
out=r/'qa/fixtures/images'
for name in added:shutil.copyfile(stage/name,out/name)
manifest=[{'path':p.name,'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in sorted(out.iterdir())if p.is_file()]
(r/'qa/fixtures/images.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps({'added':len(added),'total':len(manifest),'jpeg':sum(p['path'].endswith('.jpg') for p in manifest)}))
