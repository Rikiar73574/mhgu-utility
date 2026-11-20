"""Native pad conversion bypasses GOT: game capture must still be exclusive."""
import ast,struct,os,sys,subprocess
from pathlib import Path
source=Path(__file__).with_name('arm_smoke.py');tree=ast.parse(source.read_text());prefix=[]
for node in tree.body:
 if isinstance(node,ast.Expr) and isinstance(node.value,ast.Call) and isinstance(node.value.func,ast.Name) and node.value.func.id=='call':break
 prefix.append(node)
ns={'__file__':str(source)};exec(compile(ast.Module(body=prefix,type_ignores=[]),str(source),'exec'),ns)
u,put,get,call=map(ns.get,['u','put','get','call']);MAIN,STUB,HEAP=map(ns.get,['MAIN','STUB','HEAP'])
from unicorn.arm_const import *
from unicorn import UC_HOOK_CODE
fail_hook=int(os.environ.get('COMPENDIUM_INPUT_FAIL_HOOK','-1'))
entries=[0x86d34c,0x86d588,0x86d7b4,0x86dc98,0x86e1b4,0x3ddf1c];hooks={};low=high=0;axes=(0,0,0,0)
originals={STUB+0x8000+i*4 for i in range(6)}
for p in originals:u.mem_write(p,bytes.fromhex('1eff2fe1'))
def install(uc,at,size,ctx):
 if ns['imports'].get(at)!='forge_hook_create':return False
 target=uc.reg_read(UC_ARM_REG_R1)
 if target-MAIN not in entries:return False
 i=entries.index(target-MAIN)
 if i==fail_hook:uc.reg_write(UC_ARM_REG_R0,0);return True
 hooks[i]=uc.reg_read(UC_ARM_REG_R2)
 put(uc.reg_read(UC_ARM_REG_R3),STUB+0x8000+i*4);uc.reg_write(UC_ARM_REG_R0,uc.reg_read(UC_ARM_REG_R0));return True
ns['storage_callback']=install
def native(uc,at,size,ctx):
 if at not in originals:return
 if at==STUB+0x8014:
  obj=uc.reg_read(UC_ARM_REG_R0)
  for off,size in [(0x6b9c,16),(0x6bc0,28),(0x6bf4,16),(0x6c18,28),(0x6c34,16)]:uc.mem_write(obj+off,b'\xff'*size)
  return
 out=uc.reg_read(UC_ARM_REG_R3)
 # Deliberately bypass the patched HID import, as the native recomp can do.
 uc.mem_write(out,struct.pack('<IIiiiiH',low,high,*axes,0xffff))
u.hook_add(UC_HOOK_CODE,native,begin=STUB+0x8000,end=STUB+0x8018)
# KatLib owns the fullkey entry before utility loads. Standard Forge hooks
# must chain this entry, rather than rejecting it as an incompatible game.
put(MAIN+entries[0],0xe51ff004);put(MAIN+entries[0]+4,STUB+0x8800)
call(ns['symbols']['forge_onInit']);assert len(hooks)==(fail_hook if fail_hook>=0 else 6),hooks
out=HEAP+0x4000;padid=HEAP+0x4100;put(padid,0)
def poll(a=0,b=0,stick=(0,0,0,0),style=0):
 global low,high,axes
 low,high,axes=a,b,stick;u.mem_write(out,b'\xcc'*32)
 call(hooks[style],HEAP+0x4200,0,padid,out)
 assert bytes(u.mem_read(out+26,6))==b'\xcc'*6,'overwrote native output boundary'
 return bytes(u.mem_read(out,26))
if fail_hook>=0:
 # An installed prefix of native hooks must remain a transparent chain if a
 # later hook failed. The chord must not silently capture a disabled utility.
 if hooks:
  for b in [0x88,0xffffffff,0,8,0,1]:
   actual=poll(b,0xffffffff,(32767,-32767,12000,-12000))
   assert actual==struct.pack('<IIiiiiH',b,0xffffffff,32767,-32767,12000,-12000,0xffff),('partial install captured input',fail_hook,b,actual.hex())
 ns['raw']=(1<<10)|(1<<12);ns['texts'].clear()
 call(ns['symbols']['forge_onImGuiFreeRender'])
 assert not ns['texts'],'partially installed utility became ready'
 # No HID slots should be published before all native hooks have succeeded.
 assert all(get(slot) in ns['imports'] and ns['imports'][get(slot)]=='HID' for slot in ns['slots'])
 print('Partial native install failure passes original input:',fail_hook)
 sys.exit(0)
processed=HEAP+0x4400
ranges=[(0x6b9c,16),(0x6bc0,28),(0x6bf4,16),(0x6c18,28),(0x6c34,16)]
def process():
 u.mem_write(processed,b'\xcc'*0x7000)
 call(hooks[5],processed)
 for start,end in [(0x6b98,0x6b9c),(0x6bac,0x6bc0),(0x6bf0,0x6bf4),(0x6c04,0x6c18),(0x6c44,0x6c48)]:
  assert bytes(u.mem_read(processed+start,end-start))==b'\xcc'*(end-start),'mapper metadata overwritten'
 return b''.join(bytes(u.mem_read(processed+off,size))for off,size in ranges)
assert process()==b'\xff'*sum(size for _,size in ranges),'closed utility blocked mapped pad'
assert struct.unpack_from('<I',poll(1))[0]==1,'passive utility blocked gameplay' 
assert struct.unpack_from('<I',poll(8))[0]==0,'Start leaked before chord could form'
assert poll(0x88)==bytes(26),'opening chord leaked'
assert process()==bytes(sum(size for _,size in ranges)),'processed menu buttons, repeats or sticks leaked'
# Keyboard WASD and gamepad sticks converge on this same native pad output.
assert poll(0xf0000000,0,(32767,32767,0,0))==bytes(26),'keyboard-mapped movement leaked'
ns['raw']=1<<0;call(ns['symbols']['forge_input_isDown'],0)
assert u.reg_read(UC_ARM_REG_R0)==1,'utility lost the unfiltered Forge input'
ns['raw']=0
for style in range(5):
 for bit in range(64):
  assert poll(1<<bit if bit<32 else 0,1<<(bit-32) if bit>=32 else 0,(32767,-32767,9000,-9000),style)==bytes(26),(style,bit)
 assert poll(0,0,(1000,-2000,3000,-4000),style)==bytes(26),'sticks leaked with no buttons'
# Closing chord and held actions drain until every action releases.
poll();assert poll(0x88)==bytes(26);assert poll(1)==bytes(26)
assert poll(0,0x80000000)==bytes(26),'upper-word button escaped drain'
assert poll()==bytes(26)
assert struct.unpack_from('<I',poll(1))[0]==1,'gameplay did not resume'
assert struct.unpack_from('<iiii',poll(0,0,(1000,-2000,3000,-4000)),8)==(1000,-2000,3000,-4000)
# Normal native Start is replayed on release, while other inputs survive.
assert struct.unpack_from('<I',poll(8|1))[0]==1
assert struct.unpack_from('<I',poll())[0]==8
assert struct.unpack_from('<I',poll())[0]==0
print('Native input capture passed: all 64 button bits, five styles, sticks, chord/release drain and passive gameplay')

for failed in range(6):
 env=dict(os.environ,COMPENDIUM_INPUT_FAIL_HOOK=str(failed))
 subprocess.run([sys.executable,__file__],env=env,check=True)
