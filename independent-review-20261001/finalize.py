from pathlib import Path
import json,hashlib,re,subprocess,datetime
p=Path(__file__).resolve().parent;root=p.parent
manifest=json.loads((p/'evidence/snapshot-manifest.json').read_text(encoding='utf-8'))
drift=[];snapshot_drift=[]
for row in manifest:
    for base,target in [(root,drift),(p/'snapshot',snapshot_drift)]:
        f=base/row['path']
        if not f.is_file() or hashlib.sha256(f.read_bytes()).hexdigest()!=row['sha256']:target.append(row['path'])
results={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'baseline':(p/'evidence/baseline.txt').read_text().strip(),'head_at_finish':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'snapshot_files':len(manifest),'snapshot_hash_mismatches':snapshot_drift,'source_changes_since_snapshot':drift,'checks':{}}
for name in ['safe-rust-tests','probes','vitest','typecheck']:
    log=(p/f'evidence/{name}.log').read_text(encoding='utf-8-sig',errors='replace')
    records=re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored',log)
    row={'exit':(p/f'evidence/{name}.exit.txt').read_text(encoding='utf-8-sig').strip(),'passed':sum(int(x[1]) for x in records),'failed':sum(int(x[2]) for x in records),'ignored':sum(int(x[3]) for x in records),'log':f'evidence/{name}.log'}
    if name=='vitest':row.update({'passed':50,'failed':3,'note':'Assertion failures plus fork shutdown timeout messages; raw log preserved.'})
    if name=='probes':row['meaning']='5 current defects reproduced; 1 fixed event-write behavior confirmed'
    results['checks'][name]=row
results['not_run']=['full workspace/default native build','actual UI clicks/WebView2 runtime','installer/fresh VM','new full 30min soak','real model/account','power failure','full vulnerability/license audit']
(p/'evidence/results.json').write_text(json.dumps(results,ensure_ascii=False,indent=2),encoding='utf-8')
(p/'evidence/status-after.txt').write_bytes(subprocess.check_output(['git','status','--short'],cwd=root))
patterns=[re.compile(rb'sk-[A-Za-z0-9_-]{24,}'),re.compile(rb'ghp_[A-Za-z0-9]{20,}'),re.compile(rb'-----BEGIN [A-Z ]*PRIVATE KEY-----')]
matches=[]
for row in manifest:
    data=(p/'snapshot'/row['path']).read_bytes()
    if any(pattern.search(data) for pattern in patterns):matches.append(row['path'])
(p/'evidence/credential-pattern-scan.json').write_text(json.dumps({'matched_paths':matches,'scope':'Known token/private-key patterns only; no general secret-free certification.'},indent=2),encoding='utf-8')
print(json.dumps(results,ensure_ascii=True));print(json.dumps({'credential_pattern_paths':matches}))
