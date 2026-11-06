from pathlib import Path
import json,hashlib
p=Path(__file__).resolve().parent
s=json.loads((p.parent/'session-20260930.json').read_text(encoding='utf-8-sig'))
rows=[];seen=set();counts={}
automatic_prefixes=['[CONTEXT COMPACTION','[IMPORTANT: Background process','[ASYNC DELEGATION','Continue from the compressed','You\'ve reached the maximum']
for i,m in enumerate(s['messages']):
    if m.get('role')!='user':continue
    text=m.get('content') or ''
    if not isinstance(text,str):text=json.dumps(text,ensure_ascii=False)
    category='human-message-candidate'
    if text.startswith(tuple(automatic_prefixes)):category='system-generated-user-role'
    elif text.startswith('[OUT-OF-BAND USER MESSAGE'):category='out-of-band-user-wrapper'
    h=hashlib.sha256(text.encode()).hexdigest(); duplicate=h in seen;seen.add(h)
    counts[category]=counts.get(category,0)+1
    rows.append({'index':i,'timestamp':m.get('timestamp'),'category':category,'duplicate_exact_text':duplicate,'sha256':h,'preview':text[:240]})
(p/'evidence/session-role-classification.json').write_text(json.dumps({'counts':counts,'rows':rows,'note':'role=user is not sufficient proof of a fresh human instruction. Classification is heuristic; export provenance preserved.'},ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps({'counts':counts,'exact_unique_user_role_texts':len(seen)}))
