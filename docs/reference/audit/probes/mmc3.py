import os,re
# MMC_EDITOR_DIR: the local MMC_Editor folder, with a trailing slash; never committed.
R=os.environ['MMC_EDITOR_DIR']
def strs(p):
    b=open(p,'rb').read()
    a=[x.decode('latin1') for x in re.findall(rb'[\x20-\x7e]{4,300}',b)]
    u=[x.decode('utf-16le') for x in re.findall(rb'(?:[\x20-\x7e]\x00){4,300}',b)]
    return a,u
a,u=strs(R+'Plugins/PortraitManagerPlugin.dll')
print('UTF16 strings in PortraitManagerPlugin:',len(u));
for s in u:
    if re.search(r'portrait|plpo|dds|import|export|folder|id|format|srgb|bc7|r8g8|size|512|256|game|madden|profile',s,re.I): print(' ',s[:200])
print('--- meta names'); print([s for s in a if re.search(r'Import|Export|Portrait|Dds|DDS|Plpo|Bc7|Srgb',s)][:80])
b,c=strs(R+'MMCEditor.exe')
print('--- MMC changelog lines'); 
for s in c:
    if s.startswith('- ') and re.search(r'portrait|fbmod|mod manager|security|command|madden 27',s,re.I): print(' ',s[:220])
print([s for s in c if re.search(r'^v?1\.1\.0|version',s,re.I)][:12])
print('--- args'); 
for n in ['MMCEditor.exe','FrostyCore.dll','FrostyModSupport.dll','../MMC_ModManager_v1.1.0.7/MMCModManager.exe']:
    x,y=strs(R+n); print(n,[s for s in x+y if re.search(r'GetCommandLineArgs|-launch|Command line|/launch|-modpack|-pack|-noupdate|-mod\b',s)][:15])
x,y=strs(R+'FrostyCore.dll'); print('fbmod magic?',[s for s in x+y if re.search(r'FBMOD|FROSTY|plpo_id|assets/added',s)][:20])
x,y=strs(R+'Plugins/PortraitManagerPlugin.dll'); print('plpo',[s for s in x+y if re.search(r'plpo|assets/added|added',s,re.I)][:30])
