import re,os,json,bisect
root=os.environ['TEMP']+'/maddentool-audit/'
d=root+'mf201/.vite/renderer/main_window/assets/'
r=open(d+'index-CqbtHSxo.js',encoding='utf8',errors='replace').read()
pr=open(d+'ProgressionReport-DEHzOFXM.js',encoding='utf8',errors='replace').read()
g=json.load(open(root+'graph.json'));routes=json.load(open(root+'routes.json'))
edges=g['edges']
inv=json.load(open(os.path.join(os.path.dirname(os.path.abspath(__file__)),'..','..','myfranchise-2.0.1-inventory.json'),encoding='utf8'))
# reach
reach={}
def bfs(s):
    seen={s};st=[s]
    while st:
        x=st.pop()
        for y in edges.get(x,[]):
            if y not in seen: seen.add(y);st.append(y)
    return seen
owners={}
for p,c in routes.items():
    for n in bfs(c): owners.setdefault(n,set()).add(p)
app=bfs('App')
for n in app: owners.setdefault(n,set()).add('<App>')
prn=set(re.findall(r'__name:"([A-Za-z0-9_]+)"',pr))
print('progression chunk names',prn)
allc=set(inv['vue_components'])
print('unreachable (no route/App)',sorted(allc-set(owners)-prn))
# who refs each (direct parents)
par={}
for k,v in edges.items():
    for x in v: par.setdefault(x,set()).add(k)
print('no direct parents',sorted(c for c in allc if c not in par and c not in routes.values() and c not in prn))
json.dump({k:sorted(v) for k,v in par.items()},open(root+'parents.json','w'))
json.dump({k:sorted(v) for k,v in owners.items()},open(root+'owners.json','w'))
# IPC callers
stores=[(m.start(),'store:'+m.group(1)) for m in re.finditer(r'WS\("([\w-]+)"',r)]
names=[(m.start(),m.group(1)) for m in re.finditer(r'__name:"([A-Za-z0-9_]+)"',r)]
marks=sorted(stores+names)
pos=[p for p,_ in marks]
def enc(i):
    k=bisect.bisect_right(pos,i)-1
    return marks[k][1] if k>=0 else '<module>'
calls={}
for m in re.finditer(r'electronAPI\??\.?(?:\?\.)?(\w+)\??\.(\w+)',r):
    calls.setdefault(m.group(1)+'.'+m.group(2),[]).append(enc(m.start()))
for m in re.finditer(r'electronAPI\??\.?(?:\?\.)?(\w+)\??\.(\w+)',pr):
    calls.setdefault(m.group(1)+'.'+m.group(2),[]).append('ProgressionReport')
for k in sorted(calls): print(k,len(calls[k]),sorted(set(calls[k]))[:8])
json.dump(calls,open(root+'ipc_callers.json','w'))
# preload methods vs used
pre=open(root+'mf201/.vite/build/preload.js',encoding='utf8').read()
print('tools ids', re.search(r'var OS=\(E=>\((.{0,1600}?)\)\)\(OS',r).group(1)[:1600] if re.search(r'var OS=\(E=>\((.{0,1600}?)\)\)\(OS',r) else None)
# FAQ strings size
faqs=re.findall(r'\{summary:"((?:[^"\]|\.)*)",faqs:\[(.*?)\]\}',r)
print('faq blocks',len(faqs),sum(len(a)+len(b) for a,b in faqs))
with open(root+'faqs.txt','w',encoding='utf8') as f:
    for a,b in faqs:
        f.write('SUMMARY: '+a+'\n')
        for q,an in re.findall(r'question:(?:"|`)(.*?)(?:"|`),answer:(?:"|`)(.*?)(?:"|`)\}',b,flags=re.S):
            f.write(' Q: '+q+'\n A: '+an+'\n')
        f.write('\n')
