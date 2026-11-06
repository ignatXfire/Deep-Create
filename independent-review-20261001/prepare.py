from pathlib import Path
import subprocess, json, shutil, hashlib, datetime
p=Path(__file__).resolve().parent; root=p.parent
(p/'evidence').mkdir(exist_ok=True); (p/'tmp').mkdir(exist_ok=True)
def git(*args): return subprocess.check_output(['git',*args],cwd=root)
(p/'evidence/baseline.txt').write_bytes(git('rev-parse','HEAD'))
(p/'evidence/status-before.txt').write_bytes(git('status','--short'))
(p/'evidence/uncommitted.patch').write_bytes(git('diff','--','crates/dc-core/MEMORY_LEAK_EVIDENCE.md','crates/dc-core/tests/extended_stability_soak.rs'))
(p/'evidence/changes-since-prior.stat.txt').write_bytes(git('diff','--stat','b194df4','HEAD','--','crates','ui','tools','Cargo.toml'))
(p/'evidence/recent-commits.txt').write_bytes(git('log','-25','--format=%h %aI %s'))
(p/'evidence/worktrees.txt').write_bytes(git('worktree','list','--porcelain'))
names=git('ls-files','-z').decode('utf-8').split('\0')
selected=[]
for name in filter(None,names):
    if name.startswith(('crates/','ui/','tools/','docs/')) or ('/' not in name and name.endswith(('.md','.toml','.lock','.json','.yaml','.ts'))):
        src=root/name
        if not src.is_file() or src.stat().st_size>5_000_000: continue
        if src.suffix.lower() in {'.exe','.dll','.pdb','.zip','.png','.jpg'}: continue
        dest=p/'snapshot'/name; dest.parent.mkdir(parents=True,exist_ok=True); shutil.copyfile(src,dest)
        selected.append({'path':name,'bytes':dest.stat().st_size,'sha256':hashlib.sha256(dest.read_bytes()).hexdigest()})
(p/'evidence/snapshot-manifest.json').write_text(json.dumps(selected,ensure_ascii=False,indent=2),encoding='utf-8')
s=json.loads((root/'session-20260930.json').read_text(encoding='utf-8-sig'))
msgs=s.get('messages',[])
meta={k:v for k,v in s.items() if k!='messages'}
meta.update({'sha256':hashlib.sha256((root/'session-20260930.json').read_bytes()).hexdigest(),'roles':{},'first_message_keys':list(msgs[0]) if msgs else []})
for m in msgs: meta['roles'][m.get('role','?')]=meta['roles'].get(m.get('role','?'),0)+1
(p/'evidence/session-metadata.json').write_text(json.dumps(meta,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps({'snapshot_files':len(selected),'snapshot_bytes':sum(x['bytes'] for x in selected),'session':meta},ensure_ascii=True))
