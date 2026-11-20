//! Capture at the game's native pad conversion boundary as well as its HID imports.
//! Other Forge mods can supply native pad state without visiting the main HID GOT.
//! Forge still reads raw controller state directly from the host.
//! Both HID calls and drawing execute on the recomp's game thread.
use core::{cell::UnsafeCell,ffi::c_void,ptr};
use crate::control::Gate;
struct Local<T>(UnsafeCell<T>);unsafe impl<T> Sync for Local<T>{}
static GATE:Local<Gate>=Local(UnsafeCell::new(Gate::new()));
static ORIGINAL:Local<[usize;5]>=Local(UnsafeCell::new([0;5]));
static POLLS:Local<u32>=Local(UnsafeCell::new(0));
static READY:core::sync::atomic::AtomicBool=core::sync::atomic::AtomicBool::new(false);
pub fn ready()->bool{READY.load(core::sync::atomic::Ordering::Relaxed)}
pub fn open()->bool{unsafe{(*GATE.0.get()).open}}
pub fn show(){unsafe{(*GATE.0.get()).open=true;}}
pub fn toggle(){unsafe{let g=&mut *GATE.0.get();if g.open{g.close();}else{g.open=true;}}}
pub fn close(){unsafe{(*GATE.0.get()).close();}}
unsafe fn run(i:usize,s:*mut u8,id:*const u32){unsafe{
 let orig=(*ORIGINAL.0.get())[i];
 if orig==0{return;}
 let f:unsafe extern "C" fn(*mut u8,*const u32)=core::mem::transmute(orig);f(s,id);
 if !ready()||s.is_null()||id.is_null()||ptr::read_unaligned(id)!=0{return;}
 *POLLS.0.get()=(*POLLS.0.get()).wrapping_add(1);
 let b=ptr::read_unaligned(s.add(8).cast::<u32>());
 let gate=&mut *GATE.0.get();let was_capture=gate.capturing();let out=gate.filter(b);
 ptr::write_unaligned(s.add(8).cast::<u32>(),out);
 // Clear both sticks, high button bits and every action while capturing/draining.
 if was_capture||gate.capturing() {ptr::write_bytes(s.add(12),0,20);}
}}
unsafe extern "C" fn full(s:*mut u8,id:*const u32){unsafe{run(0,s,id)}}
unsafe extern "C" fn dual(s:*mut u8,id:*const u32){unsafe{run(1,s,id)}}
unsafe extern "C" fn left(s:*mut u8,id:*const u32){unsafe{run(2,s,id)}}
unsafe extern "C" fn right(s:*mut u8,id:*const u32){unsafe{run(3,s,id)}}
unsafe extern "C" fn handheld(s:*mut u8,id:*const u32){unsafe{run(4,s,id)}}
const NAMES:[&[u8];5]=[b"_ZN2nn3hid12GetNpadStateEPNS0_16NpadFullKeyStateERKj",b"_ZN2nn3hid12GetNpadStateEPNS0_16NpadJoyDualStateERKj",b"_ZN2nn3hid12GetNpadStateEPNS0_16NpadJoyLeftStateERKj",b"_ZN2nn3hid12GetNpadStateEPNS0_17NpadJoyRightStateERKj",b"_ZN2nn3hid12GetNpadStateEPNS0_17NpadHandheldStateERKj"];
// Native 1.4.0 pad converters: (receiver, unused, NpadId const&, pad output).
// All five write buttons at +0/+4 and sticks at +8..+0x14, plus a halfword
// at +0x18. Connection metadata lives on the receiver and stays untouched.
#[repr(C)]struct Hook([u32;19]);
static mut HOOKS:[Hook;6]=[const{Hook([0;19])};6];
static mut NATIVE:[u32;6]=[0;6];
unsafe extern "C"{#[link_name="forge_hook_create"]fn hook(out:*mut Hook,target:u32,detour:u32,original:*mut u32)->*mut Hook;}
unsafe fn native(i:usize,object:u32,arg:u32,id:*const u32,out:*mut u8){unsafe{
 let before=*POLLS.0.get();let was_capture=(*GATE.0.get()).capturing();
 let f:unsafe extern "C" fn(u32,u32,*const u32,*mut u8)=core::mem::transmute(NATIVE[i]);
 f(object,arg,id,out);
 // A partially installed chain must never capture input for a disabled UI.
 if !ready()||out.is_null(){return;}
 // An earlier native hook can bypass the GOT. Filter its already-converted
 // buttons here, once, using the native Start/Left mapping (bits 3 and 7).
 // The remaining bits only signal that a physical action is still held.
 if *POLLS.0.get()==before&&!id.is_null()&&ptr::read_unaligned(id)==0{
  let b=ptr::read_unaligned(out.cast::<u32>());let hi=ptr::read_unaligned(out.add(4).cast::<u32>());
  let raw=crate::control::native_buttons(b,hi);
  let filtered=(*GATE.0.get()).filter(raw);
  ptr::write_unaligned(out.cast::<u32>(),(b&!8)|if filtered&crate::control::START!=0{8}else{0});
 }
 if was_capture||(*GATE.0.get()).capturing(){ptr::write_bytes(out,0,0x1a);}
}}
unsafe extern "C" fn native_full(a:u32,b:u32,c:*const u32,d:*mut u8){unsafe{native(0,a,b,c,d)}}
unsafe extern "C" fn native_dual(a:u32,b:u32,c:*const u32,d:*mut u8){unsafe{native(1,a,b,c,d)}}
unsafe extern "C" fn native_left(a:u32,b:u32,c:*const u32,d:*mut u8){unsafe{native(2,a,b,c,d)}}
unsafe extern "C" fn native_right(a:u32,b:u32,c:*const u32,d:*mut u8){unsafe{native(3,a,b,c,d)}}
unsafe extern "C" fn native_handheld(a:u32,b:u32,c:*const u32,d:*mut u8){unsafe{native(4,a,b,c,d)}}
// The game's sPad remaps the platform devices into separate gameplay and
// menu states, with its own repeat masks. Capture after that final update.
unsafe extern "C" fn processed(object:u32){unsafe{
 let original:unsafe extern "C" fn(u32)=core::mem::transmute(NATIVE[5]);
 let capture=ready()&&(*GATE.0.get()).capturing();original(object);
 if object!=0&&ready()&&(capture||(*GATE.0.get()).capturing()){
  for off in [0x6b98,0x6bf0]{
   // Keep each mapper's vtable and mapping configuration intact.
   ptr::write_bytes((object+off+4)as *mut u8,0,16);
   ptr::write_bytes((object+off+0x28)as *mut u8,0,0x1c);
  }
  ptr::write_bytes((object+0x6c34)as *mut u8,0,16);
 }
}}
// Known 1.4.0 MOD0 location is checked before dereferencing its dynamic tables.
pub fn install()->bool{unsafe{
 let base=forge::mem::text_addr();
 let read=|off:u32|ptr::read_unaligned((base+off) as *const u32);
 let m=read(4);if m>0x2000000||read(m)!=0x30444f4d{return false;}
 let dynamic=m.wrapping_add(read(m+4));if dynamic>0x2100000{return false;}
 let(mut sym,mut strings,mut rel,mut size)=(0,0,0,0);
 for i in 0..128 {let tag=read(dynamic+i*8);let v=read(dynamic+i*8+4);match tag{0=>break,5=>strings=v,6=>sym=v,23=>rel=v,2=>size=v,_=>{}}}
 if sym==0||strings==0||rel==0||size==0||size>0x10000||[sym,strings,rel+size].iter().any(|v|*v>0x2100000){return false;}
 let mut slots=[0u32;5];
 for i in 0..size/8 {let off=read(rel+i*8);let info=read(rel+i*8+4);if info&255!=22{continue;}
  let so=sym+(info>>8)*16;if so>0x2100000-16{continue;}let name=strings+read(so);if name>0x2100000-100{continue;}
  for j in 0..5 {let n=NAMES[j];let p=(base+name) as *const u8;if core::slice::from_raw_parts(p,n.len())==n&&*p.add(n.len())==0 {if off<0x2100000&&off%4==0{slots[j]=off;}}}
 }
 if slots.iter().any(|x|*x==0){return false;}
 for j in 0..5 {let original=read(slots[j]);if original==0{return false;}(*ORIGINAL.0.get())[j]=original as usize;}
 let offsets=[0x86d34c,0x86d588,0x86d7b4,0x86dc98,0x86e1b4];
 // KatLib and other Forge mods can already own this entry. Forge chains
 // those hooks through our original trampoline. Verify the unchanged body
 // as well as a stock prologue or the standard Forge entry jump.
 if offsets.iter().any(|&at|{
  let first=read(at);let second=read(at+4);
  let entry=(first==0xe92d48f0&&second==0xe24dd028)||(first==0xe51ff004&&second!=0&&second&3==0);
  !entry||read(at+8)!=0xe1a0700d||read(at+12)!=0xe1a06000
 }){return false;}
 // Native mapped-pad update. Its gameplay and menu mappers have separate
 // action masks; neither controller conversion alone covers both.
 if read(0x3ddf1c)!=0xe92d4070||read(0x3ddf1c+4)!=0xe1a04000{return false;}
 let native_detours=[native_full as *const c_void,native_dual as *const c_void,native_left as *const c_void,native_right as *const c_void,native_handheld as *const c_void];
 for j in 0..5{if hook(&raw mut HOOKS[j],base+offsets[j],native_detours[j]as u32,&raw mut NATIVE[j]).is_null()||NATIVE[j]==0{return false;}}
 if hook(&raw mut HOOKS[5],base+0x3ddf1c,processed as *const c_void as u32,&raw mut NATIVE[5]).is_null()||NATIVE[5]==0{return false;}
 let detours=[full as *const c_void,dual as *const c_void,left as *const c_void,right as *const c_void,handheld as *const c_void];
 for j in 0..5 {ptr::write_volatile((base+slots[j]) as *mut u32,detours[j] as u32);}
 READY.store(true,core::sync::atomic::Ordering::Relaxed);true
}}
