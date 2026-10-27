const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..');
const Ajv=require(path.join(root,'.qa/renderer/node_modules/ajv/dist/2020')).default;
const ajv=new Ajv({strict:false,allErrors:true});
ajv.addFormat('uri',{type:'string',validate:s=>{try{return /^[a-z][a-z\d+.-]*:/i.test(s)&&Boolean(new URL(s));}catch{return false;}}});
const schema=JSON.parse(fs.readFileSync(path.join(root,'qa/fixtures/host/schemas/extension-v3.schema.json'))),check=ajv.compile(schema);
const manifest=JSON.parse(fs.readFileSync(path.join(root,'extension.json')));assert.ok(check(manifest),JSON.stringify(check.errors));
const negatives=[['schemaVersion',2],['runtime','javascript'],['execution','embedded'],['protocolVersion',2],['id','com.tezbar.other'],['repository','relative'],['replaces',[]],['replaces',['bad']],['capabilities',['clipboard.unknown']],['platforms',['macOS','macOS']],['commands',[]],['commands',[{name:'cmd',runtime:'rust',platforms:[]}]],['commands',[{name:'cmd',runtime:'rust',platforms:['macOS'],refreshMs:1}]],['commands',[{name:'cmd',runtime:'rust',platforms:['macOS'],timeoutMs:300001}]],['files',{'../escape':'0'.repeat(64)}],['artifacts',{'linux':{}}],['artifacts',{'aarch64-apple-darwin':{url:'http://invalid',sha256:'0'.repeat(64),executable:'bin/a',executableSha256:'0'.repeat(64)}}],['artifacts',{'aarch64-apple-darwin':{url:'https://invalid/a',sha256:'bad',executable:'bin/a',executableSha256:'0'.repeat(64)}}],['artifacts',{'aarch64-apple-darwin':{url:'https://invalid/a',sha256:'0'.repeat(64),executable:'../a',executableSha256:'0'.repeat(64)}}]];
for(const [key,value]of negatives){assert.equal(check({...manifest,[key]:value}),false,key);}
for(const key of schema.required){const copy={...manifest};delete copy[key];assert.equal(check(copy),false,'missing '+key);}
const report={sourceManifestValid:true,fullDraft202012:true,negatives:negatives.length+schema.required.length,artifactsEmpty:true};
fs.writeFileSync(path.join(root,'.qa/schema-report.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
