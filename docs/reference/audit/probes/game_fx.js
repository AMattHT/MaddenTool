const P=process.argv[2],files=process.argv.slice(3),fs=require('fs');const Parser=require(P+'/M27Parser.js'),Writer=require(P+'/M27Writer.js');
const DS=0x46,BS=5876,M=Parser.M27_FIELDS,R=Parser.M27_RATINGS;const mapped=new Set;
for(const f of Object.values(M)){const n=f.len||(f.u16?2:f.u16x5?10:1);for(let i=0;i<n;i++)mapped.add(f.off+i)}for(const o of Object.values(R))mapped.add(o);
const j=(b,i)=>{const s=DS+i*BS;let e=s;while(e<s+0x1600&&b[e])e++;return{txt:b.toString('utf8',s,e),len:e-s,tail:b.subarray(e,s+0x1600).some(x=>x)}};
const names=[];
for(const f of files){const a=fs.readFileSync(f);
 const hdr={magic:a.toString('latin1',0,8),ver:a.readUInt16LE(8),hs:a.readUInt16LE(10),payload:a.readUInt32LE(0xe),p2:a.readUInt32LE(0x12),ts:[0x16,0x18,0x1a,0x1c,0x1e,0x20].map(o=>a.readUInt16LE(o)).join('-'),product:a.toString('latin1',0x22,0x3e).replace(/\0+/g,''),u3e:a.readUInt32LE(0x3e),count:a.readUInt32LE(0x42)};
 const all=Parser.parseM27Prospects(a,{dataStartOffset:DS});const ps=all.filter(p=>p.firstName);
 let nonzero=0;for(let i=0;i<455;i++){const s=DS+i*BS;if(a.subarray(s,s+BS).some(x=>x))nonzero++}
 const dr=ps.filter(p=>p.draftRound<63).length,un=ps.length-dr;const rounds={};ps.forEach(p=>rounds[p.draftRound]=(rounds[p.draftRound]||0)+1);
 const dna={};ps.forEach(p=>{const n=(p.personaDNA||[]).length;dna[n]=(dna[n]||0)+1});
 const un2=new Map;for(let i=0;i<ps.length;i++){const s=DS+i*BS+0x1600;for(let o=0;o<244;o++)if(a[s+o]&&!mapped.has(o))un2.set(o,(un2.get(o)||0)+1)}
 const k6b=ps.every((p,i)=>a[DS+i*BS+0x1600+0x6b]===127),k7d=ps.every((p,i)=>a[DS+i*BS+0x1600+0x7d]===1);
 let maxj=0,tail=0,bad=0,bt=0,skin=0,asset=0,pid0=0;const assets=new Set,gear=new Set;ps.forEach((p,i)=>{const v=j(a,i);maxj=Math.max(maxj,v.len);if(v.tail)tail++;try{const o=JSON.parse(v.txt);if(o.bodyType)bt++;if(o.skinTone!=null)skin++;(o.loadouts||[]).forEach(l=>(l.loadoutElements||[]).forEach(e=>e.itemAssetName&&gear.add(e.itemAssetName)))}catch{bad++}if(p.assetName)asset++;if(!p.PID)pid0++});
 const b=Writer.writeM27DraftClass(a,ps,{dataStartOffset:DS});let hd=0;for(let i=0;i<DS;i++)if(a[i]!==b[i])hd++;let vb=0,ab=0,offs={};for(let i=0;i<455;i++){const s=DS+i*BS;let v=0,t=0;for(let k=0;k<0x1600;k++)if(a[s+k]!==b[s+k]){v=1;break}for(let k=0x1600;k<BS;k++)if(a[s+k]!==b[s+k]){t=1;const o='0x'+(k-0x1600).toString(16);offs[o]=(offs[o]||0)+1}vb+=v;ab+=t}
 console.log(f.split(/[\/]/).pop(),JSON.stringify(hdr));console.log(' len',a.length,'parsed',ps.length,'nonzeroBlocks',nonzero,'drafted',dr,'undrafted',un,'rounds',JSON.stringify(rounds),'dnaSlots',JSON.stringify(dna));
 console.log(' unmappedNZ',[...un2.entries()].map(([o,c])=>'0x'+o.toString(16)+':'+c).join(' ')||'none','const6b',k6b,'const7d',k7d,'maxJson',maxj,'nonzeroAfterJson',tail,'jsonBad',bad,'bodyTypeKey',bt,'skinToneKey',skin,'assetName',asset,'pid0',pid0,'gearAssets',gear.size);
 console.log(' noop: identical',a.equals(b),'hdrDiff',hd,'visBlocksChanged',vb,'attrBlocksChanged',ab,JSON.stringify(offs));
 names.push(new Set(ps.map(p=>p.firstName+' '+p.lastName)));
 console.log(' sample',ps.slice(0,3).map(p=>[p.firstName,p.lastName,'pos',p.position,'ovr',p.overall,'rd',p.draftRound,'pk',p.draftPick].join(' ')).join(' | '));
}
for(let i=0;i<names.length;i++)for(let k=i+1;k<names.length;k++){let c=0;names[i].forEach(n=>{if(names[k].has(n))c++});console.log('nameOverlap',i+1,k+1,c)}
