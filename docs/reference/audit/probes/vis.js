const P=process.argv[2],tpl=process.argv[3],fs=require('fs');const Parser=require(P+'/M27Parser.js'),Writer=require(P+'/M27Writer.js');
const a=fs.readFileSync(tpl);const ps=Parser.parseM27Prospects(a,{dataStartOffset:0x46}).filter(p=>p.firstName);const b=Writer.writeM27DraftClass(a,ps,{dataStartOffset:0x46});
function j(buf,i){const s=0x46+i*5876;let e=s;while(e<s+0x1600&&buf[e])e++;return JSON.parse(buf.toString('utf8',s,e))}
let addedBT=0,addedSkin=0,semEq=0,loadoutsLost=0,elLost=0,keyorder=0;const sample=[];
for(let i=0;i<ps.length;i++){const x=j(a,i),y=j(b,i);if(!x.bodyType&&y.bodyType)addedBT++;if(x.skinTone==null&&y.skinTone!=null)addedSkin++;
 const n=o=>JSON.stringify(o);if(n(x)===n(y))semEq++;
 if(x.loadouts.length!==y.loadouts.length)loadoutsLost++;
 const cnt=o=>o.loadouts.reduce((s,l)=>s+(l.loadoutElements||[]).length,0);if(cnt(x)!==cnt(y))elLost++;
 if(sample.length<1&&n(x)!==n(y))sample.push({game:x.loadouts.map(l=>({t:l.loadoutType,c:l.loadoutCategory,n:(l.loadoutElements||[]).length})),out:y.loadouts.map(l=>({t:l.loadoutType,c:l.loadoutCategory,n:(l.loadoutElements||[]).length})),gameKeys:Object.keys(x),outKeys:Object.keys(y),gameBody:x.bodyType,outBody:y.bodyType})}
console.log({n:ps.length,addedBodyType:addedBT,addedSkinTone:addedSkin,identicalJson:semEq,loadoutCountChanged:loadoutsLost,elementCountChanged:elLost});console.log(JSON.stringify(sample[0]));
const bt={};for(let i=0;i<ps.length;i++){const x=j(a,i);const k=(x.bodyType||'none')+'/'+ps[i].bodyTypeId;bt[k]=(bt[k]||0)+1}console.log('bodyType/json vs byte',JSON.stringify(bt));
