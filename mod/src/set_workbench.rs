//! Dedicated native Set Builder navigation and persistence integration.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Screen{Search,Settings,Saved,SavedDetail}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Back{Stay,General(u8)}
pub struct Navigation{pub requirements:bool,pub priorities:bool,pub dropdown:Option<u8>,pub screen:Screen,pub group:Option<u8>,pub renaming:bool,pub return_tab:u8}
impl Navigation{
 pub const fn new()->Self{Self{requirements:false,priorities:false,dropdown:None,screen:Screen::Search,group:None,renaming:false,return_tab:0}}
 pub fn back(&mut self)->Back{
  if self.renaming{self.renaming=false;return Back::Stay;}
  if self.dropdown.take().is_some(){return Back::Stay;}
  if self.group.take().is_some(){return Back::Stay;}
  if self.priorities||self.requirements{self.priorities=false;self.requirements=false;return Back::Stay;}
  self.screen=match self.screen{Screen::SavedDetail=>Screen::Saved,Screen::Settings|Screen::Saved=>Screen::Search,Screen::Search=>return Back::General(self.return_tab)};
  Back::Stay
 }
}

#[cfg(not(test))]mod live{
 use super::*;
 use core::cell::UnsafeCell;
 use crate::{set_builder as builder,set_data as data,set_player as player,set_saved as saved,set_skills::PriorityProfile,Ui};
 pub const WEAPONS:[&str;15]=["Great Sword","Long Sword","Sword & Shield","Dual Blades","Hammer","Hunting Horn","Lance","Gunlance","Switch Axe","Charge Blade","Insect Glaive","Light Bowgun","Heavy Bowgun","Bow","Unknown"];
 pub struct State{pub choice:usize,pub nav:Navigation,pub row:usize,pub notice:&'static str,pub slots_override:i8,pub selected_saved:usize,pub saved_context:bool,pub compare:Option<saved::Saved>,pub comparing:bool,old_query:[u8;48],old_len:usize}
 struct Local(UnsafeCell<State>);unsafe impl Sync for Local{}
 static STATE:Local=Local(UnsafeCell::new(State{choice:0,nav:Navigation::new(),row:0,notice:"",slots_override:-1,selected_saved:0,saved_context:false,compare:None,comparing:false,old_query:[0;48],old_len:0}));
 pub fn state()->&'static State{unsafe{&*STATE.0.get()}}
 fn mutable()->&'static mut State{unsafe{&mut*STATE.0.get()}}
 pub fn enter(previous:u8){saved::load();let s=mutable();s.nav=Navigation::new();s.nav.return_tab=if previous<=5{previous}else{0};s.row=0;s.saved_context=false;s.notice="";}
 pub fn search_started(){let s=mutable();s.nav.screen=Screen::Search;s.saved_context=false;s.notice="";}
 pub fn allows_text()->bool{state().nav.renaming||state().nav.screen==Screen::Search&&!builder::state().show_results}
 pub fn slots()->u8{if state().slots_override>=0{state().slots_override as u8}else{player::snapshot().weapon_slots.unwrap_or(0)}}
 pub fn profile()->PriorityProfile{let p=player::snapshot();let prefs=saved::preferences();let mut out=PriorityProfile::new(p.weapon_type.unwrap_or(14),p.element.unwrap_or(0),p.status_element.unwrap_or(0));out.order=prefs.order;out.trees=prefs.trees;out.groups=prefs.groups;out}
 pub fn tab(index:usize){builder::browse();builder::dismiss_details();let s=mutable();s.nav.screen=match index{1=>Screen::Settings,2=>Screen::Saved,_=>Screen::Search};s.nav.group=None;s.nav.dropdown=None;s.nav.requirements=false;s.nav.priorities=false;s.row=0;s.saved_context=false;s.comparing=false;}
 pub fn back(u:&mut Ui)->bool{
  let s=mutable();if s.comparing{s.comparing=false;return true;}
  if s.nav.renaming{u.query=s.old_query;u.len=s.old_len;u.pc_search=false;u.typing=false;u.dirty=true;}
  if s.nav.screen!=Screen::Search||s.nav.renaming||s.nav.group.is_some(){let dropdown=s.nav.dropdown.is_some();s.nav.back();if !dropdown{s.row=0;}return true;}
  false
 }
 pub fn leave(u:&mut Ui){builder::browse();u.builder=false;u.tab=state().nav.return_tab;u.detail=usize::MAX;u.depth=0;u.len=0;u.selected=0;u.dirty=true;}
 fn save_preferences(p:saved::Preferences){if saved::set_preferences(p){mutable().notice="Settings saved. Start a new search to apply them.";}else{mutable().notice="Settings save failed; previous settings retained.";}}
 pub fn group_skills(group:u8,rows:&mut[u8;206])->usize{
  let p=profile();let mut classes=[0u8;206];
  for a in data::ACTIVATIONS{if a.points>0&&builder::applicable(a.tree){classes[a.tree as usize]=classes[a.tree as usize].max(p.activation_preference(a.tree,i16::from(a.points)).0);}}
  let mut count=0;for t in 1..206{if classes[t]==4-group{rows[count]=t as u8;count+=1;}}
  rows[..count].sort_unstable_by_key(|&t|(core::cmp::Reverse(p.preference(t).1),data::TREE_NAMES[t as usize]));count
 }
 pub fn open_section(priorities:bool){let s=mutable();s.nav.requirements=!priorities;s.nav.priorities=priorities;s.nav.group=None;s.row=0;}
 pub fn dropdown_count(field:u8)->usize{match field{0=>12,1=>13,_=>5}}
 pub fn open_dropdown(field:u8){let prefs=saved::preferences();let choice=match field{0=>if prefs.village==255{0}else{usize::from(prefs.village)+1},1=>if prefs.hub==255{0}else{usize::from(prefs.hub)+1},3=>(state().slots_override+1)as usize,_=>0};let s=mutable();s.nav.dropdown=Some(field);s.choice=choice;}
 pub fn choose(index:usize){let Some(field)=state().nav.dropdown else{return;};if index>=dropdown_count(field){return;}let mut prefs=saved::preferences();
  match field{0=>prefs.village=if index==0{255}else{index as u8-1},1=>prefs.hub=if index==0{255}else{index as u8-1},3=>{let s=mutable();s.slots_override=index as i8-1;s.nav.dropdown=None;s.notice="Weapon sockets apply to the next search.";return;},4=>{let Some(group)=state().nav.group else{return;};let mut rows=[0;206];let n=group_skills(group,&mut rows);if state().row>=n{return;}let tree=rows[state().row];prefs.groups[tree as usize]=if index==0{0}else{prefs.order[index-1]+1};},_=>return}
  save_preferences(prefs);mutable().nav.dropdown=None;if field==4{let mut rows=[0;206];let n=state().nav.group.map_or(0,|g|group_skills(g,&mut rows));mutable().row=state().row.min(n.saturating_sub(1));}
 }
 pub fn reset(){if saved::set_preferences(saved::Preferences::new()){let s=mutable();s.slots_override=-1;s.notice="Set Builder settings reset.";}else{mutable().notice="Settings save failed; previous settings retained.";}}
 pub fn setting(row:usize){
  if state().nav.priorities{open_group(row);}
  else if state().nav.requirements{if row==2{let mut p=saved::preferences();p.owned_only=!p.owned_only;save_preferences(p);}else if row<4{open_dropdown(row as u8);}}
  else{match row{0=>open_section(false),1=>open_section(true),_=>reset()}}
 }
 pub fn open_group(row:usize){if row<4{let s=mutable();s.nav.group=Some(saved::preferences().order[row]);s.row=0;}}
 pub fn rename_start(u:&mut Ui,index:usize){let Some(build)=saved::entries().get(index)else{return;};let s=mutable();s.old_query=u.query;s.old_len=u.len;s.nav.renaming=true;s.selected_saved=index;u.query=build.name;u.len=usize::from(build.name_len);u.editor.end(u.len,false);u.pc_search=true;u.typing=false;u.dirty=true;}
 pub fn rename_finish(u:&mut Ui){let name=core::str::from_utf8(&u.query[..u.len]).unwrap_or("");if saved::rename(state().selected_saved,name){mutable().notice="Build renamed.";back(u);}else{mutable().notice="Could not rename build. Use a nonempty name up to 48 bytes.";}}
 pub fn save_current(){let Some(build)=builder::saved_build()else{mutable().notice="No complete build to save.";return;};mutable().notice=if saved::add(build){"Build saved."}else{"Could not save build. Check storage or the 12-build limit."};}
 pub fn remove(index:usize){if saved::remove(index){let s=mutable();s.row=s.row.min(saved::entries().len().saturating_sub(1));s.notice="Saved build deleted.";s.nav.screen=Screen::Saved;}else{mutable().notice="Could not delete saved build.";}}
 pub fn open_saved(index:usize){let Some(build)=saved::entries().get(index).copied()else{return;};if builder::restore_saved(&build){let s=mutable();s.nav.screen=Screen::SavedDetail;s.selected_saved=index;s.saved_context=true;s.notice="Saved loadout. Availability must be revalidated with a new search.";}else{mutable().notice="Build data or its required charm is no longer available on this character.";}}
 pub fn revalidate(u:&mut Ui){let Some(build)=saved::entries().get(state().selected_saved).copied()else{return;};builder::restore_goals(&build);tab(0);builder::start();u.dirty=true;}
 pub fn compare_current(){let build=builder::saved_build();let s=mutable();if s.compare.is_none(){s.compare=build;s.notice="Build pinned for comparison. Choose another result and Compare again.";}else{s.comparing=true;builder::browse();}}
 pub fn clear_comparison(){let s=mutable();s.compare=None;s.comparing=false;s.notice="Comparison cleared.";}
 pub fn command(u:&mut Ui,cmd:u8,index:usize){match cmd{
  0=>tab(index),1=>mutable().row=index,2=>setting(index),3=>open_group(index),5=>save_current(),6=>open_saved(index),7=>rename_start(u,index),8=>remove(index),9=>rename_finish(u),10=>{back(u);},11=>compare_current(),12=>clear_comparison(),13=>revalidate(u),14=>builder::scroll_summary(index!=0),15=>mutable().row=index,16=>{u.pc_search=false;u.typing=true;},18=>{mutable().saved_context=false;builder::run_toggle();u.dirty=true;},19=>builder::add_missing(),20=>open_section(index!=0),21=>open_dropdown(index as u8),22=>choose(index),23=>setting(index),25=>{mutable().row=index;open_dropdown(4);},26=>reset(),_=>{}
 }}
 pub fn input(u:&mut Ui,p:u32)->bool{
  if let Some(field)=state().nav.dropdown{let n=dropdown_count(field);let s=mutable();if p&(1<<15)!=0{s.choice=(s.choice+1).min(n-1);}if p&(1<<13)!=0{s.choice=s.choice.saturating_sub(1);}if p&(1<<14)!=0{s.choice=(s.choice+7).min(n-1);}if p&(1<<12)!=0{s.choice=s.choice.saturating_sub(7);}if p&1!=0{choose(state().choice);}return true;}

  if p&((1<<6)|(1<<7))!=0{let index=match state().nav.screen{Screen::Search=>0,Screen::Settings=>1,_=>2};tab(if p&(1<<7)!=0{(index+1)%3}else{(index+2)%3});return true;}
  if state().nav.renaming{if p&1!=0{rename_finish(u);}return true;}
  if state().comparing{return true;}
  match state().nav.screen{
   Screen::Settings=>{let group=state().nav.group;let mut rows=[0;206];let n=group.map_or(if state().nav.requirements||state().nav.priorities{4}else{3},|g|group_skills(g,&mut rows));
    if p&(1<<15)!=0{mutable().row=(state().row+1).min(n.saturating_sub(1));}if p&(1<<13)!=0{mutable().row=state().row.saturating_sub(1);}
    if group.is_some(){if p&1!=0{open_dropdown(4);}}
    else{if p&1!=0{setting(state().row);}}true},
   Screen::Saved=>{let n=saved::entries().len();if p&(1<<15)!=0{mutable().row=(state().row+1).min(n.saturating_sub(1));}if p&(1<<13)!=0{mutable().row=state().row.saturating_sub(1);}if p&1!=0{open_saved(state().row);}if p&(1<<2)!=0{rename_start(u,state().row);}if p&(1<<5)!=0{remove(state().row);}true},
   Screen::Search|Screen::SavedDetail=>{if !builder::state().show_results{return false;}if p&(1<<8)!=0{save_current();}if p&(1<<9)!=0{compare_current();}false},
  }
 }
}
#[cfg(not(test))]pub use live::*;
