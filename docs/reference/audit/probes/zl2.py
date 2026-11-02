import zlib,os,struct
p=os.environ['TEMP']+'/maddentool-audit/fx/'
b=open(p+'career-m27.fixture','rb').read(); e=open(p+'edit-copy.edit','rb').read()
off=b.find(b'\x78\x9c',0x20);print('zlib off',hex(off),'pre',b[0x38:off].hex())
d=zlib.decompressobj();u=d.decompress(b[off:]);tail=d.unused_data
print('unpacked',len(u),'compressed',len(b)-off-len(tail),'tail',len(tail),'tail nonzero',sum(1 for x in tail if x),'tail head',tail[:24].hex())
print('u32 at 0x40..',[hex(struct.unpack('<I',b[i:i+4])[0]) for i in range(0x38,off,4)])
for lvl in (1,3,5,6,7,8,9):
    c=zlib.compress(u,lvl);print(' lvl',lvl,len(c),'matches',b[off:off+len(c)]==c)
offe=e.find(b'\x78\x9c',0x20);d=zlib.decompressobj();ue=d.decompress(e[offe:]);te=d.unused_data
print('edit compressed',len(e)-offe-len(te),'tail',len(te),'hdr diff',[ (hex(i),b[i],e[i]) for i in range(offe) if b[i]!=e[i]])
print('unpacked head',u[:48].hex(),'contains FTC',u.find(b'FTC'), u[:4])
