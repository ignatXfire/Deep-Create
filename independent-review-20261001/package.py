from pathlib import Path
import json,zipfile,hashlib,subprocess
p=Path(__file__).resolve().parent;root=p.parent
subprocess.run(['python',str(p/'finalize.py')],check=True)
manifest=json.loads((p/'evidence/snapshot-manifest.json').read_text(encoding='utf-8'))
allowed={x['path'] for x in manifest}
scan=json.loads((p/'evidence/credential-pattern-scan.json').read_text(encoding='utf-8'))
if scan['matched_paths']:raise RuntimeError('Review known credential patterns before packaging; paths saved, values never printed')
selected=[]
for f in p.rglob('*'):
    if not f.is_file():continue
    rel=f.relative_to(p)
    if rel.parts[0] in {'target','tmp','__pycache__'}:continue
    if rel.parts[0]=='snapshot' and Path(*rel.parts[1:]).as_posix() not in allowed:continue
    selected.append(f)
archive=root/'DeepCreate_independent_review_2026-10-01.zip'
with zipfile.ZipFile(archive,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=6) as z:
    for f in selected:z.write(f,'independent-review-20261001/'+f.relative_to(p).as_posix())
    z.write(root/'INDEPENDENT_REVIEW_2026-10-01.md','INDEPENDENT_REVIEW_2026-10-01.md')
with zipfile.ZipFile(archive) as z:
    assert z.testzip() is None
    assert len(z.namelist())==len(selected)+1
    for f in selected:
        assert hashlib.sha256(z.read('independent-review-20261001/'+f.relative_to(p).as_posix())).digest()==hashlib.sha256(f.read_bytes()).digest()
digest=hashlib.sha256(archive.read_bytes()).hexdigest()
archive.with_suffix('.zip.sha256').write_text(digest+'  '+archive.name+'\n',encoding='utf-8')
verification={'zip':str(archive),'entries':len(selected)+1,'bytes':archive.stat().st_size,'sha256':digest,'crc':'passed','entry_hashes':'all matched','snapshot_files':len(manifest)}
(p/'evidence/archive-verification.json').write_text(json.dumps(verification,indent=2),encoding='utf-8')
print(json.dumps(verification))
