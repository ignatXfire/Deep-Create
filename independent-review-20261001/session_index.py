from pathlib import Path
import json,re
p=Path(__file__).resolve().parent
s=json.loads((p.parent/'session-20260930.json').read_text(encoding='utf-8-sig'))
users=[]; claims=[]; tools={}
for i,m in enumerate(s['messages']):
    c=m.get('content') or ''
    if not isinstance(c,str): c=json.dumps(c,ensure_ascii=False)
    item={'index':i,'id':m.get('id'),'timestamp':m.get('timestamp'),'text':c}
    if m.get('role')=='user': users.append(item)
    if m.get('role')=='assistant' and not m.get('tool_calls') and re.search(r'готов|passed|процент|[0-9]+%|исправлен|подтвержд|реальн|заверш',c,re.I): claims.append(item)
    if m.get('role')=='tool': tools[m.get('tool_name') or '?']=tools.get(m.get('tool_name') or '?',0)+1
(p/'evidence/session-user-messages.json').write_text(json.dumps(users,ensure_ascii=False,indent=2),encoding='utf-8')
(p/'evidence/session-claims.json').write_text(json.dumps(claims,ensure_ascii=False,indent=2),encoding='utf-8')
(p/'evidence/session-index.json').write_text(json.dumps({'user_messages':len(users),'candidate_assertions':len(claims),'tools':tools,'note':'Assertions are search candidates, NOT accepted facts.'},ensure_ascii=False,indent=2),encoding='utf-8')
print('USER MESSAGES (previews; full text in evidence)')
for x in users: print(str(x['index'])+' '+x['text'].replace('\n',' ')[:210])
print('LAST ASSERTIONS (not verified)')
for x in claims[-8:]:print(str(x['index'])+' '+x['text'].replace('\n',' ')[:550])
