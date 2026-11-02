const P=process.argv[2], tpl=process.argv[3], extra=process.argv.slice(4);
const fs=require('fs');const Parser=require(P+'/M27Parser.js');const Writer=require(P+'/M27Writer.js');
const DS=0x46,BS=5876;
function diff(path){
 const a=fs.readFileSync(path);const ps=Parser.parseM27Prospects(a,{dataStartOffset:DS}).filter(p=>p.firstName);
 const b=Writer.writeM27DraftClass(a,ps,{dataStartOffset:DS});
 let hdr=0,attr=new Map,visBlocks=0,attrBlocks=0,zeroTail=0;
 for(let i=0;i<DS;i++) if(a[i]!==b[i]) hdr++;
 for(let i=0;i<455;i++){const s=DS+i*BS;let v=false,at=false;for(let k=0;k<0x1600;k++) if(a[s+k]!==b[s+k]){v=true;break}
  for(let k=0x1600;k<BS;k++) if(a[s+k]!==b[s+k]){at=true;const o=k-0x1600;attr.set(o,(attr.get(o)||0)+1)}
  if(v)visBlocks++;if(at)attrBlocks++}
 const offs=[...attr.entries()].sort((x,y)=>x[0]-y[0]).map(([o,c])=>'0x'+o.toString(16)+':'+c);
 console.log(path.split(/[\/]/).pop(),'prospects',ps.length,'identical',a.equals(b),'hdrDiff',hdr,'visualBlocksChanged',visBlocks,'attrBlocksChanged',attrBlocks,'attrOffsets',offs.slice(0,50).join(' '));
 return {a,ps};
}
const {a,ps}=diff(tpl);for(const e of extra)diff(e);
// byte coverage on game-generated template
const M=Parser.M27_FIELDS,R=Parser.M27_RATINGS;const mapped=new Set;
for(const f of Object.values(M)){const n=f.len||(f.u16?2:f.u16x5?10:1);for(let i=0;i<n;i++)mapped.add(f.off+i)}
for(const o of Object.values(R))mapped.add(o);
const nz=new Map,vals=new Map;
for(let i=0;i<ps.length;i++){const s=DS+i*BS+0x1600;for(let o=0;o<244;o++){const v=a[s+o];if(v){nz.set(o,(nz.get(o)||0)+1)};if(!vals.has(o))vals.set(o,new Set);vals.get(o).add(v)}}
const un=[...nz.keys()].filter(o=>!mapped.has(o)).sort((x,y)=>x-y).map(o=>'0x'+o.toString(16)+'(nz='+nz.get(o)+',distinct='+vals.get(o).size+')');
console.log('mapped',mapped.size,'of 244; unmapped nonzero in game template:',un.join(' ')||'none');
// visual json keys + region
let maxj=0,keys=new Map;for(let i=0;i<ps.length;i++){const s=DS+i*BS;let e=s;while(e<s+0x1600&&a[e]!==0)e++;maxj=Math.max(maxj,e-s);try{const j=JSON.parse(a.toString('utf8',s,e));for(const k of Object.keys(j))keys.set(k,(keys.get(k)||0)+1);}catch(x){keys.set('PARSEFAIL',(keys.get('PARSEFAIL')||0)+1)}
 let tail=false;for(let k=e;k<s+0x1600;k++)if(a[k]){tail=true;break} if(tail)keys.set('NONZERO_AFTER_JSON',(keys.get('NONZERO_AFTER_JSON')||0)+1)}
console.log('visual keys',[...keys.entries()].join(' '),'maxJson',maxj);
// duplicates of block tails
