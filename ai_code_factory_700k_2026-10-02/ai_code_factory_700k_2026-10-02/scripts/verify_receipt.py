#!/usr/bin/env python3
import argparse, hashlib, json
from pathlib import Path

def sha256(p: Path) -> str:
    h=hashlib.sha256()
    with p.open('rb') as f:
        for b in iter(lambda:f.read(1024*1024), b''): h.update(b)
    return h.hexdigest()

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('receipt'); ap.add_argument('--repo',default='.')
    a=ap.parse_args(); r=json.loads(Path(a.receipt).read_text(encoding='utf-8')); repo=Path(a.repo)
    errors=[]
    for key in ('task_id','attempt_id','base_sha','head_sha','model','commands','artifacts','status'):
        if key not in r: errors.append(f'missing {key}')
    for i,c in enumerate(r.get('commands',[])):
        if c.get('exit_code') != 0: errors.append(f'command[{i}] exit={c.get("exit_code")}')
        if c.get('failed',0): errors.append(f'command[{i}] failed={c.get("failed")}')
    for a in r.get('artifacts',[]):
        p=repo/a['path']
        if not p.exists(): errors.append(f'missing artifact {p}')
        elif sha256(p).lower()!=a['sha256'].lower(): errors.append(f'hash mismatch {p}')
    if errors:
        print(json.dumps({'valid':False,'errors':errors},indent=2)); raise SystemExit(2)
    print(json.dumps({'valid':True,'task_id':r['task_id'],'head_sha':r['head_sha']},indent=2))
if __name__=='__main__': main()
