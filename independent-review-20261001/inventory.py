"""Fresh metadata inventory. A listed file is not a semantically reviewed file."""
import os,json
from pathlib import Path
from collections import Counter
p=Path(__file__).resolve().parent; root=p.parent
excluded={'.git','node_modules','.venv','__pycache__','toolchain','pnpm-store','pnpm-cache','npm-cache','pnpm-state'}
rows=[]; skips=[]; errors=[]
def walk(folder):
    try:
        with os.scandir(folder) as entries:
            for e in entries:
                rel=str(Path(e.path).relative_to(root))
                try:
                    st=e.stat(follow_symlinks=False)
                    if getattr(st,'st_file_attributes',0)&1024: skips.append({'path':rel,'reason':'reparse'}); continue
                    if e.is_dir(follow_symlinks=False):
                        if e.name in excluded or e.name=='target' or e.name.startswith(('target-','independent-review-')): skips.append({'path':rel,'reason':'build/cache/git/audit'}); continue
                        walk(e.path)
                    else: rows.append({'path':rel,'bytes':st.st_size,'mtime_ns':st.st_mtime_ns})
                except OSError as ex:errors.append({'path':rel,'error':str(ex)})
    except OSError as ex:errors.append({'path':str(folder),'error':str(ex)})
walk(root)
(p/'evidence/inventory.json').write_text(json.dumps({'files':rows,'excluded_subtrees':skips,'errors':errors},ensure_ascii=False,indent=2),encoding='utf-8')
summary={'files_metadata_only':len(rows),'bytes':sum(x['bytes'] for x in rows),'excluded_subtrees':len(skips),'errors':len(errors),'top_level_counts':dict(Counter(Path(x['path']).parts[0] for x in rows))}
(p/'evidence/inventory-summary.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps({k:v for k,v in summary.items() if k!='top_level_counts'}))
