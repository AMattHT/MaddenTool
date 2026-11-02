const lib=process.argv[2],o=process.argv[3];const fs=require('fs');const FF=require(lib);
(async()=>{
 fs.copyFileSync(process.argv[4],o);
 const ff=await FF.create(o,{keepTablesOpen:true});
 const u0=Buffer.from(ff.unpackedFileContents);const orig=fs.readFileSync(o);
 const t=ff.getTableByName('Player');await t.readRecords();
 const rec=t.records.find(r=>!r.isEmpty);const old=rec.Weight;rec.Weight=old===100?101:old+1;
 await ff.save(o+'.edit');
 const s=fs.readFileSync(o+'.edit');let hdr=0;for(let i=0;i<0x48;i++)if(orig[i]!==s[i])hdr++;
 const f2=await FF.create(o+'.edit',{keepTablesOpen:true});const u1=Buffer.from(f2.unpackedFileContents);let d=0,fi=-1,li=-1;for(let i=0;i<Math.min(u0.length,u1.length);i++)if(u0[i]!==u1[i]){d++;if(fi<0)fi=i;li=i}
 console.log('edit weight',old,'-> savedLen',s.length,'origLen',orig.length,'headerbytes changed',hdr,'unpackedLen',u0.length,u1.length,'unpackedDiff',d,'range',fi,li);
 const t2=f2.getTableByName('Player');await t2.readRecords();const r2=t2.records[rec.index];console.log('reopen',r2.Weight);
 process.exit(0)})().catch(e=>{console.error(e);process.exit(1)})
