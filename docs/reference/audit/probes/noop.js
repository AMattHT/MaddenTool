const lib=process.argv[2],f=process.argv[3],o=process.argv[4];const fs=require('fs'),zlib=require('zlib');const FF=require(lib);
(async()=>{const orig=fs.readFileSync(f);
 const ff=await FF.create(o,{keepTablesOpen:false,schemaOverride:undefined});
 const u0=Buffer.from(ff.unpackedFileContents);
 await ff.save(o+'.saved');
 const s=fs.readFileSync(o+'.saved');
 console.log('orig',orig.length,'saved',s.length,'identical',orig.equals(s));
 let hd=0;for(let i=0;i<Math.min(0x100,s.length);i++)if(orig[i]!==s[i])hd++;console.log('hdrdiff first 256',hd);
 const f2=await FF.create(o+'.saved',{});const u1=Buffer.from(f2.unpackedFileContents);console.log('unpacked lens',u0.length,u1.length,'unpackedIdentical',u0.equals(u1));
 let d=0,first=-1;const n=Math.min(u0.length,u1.length);for(let i=0;i<n;i++)if(u0[i]!==u1[i]){d++;if(first<0)first=i}console.log('unpacked bytes diff',d,'first',first);
 console.log('head orig',orig.slice(0,0x80).toString('latin1').replace(/[^\x20-\x7e]/g,'.'));
 console.log('head hex',orig.slice(0,0x60).toString('hex'));
 process.exit(0)})().catch(e=>{console.error(e);process.exit(1)})
