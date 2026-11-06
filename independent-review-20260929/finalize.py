"""Save actual results and source drift; archive is created separately after completion."""
from pathlib import Path
import json, hashlib, re, subprocess, datetime
p = Path(__file__).resolve().parent
root = p.parent
manifest = json.loads((p/'evidence/snapshot-manifest.json').read_text(encoding='utf-8-sig'))
drift = []
snapshot_drift = []
for row in manifest:
    rel = row['Path'].lstrip('/\\')
    for base, dest in [(root, drift), (p/'snapshot',snapshot_drift)]:
        f = base/rel
        if not f.is_file() or hashlib.sha256(f.read_bytes()).hexdigest().upper() != row['SHA256']:
            dest.append(rel)
head = subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()
results = {'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(), 'baseline':(p/'evidence/baseline.txt').read_text(encoding='utf-8-sig').strip(), 'head_at_finish':head, 'snapshot_files':len(manifest),'snapshot_hash_mismatches':snapshot_drift,'source_changes_since_copy':drift,'checks':{}}
for name in ['typecheck','vitest','safe-rust-tests','probes']:
    exitfile=p/f'evidence/{name}.exit.txt'
    log=(p/f'evidence/{name}.log').read_text(encoding='utf-8-sig',errors='replace')
    records=re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored',log)
    results['checks'][name]={'exit':exitfile.read_text(encoding='utf-8-sig').strip() if exitfile.exists() else 'NOT_COMPLETED','passed':sum(int(x[1]) for x in records),'failed':sum(int(x[2]) for x in records),'ignored':sum(int(x[3]) for x in records),'log':f'evidence/{name}.log'}
results['checks']['vitest']['passed']=53 if re.search(r'Tests\s+53 passed',log:=(p/'evidence/vitest.log').read_text(encoding='utf-8-sig')) else None
results['checks']['workspace']='INTERRUPTED during dependency compilation; not evidence of success or test failure'
results['checks']['probe_initial_attempt']='exit=101: missing parent entity in audit fixture, FK rejection; fixed probe only; original log preserved'
(p/'evidence/results.json').write_text(json.dumps(results,ensure_ascii=False,indent=2),encoding='utf-8')
inventory=json.loads((p/'evidence/inventory.json').read_text(encoding='utf-8'))
recent=sorted((x for x in inventory['files'] if x['path'].startswith('.local') and not x['path'].endswith(('.exe','.dll','.lib','.pdb','.rlib','.rmeta','.obj'))), key=lambda x:x['mtime_ns'],reverse=True)[:40]
(p/'evidence/recent-local-files.json').write_text(json.dumps(recent,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(results,ensure_ascii=True))
