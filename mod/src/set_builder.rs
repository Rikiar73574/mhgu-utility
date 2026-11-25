//! Set Builder adapter: read-only progression/equipment, resumable search and wishlist.
use core::cell::UnsafeCell;
use crate::{armor_data as armor,set_data as data,set_solver as solver,set_player as player,set_skills as skills,Ui};
use crate::set_solver::Catalog as _;
pub const LIMIT:usize=24;
#[derive(Clone,Copy,PartialEq,Eq)]pub enum Phase{Choose,Preparing,Searching,Paused,Complete,Unavailable}
pub struct State{
 pub phase:Phase,pub status:&'static str,pub goals:[usize;6],pub goal_count:usize,pub show_results:bool,
 pub result:usize,pub piece:usize,pub jewels:bool,pub jewel_row:usize,pub results:[Option<solver::Solution>;LIMIT],pub count:usize,pub visited:u64,
 pub effect:Option<usize>,pub effect_line:usize,pub effect_lines:usize,pub skills:bool,pub skill_row:usize,pub skill_rows:[u8;206],pub skill_count:usize,pub active_count:usize,pub totals:skills::Totals,
 pub priority:Option<skills::PriorityProfile>,cached:Option<solver::Solution>,ranks:[[u32;12];LIMIT],pub search:skills::SearchBudget,
 focus:Option<usize>,focus_mask:crate::set_focus::Mask,focus_targets:[(u16,i32,i32);206],focus_scan:usize,focus_count:usize,focus_task:usize,focus_found:u8,
 pub weapon_slots:u8,pub owned_only:bool,pub progress:usize,pub armor_count:[u16;5],pub jewel_count:usize,
 armor_ids:[[u16;1200];5],jewel_ids:[u16;256],owned:[u32;player::MAX_EQUIPMENT],owned_count:usize,
 solver:Option<solver::Solver>,bonus:Option<solver::DecorationRefiner>,bonus_penalties:u32,
}
struct Local(UnsafeCell<State>);unsafe impl Sync for Local{}
static STATE:Local=Local(UnsafeCell::new(State{phase:Phase::Choose,status:"Search by weapon and priorities, or choose optional required skills.",goals:[usize::MAX;6],goal_count:0,show_results:false,result:0,piece:0,jewels:false,jewel_row:0,results:[None;LIMIT],count:0,visited:0,effect:None,effect_line:0,effect_lines:0,skills:false,skill_row:0,skill_rows:[0;206],skill_count:0,active_count:0,totals:skills::Totals{points:[0;206],present:[false;206]},priority:None,cached:None,ranks:[[u32::MAX;12];LIMIT],search:skills::SearchBudget::new(),focus:None,focus_mask:crate::set_focus::Mask::new(),focus_targets:[(0,0,0);206],focus_scan:0,focus_count:0,focus_task:0,focus_found:0,weapon_slots:0,owned_only:false,progress:0,armor_count:[0;5],jewel_count:0,armor_ids:[[0;1200];5],jewel_ids:[0;256],owned:[0;player::MAX_EQUIPMENT],owned_count:0,solver:None,bonus:None,bonus_penalties:0}));
pub fn state()->&'static State{unsafe{&*STATE.0.get()}}
fn mutable()->&'static mut State{unsafe{&mut*STATE.0.get()}}
pub fn open(u:&mut Ui){u.tab=5;u.builder=true;u.selected=0;u.len=0;u.detail=usize::MAX;u.dirty=true;let s=mutable();s.show_results=false;s.effect=None;capture(s);}
fn capture(s:&mut State)->bool{
 s.focus=None;s.focus_count=0;s.focus_task=0;s.owned_count=0;s.count=0;s.results.fill(None);s.cached=None;s.solver=None;s.bonus=None;
 if !player::refresh(){s.phase=Phase::Unavailable;s.status="Character equipment is unavailable. Open this in town as a hunter.";return false;}
 let p=player::snapshot();#[cfg(target_arch="arm")]let prepared={s.weapon_slots=crate::set_workbench::slots();let prefs=crate::set_saved::preferences();s.priority=Some(crate::set_workbench::profile());s.owned_only=prefs.owned_only;if prefs.village==255&&prefs.hub==255{crate::set_unlock::begin()}else{crate::set_unlock::begin_plan(prefs.village,prefs.hub)}};
 #[cfg(not(target_arch="arm"))]let prepared={s.weapon_slots=p.weapon_slots.unwrap_or(0);crate::set_unlock::begin()};
 if p.hunter.is_none()||p.gender.is_none()||p.weapon_slots.is_none()||!prepared{s.phase=Phase::Unavailable;s.status="This game build cannot verify equipment availability.";return false;}
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
pub fn start(){#[cfg(target_arch="arm")]crate::set_workbench::search_started();let s=mutable();if !capture(s){return;}
 s.effect=None;s.skills=false;s.skill_row=0;s.jewels=false;s.jewel_row=0;s.search=skills::SearchBudget::new();s.show_results=true;s.phase=Phase::Preparing;s.progress=0;s.armor_count=[0;5];s.jewel_count=0;s.count=0;s.result=0;s.piece=0;s.visited=0;s.solver=None;s.status="Checking progression availability (not crafting stock)...";
}
pub fn back(u:&mut Ui){let s=mutable();if s.effect.take().is_some(){return;}#[cfg(target_arch="arm")]if crate::set_workbench::back(u){return;}if s.skills{s.skills=false;return;}if s.jewels{s.jewels=false;return;}if s.show_results{s.show_results=false;if matches!(s.phase,Phase::Preparing|Phase::Searching){s.phase=Phase::Paused;}u.selected=0;u.dirty=true;}else{#[cfg(target_arch="arm")]crate::set_workbench::leave(u);#[cfg(not(target_arch="arm"))]{u.builder=false;u.tab=0;u.dirty=true;}}}
pub fn run_toggle(){let s=mutable();s.show_results=true;match s.phase{Phase::Searching|Phase::Preparing=>{s.phase=Phase::Paused;s.status="Search paused. Y resumes.";},Phase::Paused=>{s.phase=if s.progress==armor::ARMOR.len()+data::DECO.len(){Phase::Searching}else{Phase::Preparing};s.search=skills::SearchBudget::new();s.status="Improving the best builds found so far...";},_=>start()}}
pub fn dismiss_details(){let s=mutable();s.effect=None;s.skills=false;s.jewels=false;}
pub fn browse(){let s=mutable();if matches!(s.phase,Phase::Preparing|Phase::Searching){s.phase=Phase::Paused;s.status="Search paused while browsing. Y resumes.";}}

fn put(g:&mut solver::Gear,tree:u8,value:i8,s:&State){for i in 0..s.goal_count{if data::ACTIVATIONS[s.goals[i]].tree==tree{g.points[i]+=i16::from(value);g.present|=1<<i;}}
 if let Some(id)=s.focus{if data::ACTIVATIONS[id].tree==tree{g.points[s.goal_count]+=i16::from(value);g.present|=1<<s.goal_count;}}
 if tree==data::SKILL_PLUS{g.extra[0]+=i16::from(value);}if tree==data::TALISMAN_BOOST{g.extra[1]+=i16::from(value);}if tree==data::TORSO_UP{g.torso_up=true;}
}
struct Catalog<'a>(&'a State,bool);
impl Catalog<'_>{fn index(&self,k:usize,i:u32)->u32{if self.1{self.0.focus_mask.index(k,i)}else{i}}}
impl solver::Catalog for Catalog<'_>{
 fn verified(&self,kind:usize,index:u32)->bool{match kind{0..=4=>self.0.armor_ids[kind][self.index(kind,index)as usize]&0x8000==0,6=>self.0.jewel_ids[self.index(6,index)as usize]&0x8000==0,_=>true}}

 fn armor_count(&self,slot:usize)->u32{if self.1{self.0.focus_mask.count(slot)}else{u32::from(self.0.armor_count[slot])}}
 fn armor(&self,slot:usize,index:u32)->solver::Gear{let a=&armor::ARMOR[(self.0.armor_ids[slot][self.index(slot,index)as usize]&0x7fff)as usize];let mut g=solver::Gear::default();g.slots=a.slots;for &(tree,value)in &armor::SKILLS[a.skill as usize..a.skill as usize+a.skill_count as usize]{put(&mut g,data::ARMOR_TREES[tree as usize],value,self.0);}g}
 fn charm_count(&self)->u32{player::snapshot().charm_count as u32+1}
 fn charm(&self,index:u32)->solver::Gear{let mut g=solver::Gear::default();if index>0{let a=&player::snapshot().charms[index as usize-1];g.slots=a.slots;for skill in a.skills{if skill.tree!=0{put(&mut g,skill.tree,skill.points,self.0);}}}g}
 fn jewel_count(&self)->u32{if self.1{self.0.focus_mask.count(6)}else{self.0.jewel_count as u32}}
 fn jewel(&self,index:u32)->solver::Jewel{let a=&data::DECO[(self.0.jewel_ids[self.index(6,index)as usize]&0x7fff)as usize];let mut g=solver::Gear::default();for &(tree,value)in &data::POINTS[a.skill as usize..a.skill as usize+a.skill_count as usize]{put(&mut g,tree,value,self.0);}solver::Jewel{points:g.points,present:g.present,slots:a.slots,extra:g.extra}}
}
// Seed targets are discovered from the captured catalog, never hardcoded.
fn prepare_focus(s:&mut State){s.focus=None;s.focus_count=0;s.focus_scan=0;}
// Inspect one bonus tree per slice while planning seeds.
fn scan_focus(s:&mut State){
 if s.goal_count==solver::MAX_GOALS{s.focus_scan=data::ACTIVATIONS.len();return;}
 while s.focus_scan<data::ACTIVATIONS.len(){let id=s.focus_scan;s.focus_scan+=1;let a=&data::ACTIVATIONS[id];
  if a.points<=0||!applicable(a.tree)||s.goals[..s.goal_count].iter().any(|&g|data::ACTIVATIONS[g].tree==a.tree){continue;}
  if data::ACTIVATIONS.iter().any(|b|b.tree==a.tree&&b.points>0&&b.points<a.points){continue;}s.focus=Some(id);
  let c=Catalog(s,false);let mut potential=0i32;let mut overlap=0i32;
  for k in 0..5{let mut max=0;let mut shared=0;for i in 0..c.armor_count(k){if !c.verified(k,i){continue;}let g=c.armor(k,i);let p=i32::from(g.points[s.goal_count]).max(0);max=max.max(p);if g.points[..s.goal_count].iter().any(|&v|v>0){shared=shared.max(p);}}potential+=max;overlap+=shared;}
  let charm=(0..c.charm_count()).map(|i|i32::from(c.charm(i).points[s.goal_count]).max(0)).max().unwrap_or(0);potential+=charm;overlap+=charm;
  potential+=(0..c.jewel_count()).filter(|&i|c.verified(6,i)).map(|i|{let j=c.jewel(i);i32::from(j.points[s.goal_count]).max(0)*3/i32::from(j.slots.max(1))}).max().unwrap_or(0)*7;
  if potential>=i32::from(a.points){s.focus_targets[s.focus_count]=(id as u16,overlap*100/i32::from(a.points),potential*100/i32::from(a.points));s.focus_count+=1;}
  s.focus=None;if s.focus_scan==data::ACTIVATIONS.len(){break;}return;
 }
 s.focus_targets[..s.focus_count].sort_unstable_by_key(|&(id,overlap,potential)|(core::cmp::Reverse(s.priority.map_or(0,|p|{let(c,w)=p.activation_preference(data::ACTIVATIONS[id as usize].tree,i16::from(data::ACTIVATIONS[id as usize].points));if c==0{0}else{(4-p.order.iter().position(|&v|v==4-c).unwrap_or(3)as u32)*1000+w}})),core::cmp::Reverse(overlap),core::cmp::Reverse(potential),id));
}
pub fn weapon_slots()->u8{state().weapon_slots}
fn focus_query(s:&State)->solver::Query{let mut q=solver::Query::default();q.goal_count=(s.goal_count+usize::from(s.focus.is_some()))as u8;q.weapon_slots=s.weapon_slots;q.native_wrapping=true;
 for i in 0..q.goal_count as usize{let id=if i==s.goal_count{s.focus.unwrap()}else{s.goals[i]};let a=&data::ACTIVATIONS[id];q.goals[i]=solver::Goal{threshold:i16::from(a.points),at_least:a.points>0};if(144..=179).contains(&a.tree){q.boost_nonzero|=1<<i;}}q
}
// Tasks: original seed, bonus-tree seeds, verified broad pass, full broad pass.
fn start_focus(s:&mut State,task:usize){s.focus_task=task;s.focus=if task>0&&task<=s.focus_count{Some(s.focus_targets[task-1].0 as usize)}else{None};s.focus_mask=crate::set_focus::Mask::new();s.focus_found=0;s.solver=None;}
fn advance_focus(s:&mut State){if s.focus_task<s.focus_count+2{start_focus(s,s.focus_task+1);}else{s.phase=Phase::Complete;s.status=if s.count==0{"Search complete: no matching build."}else{"Search complete. Ranked by activated skills."};}}

pub fn tick(){
 #[cfg(not(test))]let now=forge::imgui::get_time();
 #[cfg(test)]let now=0.0;
 tick_at(now);
}
// The deadline is cooperative: one existing slice may overrun the 1 ms window.
// The slice ceiling also bounds work if the clock stops advancing.
pub(crate) fn run_slices<T>(work:&mut T,mut clock:impl FnMut()->u64,active:impl Fn(&T)->bool,mut slice:impl FnMut(&mut T)->bool){
 if !active(work){return;}
 let start=clock();let mut previous=start;
 for _ in 0..8{
  if !slice(work)||!active(work){return;}
  let now=clock();
  if now<previous||now-start>=19_200{return;}
  previous=now;
 }
}
fn scheduling_ticks()->u64{
 #[cfg(target_arch="arm")]unsafe{
  // GetSystemTick is supplied by the recomp kernel at the Switch's 19.2 MHz.
  let lo:u32;let hi:u32;
  core::arch::asm!("svc #0x1e",out("r0")lo,out("r1")hi,options(nostack));
  (u64::from(hi)<<32)|u64::from(lo)
 }
 #[cfg(all(not(target_arch="arm"),not(test)))]{(crate::preview_time()*19_200_000.0)as u64}
 #[cfg(all(not(target_arch="arm"),test))]{0}
}
fn search_active(s:&State)->bool{s.show_results&&matches!(s.phase,Phase::Preparing|Phase::Searching)}
fn progress(s:&State)->(Phase,usize,u64,usize,usize,bool,u8,bool,bool,usize){
 (s.phase,s.progress,s.visited,s.focus_scan,s.focus_task,s.focus_mask.ready,s.focus_found,s.solver.is_some(),s.bonus.is_some(),s.count)
}
pub fn tick_at(now:f64){tick_with_clock(now,scheduling_ticks);}
pub(crate) fn tick_with_clock(now:f64,clock:impl FnMut()->u64){
 let s=mutable();if !s.show_results{return;}
 if !crate::set_unlock::ready(){s.solver=None;s.count=0;s.results.fill(None);s.phase=Phase::Unavailable;s.status="Character or scene changed. Return to skill selection and search again.";return;}
 run_slices(s,clock,search_active,|s|{let before=progress(s);tick_slice(s,now);progress(s)!=before});
}
fn tick_slice(s:&mut State,now:f64){
 if s.phase==Phase::Preparing{if !crate::set_unlock::tick(){return;}let end=(s.progress+32).min(armor::ARMOR.len()+data::DECO.len());let p=player::snapshot();
  while s.progress<end{let i=s.progress;s.progress+=1;
   if i<armor::ARMOR.len(){let a=&armor::ARMOR[i];if s.owned_only&&!owns_in(s,i){continue;}if a.native==65535||a.hunter!=2&&Some(a.hunter)!=p.hunter||a.gender!=2&&Some(a.gender)!=p.gender{continue;}
    let available=if owns_in(s,i){Some(true)}else{crate::set_unlock::armor(a.slot,a.native)};
    match available{Some(true)|None=>{let n=s.armor_count[a.slot as usize]as usize;if n>=1200{s.phase=Phase::Unavailable;s.status="Armor candidate capacity exceeded; search was not truncated.";return;}s.armor_ids[a.slot as usize][n]=i as u16|if available.is_none(){0x8000}else{0};s.armor_count[a.slot as usize]+=1;},Some(false)=>{}}
   }else{let id=i-armor::ARMOR.len();let available=crate::set_unlock::decoration(data::DECO[id].native);match available{Some(true)|None=>{if s.jewel_count==256{s.phase=Phase::Unavailable;s.status="Decoration capacity exceeded.";return;}s.jewel_ids[s.jewel_count]=id as u16|if available.is_none(){0x8000}else{0};s.jewel_count+=1;},Some(false)=>{}}}
  }
  if s.progress==armor::ARMOR.len()+data::DECO.len(){if s.armor_count.contains(&0){s.phase=Phase::Complete;s.status="No available armor for at least one equipment slot.";return;}
   prepare_focus(s);start_focus(s,0);s.phase=Phase::Searching;s.status="Searching focused skill combinations...";
  }return;
 }
 if s.phase!=Phase::Searching{return;}
 if stop_search(s,now){return;}
 if s.focus_scan<data::ACTIVATIONS.len(){scan_focus(s);return;}
 if s.focus_task<=s.focus_count+1&&!s.focus_mask.ready{
  let mut mask=core::mem::replace(&mut s.focus_mask,crate::set_focus::Mask::new());let work=mask.tick(&Catalog(s,false),s.goal_count+usize::from(s.focus.is_some()),s.focus_task<=s.focus_count,2048);s.focus_mask=mask;s.visited+=u64::from(work);return;
 }
 if s.bonus.is_none(){
  // Seed budgets only affect prioritization. The final pass retains every candidate.
  let seed=s.focus_task<=s.focus_count;let limit=if seed{131_072}else if s.focus_task==s.focus_count+1{20_000_000}else{u64::MAX};
  if (seed&&s.focus_found>=1||s.focus_task==s.focus_count+1&&s.focus_found>=30)||s.solver.as_ref().is_some_and(|e|e.work_done>=limit){advance_focus(s);return;}
  if s.solver.is_none(){s.solver=Some(solver::Solver::new(focus_query(s)));}
 }
 if let Some(mut bonus)=s.bonus.take(){
  let before=bonus.work_done;
  let (found,done)=bonus.tick(s.jewel_count as u32,|j|data::DECO[(s.jewel_ids[j as usize]&0x7fff)as usize].slots,|candidate|bonus_score(s,candidate).filter(|score|score[1]<=s.bonus_penalties),32);
  s.visited+=bonus.work_done-before;
  if done{let score=rank(s,&found);retain(s,found,score);s.focus_found=s.focus_found.saturating_add(1);s.status="Searching other armor and charm combinations...";}
  else{s.bonus=Some(bonus);}
  stop_search(s,now);return;
 }
 let Some(mut engine)=s.solver.take()else{return;};let before=engine.work_done;let result=engine.tick(&Catalog(s,s.focus_task<=s.focus_count+1),256);s.visited+=engine.work_done-before;s.solver=Some(engine);
 match result{solver::Tick::Working=>{},solver::Tick::Found(found)=>{
  let mut found=found;
  if s.focus_task<=s.focus_count+1{for k in 0..5{found.armor[k]=s.focus_mask.index(k,found.armor[k]);}for p in &mut found.jewels[..found.jewel_count as usize]{p.jewel=s.focus_mask.index(6,p.jewel);}}
  if let Some(score)=bonus_score(s,&mut found){
   s.status="Refining decorations for this build...";s.bonus_penalties=score[1];
   let (capacity,_)=slots_for(s,&found);
   s.bonus=solver::DecorationRefiner::new(found,capacity,|j|data::DECO[(s.jewel_ids[j as usize]&0x7fff)as usize].slots,score);
   if let Some(engine)=s.solver.as_mut(){engine.next_equipment();}
   stop_search(s,now);return;
  }

 },solver::Tick::Complete=>{advance_focus(s);},solver::Tick::Cancelled=>{s.phase=Phase::Paused;s.status="Search paused.";},solver::Tick::InvalidCatalog=>{s.phase=Phase::Unavailable;s.status="The equipment snapshot or search data is invalid.";}}
 if s.phase==Phase::Searching{stop_search(s,now);}

}
pub fn armor_id(slot:usize)->Option<usize>{let s=state();if !s.show_results{return None;}let r=s.results.get(s.result).filter(|_|s.result<s.count)?.as_ref()?;Some((s.armor_ids[slot][r.armor[slot]as usize]&0x7fff)as usize)}
pub fn jewel_id(index:usize)->usize{(state().jewel_ids[index]&0x7fff)as usize}
pub fn add_missing(){for slot in 0..5{if let Some(id)=armor_id(slot){let node=crate::wishlist::ARMOR_BASE+id;if !owns(id)&&!crate::wishlist::contains(node,0){crate::wishlist::toggle(node,0);if crate::wishlist::failed(){return;}}}}}
pub const SKILL_PAGE:usize=7;pub const POINTS_PAGE:usize=5;pub const EFFECT_PAGE:usize=16;
pub fn show_effect(id:usize){if id>=data::ACTIVATIONS.len(){return;}browse();let s=mutable();s.effect=Some(id);s.effect_line=0;s.effect_lines=0;}
pub fn effect_lines(count:usize){let s=mutable();s.effect_lines=count;s.effect_line=s.effect_line.min(count.saturating_sub(EFFECT_PAGE));}
fn scroll_effect(down:bool,amount:usize){let s=mutable();s.effect_line=if down{s.effect_line.saturating_add(amount).min(s.effect_lines.saturating_sub(EFFECT_PAGE))}else{s.effect_line.saturating_sub(amount)};}
pub fn input(u:&mut Ui,p:u32){
 if state().effect.is_some(){if p&(1<<15)!=0{scroll_effect(true,1);}if p&(1<<13)!=0{scroll_effect(false,1);}if p&(1<<14)!=0{scroll_effect(true,EFFECT_PAGE);}if p&(1<<12)!=0{scroll_effect(false,EFFECT_PAGE);}return;}
if p&((1<<12)|(1<<13)|(1<<14)|(1<<15)|1|(1<<5))!=0&&state().show_results{browse();}
 if state().skills{let s=mutable();if p&(1<<15)!=0{s.skill_row=(s.skill_row+1).min(s.skill_count.saturating_sub(1));}if p&(1<<13)!=0{s.skill_row=s.skill_row.saturating_sub(1);}if p&(1<<14)!=0{s.skill_row=(s.skill_row+POINTS_PAGE).min(s.skill_count.saturating_sub(1));}if p&(1<<12)!=0{s.skill_row=s.skill_row.saturating_sub(POINTS_PAGE);}if p&1!=0{let tree=s.skill_rows[s.skill_row];if let Some(id)=activation(tree,s.totals.points[tree as usize]){show_effect(id);}}return;}
 if state().jewels{
 let s=mutable();let count=s.results[s.result].as_ref().map_or(0,|r|r.jewel_count as usize);
 if p&(1<<15)!=0{s.jewel_row=(s.jewel_row+1).min(count.saturating_sub(1));}if p&(1<<13)!=0{s.jewel_row=s.jewel_row.saturating_sub(1);}
 if p&(1<<14)!=0{s.jewel_row=(s.jewel_row+11).min(count.saturating_sub(1));}if p&(1<<12)!=0{s.jewel_row=s.jewel_row.saturating_sub(11);}
 if p&1!=0{open_jewel(u,s.jewel_row);}return;
 }if state().show_results{
 let s=mutable();if p&(1<<14)!=0&&s.count>0{s.result=(s.result+1).min(s.count-1);}if p&(1<<12)!=0{s.result=s.result.saturating_sub(1);}if p&(1<<15)!=0{s.piece=(s.piece+1).min(6);}if p&(1<<13)!=0{s.piece=s.piece.saturating_sub(1);}
 if p&(1<<4)!=0{show_skills();return;}
 if p&1!=0{if state().piece==6{show_skills();}else if state().piece==5{#[cfg(target_arch="arm")]show_skills();#[cfg(not(target_arch="arm"))]show_jewels();}else if let Some(id)=armor_id(state().piece){crate::enter(u,crate::catalog::ENTRIES.len()+id);}}
 if p&(1<<5)!=0{add_missing();}if p&(1<<2)!=0{run_toggle();}
 }else{if p&(1<<4)!=0&&u.count>0{show_effect(u.results[u.selected]as usize);return;}if p&(1<<15)!=0&&u.count>0{u.selected=(u.selected+1).min(u.count-1);}if p&(1<<13)!=0{u.selected=u.selected.saturating_sub(1);}if p&1!=0&&u.count>0{toggle(u.results[u.selected]as usize);}if p&(1<<2)!=0{start();}}
}

pub fn select_result(index:usize){browse();let s=mutable();s.result=index.min(s.count.saturating_sub(1));}
pub fn page(u:&mut Ui,down:bool){if state().effect.is_some(){scroll_effect(down,EFFECT_PAGE);}else if state().skills{let s=mutable();s.skill_row=if down{(s.skill_row+POINTS_PAGE).min(s.skill_count.saturating_sub(1))}else{s.skill_row.saturating_sub(POINTS_PAGE)};}else if state().jewels{let s=mutable();let count=s.results[s.result].as_ref().map_or(0,|r|r.jewel_count as usize);s.jewel_row=if down{(s.jewel_row+11).min(count.saturating_sub(1))}else{s.jewel_row.saturating_sub(11)};}else if state().show_results{let s=state();select_result(if down{s.result.saturating_add(1)}else{s.result.saturating_sub(1)});}else{u.selected=if down{u.selected.saturating_add(SKILL_PAGE).min(u.count.saturating_sub(1))}else{u.selected.saturating_sub(SKILL_PAGE)};}}

/// High bits label incomplete source coverage without discarding candidates.
pub fn unverified()->bool{let s=state();let Some(r)=s.results[s.result].as_ref()else{return false;};(0..5).any(|slot|s.armor_ids[slot][r.armor[slot]as usize]&0x8000!=0)||r.jewels[..r.jewel_count as usize].iter().any(|p|s.jewel_ids[p.jewel as usize]&0x8000!=0)}
pub fn show_jewels(){browse();let s=mutable();s.jewels=true;s.jewel_row=0;}
pub fn open_jewel(u:&mut Ui,index:usize){let s=state();if let Some(p)=s.results[s.result].as_ref().and_then(|r|r.jewels.get(index).filter(|_|index<r.jewel_count as usize)){let id=jewel_id(p.jewel as usize);crate::enter(u,data::DECO[id].entry as usize);}}

fn id(s:&State,r:&solver::Solution,slot:usize)->usize{(s.armor_ids[slot][r.armor[slot]as usize]&0x7fff)as usize}
fn evaluate(s:&State,r:&solver::Solution)->skills::Totals{
 let mut e=skills::Evaluator::new();let mut torso=0;
 for slot in 0..5{let a=&armor::ARMOR[id(s,r,slot)];for &(tree,p)in &armor::SKILLS[a.skill as usize..a.skill as usize+a.skill_count as usize]{let tree=data::ARMOR_TREES[tree as usize];e.add(slot,tree,i16::from(p));if slot!=1&&tree==data::TORSO_UP{torso+=1;}}}
 if r.charm>0{for p in player::snapshot().charms[r.charm as usize-1].skills{if p.tree!=0{e.add(5,p.tree,i16::from(p.points));}}}
 for p in &r.jewels[..r.jewel_count as usize]{let d=&data::DECO[(s.jewel_ids[p.jewel as usize]&0x7fff)as usize];for &(tree,v)in &data::POINTS[d.skill as usize..d.skill as usize+d.skill_count as usize]{e.add(p.gear as usize,tree,i16::from(v));}}
 e.set_torso_count(torso);e.finish()
}
fn slots_for(s:&State,r:&solver::Solution)->([u8;7],[u8;7]){let mut capacity=[0;7];let mut used=[0;7];for slot in 0..5{capacity[slot]=armor::ARMOR[id(s,r,slot)].slots;}if r.charm>0{capacity[5]=player::snapshot().charms[r.charm as usize-1].slots;}capacity[6]=s.weapon_slots;for p in &r.jewels[..r.jewel_count as usize]{used[p.gear as usize]+=data::DECO[(s.jewel_ids[p.jewel as usize]&0x7fff)as usize].slots;}(capacity,used)}
pub fn slots()->([u8;7],[u8;7]){let s=state();s.results.get(s.result).filter(|_|s.result<s.count).and_then(|r|r.as_ref()).map(|r|slots_for(s,r)).unwrap_or(([0;7],[0;7]))}
fn rank(s:&State,r:&solver::Solution)->[u32;12]{rank_with_totals(s,r,&evaluate(s,r))}
// Preview may supply weapon priorities; absent a profile, preserve released ranking. Unknown source coverage
// must not veto every jewel; those builds keep the explicit unverified label.
fn rank_with_totals(s:&State,r:&solver::Solution,all:&skills::Totals)->[u32;12]{let mut unknown=0;let mut missing=0;let mut defense=0;for slot in 0..5{unknown+=u32::from(s.armor_ids[slot][r.armor[slot]as usize]&0x8000!=0);missing+=u32::from(!owns_in(s,id(s,r,slot)));defense+=u32::from(armor::ARMOR[id(s,r,slot)].defense);}for p in &r.jewels[..r.jewel_count as usize]{unknown+=u32::from(s.jewel_ids[p.jewel as usize]&0x8000!=0);}
 let (negative,active,tiers,_,_,excess)=quality(all);let priority=s.priority.map_or_else(||{let k=skills::skill_rank(active,negative,tiers);[k[0],k[1],k[2],0,0,0]},|p|p.rank(all,data::ACTIVATIONS.iter().filter(|a|applicable(a.tree)).map(|a|(a.tree,a.points))));
 let(c,u)=slots_for(s,r);let total=c.iter().map(|&v|u32::from(v)).sum::<u32>();let used=u.iter().map(|&v|u32::from(v)).sum::<u32>();[priority[0],priority[1],priority[2],priority[3],priority[4],priority[5],unknown,excess,used,21-(total-used),missing,65535-defense]
}
/// Count only the strongest activation per tree; intermediate points guide the
/// bounded spare-slot search but never outrank an actual active skill.
fn quality(all:&skills::Totals)->(u32,u32,u32,u32,u32,u32){
 let mut positive=[0i16;206];let mut next=[i16::MAX;206];let mut negatives=[false;206];
 for a in data::ACTIVATIONS{if !applicable(a.tree){continue;}let tree=a.tree as usize;let p=all.points[tree];
  if a.points<0&&all.present[tree]&&p<=i16::from(a.points){negatives[tree]=true;}
  if a.points>0{let threshold=i16::from(a.points);if p>=threshold{positive[tree]=positive[tree].max(threshold);}else{next[tree]=next[tree].min(threshold);}}
 }
 let mut active=0;let mut tiers=0;let mut near=0;let mut sum=0;let mut excess=0;
 for tree in 1..206{if positive[tree]>0{active+=1;tiers+=positive[tree]as u32;excess+=u32::from(skills::tier_score(all.points[tree],positive[tree],core::iter::once(positive[tree])).1);}
  if next[tree]!=i16::MAX&&all.points[tree]>0{let progress=all.points[tree]as u32*1000/next[tree]as u32;near=near.max(progress);sum+=progress;}
 }
 (negatives.iter().filter(|&&v|v).count()as u32,active,tiers,near,sum,excess)
}
fn bonus_score(s:&State,r:&mut solver::Solution)->Option<[u32;13]>{
 let all=evaluate(s,r);
 if s.focus_task<=s.focus_count+1&&r.jewels[..r.jewel_count as usize].iter().any(|p|s.jewel_ids[p.jewel as usize]&0x8000!=0){return None;}
 if let Some(id)=s.focus{let a=&data::ACTIVATIONS[id];if all.points[a.tree as usize]<i16::from(a.points){return None;}}
 for i in 0..s.goal_count{let a=&data::ACTIVATIONS[s.goals[i]];let p=all.points[a.tree as usize];if a.points>0&&p<i16::from(a.points)||a.points<0&&p>i16::from(a.points){return None;}r.totals[i]=i32::from(p);}
 let (negative,active,tiers,near,sum,excess)=quality(&all);let priority=s.priority.map_or_else(||{let k=skills::skill_rank(active,negative,tiers);[k[0],k[1],k[2],0,0,0]},|p|p.rank(&all,data::ACTIVATIONS.iter().filter(|a|applicable(a.tree)).map(|a|(a.tree,a.points))));let ranked=rank_with_totals(s,r,&all);
 let (near,sum)=s.priority.map_or((near,sum),|p|(p.progress(&all,data::ACTIVATIONS.iter().filter(|a|applicable(a.tree)).map(|a|(a.tree,a.points))),0));
 Some([priority[0],priority[1],priority[2],priority[3],priority[4],priority[5],u32::MAX-near,u32::MAX-sum,ranked[6],excess,ranked[8],ranked[9],ranked[11]])
}

fn signature(all:&skills::Totals)->[i8;206]{skills::skill_signature(&all.points,data::ACTIVATIONS.iter().filter(|a|applicable(a.tree)).map(|a|(a.tree,a.points)))}
pub fn applicable(tree:u8)->bool{player::snapshot().hunter!=Some(0)||!(45..=69).contains(&tree)}
fn stop_search(s:&mut State,now:f64)->bool{
 let Some(reason)=s.search.poll(now)else{return false;};s.phase=Phase::Paused;
 s.status=match reason{skills::SearchStop::Variations=>"Found 5 different active-skill combinations. Y continues.",skills::SearchStop::Time=>"60-second search finished. Best variations so far; Y continues."};true
}
fn retain(s:&mut State,r:solver::Solution,score:[u32;12]){
 let key=signature(&evaluate(s,&r));
 // Keep the strongest representative of each active-skill combination.
 let previous=s.results[..s.count].iter().position(|old|old.as_ref().is_some_and(|old|signature(&evaluate(s,old))==key));
 if let Some(i)=previous{if s.ranks[i]<=score{return;}for j in i..s.count-1{s.results[j]=s.results[j+1];s.ranks[j]=s.ranks[j+1];}s.count-=1;}
 let at=(0..s.count).find(|&i|score<s.ranks[i]).unwrap_or(s.count);if at==LIMIT{return;}let end=s.count.min(LIMIT-1);for j in(at..end).rev(){s.results[j+1]=s.results[j];s.ranks[j+1]=s.ranks[j];}s.results[at]=Some(r);s.ranks[at]=score;s.count=(s.count+1).min(LIMIT);s.result=s.result.min(s.count-1);if previous.is_none(){s.search.observe(key);}
}
pub fn activation(tree:u8,p:i16)->Option<usize>{let mut selected=None;for(i,a)in data::ACTIVATIONS.iter().enumerate(){if a.tree==tree&&((a.points>0&&p>=i16::from(a.points))||(a.points<0&&p<=i16::from(a.points))){if selected.is_none_or(|old:usize|a.points.abs()>data::ACTIVATIONS[old].points.abs()){selected=Some(i);}}}selected}
fn active(tree:u8,p:i16)->bool{applicable(tree)&&(activation(tree,p).is_some()||tree==data::TORSO_UP&&p>0)}
pub fn skill_name(tree:u8)->&'static str{let p=state().totals.points[tree as usize];activation(tree,p).map(|i|data::ACTIVATIONS[i].name).unwrap_or(if tree==data::TORSO_UP&&p>0{"Torso Up"}else{"Not active"})}
pub fn target(tree:u8)->Option<i8>{let s=state();s.goals[..s.goal_count].iter().map(|&i|&data::ACTIVATIONS[i]).find(|a|a.tree==tree).map(|a|a.points)}
pub fn refresh_summary(){let s=mutable();let Some(r)=s.results.get(s.result).filter(|_|s.result<s.count).copied().flatten()else{return;};if s.cached==Some(r){return;}s.totals=evaluate(s,&r);s.cached=Some(r);s.skill_count=0;s.active_count=0;
 for tree in 1..206{if s.totals.present[tree]&&s.totals.points[tree]!=0{s.skill_rows[s.skill_count]=tree as u8;s.skill_count+=1;if s.totals.points[tree]>0&&active(tree as u8,s.totals.points[tree]){s.active_count+=1;}}}
 let goals=s.goals;let n=s.goal_count;let points=s.totals.points;s.skill_rows[..s.skill_count].sort_unstable_by_key(|&tree|{let requested=goals[..n].iter().any(|&i|data::ACTIVATIONS[i].tree==tree);let p=points[tree as usize];(if requested{0}else if active(tree,p)&&p<0{1}else if active(tree,p){2}else{3},tree)});s.skill_row=s.skill_row.min(s.skill_count.saturating_sub(1));
}
pub fn show_skills(){browse();refresh_summary();let s=mutable();s.skills=true;s.skill_row=0;}

#[cfg(target_arch="arm")]
pub fn saved_build()->Option<crate::set_saved::Saved>{
 let s=state();if !s.show_results{return None;}let r=s.results.get(s.result).filter(|_|s.result<s.count)?.as_ref()?;let mut out=crate::set_saved::Saved::new();
 let mut name=crate::Text::new();for(i,&goal)in s.goals[..s.goal_count].iter().enumerate(){let _=core::fmt::Write::write_fmt(&mut name,format_args!("{}{}",if i==0{""}else{" + "},data::ACTIVATIONS[goal].name));}
 if name.str().is_empty(){let _=core::fmt::Write::write_str(&mut name,"Saved build");}let mut end=name.str().len().min(48);while !name.str().is_char_boundary(end){end-=1;}out.set_name(&name.str()[..end]).ok()?;
 for slot in 0..5{out.armor[slot]=armor::ARMOR[id(s,r,slot)].key;}
 if r.charm>0{let c=&player::snapshot().charms[r.charm as usize-1];out.charm_box=c.box_index;out.charm_slots=c.slots;for i in 0..2{out.charm_trees[i]=c.skills[i].tree;out.charm_points[i]=c.skills[i].points;}}
 out.jewel_count=r.jewel_count;for(i,p)in r.jewels[..r.jewel_count as usize].iter().enumerate(){out.jewels[i]=crate::set_saved::SavedJewel{native:data::DECO[jewel_id(p.jewel as usize)].native,gear:p.gear};}
 out.goal_count=s.goal_count as u8;for(i,&goal)in s.goals[..s.goal_count].iter().enumerate(){out.goal_trees[i]=data::ACTIVATIONS[goal].tree;out.goal_points[i]=data::ACTIVATIONS[goal].points;}
 let p=s.priority.unwrap_or_else(crate::set_workbench::profile);out.weapon=[p.weapon,p.element,p.status,s.weapon_slots];Some(out)
}
#[cfg(target_arch="arm")]
pub fn restore_goals(build:&crate::set_saved::Saved){let s=mutable();s.goal_count=0;s.goals.fill(usize::MAX);for i in 0..build.goal_count as usize{if let Some(id)=data::ACTIVATIONS.iter().position(|a|a.tree==build.goal_trees[i]&&a.points==build.goal_points[i]){s.goals[s.goal_count]=id;s.goal_count+=1;}}}
#[cfg(target_arch="arm")]
pub fn restore_saved(build:&crate::set_saved::Saved)->bool{
 if build.validate().is_err()||!capture(mutable()){return false;}let mut ids=[0usize;5];let player=player::snapshot();
 for slot in 0..5{let Some(id)=armor::ARMOR.iter().position(|a|a.key==build.armor[slot]&&a.slot as usize==slot&&(a.hunter==2||Some(a.hunter)==player.hunter)&&(a.gender==2||Some(a.gender)==player.gender))else{return false;};ids[slot]=id;}
 let charm=if build.charm_box==65535{0}else{let Some(i)=player.charms[..player.charm_count].iter().position(|c|c.slots==build.charm_slots&&(0..2).all(|i|c.skills[i].tree==build.charm_trees[i]&&c.skills[i].points==build.charm_points[i]))else{return false;};i as u32+1};
 let mut r=solver::Solution{armor:[0;5],charm,jewels:[solver::Placement::default();solver::MAX_JEWELS],jewel_count:build.jewel_count,totals:[0;6]};
 for(i,j)in build.jewels[..build.jewel_count as usize].iter().enumerate(){let Some(id)=data::DECO.iter().position(|d|d.native==j.native)else{return false;};r.jewels[i]=solver::Placement{jewel:id as u32,gear:j.gear};}
 let s=mutable();for slot in 0..5{s.armor_count[slot]=1;s.armor_ids[slot][0]=ids[slot]as u16|if owns_in(s,ids[slot]){0}else{0x8000};}
 s.jewel_count=data::DECO.len();for i in 0..s.jewel_count{s.jewel_ids[i]=i as u16|0x8000;}
 s.weapon_slots=build.weapon[3];s.priority=Some(skills::PriorityProfile::new(build.weapon[0],build.weapon[1],build.weapon[2]));let(c,u)=slots_for(s,&r);if (0..7).any(|i|u[i]>c[i]){return false;}
 s.results.fill(None);s.results[0]=Some(r);s.count=1;s.result=0;s.show_results=true;s.cached=None;s.phase=Phase::Complete;s.status="Saved build. Run a new search to verify current progression.";
 restore_goals(build);refresh_summary();true
}
#[cfg(target_arch="arm")]
pub fn saved_totals(build:&crate::set_saved::Saved)->Option<skills::Totals>{
 let mut e=skills::Evaluator::new();let mut torso=0;
 for slot in 0..5{let a=armor::ARMOR.iter().find(|a|a.key==build.armor[slot]&&a.slot as usize==slot)?;for &(tree,p)in &armor::SKILLS[a.skill as usize..a.skill as usize+a.skill_count as usize]{let tree=data::ARMOR_TREES[tree as usize];e.add(slot,tree,i16::from(p));if slot!=1&&tree==data::TORSO_UP{torso+=1;}}}
 if build.charm_box!=65535{for i in 0..2{e.add(5,build.charm_trees[i],i16::from(build.charm_points[i]));}}
 for p in &build.jewels[..build.jewel_count as usize]{let d=data::DECO.iter().find(|d|d.native==p.native)?;for &(tree,value)in &data::POINTS[d.skill as usize..d.skill as usize+d.skill_count as usize]{e.add(p.gear as usize,tree,i16::from(value));}}
 e.set_torso_count(torso);Some(e.finish())
}
#[cfg(target_arch="arm")]
pub fn scroll_summary(down:bool){let s=mutable();s.skills=true;s.skill_row=if down{(s.skill_row+POINTS_PAGE).min(s.skill_count.saturating_sub(1))}else{s.skill_row.saturating_sub(POINTS_PAGE)};}
