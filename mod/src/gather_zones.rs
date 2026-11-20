//! Static source locations change only with the quest map/rank, never per frame.
use core::cell::UnsafeCell;
struct Cache{key:Option<(usize,u8)>,zones:[u32;crate::catalog::ENTRIES.len()]}
struct Local(UnsafeCell<Cache>);unsafe impl Sync for Local{}
static CACHE:Local=Local(UnsafeCell::new(Cache{key:None,zones:[0;crate::catalog::ENTRIES.len()]}));
pub fn zones(entry:u16,loc:usize,rank:u8)->u32{unsafe{
 let c=&mut*CACHE.0.get();if c.key!=Some((loc,rank)){
  c.zones.fill(0);
  for r in crate::world::GATHER{if !r.supply&&r.loc as usize==loc&&r.rank==rank&&r.zone<32{if let Some(mask)=c.zones.get_mut(r.entry as usize){*mask|=1u32<<r.zone;}}}
  c.key=Some((loc,rank));
 }
 c.zones.get(entry as usize).copied().unwrap_or(0)
}}
