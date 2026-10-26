#!/usr/bin/env python3
"""Final validation matrix; logs, homes, tool state and artifacts stay clone-owned.

Requires the already-installed toolchains/targets and Pillow; installs no OS tools.
Use qa/run.py as the outer wrapper. Development npm dependencies use exact locks.
"""
from pathlib import Path
import subprocess,sys,json,shutil,hashlib,os
r=Path(__file__).resolve().parents[1];q=r/'.qa';q.mkdir(exist_ok=True)
results=[]
def run(label,cmd,expected=0):
 p=subprocess.run(cmd,cwd=r.parent,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True)
 (q/(label+'.log')).write_text(p.stdout)
 results.append({'name':label,'command':cmd,'exitCode':p.returncode,'logSha256':hashlib.sha256(p.stdout.encode()).hexdigest()})
 print(label,p.returncode,flush=True)
 if expected=='windows-tools':assert p.returncode==0 or any(x in p.stdout for x in ['dlltool','linker `x86_64-w64-mingw32-gcc` not found']),p.stdout[-2000:]
 else:assert p.returncode==expected,p.stdout[-4000:]
for name in ['package.json','package-lock.json','tsconfig.json','raycast-env.d.ts','src','test']:
 src=r.parent/name;out=q/'v1'/name;out.parent.mkdir(parents=True,exist_ok=True)
 if src.is_dir():shutil.copytree(src,out,dirs_exist_ok=True)
 else:shutil.copyfile(src,out)
for name in ['package.json','package-lock.json']:
 out=q/'renderer'/name;out.parent.mkdir(exist_ok=True);shutil.copyfile(r/'qa'/name,out)
for name in ['v1','renderer']:run(name+'-install',['npm','ci','--ignore-scripts','--no-audit','--no-fund','--prefix',str(q/name)])
run('v1-tests',['npm','--prefix',str(q/'v1'),'test'])
run('v1-typecheck',['npm','--prefix',str(q/'v1'),'run','typecheck'],2)
assert "Cannot find module 'vitest'"in(q/'v1-typecheck.log').read_text()
for name in ['host-patch','host-contract','producers','jpeg-investigation']:run(name,[sys.executable,'-B',str(r/'qa'/f'{name}.py')])
run('source-oracle',['node',str(r/'qa/source-oracle.cjs')])
run('save-boundaries',[sys.executable,'-B',str(r/'qa/save-boundaries.py')])
run('schema',['node',str(r/'qa/schema.cjs')])
for toolchain in ['stable','1.85.0']:
 for name,args in [('fmt',['fmt','--check']),('tests',['test','--locked','--all-targets']),('clippy',['clippy','--locked','--all-targets'])]:
  cmd=['cargo','+'+toolchain]+args+['--manifest-path',str(r/'Cargo.toml')]
  if name=='clippy':cmd+=['--','-D','warnings']
  # Exact generated contracts are rustfmt'd before fmt verification.
  if name=='fmt':run(toolchain+'-format',['cargo','+'+toolchain,'fmt','--manifest-path',str(r/'Cargo.toml')])
  # Installer contract tests require a closed ZIP; build/package first below.
  if name=='tests' and not(r/'image-diff-checker-aarch64-apple-darwin-dev.zip').exists():
   run('bootstrap-release',['cargo','+stable','build','--release','--locked','--manifest-path',str(r/'Cargo.toml')]);run('bootstrap-notices',[sys.executable,'-B',str(r/'qa/notices.py')]);run('bootstrap-package',[sys.executable,'-B',str(r/'qa/package.py')])
  run(toolchain+'-'+name,cmd)
run('windows-check',['cargo','+stable','check','--locked','--all-targets','--target','x86_64-pc-windows-gnu','--manifest-path',str(r/'Cargo.toml')])
run('windows-link',['cargo','+stable','build','--release','--locked','--target','x86_64-pc-windows-gnu','--manifest-path',str(r/'Cargo.toml')],'windows-tools')
run('fresh-build',[sys.executable,'-B',str(r/'qa/fresh-build.py')])
fresh=json.loads((q/'fresh-build.json').read_text())
run('notices',[sys.executable,'-B',str(r/'qa/notices.py')])
run('package',[sys.executable,'-B',str(r/'qa/package.py'),fresh['binary']])
package=json.loads((q/'package.json').read_text())
for arg in ['--protocol-info','--self-test']:run('extracted'+arg,[package['binary'],arg])
run('extracted-process',[sys.executable,'-B',str(r/'qa/process.py'),package['binary']])
run('renderer',['node',str(r/'qa/renderer.cjs')])
(r/'qa/validation-matrix.json').write_text(json.dumps(results,indent=2)+'\n')
print('Final matrix complete; Windows native GUI/MSVC/full host remain unrun.')
