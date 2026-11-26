//! Read-only progression availability. Native creation conditions are opaque
//! quest/progress flags, never rarity or HR. Material *sources* are tested;
//! current item quantities and zeny are never inspected.
pub trait Memory{fn word(&self,address:u32)->Option<u32>;fn byte(&self,address:u32)->Option<u8>;}
fn valid(p:u32)->bool{(0x08000000..0xa7fb0000).contains(&p)&&p&3==0}
fn bit(m:&impl Memory,p:u32,n:u32)->Option<bool>{Some(m.byte(p.checked_add(n/8)?)?&(1<<(n&7))!=0)}
/// Exact f_7615bc/f_7615cc condition semantics. Gate 110 is an OR of
/// five completed quests; gates 88..105 are save unlock flags. The native
/// table maps the remaining gates to story bits in sMhGameData +0x500.
pub fn gate(m:&impl Memory,b:u32,g:u16)->Option<bool>{
 if g<4{return Some(true);}if g>111{return None;}
 let save=m.word(b+0x1897f78)?;if !valid(save){return None;}
 if (88..=105).contains(&g){return bit(m,save+0x3614,u32::from(g)-62);}
 if g==110{
  // f_723da4 maps quest IDs through the resident 12-byte quest index.
  let manager=m.word(b+0x188ad90)?;if !valid(manager){return None;}
  let resource=m.word(manager+0xb4)?;if !valid(resource){return None;}
  let count=m.word(resource+0x64)?;let rows=m.word(resource+0x6c)?;
  if count==0||count>20000||!valid(rows){return None;}
  let mut unknown=false;
  for quest in [0x3ed,0x2c89,0x2cc2,0x2cc3,0x2cc4]{
   let mut index=0;for i in 1..count{if m.word(rows+i*12+4)?==quest{index=i;break;}}
   // Native index zero is its sentinel. Missing quest metadata is unknown.
   if index==0{unknown=true;continue;}if bit(m,save+0x42c,index)?{return Some(true);}
  }return if unknown{None}else{Some(false)};
 }
 let progress=m.word(b+0x1884c58)?;if !valid(progress){return None;}
 let n=m.word(b+0x162ea7c+u32::from(g)*4)?;if n>16383{return None;}bit(m,progress+0x500,n)
}
/// AND of the recipe's progression condition and every ingredient's potential
/// acquisition. Unknown ingredients do not become fabricated availability.
pub fn recipe_available(condition:Option<bool>,materials:&[u16],mut obtainable:impl FnMut(u16)->Option<bool>)->Option<bool>{
 if condition==Some(false){return Some(false);}let mut unknown=condition.is_none();
 for &item in materials{match obtainable(item){Some(false)=>return Some(false),None=>unknown=true,Some(true)=>{}}}
 if unknown{None}else{Some(true)}
}
pub fn alternatives(values:impl IntoIterator<Item=Option<bool>>)->Option<bool>{let mut unknown=false;let mut any=false;for v in values{any=true;match v{Some(true)=>return Some(true),None=>unknown=true,Some(false)=>{}}}if !any||unknown{None}else{Some(false)}}
pub fn source_matches(rank:u8,location:u8,monsters:&[u8],quest:usize,source_rank:u8,entity:u16,kind:u8)->bool{
 rank==source_rank&&match kind{0=>entity==u16::from(location),1=>entity<=255&&monsters.contains(&(entity as u8)),2=>entity as usize==quest,_=>false}
}
/// A matching source or historical acquisition is positive evidence. The
/// quest graph omits some shops/trades/combinations, so its negatives cannot
/// prove an ingredient inaccessible without a separate completeness audit.
pub fn ingredient_available(source:Option<bool>,acquired:Option<bool>)->Option<bool>{
 if source==Some(true)||acquired==Some(true){Some(true)}else{None}
}

/// Scenario source coverage only. Hub levels 8..11 mean G1..G4;
/// special quest categories have no reviewed star meaning here.
pub fn quest_star_match(hub:&str,rank:u8,stars:u8,village:u8,guild:u8)->bool{
 if village>10||guild>11||stars==0{return false;}
 match hub{
  "Village"=>rank<2&&stars<=10&&stars<=village,
  "Guild"=>if rank<2{stars<=7&&stars<=guild.min(7)}else{rank==2&&stars<=4&&guild>=8&&stars<=guild-7},
  _=>false
 }
}
/// None delegates this quest to the character's actual unlock checks.
pub fn quest_override(hub:&str,rank:u8,stars:u8,village:u8,guild:u8)->Option<bool>{
 match hub{"Village" if village!=255=>Some(quest_star_match(hub,rank,stars,village,if guild==255{0}else{guild})),"Guild" if guild!=255=>Some(quest_star_match(hub,rank,stars,if village==255{0}else{village},guild)),_=>None}
}
/// Planning may exclude an indexed material with no matching scenario route.
/// Positive source coverage cannot verify native recipe/event/request gates.
pub fn planning_recipe(materials:&[u16],mut source:impl FnMut(u16)->Option<bool>)->Option<bool>{
 for &item in materials{if source(item)==Some(false){return Some(false);}}None
}

/// Coarse standard-equipment tier boundary. Rarity11 is the special deviant
/// tier (including LR/HR base pieces) and keeps its native recipe gates.
/// This is independent of native recipe/source positives;
/// it never substitutes rarity for the finer quest/story unlock checks.
/// Hub overrides 8..11 explicitly plan G1..G4, while 255 uses actual HR.
pub fn armor_rank_allowed(rarity:u8,hr:Option<u16>,hub_override:u8)->bool{
 if !(1..=11).contains(&rarity){return false;}
 if rarity<=7||rarity==11{return true;}
 if hub_override==255{hr.is_some_and(|hr|hr>=8)}else{(8..=11).contains(&hub_override)}
}

#[cfg(not(test))]mod live{
 use super::*;use core::{cell::UnsafeCell,mem,ptr};use crate::{quest,quest_data,set_data};
 const ITEMS:usize=4096;
 struct State{valid:bool,plan:bool,village:u8,guild:u8,save:u32,gc:u32,actor:u32,progress:usize,materials:[Option<bool>;ITEMS]}
 struct Local(UnsafeCell<State>);unsafe impl Sync for Local{}
 static STATE:Local=Local(UnsafeCell::new(State{valid:false,plan:false,village:0,guild:0,save:0,gc:0,actor:0,progress:0,materials:[None;ITEMS]}));
 struct Direct;impl Memory for Direct{fn word(&self,a:u32)->Option<u32>{Some(word(a))}fn byte(&self,a:u32)->Option<u8>{Some(byte(a))}}
 fn word(p:u32)->u32{unsafe{ptr::read_volatile(p as *const u32)}}fn byte(p:u32)->u8{unsafe{ptr::read_volatile(p as *const u8)}}
 fn identity()->Option<(u32,u32,u32)>{
  let b=forge::mem::text_addr();let gc=word(b+0x188ca48);let save=word(b+0x1897f78);let manager=word(b+0x1885b58);
  if !valid(gc)||!valid(save)||!valid(manager)||byte(gc+0x4c)>7||word(save)!=b+0x176af48{return None;}
  let slot=word(gc+0x30);if slot>=4{return None;}let actor=word(manager+0x1c+slot*4);if !valid(actor){return None;}
  // The same pure quest appearance and entry-check signatures used by the
  // quest planner. Native creation-gate table/getter guards pin MHGU 1.4.
  if word(b+0x54af8c)!=0xe92d4830||word(b+0x54af90)!=0xe1a05002||word(b+0x54ce08)!=0xe92d47f0||word(b+0x3ba50c)!=0xe92d48f0||word(b+0x5615bc)!=0xe3500004||word(b+0x4a9390)!=0xe1d000bc||word(b+0x162ea7c+20*4)!=387{return None;}
  Some((save,gc,actor))
 }
 /// Guard for a prepared cache; false after departure/character replacement.
 pub fn ready()->bool{unsafe{let s=&*STATE.0.get();s.valid&&identity()==Some((s.save,s.gc,s.actor))}}
 pub fn begin()->bool{unsafe{
  let s=&mut*STATE.0.get();s.valid=false;s.plan=false;s.progress=0;s.materials.fill(None);
  let Some((save,gc,actor))=identity()else{return false;};s.save=save;s.gc=gc;s.actor=actor;
  for source in quest_data::SOURCES{if let Some(v)=s.materials.get_mut(source.item as usize){*v=Some(false);}}
  s.valid=true;true
 }}
 /// Start a read-only planning cache; no progression state is changed.
 pub fn begin_plan(village:u8,hub:u8)->bool{
  if (village!=255&&village>10)||(hub!=255&&hub>11){unsafe{let s=&mut*STATE.0.get();s.valid=false;s.plan=false;}return false;}
  if !begin(){return false;}unsafe{let s=&mut*STATE.0.get();s.plan=true;s.village=village;s.guild=hub;}true
 }
 pub fn planning()->bool{unsafe{(*STATE.0.get()).plan}}
 fn eligible(q:&quest::Quest,actor:u32)->Option<bool>{unsafe{
  if q.entry.contains(&255){return None;}let b=forge::mem::text_addr();
  let mut data=[0u32;68];let bytes=core::slice::from_raw_parts_mut(data.as_mut_ptr()as *mut u8,272);bytes[0x16..0x19].copy_from_slice(&q.entry);
  let mut resource=[0u32;28];resource[0x64/4]=1;resource[0x6c/4]=data.as_ptr()as u32;
  let r=if q.id>=1000000{let manager=word(b+0x188ae38);if !valid(manager){return None;}let lookup:unsafe extern "C" fn(u32,u32)->u32=mem::transmute((b+0x3ba50c)as usize);let r=lookup(manager,q.id);if !valid(r){return None;}r}else{resource.as_ptr()as u32};
  let check:unsafe extern "C" fn(u32,u32)->u32=mem::transmute((b+0x54ce08)as usize);Some(check(r,actor)==1)
 }}
 /// At most sixteen quests per frame. Planning uses indexed stars only;
 /// normal mode uses native checks. Callers distinguish failure with ready().
 pub fn tick()->bool{if !ready(){return false;}unsafe{
  let s=&mut*STATE.0.get();
  for i in s.progress..(s.progress+16).min(quest_data::QUESTS.len()){
   let q=&quest_data::QUESTS[i];let override_value=if s.plan{quest_override(q.hub,q.rank,q.stars,s.village,s.guild)}else{None};let available=if let Some(value)=override_value{Some(value)}else{let f:unsafe extern "C" fn(u32,u32,u32)->u32=mem::transmute((forge::mem::text_addr()+0x54af8c)as usize);if f(q.id,255,0)&255==1{eligible(q,s.actor)}else{Some(false)}};
   if available!=Some(false){for source in quest_data::SOURCES{
    // Same exact monster/gather/reward rank matching as quest::covers,
    // applied once per source rather than repeating its per-item scan.
    if !source_matches(q.rank,q.loc,q.monsters,i,source.rank,source.entity,source.kind){continue;}
    if let Some(v)=s.materials.get_mut(source.item as usize){if available==Some(true){*v=Some(true);}else if *v!=Some(true){*v=None;}}
   }}s.progress=i+1;
  }s.progress==quest_data::QUESTS.len()
 }}
 pub fn progress()->(usize,usize){unsafe{((*STATE.0.get()).progress,quest_data::QUESTS.len())}}
 /// Raw source-index coverage: accessible quest route, no accessible indexed
 /// quest route, unindexed/unsupported. The middle count is NOT proven locked.
 pub fn coverage()->(usize,usize,usize){unsafe{let s=&*STATE.0.get();let mut n=[0usize;3];for v in &s.materials{n[match v{Some(true)=>0,Some(false)=>1,None=>2}]+=1;}(n[0],n[1],n[2])}}
 fn material(item:u16)->Option<bool>{unsafe{
  if item>=quest::POINT_BASE{return alternatives(quest::point_group(item,quest_data::POINT_GROUPS)?.members.iter().map(|&(member,_)|material(member)));}
  let s=&*STATE.0.get();let v=s.materials.get(item as usize).copied().flatten();if v==Some(true){return v;}
  // f_39443c's historical-acquisition bits, only as a positive fallback
  // when currently indexed quest sources do not prove acquisition. Never
  // stock counts; this evidence cannot override a false recipe gate.
  let ids=crate::weapon_data::GAME_ITEMS;let native=ids.binary_search_by_key(&item,|r|r.0).ok().and_then(|i|ids.get(i)).map(|r|r.1);
  let acquired=native.and_then(|native|{let p=word(forge::mem::text_addr()+0x18583a8);if !valid(p)||native>=4096{None}else{bit(&Direct,p+0x9c,u32::from(native))}});
  ingredient_available(v,acquired)
 }}
 fn planned_material(item:u16)->Option<bool>{unsafe{
  if item>=quest::POINT_BASE{return alternatives(quest::point_group(item,quest_data::POINT_GROUPS)?.members.iter().map(|&(member,_)|planned_material(member)));}
  (*STATE.0.get()).materials.get(item as usize).copied().flatten()
 }}
 fn recipe(g:u16,materials:&[(u16,u8)])->Option<bool>{
  let mut ids=[0u16;8];if materials.len()>ids.len(){return None;}for(i,&(item,_))in materials.iter().enumerate(){ids[i]=item;}
  if planning(){return planning_recipe(&ids[..materials.len()],planned_material);}
  recipe_available(gate(&Direct,forge::mem::text_addr(),g),&ids[..materials.len()],material)
 }
 fn prepared()->bool{ready()&&unsafe{(*STATE.0.get()).progress==quest_data::QUESTS.len()}}
 pub fn decoration(native:u16)->Option<bool>{
  if !prepared(){return None;}let deco=set_data::DECO.iter().find(|d|d.native==native)?;
  alternatives(set_data::RECIPES[deco.recipe as usize..deco.recipe as usize+deco.recipe_count as usize].iter().map(|r|recipe(r.gate,&set_data::MATERIALS[r.material as usize..r.material as usize+r.material_count as usize])))
 }
 pub fn armor(a:&crate::armor_data::Armor)->Option<bool>{
  let(slot,native)=(a.slot,a.native);
  if !prepared()||slot>=5||native==0{return None;}
  // Owned equipment is already handled by the caller. Unknown source coverage
  // or a remembered recipe must not admit missing G armor at a lower rank.
  let hub=unsafe{let s=&*STATE.0.get();if s.plan{s.guild}else{255}};
  if !armor_rank_allowed(a.rarity,crate::set_player::snapshot().hr,hub){return Some(false);}
  alternatives(set_data::ARMOR_RECIPES.iter().filter_map(|r|{
   if r.slot!=slot{return None;}let variant=r.native.iter().position(|&id|id==native)?;
   if planning(){return Some(planning_recipe(&set_data::ARMOR_MATERIALS[r.material as usize..r.material as usize+r.material_count as usize],planned_material));}
   let remembered=unsafe{let s=&*STATE.0.get();bit(&Direct,s.save+0x16c8+u32::from(slot)*0x360,u32::from(r.index)*4+variant as u32)};
   let physical=recipe_available(gate(&Direct,forge::mem::text_addr(),r.gate),&set_data::ARMOR_MATERIALS[r.material as usize..r.material as usize+r.material_count as usize],material);
   // A remembered recipe can resolve missing source data, but cannot
   // override a proven progression gate or inaccessible ingredient.
   if physical==Some(false){return Some(Some(false));}
   if remembered==Some(true){return Some(Some(true));}
   // Native material-points category is deliberately opaque until its
   // membership is proved. Missing point metadata cannot become availability.
   Some(if r.point_category!=0||r.point_count!=0{None}else{physical})
  }))
 }
}
#[cfg(not(test))]pub use live::*;
