#[allow(dead_code)]mod weapon_data{include!("../build/generated/weapon_data.rs");}
#[path="../mod/src/set_player.rs"]mod set_player;
use set_player::*;
use std::{collections::BTreeMap,cell::Cell};
struct Fake{bytes:BTreeMap<u32,u8>,slots:Option<u8>,changed:Cell<bool>}
impl Fake{
 fn put_byte(&mut self,a:u32,v:u8){self.bytes.insert(a,v);}
 fn put_half(&mut self,a:u32,v:u16){for(i,b)in v.to_le_bytes().iter().enumerate(){self.put_byte(a+i as u32,*b);}}
 fn put_word(&mut self,a:u32,v:u32){for(i,b)in v.to_le_bytes().iter().enumerate(){self.put_byte(a+i as u32,*b);}}
 fn record(&mut self,index:u32,ty:u16,id:u16){let p=BOX+0x1c+index*44;self.put_half(p+4,ty);self.put_half(p+6,id);}
 fn charm(&mut self,index:u32,slots:u8,trees:[u8;2],points:[i8;2]){self.record(index,6,1);let p=BOX+0x1c+index*44;self.put_byte(p+0x14,slots);for k in 0..2{self.put_byte(p+0x10+k,trees[k as usize]);self.put_byte(p+0x12+k,points[k as usize]as u8);}}
}
const BOX:u32=0x08000000;const SAVE:u32=0x09000000;const GC:u32=0x0a000000;const MAN:u32=0x0b000000;const ACT:u32=0x0c000000;const PD:u32=0x0d000000;
impl Memory for Fake{
 fn word(&self,a:u32)->Option<u32>{if a==0x1897f78&&self.changed.get(){return Some(0);}Some(u32::from_le_bytes(core::array::from_fn(|i|*self.bytes.get(&(a+i as u32)).unwrap_or(&0))))}
 fn half(&self,a:u32)->Option<u16>{Some(u16::from_le_bytes(core::array::from_fn(|i|*self.bytes.get(&(a+i as u32)).unwrap_or(&0))))}
 fn byte(&self,a:u32)->Option<u8>{Some(*self.bytes.get(&a).unwrap_or(&0))}
 fn equipment_slots(&self,_a:u32)->Option<u8>{self.slots}
}
fn setup()->Fake{let mut m=Fake{bytes:BTreeMap::new(),slots:Some(3),changed:Cell::new(false)};for(a,v)in [(0x1897f78,SAVE),(0x1856fe8,BOX),(0x188ca48,GC),(0x1885b58,MAN),(SAVE,0x176af48),(BOX,0x173e3d0),(MAN+0x1c,ACT),(ACT+0x13c4,PD),(PD+0x14c,14),(SAVE+0x958,3<<21),(SAVE+0x3614,7)]{m.put_word(a,v);}m.put_half(PD+0x554,650);m.put_half(PD+0x18+4,21);m.put_half(PD+0x18+6,1);m}
#[test]fn all_two_thousand_charms_are_preserved(){let mut m=setup();for i in 0..2000{m.charm(i,3,[35,25],[8,-10]);}let mut s=Snapshot::new();assert_eq!(decode(&m,0,&mut s),Ok(()));assert_eq!(s.charm_count,2000);assert_eq!(s.charms[1999].box_index,1999);assert_eq!(s.charms[1999].skills[1],Skill{tree:25,points:-10});assert_eq!(s.gender,Some(0));assert_eq!(s.hunter,Some(0));assert_eq!(s.weapon_slots,Some(3));assert_eq!(s.hr,Some(650));assert_eq!(s.village_stars,None);}
#[test]fn native_armor_identity_uses_actual_id_and_zero_based_slot(){let mut m=setup();for i in 0..5{m.record(i,(i+1)as u16,100+i as u16);m.put_half(BOX+0x1c+i*44+8,999);}let mut s=Snapshot::new();assert!(decode(&m,0,&mut s).is_ok());assert_eq!(s.owned_count,5);for i in 0..5{assert!(s.owns(i,100+i as u16));assert!(!s.owns(i,999));}}
#[test]fn equipment_expansion_capacity_matches_native_formula(){assert_eq!(capacity(0,0),1000);assert_eq!(capacity(3<<21,7),2000);let mut m=setup();m.put_word(SAVE+0x958,0);m.put_word(SAVE+0x3614,0);m.charm(1000,0,[1,0],[1,0]);let mut s=Snapshot::new();assert_eq!(decode(&m,0,&mut s),Err(Status::InvalidData));assert_eq!(s.charm_count,0);}
#[test]fn unknown_and_invalid_data_clear_previous_results(){let mut m=setup();m.charm(0,1,[205,2],[1,-5]);let mut s=Snapshot::new();assert!(decode(&m,0,&mut s).is_ok());m.put_byte(BOX+0x1c+0x14,4);assert_eq!(decode(&m,0,&mut s),Err(Status::InvalidData));assert_eq!(s.charm_count,0);assert_eq!(s.gender,None);m.put_byte(BOX+0x1c+0x14,1);m.slots=None;assert_eq!(decode(&m,0,&mut s),Err(Status::Unavailable));}
#[test]fn unknown_trees_gender_hr_and_prowler_fail_closed(){let mut m=setup();let mut s=Snapshot::new();m.charm(0,1,[206,0],[1,0]);assert_eq!(decode(&m,0,&mut s),Err(Status::InvalidData));m.charm(0,1,[1,0],[1,0]);m.put_byte(PD+0x4d8,2);assert!(decode(&m,0,&mut s).is_err());m.put_byte(PD+0x4d8,1);m.put_half(PD+0x554,0);assert!(decode(&m,0,&mut s).is_err());m.put_half(PD+0x554,2);m.put_word(PD+0x14c,15);assert_eq!(decode(&m,0,&mut s),Err(Status::Unavailable));}
#[test]fn ranged_type_mapping_and_build_guard(){let mut m=setup();let mut s=Snapshot::new();for wt in [4,6,10]{m.put_word(PD+0x14c,wt);m.put_half(PD+0x18+4,(wt+7)as u16);assert!(decode(&m,0,&mut s).is_ok());assert_eq!(s.hunter,Some(1));}m.put_word(BOX,0);assert_eq!(decode(&m,0,&mut s),Err(Status::UnsupportedBuild));}
#[test]fn missing_memory_is_unavailable(){struct Missing;impl Memory for Missing{fn word(&self,_:u32)->Option<u32>{None}fn half(&self,_:u32)->Option<u16>{None}fn byte(&self,_:u32)->Option<u8>{None}}let mut s=Snapshot::new();assert_eq!(decode(&Missing,0,&mut s),Err(Status::Unavailable));}
#[test]fn character_replacement_does_not_publish_partial_snapshot(){struct Race{m:Fake,reads:Cell<usize>}impl Memory for Race{fn word(&self,a:u32)->Option<u32>{if a==0x1897f78{let n=self.reads.get();self.reads.set(n+1);if n>0{return Some(0);}}Memory::word(&self.m,a)}fn half(&self,a:u32)->Option<u16>{Memory::half(&self.m,a)}fn byte(&self,a:u32)->Option<u8>{Memory::byte(&self.m,a)}fn equipment_slots(&self,a:u32)->Option<u8>{self.m.equipment_slots(a)}}let mut m=setup();m.charm(1999,3,[35,25],[8,-10]);let r=Race{m,reads:Cell::new(0)};let mut s=Snapshot::new();assert_eq!(decode(&r,0,&mut s),Err(Status::Changed));assert_eq!(s.charm_count,0);assert_eq!(s.hr,None);}

#[test]fn absent_and_mismatched_equipped_records_do_not_publish_fake_zero_slots(){let mut m=setup();m.slots=Some(0);let mut s=Snapshot::new();assert_eq!(decode(&m,0,&mut s),Ok(()));assert_eq!(s.weapon_slots,Some(0));m.put_half(PD+0x18+6,0);assert_eq!(decode(&m,0,&mut s),Err(Status::InvalidData));assert_eq!(s.weapon_slots,None);m.put_half(PD+0x18+6,1);m.put_half(PD+0x18+4,7);assert_eq!(decode(&m,0,&mut s),Err(Status::InvalidData));m.put_half(PD+0x18+4,0);assert_eq!(decode(&m,0,&mut s),Err(Status::InvalidData));}
#[test]fn inline_weapon_level_row_requires_valid_accessor_and_preserves_zero_slots(){let mut m=setup();let row=0x0e000000;let v=0x1700000;let f=0x100000;m.put_word(row,v);m.put_word(row+4,2);m.put_word(v+0x38,f);m.put_word(f,0xe5d00012);m.put_word(f+4,0xe12fff1e);assert_eq!(decode_weapon_row(&m,0,row),Some(0));m.put_byte(row+0x12,3);assert_eq!(decode_weapon_row(&m,0,row),Some(3));m.put_byte(row+0x12,4);assert_eq!(decode_weapon_row(&m,0,row),None);m.put_byte(row+0x12,0);m.put_word(f,0);assert_eq!(decode_weapon_row(&m,0,row),None);assert_eq!(decode_weapon_row(&m,0,0),None);}

#[test]fn equipped_weapon_identity_is_retained_in_preview_numbering(){
 let mut m=setup();let mut s=Snapshot::new();
 for (native,preview) in [(0,0),(1,2),(2,4),(3,6),(4,12),(6,11),(7,1),(8,8),(9,7),(10,13),(11,3),(12,5),(13,10),(14,9)]{
  m.put_word(PD+0x14c,native);m.put_half(PD+0x18+4,(native as u16+7)|(3<<5));m.put_half(PD+0x18+6,5);
  assert_eq!(decode(&m,0,&mut s),Ok(()));assert_eq!(s.weapon_type,Some(preview));assert_eq!(s.weapon_id,Some(5));assert_eq!(s.weapon_level,Some(4));
 }
 m.put_word(PD+0x14c,5);m.put_half(PD+0x18+4,12);assert_eq!(decode(&m,0,&mut s),Err(Status::Unavailable));assert_eq!(s.weapon_type,None);
}
#[test]fn exact_native_weapon_levels_resolve_element_and_status_without_assuming_raw(){
 for (id,expected) in [(1,(Some(0),Some(0))),(5,(Some(27),Some(0))),(6,(Some(28),Some(0))),(7,(Some(29),Some(0))),(52,(Some(30),Some(0))),(76,(Some(31),Some(0))),(50,(Some(0),Some(1))),(49,(Some(0),Some(2))),(56,(Some(0),Some(3))),(39,(Some(0),Some(92)))]{
  assert_eq!(weapon_elements(0,id,1),expected,"native GS id {id}");
 }
 assert_eq!(weapon_elements(11,118,1),(Some(28),Some(3))); // water/sleep dual blades
 assert_eq!(weapon_elements(1,3,1),(None,None)); // fallback Bla 15 is not a structured match
 assert_eq!(weapon_elements(0,u16::MAX,1),(None,None));
 assert_eq!(weapon_elements(0,5,0),(None,None));assert_eq!(weapon_elements(0,5,32),(None,None));
}
#[test]fn capture_publishes_element_only_for_current_identity_and_invalidates_it(){
 let mut m=setup();m.put_word(PD+0x14c,0);m.put_half(PD+0x18+4,7);m.put_half(PD+0x18+6,5);let mut s=Snapshot::new();
 assert_eq!(decode(&m,0,&mut s),Ok(()));assert_eq!(s.element,Some(27));assert_eq!(s.status_element,Some(0));
 m.put_half(PD+0x18+6,u16::MAX);assert_eq!(decode(&m,0,&mut s),Ok(()));assert_eq!(s.element,None);assert_eq!(s.status_element,None);
 m.put_half(PD+0x18+6,5);assert!(decode(&m,0,&mut s).is_ok());m.slots=None;
 assert_eq!(decode(&m,0,&mut s),Err(Status::Unavailable));assert_eq!((s.weapon_type,s.weapon_id,s.weapon_level,s.element,s.status_element),(None,None,None,None,None));
}
#[test]fn equipped_weapon_upgrade_change_during_capture_clears_identity(){
 struct UpgradeRace{m:Fake,reads:Cell<usize>}
 impl Memory for UpgradeRace{
  fn word(&self,a:u32)->Option<u32>{self.m.word(a)}
  fn half(&self,a:u32)->Option<u16>{if a==PD+0x18+4{let n=self.reads.get();self.reads.set(n+1);if n>0{return self.m.half(a).map(|v|v+32);}}self.m.half(a)}
  fn byte(&self,a:u32)->Option<u8>{self.m.byte(a)}
  fn equipment_slots(&self,a:u32)->Option<u8>{self.m.equipment_slots(a)}
 }
 let r=UpgradeRace{m:setup(),reads:Cell::new(0)};let mut s=Snapshot::new();assert_eq!(decode(&r,0,&mut s),Err(Status::Changed));assert_eq!((s.weapon_type,s.weapon_id,s.weapon_level,s.element,s.status_element),(None,None,None,None,None));
}
