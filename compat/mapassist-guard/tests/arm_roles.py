"""Run the built ARM guard against native multiplayer record layouts."""
from pathlib import Path
import struct,sys
from elftools.elf.elffile import ELFFile
from unicorn import Uc,UC_ARCH_ARM,UC_MODE_ARM,UC_HOOK_CODE,UC_HOOK_INTR
from unicorn.arm_const import *
R=Path(__file__).resolve().parents[1]
u=Uc(UC_ARCH_ARM,UC_MODE_ARM);u.ctl_set_cpu_model(UC_CPU_ARM_MAX)
u.reg_write(UC_ARM_REG_C1_C0_2,0xf00000);u.reg_write(UC_ARM_REG_FPEXC,0x40000000)
BASE,MAIN,HEAP,STUB,STACK,STOP,PEER=0x4000000,0x200000,0xc000000,0x3000000,0xa000000,0x3100000,0x3210000
for a,n in [(BASE,0x1000000),(MAIN,0x1b00000),(HEAP,0x10000),(STUB,0x10000),(STACK,0x10000),(STOP,0x1000),(0x3200000,0x60000)]:u.mem_map(a,n)
def put(a,v):u.mem_write(a,struct.pack('<I',v))
def get(a):return struct.unpack('<I',u.mem_read(a,4))[0]
imports={};symbols={}
image=R/'../../mod/target/armv7a-none-eabi/release/libhunters_compendium.so' if '--utility' in sys.argv else R/'target/armv7a-none-eabi/release/libmapassist_role_guard.so'
with image.open('rb')as f:
 e=ELFFile(f)
 for s in e.iter_segments():
  if s['p_type']=='PT_LOAD':u.mem_write(BASE+s['p_vaddr'],s.data())
 dyn=e.get_section_by_name('.dynsym')
 for s in dyn.iter_symbols():
  if not s.name:continue
  if s['st_shndx']=='SHN_UNDEF':
   at=STUB+4*len(imports);imports[at]=s.name;symbols[s.name]=at;put(at,0xe12fff1e)
  else:symbols[s.name]=BASE+s['st_value']
 for section in ['.rel.dyn','.rel.plt']:
  rs=e.get_section_by_name(section)
  if rs is None:continue
  for r in rs.iter_relocations():
   at=BASE+r['r_offset'];t=r['r_info_type'];assert t in (21,22,23),t
   put(at,BASE+get(at)if t==23 else symbols[dyn.get_symbol(r['r_info_sym']).name])
hooks=[];calls=[]
def cb(uc,at,size,ctx):
 if at in [STUB+0x100,STUB+0x104,STUB+0x108]:calls.append(at);uc.reg_write(UC_ARM_REG_R0,1);return
 name=imports.get(at)
 if name=='forge_mem_getMainTextAddr':uc.reg_write(UC_ARM_REG_R0,MAIN)
 elif name=='forge_hook_create':
  original=STUB+0x100+4*len(hooks);put(original,0xe12fff1e);put(uc.reg_read(UC_ARM_REG_R3),original);hooks.append(uc.reg_read(UC_ARM_REG_R1))
  uc.reg_write(UC_ARM_REG_R0,1)
 elif name:uc.reg_write(UC_ARM_REG_R0,0)
u.hook_add(UC_HOOK_CODE,cb,begin=STUB,end=STUB+0xffff)
def intr(uc,no,ctx):
 assert get(uc.reg_read(UC_ARM_REG_PC)-4)==0xef000006
 p=uc.reg_read(UC_ARM_REG_R0);at=uc.reg_read(UC_ARM_REG_R2)
 u.mem_write(p,struct.pack('<QQIIIIII',0x3200000,0x60000,4,0,3,0,0,0));uc.reg_write(UC_ARM_REG_R0,0 if 0x3200000<=at<0x3260000 else 1)
u.hook_add(UC_HOOK_INTR,intr)
def call(name,*args):
 u.reg_write(UC_ARM_REG_SP,STACK+0xf000);u.reg_write(UC_ARM_REG_LR,STOP)
 for r,v in zip([UC_ARM_REG_R0,UC_ARM_REG_R1,UC_ARM_REG_R2,UC_ARM_REG_R3],args):u.reg_write(r,v)
 u.emu_start(symbols[name],STOP,count=10000000);assert u.reg_read(UC_ARM_REG_PC)==STOP,name
 return u.reg_read(UC_ARM_REG_R0)
# Supported binary fingerprint; no original MapAssist code is redistributed.
for off,v in [(0x10,0x304f524e),(0x18,0x12000),(0x3e1c,0xe280c0b4),(0x3e20,0xe16d41f8),(0x3e2c,0xe1a05001),(0x6e98,0xe59f33ac),(0xa820,0xe59fc404),(0xace4,0xe16d41f4)]:put(PEER+off,v)
put(0x819494,0xe92d4ff0);put(0xb1b284,0xe92d4830)
if '--unsupported' in sys.argv:put(PEER+0xace4,0)
call('mapassist_guard_tick',0);call('mapassist_guard_tick',0)
if '--unsupported' in sys.argv:
 assert not hooks,hooks
 print('PASS: unsupported MapAssist version leaves all native and plugin entries untouched');sys.exit()
assert len(hooks)==3,hooks
ui,layout,records,arr,manager=HEAP,HEAP+0x1000,HEAP+0x2000,HEAP+0x3000,HEAP+0x4000
put(ui,0x1974c14);put(ui+0x24,layout);put(ui+0xd8,records);put(layout+0x298,32);put(layout+0xf8,arr);put(0x1a51fe0,manager);put(manager+0x1c,HEAP+0x8000)
for i in range(32):put(ui+0x4c+i*4,0xffffffff)
for i,(tag,kind)in enumerate([(11,1),(13,1),(18,3)]):
 node=HEAP+0x5000+i*0x100;parts=HEAP+0x6000+i*0x20;sprite=HEAP+0x7000+i*0x200
 put(ui+0x4c+i*4,i);put(arr+i*4,node);put(records+0x18+i*32,kind);put(records+0x1c+i*32,0);put(node+4,tag);put(node+0xd8,parts);put(parts+4,sprite)
call('mapassist_guard_update',ui)
for i in (0,1):assert call('mapassist_guard_classify',HEAP+0x7000+i*0x200,HEAP+0x5000+i*0x100)==0
assert call('mapassist_guard_classify',HEAP+0x7400,HEAP+0x5200)==1
# Reused tag, unrelated sprite and stale enemy association cannot pass.
assert call('mapassist_guard_classify',HEAP+0x7000,HEAP+0x5200)==0
put(manager+0x1c,HEAP+0x8100);assert call('mapassist_guard_classify',HEAP+0x7400,HEAP+0x5200)==0
put(manager+0x1c,HEAP+0x8000);call('mapassist_guard_drop',layout);assert call('mapassist_guard_classify',HEAP+0x7400,HEAP+0x5200)==0
call('mapassist_guard_update',ui);assert call('mapassist_guard_classify',HEAP+0x7400,HEAP+0x5200)==1
print('PASS: built ARM guard rejects companion/foreign/stale sprites, preserves enemies, clears and recreates layouts')
