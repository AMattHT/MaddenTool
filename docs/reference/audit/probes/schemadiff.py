import gzip,json,os,sys,re
r=os.environ['TEMP']+'/maddentool-audit/mf201/schemas/'
d={}
for v in ('620','640'):
    raw=gzip.open(r+f'M27_{v}_0.gz').read();print(v,len(raw),raw[:120])
    j=json.loads(raw);d[v]=j;print(type(j), list(j.keys())[:6] if isinstance(j,dict) else len(j))
def idx(j):
    sc=j['schemas'] if isinstance(j,dict) and 'schemas' in j else j
    out={}
    for s in sc:
        out[s['name']]=[(a.get('name'),a.get('type') or a.get('_type') or a.get('assetType') or str(sorted(a.keys()))[:40],a.get('default'),None) for a in s.get('attributes',[]) if isinstance(a,dict)]
    return out
a,b=idx(d['620']),idx(d['640'])
print('tables',len(a),len(b),'added',len(set(b)-set(a)),'removed',len(set(a)-set(b)))
ch=[k for k in a if k in b and a[k]!=b[k]]
print('changed',len(ch),ch[:30])
for k in ['Player','PlayerContract','Team','Coach','SeasonInfo']:
    print(k,len(a.get(k,[])),len(b.get(k,[])),'added',[x[0] for x in b.get(k,[]) if x[0] not in [y[0] for y in a.get(k,[])]][:20],'removed',[x[0] for x in a.get(k,[]) if x[0] not in [y[0] for y in b.get(k,[])]][:20])
pc=[x[0]+':'+x[1] for x in a['PlayerContract']];print('PlayerContract 620',pc)
print('BaseContract',[x[0]+':'+x[1] for x in a.get('PlayerBaseContract',[])]);print('ContractIncentive',[x[0]+':'+x[1] for x in a.get('ContractIncentive',[])])
