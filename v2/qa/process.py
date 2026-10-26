#!/usr/bin/env python3
"""Real SDK production JSONL pipes, owned paths only, reverse effects forbidden."""
from pathlib import Path
import subprocess,os,json,selectors,time,tempfile,sys,hashlib,base64,uuid
r=Path(__file__).resolve().parents[1];run=r/'.qa/pipes';run.mkdir(exist_ok=True)
binary=Path(sys.argv[1]) if len(sys.argv)>1 else r/'.qa/target/release/image-diff-checker'
views=[];counts={'requests':0,'reverse':0,'canceledOpens':0,'maxFrame':0};transcript=[]
class Process:
 def __init__(self):
  self.package=binary.parent.parent if binary.parent.name=='bin' else r
  self.dir=Path(tempfile.mkdtemp(prefix='process-',dir=run));(self.dir/'data').mkdir();(self.dir/'temp').mkdir()
  self.p=subprocess.Popen([str(binary)],cwd=self.dir,env=os.environ.copy(),stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=(self.dir/'stderr.log').open('wb'),bufsize=0)
  self.selector=selectors.DefaultSelector();self.selector.register(self.p.stdout,selectors.EVENT_READ);self.buffer=b'';self.seq=0
  hello=self.read();assert hello['type']=='hello' and hello['id']=='com.tezbar.native.image-diff-checker' and hello['protocol']==1
  assert self.call('initialize',{'protocol':1,'extensionId':hello['id'],'platform':'macOS','packageDir':str(self.package),'dataDir':str(self.dir/'data'),'tempDir':str(self.dir/'temp'),'grantedCapabilities':[],'migrationPaths':{}})['ready']
 def send(self,value):self.p.stdin.write(json.dumps(value,separators=(',',':'),ensure_ascii=False).encode()+b'\n');self.p.stdin.flush()
 def read(self):
  end=time.monotonic()+15
  while b'\n' not in self.buffer:
   assert self.selector.select(max(0,end-time.monotonic())),'SDK frame timed out'
   chunk=os.read(self.p.stdout.fileno(),65536);assert chunk,'Unexpected EOF';self.buffer+=chunk
   assert len(self.buffer)<=16*1024*1024
  line,self.buffer=self.buffer.split(b'\n',1);counts['maxFrame']=max(counts['maxFrame'],len(line));v=json.loads(line)
  if v.get('type')=='request':counts['reverse']+=1;raise AssertionError('Unexpected host effect '+str(v))
  return v
 def request(self,method,params):
  self.seq+=1;id='h:'+str(uuid.uuid4());counts['requests']+=1;self.send({'type':'request','id':id,'method':method,'params':params});return id
 def reply(self,id):
  while True:
   v=self.read()
   if v.get('type')=='event':continue
   assert v['id']==id,(id,v);transcript.append(v);return v
 def call(self,method,params,error=False):
  v=self.reply(self.request(method,params))
  if error:assert 'error'in v,v;return v['error']
  assert 'result'in v,v;return v['result']
 def open(self,sid):return self.call('session.open',{'sessionId':sid,'commandName':'image-diff-check','preferences':{},'argumentValues':{}})
 def action(self,sid,view,values=None,kind=None,revision=1):
  action=kind or view['actions'][0]['id'];payload={'actionId':action}
  if values is not None:payload['formValues']={**values,'__formToken':view['root']['props']['formToken'],'__formRevision':revision}
  return self.call('session.call',{'sessionId':sid,'operation':'extension:invoke-action','payload':payload})
 def refresh(self,sid):return self.call('session.call',{'sessionId':sid,'operation':'extension:refresh-session','payload':{}})
 def close(self,eof=False):
  if not eof:self.call('shutdown',{})
  self.p.stdin.close();assert self.p.wait(timeout=15)==0
  assert list((self.dir/'temp').iterdir())==[];assert list((self.dir/'data').iterdir())==[]
def values(a='alpha.png',b='changed.png',t='0'):
 return {'actual':[str(r/'qa/fixtures/images'/a)],'expected':[str(r/'qa/fixtures/images'/b)],'threshold':t}
p=Process();s=p.open('main');(run/'native-form.json').write_text(json.dumps(s)+'\n');assert s['root']['type']=='Form';assert s['root']['children'][3]['props']['value']=='0'
# Source validation preserves all drafts and returns fresh submit IDs.
bad={'actual':[],'expected':[],'threshold':''};v=p.action('main',s,bad)
assert v['root']['children'][0]['props']['error']=='The item is required.';assert v['actions'][0]['id']!=s['actions'][0]['id']
assert p.action('main',v,bad,kind=s['actions'][0]['id'])['mode']=='unchanged'
# A/B/A, including an invalid intermediate selection, current all-field revisions.
token=v['root']['props']['formToken'];change=f'f:{token}:change'
for revision,draft in [(1,values()),(2,values('file.svg')),(3,values())]:
 v=p.action('main',v,draft,kind=change,revision=revision);assert v['root']['children'][0]['props']['value']==draft['actual']
assert p.action('main',v,values('edge-0.png'),kind=change,revision=2)['mode']=='unchanged'
detail=p.action('main',v,values(),revision=3);assert detail['root']['type']=='Detail';views.append(detail)
assert detail['root']['props']['markdown'].startswith('![](data:image/png;base64,')
expected=json.loads((r/'qa/oracle.json').read_text());case=next(x for x in expected['pairs']if x['a']=='alpha.png'and x['b']=='changed.png'and x['threshold']==0)
# Actual returned PNG dimensions/pixels are verified by Pillow under this owned home.
from PIL import Image
import io
im=Image.open(io.BytesIO(base64.b64decode(detail['root']['props']['markdown'].split(',')[1][:-1])));assert im.convert('RGBA').tobytes()==base64.b64decode(case['rgba'])
assert p.action('main',detail,kind=v['actions'][0]['id'])['mode']=='unchanged'
assert p.refresh('main')['root']==detail['root']
back=p.action('main',detail);assert back['root']['type']=='Form';assert back['root']['children'][0]['props']['value']==values()['actual']
again=p.action('main',back,values(),revision=0);assert again['root']==detail['root'];assert again['actions']!=detail['actions']
back=p.action('main',again)
# Decode error and unequal sizes preserve exact submitted draft, no result.
broken=run/'broken.png';broken.write_bytes(b'invalid image')
for a in [str(broken),str(r/'qa/fixtures/images/edge-0.png')]:
 draft=values();draft['actual']=[a];old=back['actions'][0]['id'];back=p.action('main',back,draft)
 assert back['root']['type']=='Form';assert 'An error occurred while comparing images.'in back['message'];assert back['root']['children'][0]['props']['value']==[a];assert old!=back['actions'][0]['id']
# Host all-fields/revision contract and stale Back are inert.
assert p.action('main',back,kind=detail['actions'][0]['id'])['mode']=='unchanged'
missing=p.action('main',back);assert 'all current form fields'in missing['message']
# Real JPEG submissions through the SDK; all-field A/B/A, stale revision, Back and fresh submit.
jpeg_cases=[('color-q95-s2.jpg','rgb.png',0),('color-q95-s1.jpg','rgb.png',0.1),('progressive.jpg','rgb.png',0.2),('cmyk.jpg','rgb.png',0),
 ('jpeg-blocks-33x19-restart1-p1.jpg','jpeg-blocks-33x19.png',0.1),
 ('jpeg-gradient-33x19-YCCK-p1.jpg','jpeg-gradient-33x19.png',0),
 ('jpeg-blocks-17x13-q10-s0-p0.jpg','jpeg-blocks-17x13-q73-s2-p0.jpg',0.1),
 ('jpeg-gradient-33x19-exif8.jpg','boundary-jpeg-gradient-33x19-exif8.jpg.png',0),
 ('jpeg-blocks-1x1-q73-s2-p0.jpg','boundary-jpeg-blocks-1x1-q73-s2-p0.jpg.png',0)]
jpeg_form=p.open('jpeg');jpeg_inputs={}
for a,b,t in jpeg_cases:
 draft=values(a,b,str(t));token=jpeg_form['root']['props']['formToken'];change=f'f:{token}:change'
 for revision,current in [(1,draft),(2,values('invalid.svg',b,str(t))),(3,draft)]:
  jpeg_form=p.action('jpeg',jpeg_form,current,kind=change,revision=revision)
 assert p.action('jpeg',jpeg_form,values('file.svg'),kind=change,revision=2)['mode']=='unchanged'
 jpeg_detail=p.action('jpeg',jpeg_form,draft,revision=3);assert jpeg_detail['root']['type']=='Detail'
 reference=next(x for x in expected['pairs']if x['a']==a and x['b']==b and x['threshold']==t)
 image=Image.open(io.BytesIO(base64.b64decode(jpeg_detail['root']['props']['markdown'].split(',')[1][:-1])))
 assert image.convert('RGBA').tobytes()==base64.b64decode(reference['rgba'])
 returned=p.action('jpeg',jpeg_detail);assert returned['root']['children'][0]['props']['value']==draft['actual']
 fresh=p.action('jpeg',returned,draft,revision=0);assert fresh['root']==jpeg_detail['root'] and fresh['actions']!=jpeg_detail['actions']
 jpeg_form=p.action('jpeg',fresh)
 for name in [a,b]:
  full=r/'qa/fixtures/images'/name;jpeg_inputs[name]=hashlib.sha256(full.read_bytes()).hexdigest()
for name,sha in jpeg_inputs.items():assert hashlib.sha256((r/'qa/fixtures/images'/name).read_bytes()).hexdigest()==sha
p.call('session.dispose',{'sessionId':'jpeg'});counts['jpegExactSubmits']=len(jpeg_cases)*2;counts['jpegABADrafts']=len(jpeg_cases);counts['jpegInputsUnchanged']=True
# Large real process frame and in-flight cancellation using owned synthetic data.
import random
large=run/'large.png';large_image=Image.frombytes('RGB',(2200,2200),random.Random(481).randbytes(2200*2200*3));large_image.save(large)
large_hash=hashlib.sha256(large.read_bytes()).hexdigest();large_values=values();large_values['actual']=[str(large)];large_values['expected']=[str(large)]
large_form=p.open('large');large_detail=p.action('large',large_form,large_values);assert large_detail['root']['type']=='Detail';assert counts['maxFrame']>1_000_000
large_form=p.action('large',large_detail)
id=p.request('session.call',{'sessionId':'large','operation':'extension:invoke-action','payload':{'actionId':large_form['actions'][0]['id'],'formValues':{**large_values,'__formToken':large_form['root']['props']['formToken'],'__formRevision':0}}})
time.sleep(0.03);p.send({'type':'cancel','requestId':id,'reason':'owned in-flight comparison cancellation'});canceled=p.reply(id);assert 'error'in canceled
canceled_view=p.refresh('large');assert canceled_view['root']['type']=='Form';assert canceled_view['root']['children'][0]['props']['value']==[str(large)]
assert hashlib.sha256(large.read_bytes()).hexdigest()==large_hash
p.call('session.dispose',{'sessionId':'large'});counts['largeFrameAndCompareCancel']=True
p.call('session.dispose',{'sessionId':'main'})
for n in range(16):p.open('cap'+str(n))
assert '16'in p.call('session.open',{'sessionId':'overflow','commandName':'image-diff-check'},error=True)['message']
for n in range(16):p.call('session.dispose',{'sessionId':'cap'+str(n)})
# Actual canceled open requests, not a simulated token flag.
for n in range(140):
 sid='cancel'+str(n);id=p.request('session.open',{'sessionId':sid,'commandName':'image-diff-check'})
 p.send({'type':'cancel','requestId':id,'reason':'owned canceled-open QA'});reply=p.reply(id)
 if 'result'in reply:p.call('session.dispose',{'sessionId':sid})
 counts['canceledOpens']+=1
view=p.open('after-cancels');assert view['root']['type']=='Form';p.call('session.dispose',{'sessionId':'after-cancels'})
p.close();q=Process();q.open('eof');q.close(eof=True)
assert counts['reverse']==0
(run/'native-views.json').write_text(json.dumps(views)+'\n');(run/'transcript.json').write_text(json.dumps(transcript,indent=2)+'\n')
(r/'.qa/process-report.json').write_text(json.dumps({**counts,'binary':str(binary),'sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'packageDir':str(binary.parent.parent if binary.parent.name=='bin' else r),'sourceDiffPixelsVerified':True,'emptyDataTempAfterExit':True,'realSDKPipes':True,'realGUI':False},indent=2)+'\n');print(json.dumps(counts))
