#!/usr/bin/env python3
"""Run a command with every mutable tool/cache/temp/home path in this clone."""
from pathlib import Path
import os,subprocess,sys,site
r=Path(__file__).resolve().parents[1];q=r/'.qa'
for p in ['home','tmp','cargo','target','npm-cache']:(q/p).mkdir(parents=True,exist_ok=True)
env=dict(os.environ,CARGO_HOME=str(q/'cargo'),CARGO_TARGET_DIR=str(q/'target'),HOME=str(q/'home'),TMPDIR=str(q/'tmp'),TEMP=str(q/'tmp'),TMP=str(q/'tmp'),npm_config_cache=str(q/'npm-cache'),PYTHONDONTWRITEBYTECODE='1',RUSTUP_HOME=os.environ.get('RUSTUP_HOME',str(Path.home()/'.rustup')))
env['PYTHONPATH']=os.pathsep.join([site.getusersitepackages(),os.environ.get('PYTHONPATH','')])
raise SystemExit(subprocess.run(sys.argv[1:],env=env,cwd=r.parent).returncode)
