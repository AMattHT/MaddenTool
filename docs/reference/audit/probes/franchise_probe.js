const lib=process.argv[2];const f=process.argv[3];
const FF=require(lib);
(async()=>{
 const t0=Date.now();const ff=await FF.create(f,{keepTablesOpen:true});
 console.log('keys',Object.keys(ff).slice(0,40).join(','));
 console.log('type',ff.gameType, 'gameYear',ff.gameYear,'expected',JSON.stringify(ff.expectedSchemaVersion),'schemaMeta',JSON.stringify(ff.schema&&ff.schema.meta),'isLegacy',ff.isLegacyFile,'packedSize',ff.packedFileContents&&ff.packedFileContents.length,'unpacked',ff.unpackedFileContents&&ff.unpackedFileContents.length,'tables',ff.tables&&ff.tables.length,'ms',Date.now()-t0);
 const pl=ff.getTableByName('Player');await pl.readRecords();console.log('Player records',pl.header.recordCapacity,pl.records.length,'fields',pl.offsetTable.length,'empty',pl.emptyRecords&&pl.emptyRecords.size);
 const names=['Contract','PlayerContract','Team','Coach','DraftPick','Seasoninfo','SeasonInfo','Stadium','FranchiseDebugModule','SeasonGame','Injury'];
 for(const n of names){try{const t=ff.getTableByName(n);if(t){await t.readRecords();console.log(n,t.header.recordCapacity,t.offsetTable.length)}else console.log(n,'-')}catch(e){console.log(n,'err',e.message)}}
 const tn=ff.tables.map(t=>t.name);require('fs').writeFileSync(process.argv[4],JSON.stringify(tn.filter(Boolean)));
 console.log('contract-like tables',[...new Set(tn.filter(n=>/contract|incentive|void|guarant|salary/i.test(n||'')))].slice(0,40).join(','));
 const p=ff.schemaList?ff.schemaList:null;
 const packed=ff.packedFileContents, un=ff.unpackedFileContents;
 console.log('first16',packed.slice(0,16).toString('hex'));
 if(process.argv[5]==='noop'){await ff.save(process.argv[6]);console.log('saved');}
 process.exit(0)
})().catch(e=>{console.error('ERR',e);process.exit(1)});
