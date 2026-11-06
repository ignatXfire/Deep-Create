"""Metadata inventory; never claims semantic reading of inventoried files."""
import os, json, hashlib, subprocess
from pathlib import Path
from collections import Counter
root = Path(__file__).resolve().parent.parent
out = Path(__file__).resolve().parent
excluded = {'node_modules', '.git', '.venv', '__pycache__', 'toolchain', 'pnpm-store', 'pnpm-cache', 'npm-cache', 'pnpm-state', out.name}
rows, skips, errors = [], [], []
def walk(folder):
    try:
        with os.scandir(folder) as entries:
            for e in entries:
                rel = str(Path(e.path).relative_to(root))
                try:
                    stat = e.stat(follow_symlinks=False)
                    if getattr(stat, 'st_file_attributes', 0) & 1024:
                        skips.append({'path':rel,'reason':'reparse point'}); continue
                    if e.is_dir(follow_symlinks=False):
                        if e.name in excluded or e.name == 'target' or e.name.startswith('target-'):
                            skips.append({'path':rel,'reason':'build/cache/git/audit exclusion'}); continue
                        walk(e.path)
                    else:
                        rows.append({'path':rel,'bytes':stat.st_size,'mtime_ns':stat.st_mtime_ns})
                except OSError as ex: errors.append({'path':rel,'error':str(ex)})
    except OSError as ex: errors.append({'path':str(folder),'error':str(ex)})
walk(root)
(out/'evidence/inventory.json').write_text(json.dumps({'files':rows,'excluded_subtrees':skips,'errors':errors},ensure_ascii=False,indent=2),encoding='utf-8')
summary={'files_metadata_only':len(rows),'bytes':sum(x['bytes'] for x in rows),'excluded_subtrees':len(skips),'errors':len(errors),'top_level_counts':dict(Counter(Path(x['path']).parts[0] for x in rows))}
(out/'evidence/inventory-summary.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(summary,ensure_ascii=True))
