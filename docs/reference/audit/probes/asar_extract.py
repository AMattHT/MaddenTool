# read-only ASAR extractor: asar_extract.py <asar> <outdir> [--list] ; skips unpacked files & binary-heavy assets when --text-only
import sys,struct,json,os
asar,out=sys.argv[1],sys.argv[2]
textonly='--text-only' in sys.argv
TEXT=('.js','.mjs','.cjs','.json','.html','.css','.map','.txt','.md','.ftx','.xml','.license','.yml','.yaml','.sql')
f=open(asar,'rb')
hdr=f.read(16)
size=struct.unpack('<I',hdr[4:8])[0]
jlen=struct.unpack('<I',hdr[12:16])[0]
head=json.loads(f.read(jlen).decode())
base=8+size
listing=[]
def walk(n,p):
    for k,v in n['files'].items():
        q=p+'/'+k if p else k
        if 'files' in v: walk(v,q)
        else:
            listing.append((q,int(v.get('size',0)),bool(v.get('unpacked')),v.get('offset')))
walk(head,'')
os.makedirs(out,exist_ok=True)
with open(os.path.join(out,'_listing.tsv'),'w',encoding='utf8') as L:
    for q,s,u,o in listing: L.write(f"{q}\t{s}\t{u}\n")
n=0
for q,s,u,o in listing:
    if u: continue
    if textonly and not q.lower().endswith(TEXT): continue
    if s>80_000_000: continue
    t=os.path.join(out,q.replace('/',os.sep))
    os.makedirs(os.path.dirname(t),exist_ok=True)
    f.seek(base+int(o)); open(t,'wb').write(f.read(s)); n+=1
print(len(listing),'entries; extracted',n)
