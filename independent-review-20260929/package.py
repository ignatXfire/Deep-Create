from pathlib import Path
import subprocess, json, hashlib, zipfile, shutil
p=Path(__file__).resolve().parent
root=p.parent
# NUL-separated names preserve Cyrillic/spaces regardless of Git quotepath setting.
tracked=subprocess.check_output(['git','ls-files','-z'],cwd=root).decode('utf-8').split('\0')
added=[]
for name in filter(None,tracked):
    if name.startswith(('crates/','ui/','tools/','docs/technical/','docs/implementation/')) or name in {'Cargo.toml','Cargo.lock','package.json','pnpm-lock.yaml','pnpm-workspace.yaml','tsconfig.json','vite.config.ts','rust-toolchain.toml','AGENTS.md'}:
        src=root/name; dest=p/'snapshot'/name
        if src.is_file() and not dest.exists():
            dest.parent.mkdir(parents=True,exist_ok=True)
            shutil.copyfile(src,dest)
            added.append(name)
manifest=[{'Path':str(f.relative_to(p/'snapshot')),'Bytes':f.stat().st_size,'SHA256':hashlib.sha256(f.read_bytes()).hexdigest().upper()} for f in sorted((p/'snapshot').rglob('*')) if f.is_file()]
(p/'evidence/snapshot-manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2),encoding='utf-8')
(p/'evidence/filename-copy-correction.json').write_text(json.dumps({'added_using_git_nul_names':added,'note':'Initial PowerShell copy omitted quoted Cyrillic filenames; corrected before packaging. Existing tested source files were not changed.'},ensure_ascii=False,indent=2),encoding='utf-8')
for f in [p/'README.md',root/'INDEPENDENT_REVIEW_2026-09-29.md']:
    f.write_text(f.read_text(encoding='utf-8').replace('207',str(len(manifest))),encoding='utf-8')
subprocess.run(['python',str(p/'finalize.py')],check=True)
archive=root/'DeepCreate_independent_review_2026-09-29.zip'
selected=[f for f in p.rglob('*') if f.is_file() and f.relative_to(p).parts[0] not in {'target','tmp'}]
with zipfile.ZipFile(archive,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=6) as z:
    for f in selected: z.write(f,'independent-review-20260929/'+f.relative_to(p).as_posix())
    z.write(root/'INDEPENDENT_REVIEW_2026-09-29.md','INDEPENDENT_REVIEW_2026-09-29.md')
with zipfile.ZipFile(archive) as z:
    bad=z.testzip()
    assert bad is None,bad
    assert len(z.namelist())==len(selected)+1
    for f in selected:
        assert hashlib.sha256(z.read('independent-review-20260929/'+f.relative_to(p).as_posix())).digest()==hashlib.sha256(f.read_bytes()).digest()
digest=hashlib.sha256(archive.read_bytes()).hexdigest()
archive.with_suffix('.zip.sha256').write_text(digest+'  '+archive.name+'\n',encoding='utf-8')
print(json.dumps({'zip':str(archive),'files':len(selected)+1,'bytes':archive.stat().st_size,'sha256':digest,'crc':'passed','entry_hashes':'all matched','snapshot_files':len(manifest),'added_filenames':len(added)}))
