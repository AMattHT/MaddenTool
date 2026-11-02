import re,os,json,bisect
root=os.environ['TEMP']+'/maddentool-audit/'
d=root+'mf201/.vite/renderer/main_window/assets/'
r=open(d+'index-CqbtHSxo.js',encoding='utf8',errors='replace').read()
pr=open(d+'ProgressionReport-DEHzOFXM.js',encoding='utf8',errors='replace').read()
inv=json.load(open(os.path.join(os.path.dirname(os.path.abspath(__file__)),'..','..','myfranchise-2.0.1-inventory.json'),encoding='utf8'))
# component defs: var=GE({__name:"X" (GE is defineComponent in index); may have other wrapper
defs=[]
for m in re.finditer(r'([A-Za-z_$][\w$]*)=(?:[A-Za-z_$][\w$]*\()?(?:[A-Za-z_$][\w$]*)\(\{__name:"([A-Za-z0-9_]+)"',r):
    defs.append((m.start(),m.group(1),m.group(2)))
noassign=[m for m in re.finditer(r'__name:"([A-Za-z0-9_]+)"',r)]
print('defs',len(defs),'names',len(noassign), 'unmatched', sorted(set(m.group(1) for m in noassign)-set(x[2] for x in defs)))
for m in noassign:
    if m.group(1) not in set(x[2] for x in defs):
        print(m.group(1),repr(r[m.start()-80:m.start()+30]))
# wrapper exports like X=Pe(Y,[["render",..]]) ignored for now
# Find positions, segments
pos=sorted([(m.start(),m.group(1)) for m in noassign])
starts=[p for p,_ in pos]
var2name={}
for p,v,n in defs: var2name.setdefault(v,[]).append(n)
dup={v:n for v,n in var2name.items() if len(n)>1}
print('dup vars',dup)
# aliases e.g. const Qf=wrapper(xyz)
def seg_end(i):
    # component body ends at next __name start (approx)
    return starts[i+1] if i+1<len(starts) else len(r)
edges={}
allv=set(var2name)
tok=re.compile(r'(?<![\w$.])([A-Za-z_$][\w$]*)(?![\w$])')
use=re.compile(r'(?:\bW|\bNE|\bpS|h|xE|Ne)\(([A-Za-z_$][\w$]*)[,)]|(?:component|is):\s*([A-Za-z_$][\w$]*)')
for i,(p,n) in enumerate(pos):
    seg=r[p:seg_end(i)]
    c=set()
    for m in tok.finditer(seg):
        t=m.group(1)
        if t in allv and len(t)>=2:
            c.update(var2name[t])
    c.discard(n)
    edges[n]=sorted(c)
json.dump({'edges':edges,'var2name':var2name},open(root+'graph.json','w'))
# routes
s=r.find('XQt=[{path:"/",component:Stt}');e=r.find('];',s)
rt=re.findall(r'path:"([^"]+)",component:([\w$]+)',r[s:e])
print(len(rt))
res={}
print('route var -> comp')
routes={}
unres=[]
for path,v in rt:
    if v in var2name: routes[path]=var2name[v][0]
    else:
        m=re.search(r'[,; ]'+re.escape(v)+r'=([^,;]{0,200})',r)
        unres.append((path,v,m.group(1) if m else None))
print(routes);print('unres',unres)
json.dump(routes,open(root+'routes.json','w'))
