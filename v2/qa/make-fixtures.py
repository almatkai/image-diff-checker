#!/usr/bin/env python3
"""Owned synthetic raster corpus; no user inputs or default output paths."""
from pathlib import Path
import struct,zlib,binascii,hashlib,json
from PIL import Image
r=Path(__file__).resolve().parents[1]; out=r/'qa/fixtures/images';out.mkdir(parents=True,exist_ok=True)
def chunk(tag,data):return struct.pack('>I',len(data))+tag+data+struct.pack('>I',binascii.crc32(tag+data)&0xffffffff)
def png(name,w,h,depth,color,rows,extra=b'',interlace=0):
 b=b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,depth,color,0,0,interlace))+extra+chunk(b'IDAT',zlib.compress(rows))+chunk(b'IEND',b'')
 (out/name).write_bytes(b)
w,h=17,13
a=Image.new('RGBA',(w,h));a.putdata([((x*17+y*5)%256,(y*23+x*9)%256,((x^y)*31)%256,(x*37+y*13)%256)for y in range(h)for x in range(w)])
a.save(out/'alpha.png');a.convert('RGB').save(out/'rgb.png')
b=a.copy();b.putpixel((8,6),(255,0,0,255));b.putpixel((1,1),(0,255,0,0));b.save(out/'changed.png')
Image.new('RGBA',(7,9),(40,80,120,0)).save(out/'transparent-hidden.png')
Image.new('RGBA',(7,9),(230,20,50,0)).save(out/'transparent-other.png')
g=Image.new('L',(w,h));g.putdata([(x*15+y*19)%256 for y in range(h)for x in range(w)]);g.save(out/'gray8.png');g.convert('1').save(out/'gray1.png')
ga=Image.new('LA',(w,h));ga.putdata([((x*15)%256,(y*29)%256)for y in range(h)for x in range(w)]);ga.save(out/'gray-alpha.png')
palette=Image.new('P',(w,h));palette.putpalette([i for n in range(256)for i in ((n*13)%256,(n*59)%256,(n*31)%256)]);palette.putdata([(x+y)%4 for y in range(h)for x in range(w)]);palette.save(out/'palette2.png',bits=2,transparency=bytes([0,73,200,255]))
a.convert('RGB').save(out/'trns-rgb.png',transparency=a.convert('RGB').getpixel((0,0)));g.save(out/'trns-gray.png',transparency=0)
for color,channels,name in [(0,1,'gray16.png'),(2,3,'rgb16.png'),(4,2,'gray-alpha16.png'),(6,4,'rgba16.png')]:
 rows=b''.join(b'\0'+b''.join(struct.pack('>H',((x*4097+y*31+c*257+128)%65536))for x in range(w)for c in range(channels))for y in range(h))
 png(name,w,h,16,color,rows)
# Adam7 with unfiltered pass rows.
raw=a.tobytes();rows=b''
for x0,y0,dx,dy in [(0,0,8,8),(4,0,8,8),(0,4,4,8),(2,0,4,4),(0,2,2,4),(1,0,2,2),(0,1,1,2)]:
 for y in range(y0,h,dy):
  row=b''.join(raw[(y*w+x)*4:(y*w+x+1)*4]for x in range(x0,w,dx))
  if row:rows+=b'\0'+row
png('adam7.png',w,h,8,6,rows,interlace=1)
for sub in [0,1,2]:
 for quality in [35,95]:a.convert('RGB').save(out/f'color-q{quality}-s{sub}.jpg',quality=quality,subsampling=sub)
g.save(out/'gray.jpg',quality=91)
a.convert('RGB').save(out/'progressive.jpg',quality=91,progressive=True,subsampling=2)
a.convert('CMYK').save(out/'cmyk.jpg',quality=95)
for n in range(1,9):
 exif=Image.Exif();exif[274]=n;a.convert('RGB').save(out/f'orientation-{n}.jpg',quality=95,subsampling=0,exif=exif)
palette.save(out/'transparent.gif',transparency=0)
p2=palette.copy();p2.putdata([(x*3+y+1)%4 for y in range(h)for x in range(w)])
palette.save(out/'animated.gif',save_all=True,append_images=[p2],duration=[100,100],loop=0,transparency=0,disposal=2)
# First-frame subrectangle on a larger transparent logical canvas.
partial=bytearray((out/'transparent.gif').read_bytes());partial[6:10]=struct.pack('<HH',23,19)
k=13+(3*(2<<(partial[10]&7)) if partial[10]&128 else 0)
while partial[k]==0x21:
 k+=2
 while partial[k]:k+=1+partial[k]
 k+=1
assert partial[k]==0x2c
partial[k+1:k+5]=struct.pack('<HH',3,2);(out/'offset-firstframe.gif').write_bytes(partial)
# Antialias slopes with repeated dark/light neighbor regions.
for n in [0,1]:
 edge=Image.new('RGBA',(9,9));edge.putdata([(v,v,v,255)for y in range(9)for x in range(9)for v in [0 if x<3 else (255 if x>4 else [40+n*25,180+n*30][x-3])]])
 edge.save(out/f'edge-{n}.png')
manifest=[{'path':p.name,'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in sorted(out.iterdir())if p.is_file()]
(r/'qa/fixtures/images.json').write_text(json.dumps(manifest,indent=2)+'\n');print(len(manifest),'synthetic images')
