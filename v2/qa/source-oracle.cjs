// Real own component/hook/validators/compare, locked useForm/Jimp/pixelmatch.
// Every effect gate is installed before loading/executing source callbacks.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),clone=path.dirname(root),qa=path.join(root,'.qa'),owned=path.join(qa,'oracle');
fs.mkdirSync(owned,{recursive:true});
const original={read:fs.readFileSync.bind(fs),write:fs.writeFileSync.bind(fs),open:fs.openSync.bind(fs),close:fs.closeSync.bind(fs),readSync:fs.readSync.bind(fs),writeSync:fs.writeSync.bind(fs),fstat:fs.fstatSync.bind(fs)};
const deps=path.join(qa,'v1/node_modules'),renderdeps=path.join(qa,'renderer/node_modules');
const ts=require(path.join(deps,'typescript')),React=require(path.join(renderdeps,'react'));
const {JSDOM,ResourceLoader}=require(path.join(renderdeps,'jsdom'));
const effects={reads:0,writes:0,denied:0,toast:[],clipboard:0,network:0};
const deny=(what)=>{effects.denied++;throw Error('QA producer denied: '+what);};
class NoResources extends ResourceLoader {fetch(){effects.network++;return deny('browser fetch');}}
const dom=new JSDOM('<div id="app"></div>',{url:'https://inert.invalid',resources:new NoResources()});
Object.assign(global,{window:dom.window,document:dom.window.document,IS_REACT_ACT_ENVIRONMENT:true});
Object.defineProperty(global,'navigator',{value:dom.window.navigator,configurable:true});
Object.defineProperty(dom.window.navigator,'clipboard',{value:new Proxy({},{get(){effects.clipboard++;return ()=>deny('clipboard');}})});
dom.window.open=()=>deny('browser open');dom.window.fetch=()=>deny('browser fetch');
const ReactDOM=require(path.join(renderdeps,'react-dom/client'));
function allowed(p,write=false){if(typeof p!=='string'&&!Buffer.isBuffer(p)&&!(p instanceof URL))return deny('descriptor/path');const full=path.resolve(p instanceof URL?require('node:url').fileURLToPath(p):String(p));const base=write?owned:root;if(full!==base&&!full.startsWith(base+path.sep))return deny('fs '+full);return full;}
const reads=new Set(['readFile','readFileSync','stat','statSync','lstat','lstatSync','access','accessSync','existsSync','realpath','realpathSync','readdir','readdirSync','readlink','readlinkSync']);
const writes=new Set(['writeFile','writeFileSync','mkdir','mkdirSync']);
const dangerous=/^(?:append|chmod|chown|copy|cp|create|fchmod|fchown|fdatasync|fstat|fsync|ftruncate|futimes|link|lutimes|mkdtemp|open|opendir|read|rename|rm|rmdir|symlink|truncate|unlink|utimes|watch|write|statfs)/;
for(const obj of [fs,fs.promises])for(const k of Object.keys(obj)){
 if(typeof obj[k]!=='function')continue;
 const fn=obj[k].bind(obj);
 if(reads.has(k)||writes.has(k)){const wrapped=function(p,...args){allowed(p,writes.has(k));effects[writes.has(k)?'writes':'reads']++;return fn(p,...args);};if(obj[k].native)wrapped.native=(p,...args)=>{allowed(p);return obj[k](p,...args);};obj[k]=wrapped;}
 else if(dangerous.test(k))obj[k]=()=>deny('fs.'+k);
}
const descriptors=new Map();
fs.openSync=(p,flags,mode)=>{const writing=typeof flags==='number'?(flags&3)!==0:/[wa+]/.test(flags);allowed(p,writing);const fd=original.open(p,flags,mode);descriptors.set(fd,writing);return fd;};
fs.closeSync=fd=>{if(!descriptors.has(fd))return deny('close descriptor');descriptors.delete(fd);return original.close(fd);};
fs.readSync=(fd,...args)=>{if(!descriptors.has(fd))return deny('read descriptor');return original.readSync(fd,...args);};
fs.writeSync=(fd,...args)=>{if(!descriptors.get(fd))return deny('write descriptor');return original.writeSync(fd,...args);};
fs.fstatSync=(fd,...args)=>{if(!descriptors.has(fd))return deny('stat descriptor');return original.fstat(fd,...args);};
require('node:module').syncBuiltinESMExports();
const os=require('node:os');for(const k of ['homedir','tmpdir'])os[k]=()=>owned;
for(const k of ['userInfo','networkInterfaces','hostname'])os[k]=()=>deny('os.'+k);
for(const name of ['child_process','net','tls','http','https','http2','dgram','dns','worker_threads']){const m=require('node:'+name);for(const k of Object.keys(m))if(typeof m[k]==='function'&&/(?:exec|spawn|fork|connect|create|request|get|lookup|resolve|Worker)/i.test(k))m[k]=()=>deny(name+'.'+k);if(m.promises)for(const k of Object.keys(m.promises))if(typeof m.promises[k]==='function')m.promises[k]=()=>deny(name+'.promises.'+k);}
global.fetch=()=>deny('fetch');global.WebSocket=()=>deny('WebSocket');
process.chdir(owned);process.chdir=()=>deny('process.chdir');process.kill=()=>deny('process.kill');
for(const k of ['HOME','TMPDIR','TMP','TEMP','USERPROFILE'])process.env[k]=owned;
require('node:module').syncBuiltinESMExports();
// Assert representative callback/promise/stream/native producers fail closed.
for(const fn of [()=>fs.readFileSync('/etc/passwd'),()=>fs.createReadStream('/etc/passwd'),()=>fs.promises.open('/etc/passwd'),()=>require('node:child_process').spawn('echo'),()=>fetch('https://invalid'),()=>os.userInfo()])assert.throws(fn,/denied/);
const negativeDenials=effects.denied;
const Jimp=require(path.join(deps,'jimp')).Jimp;
let captured={fields:{},submit:null,markdown:''};
const api=new Proxy({Toast:{Style:{Success:'success',Failure:'failure'}},showToast:async v=>{effects.toast.push(v);},Form:Object.assign(({children,actions})=>React.createElement(React.Fragment,null,children,actions),{FilePicker:React.forwardRef((p,ref)=>{captured.fields[p.id]=p;React.useImperativeHandle(ref,()=>({focus(){},reset(){}}));return null;}),Separator:()=>null,Dropdown:Object.assign(React.forwardRef((p,ref)=>{captured.fields[p.id]=p;React.useImperativeHandle(ref,()=>({focus(){},reset(){}}));return null;}),{Item:()=>null})}),ActionPanel:({children})=>children,Action:{SubmitForm:p=>{captured.submit=p.onSubmit;return null;}},Detail:p=>{captured.markdown=p.markdown;return null;}},{get(o,k){if(k in o)return o[k];return ()=>deny('Raycast '+String(k));}});
// Exact locked useForm + its two direct helpers, no rest-of-bundle execution.
const bundled=original.read(path.join(deps,'@raycast/utils/dist/main.js'),'utf8');
const tree=ts.createSourceFile('utils.js',bundled,ts.ScriptTarget.ES2022,true);
const names=['$07a180af23fbf38b$export$d4b699e2c1148419','$c950f3189e9100ee$var$validationError','$c950f3189e9100ee$export$87c0cf8eb5a167e0'];
const functions=names.map(n=>tree.statements.find(s=>s.name?.text===n)?.getText(tree));assert.ok(functions.every(Boolean));
const utilsContext=vm.createContext({$8zHUo$react:React,Proxy,Object,Array,Date});
vm.runInContext(functions.join('\n')+'\nthis.useForm='+names[2],utilsContext);
const cache={};let pixelmatch;
async function main(){
 pixelmatch=(await import(require('node:url').pathToFileURL(path.join(deps,'pixelmatch/index.js')).href)).default;
 function load(rel){if(cache[rel])return cache[rel];let filename=path.resolve(qa,'v1',rel);assert.ok(filename.startsWith(path.join(qa,'v1/src/')));if(!path.extname(filename))filename+='.ts';if(!fs.existsSync(filename))filename=path.join(filename.slice(0,-3),'index.ts');const exp={};
 const source=original.read(filename,'utf8');const req=name=>{if(name==='react')return React;if(name==='react/jsx-runtime')return require(path.join(renderdeps,'react/jsx-runtime'));if(name==='@raycast/api')return api;if(name==='@raycast/utils')return {useForm:utilsContext.useForm};if(name==='jimp')return {Jimp};if(name==='pixelmatch')return {default:pixelmatch,__esModule:true};if(name.startsWith('.'))return load(path.relative(path.join(qa,'v1'),path.resolve(path.dirname(filename),name)));return deny('module '+name);};
 const context=vm.createContext({exports:exp,require:req,Buffer,console,Promise,Error},{codeGeneration:{strings:false,wasm:false}});
 vm.runInContext(ts.transpileModule(source,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS,jsx:ts.JsxEmit.ReactJSX,esModuleInterop:true}}).outputText,context,{timeout:1000});return cache[rel]=exp;}
 const Command=load('src/image-diff-check.tsx').default,compare=load('src/utils/pixelmatch.ts').compare;
 const app=ReactDOM.createRoot(document.getElementById('app'));await React.act(async()=>app.render(React.createElement(Command)));
 assert.deepEqual(Object.keys(captured.fields),['actual','expected','threshold']);assert.equal(captured.fields.threshold.value,null);
 await React.act(async()=>{assert.equal(await captured.submit({actual:[],expected:[],threshold:''}),false);});assert.equal(effects.toast.length,0);
 assert.equal(captured.fields.actual.error,'The item is required.');assert.equal(captured.fields.expected.error,'The item is required.');assert.equal(captured.fields.threshold.error,'The threshold is required.');
 const manifest=JSON.parse(original.read(path.join(root,'qa/fixtures/images.json'),'utf8'));
 const images=[];
 for(const f of manifest.filter(f=>!f.path.startsWith('boundary-'))){const bytes=original.read(path.join(root,'qa/fixtures/images',f.path));const dest=path.join(owned,f.path);fs.writeFileSync(dest,bytes);const im=await Jimp.read(dest);images.push({path:f.path,width:im.bitmap.width,height:im.bitmap.height,rgba:im.bitmap.data.toString('base64')});}
 // Narrow compatibility observation: a 12-bit SOF1 declaration with the owned
 // grayscale entropy is accepted by the locked source; product deliberately permits 8-bit DCT only.
 const twelve=Buffer.from(original.read(path.join(owned,'gray.jpg')));
 const sof=twelve.indexOf(Buffer.from([255,192]));assert.ok(sof>0);twelve[sof+1]=193;twelve[sof+4]=12;
 fs.writeFileSync(path.join(owned,'scope-12bit.jpg'),twelve);
 const unusual=await Jimp.read(path.join(owned,'scope-12bit.jpg'));
 const jpegScope={twelveBitDeclared:{sourceAccepted:true,sourceDimensions:[unusual.bitmap.width,unusual.bitmap.height],nativeAccepted:false,fixture:'scope-12bit.jpg',note:'Owned header variation, not evidence for every 12-bit encoder'}};
 function minimalComponents(count) {
  const chunks=[Buffer.from([255,216])];
  const seg=(marker,bytes)=>{const n=Buffer.alloc(4);n[0]=255;n[1]=marker;n.writeUInt16BE(bytes.length+2,2);chunks.push(n,Buffer.from(bytes));};
  seg(219,[0,...Array(64).fill(1)]);
  seg(192,[8,0,1,0,1,count,...Array.from({length:count},(_,i)=>[i+1,17,0]).flat()]);
  for(const spec of [0,16])seg(196,[spec,1,...Array(15).fill(0),0]);
  seg(218,[count,...Array.from({length:count},(_,i)=>[i+1,0]).flat(),0,63,0]);
  chunks.push(Buffer.from([(1<<(8-count*2))-1,255,217]));return Buffer.concat(chunks);
 }
 for(const [name,bytes]of [['twoComponents',minimalComponents(2)],['unmarkedCMYK',minimalComponents(4)],...[[195,'losslessMarker'],[201,'arithmeticMarker']].map(([marker,name])=>{const b=Buffer.from(twelve);b[sof+1]=marker;return [name,b];})]) {
  const file=path.join(owned,'scope-'+name+'.jpg');fs.writeFileSync(file,bytes);
  let error=null;try {await Jimp.read(file);} catch(e){error=e.message;}
  assert.ok(error,'expected locked-source unsupported mode '+name);
  jpegScope[name]={sourceAccepted:false,sourceError:error,nativeAccepted:false};
 }
 // Execute actual submit and original diff markdown, with a malicious filename.
 const malicious=path.join(owned,'](<img src=https: evil onerror=boom>).PNG');fs.writeFileSync(malicious,original.read(path.join(owned,'alpha.png')));
 await React.act(async()=>{captured.fields.actual.onChange([malicious]);captured.fields.expected.onChange([path.join(owned,'changed.png')]);captured.fields.threshold.onChange('0');});
 await React.act(async()=>{await captured.submit({actual:[malicious],expected:[path.join(owned,'changed.png')],threshold:'0'});});
 assert.ok(captured.markdown.startsWith('![](data:image/png;base64,'));assert.ok(!captured.markdown.includes('onerror'));assert.equal(effects.toast.at(-1).title,'Success🎉');
 const hookDiff=await Jimp.read(Buffer.from(captured.markdown.split(',')[1].slice(0,-1),'base64'));
 const sourceDiff=await compare(malicious,path.join(owned,'changed.png'),'0');assert.ok(hookDiff.bitmap.data.equals(sourceDiff.diffBuffer));
 const previous=captured.markdown;
 for(const name of ['broken.png','wrong-size.png']){
  fs.writeFileSync(path.join(owned,name),name==='broken.png'?Buffer.from('broken'):original.read(path.join(owned,'gray1.png')));
 }
 // Use unequal dimensions, not a blanket decoder-error parity skip.
 fs.writeFileSync(path.join(owned,'wrong-size.png'),original.read(path.join(owned,'edge-0.png')));
 for(const name of ['broken.png','wrong-size.png']){await React.act(async()=>{await captured.submit({actual:[path.join(owned,name)],expected:[malicious],threshold:'0.1'});});assert.equal(captured.markdown,previous);assert.equal(effects.toast.at(-1).message,'An error occurred while comparing images.');}
 const pairs=[];
 const combinations=[['alpha.png','changed.png'],['edge-0.png','edge-1.png'],['transparent-hidden.png','transparent-other.png'],['animated.gif','transparent.gif'],...images.map(x=>[x.path,x.path]),...images.filter(x=>x.path.endsWith('.jpg')&&x.width===17&&x.height===13).map(x=>[x.path,'rgb.png'])];
 // Additional JPEG-to-original-PNG and JPEG-to-JPEG neighborhoods. Keep all original 605 pairs.
 for (const image of images.filter(x=>x.path.startsWith('jpeg-')&&x.path.endsWith('.jpg'))) {
  const prefix=image.path.match(/^(jpeg-(?:gradient|blocks)-\d+x\d+)/)[1];
  const reference=images.find(x=>x.path===prefix+'.png');
  if(image.width===reference.width&&image.height===reference.height) {
   combinations.push([image.path,reference.path]);
   const other=prefix+'-q73-s2-p0.jpg'; if(image.path!==other)combinations.push([image.path,other]);
  } else combinations.push([image.path,prefix+'-exif'+image.path.match(/exif(\d)/)[1]+'.jpg']);
 }
 for(const [a,b]of combinations)for(let n=0;n<=10;n++){
  const threshold=String(n/10),result=await compare(path.join(owned,a),path.join(owned,b),threshold);
  const ia=images.find(x=>x.path===a),ib=images.find(x=>x.path===b);const aa=Buffer.from(ia.rgba,'base64'),bb=Buffer.from(ib.rgba,'base64'),mask=Buffer.alloc(aa.length);
  const count=pixelmatch(aa,bb,mask,ia.width,ia.height,{threshold:n/10});assert.ok(mask.equals(result.diffBuffer));
  pairs.push({a,b,threshold:n/10,count,rgba:mask.toString('base64')});
 }
 // JPEG-derived one-channel threshold boundary; real helper execution on owned PNGs.
 let jpegBoundaries=0;
 for(const image of images.filter(x=>x.path.startsWith('jpeg-')&&x.path.endsWith('.jpg'))) {
  const reference=Buffer.from(image.rgba,'base64'), changed=Buffer.from(reference);
  changed[0]=changed[0]===255?254:changed[0]+1;
  const a=image.path,b='boundary-'+image.path+'.png';
  const im=new Jimp({width:image.width,height:image.height,data:changed});
  fs.writeFileSync(path.join(owned,b),await im.getBuffer('image/png'));
  images.push({path:b,width:image.width,height:image.height,rgba:changed.toString('base64'),generatedBoundary:true});
  for(const threshold of [0,0.00001,0.001,0.1]) {
   const result=await compare(path.join(owned,a),path.join(owned,b),String(threshold)),mask=Buffer.alloc(reference.length);
   const count=pixelmatch(reference,changed,mask,image.width,image.height,{threshold});assert.ok(mask.equals(result.diffBuffer));
   pairs.push({a,b,threshold,count,rgba:mask.toString('base64')});jpegBoundaries++;
  }
 }
 const raw=[];let seed=0x8fa719;
 function random(){seed=(Math.imul(seed,1664525)+1013904223)>>>0;return seed>>>24;}
 for(let n=0;n<100;n++){
  const w=1+n%11,h=1+n%7,a=Buffer.alloc(w*h*4),b=Buffer.alloc(a.length);for(let k=0;k<a.length;k++){a[k]=random();b[k]=(n%3===0&&k%4!==3)?a[k]:random();}
  for(const threshold of [0,0.1,0.3,0.5,1,0.1-Number.EPSILON,0.1+Number.EPSILON]){const output=Buffer.alloc(a.length),count=pixelmatch(a,b,output,w,h,{threshold});raw.push({w,h,a:a.toString('base64'),b:b.toString('base64'),threshold,count,rgba:output.toString('base64')});}
 }
 // Deliberate metric threshold boundary, both sides; 1x1 cannot be AA.
 const aa=Buffer.from([100,140,200,255]),bb=Buffer.from([89,149,190,255]);
 const dr=11,dg=-9,db=10,y=dr*.29889531+dg*.58662247+db*.11448223,i=dr*.59597799-dg*.27417610-db*.32180189,q=dr*.21147017-dg*.52261711+db*.31114694;
 const boundary=Math.sqrt((.5053*y*y+.299*i*i+.1957*q*q)/35215);
 for(const [threshold,expected]of [[boundary-1e-14,1],[boundary+1e-14,0]]){const output=Buffer.alloc(4),count=pixelmatch(aa,bb,output,1,1,{threshold});assert.equal(count,expected);raw.push({w:1,h:1,a:aa.toString('base64'),b:bb.toString('base64'),threshold,count,rgba:output.toString('base64')});}
 assert.ok(pairs.some(p=>{const b=Buffer.from(p.rgba,'base64');for(let k=0;k<b.length;k+=4)if(b[k]===255&&b[k+1]===255&&b[k+2]===0)return true;return false;}),'AA yellow must be exercised');
 await React.act(async()=>app.unmount());assert.equal(effects.denied,negativeDenials);assert.equal(effects.network,0);assert.equal(effects.clipboard,0);
 original.write(path.join(root,'qa/oracle.json'),JSON.stringify({images,pairs,raw})+'\n');
 original.write(path.join(qa,'source-oracle.json'),JSON.stringify({images:images.length,pairs:pairs.length,raw:raw.length,jpegBoundaries,jpegScope,actualComponentAndHook:true,lockedUseForm:true,executedSubmitCallbacks:4,errorsPreserveMarkdown:true,sourceEffects:effects,unexpectedProducers:effects.denied-negativeDenials,realOS:false},null,2)+'\n');
 console.log(JSON.stringify({images:images.length,pairs:pairs.length,raw:raw.length,negativeDenials,unexpectedProducers:0}));
}
main().catch(e=>{console.error(e);process.exitCode=1;});
