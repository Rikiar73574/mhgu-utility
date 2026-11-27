"""Build name-slot identifiers and the standard KS X 1001 Hangul mapping."""
from pathlib import Path
import argparse,struct
from game_names import strings
TABLES=['itemData','armorSeriesData','skillTypeData','skillData']+[f'weapon{i:02d}MsgData' for i in range(15) if i!=5]
def hash_name(s):
    h=0xcbf29ce484222325
    for b in s.encode():h=((h^b)*0x100000001b3)&0xffffffffffffffff
    return h

def generate(table,out):
    rows=[]
    for index,name in enumerate(TABLES):
        texts=strings(table/f'{name}_eng.gmd')
        stride=10 if name=='armorSeriesData' else 6 if name.startswith('weapon') else 2
        for slot,text in enumerate(texts):
            if text and (slot%10<5 if stride==10 else slot%stride==0):rows.append((hash_name(text),index,slot))
    # Identical English names may share slots; retain every mapping so runtime can
    # reject ambiguous translations rather than silently choose the wrong one.
    result='pub const TABLES:&[&str]=&['+','.join('"'+x+'"' for x in TABLES)+'];\n'
    out.parent.mkdir(parents=True,exist_ok=True)
    out.with_suffix('.bin').write_bytes(b'UNAM'+struct.pack('<I',1)+b''.join(struct.pack('<QBH',h,i,s) for h,i,s in rows))
    hangul=[ord(bytes([a,b]).decode('euc_kr')) for a in range(0xb0,0xc9) for b in range(0xa1,0xff)]
    assert len(hangul)==2350 and all(0xac00<=x<=0xd7a3 for x in hangul)
    result+='pub const HANGUL:&[u16]=&['+','.join(map(str,hangul))+'];\n'
    out.parent.mkdir(parents=True,exist_ok=True);out.write_text(result)
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('table',type=Path);p.add_argument('output',type=Path);a=p.parse_args();generate(a.table,a.output)
