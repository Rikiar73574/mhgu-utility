//! Set Builder adapter: read-only progression/equipment, resumable search and wishlist.
use core::cell::UnsafeCell;
use crate::{armor_data as armor,set_data as data,set_solver as solver,set_player as player,Ui};
pub const LIMIT:usize=24;
#[derive(Clone,Copy,PartialEq,Eq)]pub enum Phase{Choose,Preparing,Searching,Paused,Complete,Unavailable}
pub struct State{
 pub phase:Phase,pub status:&'static str,pub goals:[usize;6],pub goal_count:usize,pub show_results:bool,
 pub result:usize,pub piece:usize,pub jewels:bool,pub jewel_row:usize,pub results:[Option<solver::Solution>;LIMIT],pub count:usize,pub visited:u64,
 pub progress:usize,pub armor_count:[u16;5],pub jewel_count:usize,
 armor_ids:[[u16;1200];5],jewel_ids:[u16;256],owned:[u32;player::MAX_EQUIPMENT],owned_count:usize,
 solver:Option<solver::Solver>,
}
struct Local(UnsafeCell<State>);unsafe impl Sync for Local{}
static STATE:Local=Local(UnsafeCell::new(State{phase:Phase::Choose,status:"Select skills, then search your available equipment.",goals:[usize::MAX;6],goal_count:0,show_results:false,result:0,piece:0,jewels:false,jewel_row:0,results:[None;LIMIT],count:0,visited:0,progress:0,armor_count:[0;5],jewel_count:0,armor_ids:[[0;1200];5],jewel_ids:[0;256],owned:[0;player::MAX_EQUIPMENT],owned_count:0,solver:None}));
pub fn state()->&'static State{unsafe{&*STATE.0.get()}}
fn mutable()->&'static mut State{unsafe{&mut*STATE.0.get()}}
pub fn open(u:&mut Ui){u.builder=true;u.selected=0;u.len=0;u.detail=usize::MAX;u.dirty=true;let s=mutable();s.show_results=false;capture(s);}
fn capture(s:&mut State)->bool{
 s.owned_count=0;s.count=0;s.results.fill(None);s.solver=None;
 if !player::refresh(){s.phase=Phase::Unavailable;s.status="Character equipment is unavailable. Open this in town as a hunter.";return false;}
 let p=player::snapshot();if p.hunter.is_none()||p.gender.is_none()||p.weapon_slots.is_none()||!crate::set_unlock::begin(){s.phase=Phase::Unavailable;s.status="This game build cannot verify equipment availability.";return false;}
 s.owned_count=p.owned_count;for(i,a)in p.owned[..p.owned_count].iter().enumerate(){s.owned[i]=((a.slot as u32)<<16)|u32::from(a.native);}s.owned[..s.owned_count].sort_unstable();s.phase=Phase::Choose;s.status="Checks every owned charm. Crafting materials do not limit results.";true
}
fn owns_in(s:&State,id:usize)->bool{let a=&armor::ARMOR[id];s.owned[..s.owned_count].binary_search(&((a.slot as u32)<<16|u32::from(a.native))).is_ok()}
pub fn owns(id:usize)->bool{owns_in(state(),id)}
pub fn selected(id:usize)->bool{state().goals[..state().goal_count].contains(&id)}
pub fn toggle(id:usize){let Some(a)=data::ACTIVATIONS.get(id)else{return;};let s=mutable();
 if let Some(i)=s.goals[..s.goal_count].iter().position(|&i|data::ACTIVATIONS[i].tree==a.tree){if s.goals[i]==id{s.goals.copy_within(i+1..s.goal_count,i);s.goal_count-=1;}else{s.goals[i]=id;}}
 else if s.goal_count<6{s.goals[s.goal_count]=id;s.goal_count+=1;}
 s.phase=Phase::Choose;
}
pub fn start(){let s=mutable();if s.goal_count==0{s.status="Select at least one skill.";return;}if !capture(s){return;}
 s.jewels=false;s.jewel_row=0;s.show_results=true;s.phase=Phase::Preparing;s.progress=0;s.armor_count=[0;5];s.jewel_count=0;s.count=0;s.result=0;s.piece=0;s.visited=0;s.solver=None;s.status="Checking progression availability (not crafting stock)...";
}
pub fn back(u:&mut Ui){let s=mutable();if s.jewels{s.jewels=false;return;}if s.show_results{s.show_results=false;if matches!(s.phase,Phase::Preparing|Phase::Searching){s.phase=Phase::Paused;}u.selected=0;u.dirty=true;}else{u.builder=false;u.len=0;u.selected=0;u.dirty=true;}}
pub fn run_toggle(){let s=mutable();if s.count==LIMIT&&s.show_results{s.show_results=false;s.phase=Phase::Choose;return;}s.show_results=true;match s.phase{Phase::Searching|Phase::Preparing=>s.phase=Phase::Paused,Phase::Paused=>s.phase=if s.solver.is_some(){Phase::Searching}else{Phase::Preparing},_=>start()}}
fn put(g:&mut solver::Gear,tree:u8,value:i8,s:&State){for i in 0..s.goal_count{if data::ACTIVATIONS[s.goals[i]].tree==tree{g.points[i]+=i16::from(value);g.present|=1<<i;}}
 if tree==data::SKILL_PLUS{g.extra[0]+=i16::from(value);}if tree==data::TALISMAN_BOOST{g.extra[1]+=i16::from(value);}if tree==data::TORSO_UP{g.torso_up=true;}
}
struct Catalog<'a>(&'a State);
impl solver::Catalog for Catalog<'_>{
 fn verified(&self,kind:usize,index:u32)->bool{match kind{0..=4=>self.0.armor_ids[kind][index as usize]&0x8000==0,6=>self.0.jewel_ids[index as usize]&0x8000==0,_=>true}}

 fn armor_count(&self,slot:usize)->u32{u32::from(self.0.armor_count[slot])}
 fn armor(&self,slot:usize,index:u32)->solver::Gear{let a=&armor::ARMOR[(self.0.armor_ids[slot][index as usize]&0x7fff)as usize];let mut g=solver::Gear::default();g.slots=a.slots;for &(tree,value)in &armor::SKILLS[a.skill as usize..a.skill as usize+a.skill_count as usize]{put(&mut g,data::ARMOR_TREES[tree as usize],value,self.0);}g}
 fn charm_count(&self)->u32{player::snapshot().charm_count as u32+1}
 fn charm(&self,index:u32)->solver::Gear{let mut g=solver::Gear::default();if index>0{let a=&player::snapshot().charms[index as usize-1];g.slots=a.slots;for skill in a.skills{if skill.tree!=0{put(&mut g,skill.tree,skill.points,self.0);}}}g}
 fn jewel_count(&self)->u32{self.0.jewel_count as u32}
 fn jewel(&self,index:u32)->solver::Jewel{let a=&data::DECO[(self.0.jewel_ids[index as usize]&0x7fff)as usize];let mut g=solver::Gear::default();for &(tree,value)in &data::POINTS[a.skill as usize..a.skill as usize+a.skill_count as usize]{put(&mut g,tree,value,self.0);}solver::Jewel{points:g.points,present:g.present,slots:a.slots,extra:g.extra}}
}
pub fn tick(){let s=mutable();if !s.show_results{return;}if !crate::set_unlock::ready(){s.solver=None;s.count=0;s.results.fill(None);s.phase=Phase::Unavailable;s.status="Character or scene changed. Return to skill selection and search again.";return;}
 if s.phase==Phase::Preparing{if !crate::set_unlock::tick(){return;}let end=(s.progress+32).min(armor::ARMOR.len()+data::DECO.len());let p=player::snapshot();
  while s.progress<end{let i=s.progress;s.progress+=1;
   if i<armor::ARMOR.len(){let a=&armor::ARMOR[i];if a.native==65535||a.hunter!=2&&Some(a.hunter)!=p.hunter||a.gender!=2&&Some(a.gender)!=p.gender{continue;}
    let available=if owns_in(s,i){Some(true)}else{crate::set_unlock::armor(a.slot,a.native)};
    match available{Some(true)|None=>{let n=s.armor_count[a.slot as usize]as usize;if n>=1200{s.phase=Phase::Unavailable;s.status="Armor candidate capacity exceeded; search was not truncated.";return;}s.armor_ids[a.slot as usize][n]=i as u16|if available.is_none(){0x8000}else{0};s.armor_count[a.slot as usize]+=1;},Some(false)=>{}}
   }else{let id=i-armor::ARMOR.len();let available=crate::set_unlock::decoration(data::DECO[id].native);match available{Some(true)|None=>{if s.jewel_count==256{s.phase=Phase::Unavailable;s.status="Decoration capacity exceeded.";return;}s.jewel_ids[s.jewel_count]=id as u16|if available.is_none(){0x8000}else{0};s.jewel_count+=1;},Some(false)=>{}}}
  }
  if s.progress==armor::ARMOR.len()+data::DECO.len(){if s.armor_count.contains(&0){s.phase=Phase::Complete;s.status="No available armor for at least one equipment slot.";return;}
   let mut query=solver::Query::default();query.goal_count=s.goal_count as u8;query.weapon_slots=p.weapon_slots.unwrap_or(0);query.native_wrapping=true;
   for i in 0..s.goal_count{let a=&data::ACTIVATIONS[s.goals[i]];query.goals[i]=solver::Goal{threshold:i16::from(a.points),at_least:a.points>0};if (144..=179).contains(&a.tree){query.boost_nonzero|=1<<i;}}
   s.solver=Some(solver::Solver::new(query));s.phase=Phase::Searching;s.status="Searching all available armor and owned charms...";
  }return;
 }
 if s.phase!=Phase::Searching{return;}let Some(mut engine)=s.solver.take()else{return;};let result=engine.tick(&Catalog(s),256);s.visited=engine.work_done;s.solver=Some(engine);
 match result{solver::Tick::Working=>{},solver::Tick::Found(found)=>{
  if let Some(engine)=s.solver.as_mut(){engine.next_equipment();}
  if !s.results[..s.count].iter().flatten().any(|r|r.armor==found.armor&&r.charm==found.charm){s.results[s.count]=Some(found);s.count+=1;}
  if s.count==LIMIT{s.phase=Phase::Paused;s.status="24 distinct builds found. Refine skills to narrow the search.";}
 },solver::Tick::Complete=>{s.phase=Phase::Complete;s.status=if s.count==0{"Search complete: no matching build."}else{"Search complete."};},solver::Tick::Cancelled=>{s.phase=Phase::Paused;s.status="Search paused.";},solver::Tick::InvalidCatalog=>{s.phase=Phase::Unavailable;s.status="The equipment snapshot or search data is invalid.";}}
}
pub fn armor_id(slot:usize)->Option<usize>{let s=state();let r=s.results.get(s.result).filter(|_|s.result<s.count)?.as_ref()?;Some((s.armor_ids[slot][r.armor[slot]as usize]&0x7fff)as usize)}
pub fn jewel_id(index:usize)->usize{(state().jewel_ids[index]&0x7fff)as usize}
pub fn add_missing(){for slot in 0..5{if let Some(id)=armor_id(slot){let node=crate::wishlist::ARMOR_BASE+id;if !owns(id)&&!crate::wishlist::contains(node,0){crate::wishlist::toggle(node,0);if crate::wishlist::failed(){return;}}}}}
pub fn input(u:&mut Ui,p:u32){if state().jewels{
 let s=mutable();let count=s.results[s.result].as_ref().map_or(0,|r|r.jewel_count as usize);
 if p&(1<<15)!=0{s.jewel_row=(s.jewel_row+1).min(count.saturating_sub(1));}if p&(1<<13)!=0{s.jewel_row=s.jewel_row.saturating_sub(1);}
 if p&(1<<14)!=0{s.jewel_row=(s.jewel_row+11).min(count.saturating_sub(1));}if p&(1<<12)!=0{s.jewel_row=s.jewel_row.saturating_sub(11);}
 if p&1!=0{open_jewel(u,s.jewel_row);}return;
 }if state().show_results{
 let s=mutable();if p&(1<<14)!=0&&s.count>0{s.result=(s.result+1).min(s.count-1);}if p&(1<<12)!=0{s.result=s.result.saturating_sub(1);}if p&(1<<15)!=0{s.piece=(s.piece+1).min(5);}if p&(1<<13)!=0{s.piece=s.piece.saturating_sub(1);}
 if p&1!=0{if state().piece==5{show_jewels();}else if let Some(id)=armor_id(state().piece){crate::enter(u,crate::catalog::ENTRIES.len()+id);}}
 if p&(1<<5)!=0{add_missing();}if p&(1<<2)!=0{run_toggle();}
 }else{if p&(1<<15)!=0&&u.count>0{u.selected=(u.selected+1).min(u.count-1);}if p&(1<<13)!=0{u.selected=u.selected.saturating_sub(1);}if p&1!=0&&u.count>0{toggle(u.results[u.selected]as usize);}if p&(1<<2)!=0{start();}}
}

pub fn select_result(index:usize){let s=mutable();s.result=index.min(s.count.saturating_sub(1));}
pub fn page(u:&mut Ui,down:bool){if state().jewels{let s=mutable();let count=s.results[s.result].as_ref().map_or(0,|r|r.jewel_count as usize);s.jewel_row=if down{(s.jewel_row+11).min(count.saturating_sub(1))}else{s.jewel_row.saturating_sub(11)};}else if state().show_results{let s=state();select_result(if down{s.result.saturating_add(1)}else{s.result.saturating_sub(1)});}else{u.selected=if down{u.selected.saturating_add(10).min(u.count.saturating_sub(1))}else{u.selected.saturating_sub(10)};}}

/// High bits label incomplete source coverage without discarding candidates.
pub fn unverified()->bool{let s=state();let Some(r)=s.results[s.result].as_ref()else{return false;};(0..5).any(|slot|s.armor_ids[slot][r.armor[slot]as usize]&0x8000!=0)||r.jewels[..r.jewel_count as usize].iter().any(|p|s.jewel_ids[p.jewel as usize]&0x8000!=0)}
pub fn show_jewels(){let s=mutable();s.jewels=true;s.jewel_row=0;}
pub fn open_jewel(u:&mut Ui,index:usize){let s=state();if let Some(p)=s.results[s.result].as_ref().and_then(|r|r.jewels.get(index).filter(|_|index<r.jewel_count as usize)){let id=jewel_id(p.jewel as usize);crate::enter(u,data::DECO[id].entry as usize);}}
