//! Full-window Set Builder. Uses the same native frames, font and socket icons.
use super::*;
use crate::{set_builder as builder,set_data as data,set_player as player,set_saved as saved,set_workbench as bench};
use bench::Screen;

fn button(c:&Canvas,x:f32,y:f32,w:f32,label:&str,cmd:u8,index:usize,selected:bool){c.card(x,y,w,34.0,selected);c.hint(x+10.0,y+8.0,label,TEXT,16.0);c.hit(x,y,w,34.0,Action::Workbench(cmd,index));}
// Face buttons use native artwork; shoulder/stick glyphs share its atlas.
// Keep key labels separate from action labels so both remain readable/clickable.
fn prompt(c:&Canvas,key:usize,x:f32,y:f32)->f32{
 if c.desktop{let label=["F9","Ctrl+F","Esc","Enter","Tab","Tab","[","]","F10","F8"][key];c.text(x,y+3.0,label,GOLD,14.0);return crate::game_ui::width(14.0,label)+10.0;}
 let p=c.v(x,y);let rect=[p.x,p.y,24.0*c.s,24.0*c.s];
 if key<4{crate::game_ui::button(&c.d,key,rect);}else{crate::game_ui::control(&c.d,key-4,rect);if key<8{let label=["LB","RB","LT","RT"][key-4];let width=crate::game_ui::width(11.0,label);c.text(x+(24.0-width)/2.0,y+7.0,label,0xff16120e,11.0);}}
 32.0
}
fn action_button(c:&Canvas,x:f32,y:f32,w:f32,label:&str,cmd:u8,index:usize,key:usize){
 c.card(x,y,w,34.0,false);let inset=prompt(c,key,x+8.0,y+5.0);c.text(x+8.0+inset,y+9.0,label,TEXT,16.0);c.hit(x,y,w,34.0,Action::Workbench(cmd,index));
}
fn profile(c:&Canvas){let p=player::snapshot();let result=builder::state().show_results&&matches!(bench::state().nav.screen,Screen::Search|Screen::SavedDetail);let weapon=if result{builder::state().priority.map_or(14,|p|p.weapon)}else{p.weapon_type.unwrap_or(14)};let slots=if result{builder::weapon_slots()}else{bench::slots()};let mut line=Text::new();let _=write!(line,"{}  {}  HR {}   {}   {} weapon slots   {} owned charms",if p.gender==Some(1){"Female"}else{"Male"},if p.hunter==Some(1){"Gunner"}else{"Blademaster"},p.hr.unwrap_or(0),bench::WEAPONS[weapon as usize],slots,p.charm_count);c.text(30.0,119.0,line.str(),GOLD,17.0);}

pub(super)fn draw(c:&Canvas,u:&mut Ui){
 c.rect(0.0,0.0,1280.0,800.0,BG);c.frame(0,0.0,0.0,1280.0,800.0,u32::MAX);
 let p=c.v(8.0,6.0);crate::game_ui::title(&c.d,[p.x,p.y,1264.0*c.s,39.0*c.s]);
 c.text(52.0,16.0,"Set Builder",TEXT,21.0);let back=prompt(c,2,305.0,12.0);c.text(305.0+back,17.0,"Back",TEXT,15.0);c.hit(305.0,5.0,95.0,35.0,Action::Back);
 let screen=bench::state().nav.screen;let active=match screen{Screen::Search=>0,Screen::Settings=>1,_=>2};
 if !c.desktop{prompt(c,4,28.0,68.0);}prompt(c,5,650.0,68.0);
 for(i,name)in["Search","Settings","Saved builds"].iter().enumerate(){button(c,65.0+i as f32*192.0,63.0,178.0,name,0,i,i==active);}
 if u.typing||u.pc_search{c.text(740.0,73.0,if bench::state().nav.renaming{"Editing build name"}else{"Typing searches directly"},MUTED,15.0);}
 profile(c);c.line(20.0,149.0,1260.0,149.0,EDGE);c.clip(20.0,154.0,1240.0,590.0);
 if bench::state().nav.renaming{name_editor(c,u);}
 else if let Some(id)=builder::state().effect{effect(c,id);}
 else if bench::state().comparing{comparison(c);}
 else{match screen{Screen::Settings=>settings(c),Screen::Saved=>saved(c),Screen::Search|Screen::SavedDetail=>{if builder::state().show_results{result(c);}else if u.typing{keyboard(c,u);}else{choose(c,u);}}}}
 c.endclip();desktop::apply(u,c.action.get());
}

fn choose(c:&Canvas,u:&Ui){
 c.text(32.0,170.0,"Required skills (optional)",CYAN,23.0);let s=builder::state();
 for(i,&id)in s.goals[..s.goal_count].iter().enumerate(){let y=217.0+i as f32*61.0;c.card(30.0,y,322.0,53.0,false);c.clip(39.0,y,304.0,52.0);c.text(42.0,y+8.0,data::ACTIVATIONS[id].name,GOLD,16.0);c.text(42.0,y+31.0,"Minimum tier; stronger tiers qualify",MUTED,11.0);c.endclip();c.hit(30.0,y,322.0,53.0,Action::BuilderSelect(id));}
 if s.goal_count==0{c.paragraph(34.0,228.0,300.0,"No required skills: recommend builds using your weapon and skill priorities. All available armor and owned charms are considered.",MUTED,17.0,0,5);}
 action_button(c,30.0,638.0,322.0,"Search available equipment",18,0,0);
 c.paragraph(34.0,689.0,315.0,s.status,MUTED,13.0,0,3);
 let query=core::str::from_utf8(&u.query[..u.len]).unwrap_or("");c.card(382.0,164.0,864.0,37.0,u.pc_search);c.text(395.0,174.0,if query.is_empty(){"Type to search skills..."}else{query},MUTED,17.0);c.hit(382.0,164.0,864.0,37.0,Action::Search);
 let start=u.selected/builder::SKILL_PAGE*builder::SKILL_PAGE;let mut buf=[0;4096];
 for(i,&id)in u.results[..u.count].iter().enumerate().skip(start).take(builder::SKILL_PAGE){let id=id as usize;let a=&data::ACTIVATIONS[id];let y=217.0+(i-start)as f32*68.0;c.card(382.0,y,864.0,60.0,i==u.selected);c.text(397.0,y+9.0,if builder::selected(id){"[x]"}else{"[ ]"},GOLD,17.0);c.text(438.0,y+8.0,a.name,TEXT,18.0);c.paragraph(438.0,y+32.0,790.0,skill_effect(id,&mut buf),MUTED,12.0,0,1);c.hit(382.0,y,864.0,60.0,Action::BuilderSelect(id));}
}

fn result(c:&Canvas){
 builder::refresh_summary();let s=builder::state();let Some(r)=s.results.get(s.result).filter(|_|s.result<s.count).and_then(|r|r.as_ref())else{c.paragraph(34.0,226.0,1195.0,s.status,TEXT,20.0,0,4);action_button(c,810.0,690.0,433.0,"Search / resume",18,0,0);return;};
 let mut head=Text::new();let _=write!(head,"Build {} / {}   {} active skills   {:.1} / 60s",s.result+1,s.count,s.active_count,s.search.elapsed);c.text(32.0,169.0,"←",GOLD,21.0);c.text(63.0,169.0,head.str(),CYAN,21.0);
 c.hit(28.0,162.0,32.0,32.0,Action::BuilderResult(s.result.saturating_sub(1)));c.hit(1160.0,162.0,85.0,32.0,Action::BuilderResult((s.result+1).min(s.count-1)));
 c.text(1188.0,170.0,"→",GOLD,21.0);let mut progress=Text::new();let _=write!(progress,"{}   {} steps   {} / 5 skill variations",s.status,s.visited,s.search.variations);c.text(34.0,194.0,progress.str(),MUTED,11.0);let(capacity,used)=builder::slots();c.card(28.0,207.0,620.0,447.0,false);c.card(665.0,207.0,581.0,447.0,false);
 c.text(42.0,214.0,"Pieces, sockets & jewels",GOLD,17.0);let focus=prompt(c,8,680.0,210.0);c.text(680.0+focus,214.0,"Skills from this build",GOLD,17.0);
 let mut defense=0;let mut missing=0;
 for slot in 0..7{let y=246.0+slot as f32*57.0;let mut name=Text::new();let label=if slot<5{crate::armor::SLOTS[slot]}else if slot==5{"Charm"}else{"Weapon"};if !s.skills&&s.piece==slot{c.rect(38.0,y-4.0,600.0,52.0,0xff274653);}
  c.text(42.0,y,label,MUTED,12.0);
  let owned=if slot<5{let Some(id)=builder::armor_id(slot)else{continue;};let a=&crate::armor_data::ARMOR[id];defense+=u32::from(a.defense);let owned=builder::owns(id);if !owned{missing+=1;}let _=write!(name,"{}",a.name);owned}else if slot==5{if r.charm==0{let _=write!(name,"No charm required");}else{let a=&player::snapshot().charms[r.charm as usize-1];for(i,k)in a.skills.iter().filter(|s|s.tree!=0).enumerate(){let _=write!(name,"{}{} {:+}",if i==0{""}else{" / "},data::TREE_NAMES[k.tree as usize],k.points);}}true}else{let _=write!(name,"{}",bench::WEAPONS[s.priority.map_or(14,|p|p.weapon)as usize]);true};
  c.clip(98.0,y-3.0,405.0,25.0);c.text(100.0,y,name.str(),TEXT,15.0);c.endclip();c.text(506.0,y,if slot==6{"Profile"}else if owned{"Owned"}else{"Missing"},if owned{0xff9ecfb6}else{GOLD},12.0);set_slots(c,572.0,y,capacity[slot],used[slot]);
  let mut jewels=Text::new();for p in r.jewels[..r.jewel_count as usize].iter().filter(|p|p.gear as usize==slot){let d=&data::DECO[builder::jewel_id(p.jewel as usize)];if !jewels.str().is_empty(){let _=write!(jewels," / ");}let _=write!(jewels,"{}",catalog::ENTRIES[d.entry as usize].name);}
  c.paragraph(100.0,y+23.0,525.0,if jewels.str().is_empty(){if capacity[slot]==0{"No sockets"}else{"Empty sockets"}}else{jewels.str()},MUTED,11.0,0,2);if slot<5{if let Some(id)=builder::armor_id(slot){c.hit(40.0,y-4.0,594.0,52.0,Action::Armor(id));}}if slot<6{c.line(40.0,y+50.0,634.0,y+50.0,EDGE);}
 }
 let mut buf=[0;4096];let start=s.skill_row/builder::POINTS_PAGE*builder::POINTS_PAGE;let mut y=246.0;
 for(i,&tree)in s.skill_rows[..s.skill_count].iter().enumerate().skip(start).take(builder::POINTS_PAGE){let points=s.totals.points[tree as usize];let active=builder::activation(tree,points);let h=if active.is_some(){67.0}else{35.0};if s.skills&&i==s.skill_row{c.rect(675.0,y-3.0,559.0,h,0xff274653);}
  let threshold=active.map(|id|data::ACTIVATIONS[id].points).unwrap_or(if points<0{-10}else{10});let mut line=Text::new();let _=write!(line,"{}  {} / {}",if active.is_some(){builder::skill_name(tree)}else{data::TREE_NAMES[tree as usize]},points,threshold);
  c.clip(681.0,y-1.0,545.0,h-1.0);c.text(683.0,y,line.str(),if points<0{0xff8888ee}else if active.is_some(){TEXT}else{MUTED},16.0);if let Some(id)=active{c.paragraph(683.0,y+23.0,538.0,skill_effect(id,&mut buf),MUTED,12.0,0,2);c.hit(675.0,y,559.0,h,Action::BuilderEffect(id));}c.endclip();y+=h;
 }
 let mut points=Text::new();let _=write!(points,"All {} skill trees   Rows {}-{}",s.skill_count,start+1,(start+builder::POINTS_PAGE).min(s.skill_count));c.text(682.0,617.0,points.str(),MUTED,12.0);c.hit(665.0,207.0,581.0,28.0,Action::BuilderSkills);button(c,1110.0,610.0,53.0,"<",14,0,false);button(c,1171.0,610.0,53.0,">",14,1,false);
 let mut summary=Text::new();let _=write!(summary,"Base defense {}   Sockets {} / {} used   {} armor pieces missing",defense,used.iter().map(|&v|u32::from(v)).sum::<u32>(),capacity.iter().map(|&v|u32::from(v)).sum::<u32>(),missing);c.text(33.0,665.0,summary.str(),MUTED,14.0);
 if bench::state().nav.screen==Screen::SavedDetail{button(c,30.0,694.0,177.0,"Rename",7,bench::state().selected_saved,false);button(c,220.0,694.0,177.0,"Search to revalidate",13,0,false);}else{action_button(c,30.0,694.0,177.0,"Save build",5,0,6);action_button(c,220.0,694.0,177.0,"Compare",11,0,7);}
 action_button(c,410.0,694.0,360.0,"Add missing armor to wishlist",19,0,9);action_button(c,810.0,694.0,433.0,"Pause / continue search",18,0,0);
 c.text(33.0,734.0,if !bench::state().notice.is_empty(){bench::state().notice}else if bench::state().saved_context{"Saved loadout: run search to revalidate availability."}else if crate::set_unlock::planning(){"Planning override: recipe and quest unlock flags are unverified."}else if builder::unverified(){"Some equipment availability is unverified."}else{"Native progression availability verified."},GOLD,12.0);
}

fn setting_row(c:&Canvas,y:f32,label:&str,value:&str,row:usize,selected:bool,dropdown:bool){
 c.card(32.0,y,1212.0,68.0,selected);c.text(55.0,y+22.0,label,TEXT,21.0);c.text(585.0,y+23.0,value,GOLD,20.0);
 if selected{prompt(c,3,1144.0,y+21.0);}
 if dropdown{c.line(1192.0,y+29.0,1203.0,y+39.0,GOLD);c.line(1203.0,y+39.0,1214.0,y+29.0,GOLD);}
 c.hit(32.0,y,1212.0,68.0,Action::Workbench(23,row));
}
fn settings(c:&Canvas){
 let s=bench::state();let prefs=saved::preferences();let mut rows=[0;206];
 if let Some(field)=s.nav.dropdown{
  c.text(35.0,175.0,match field{0=>"Village progression",1=>"Hub progression",3=>"Weapon sockets",_=>"Assign skill priority"},CYAN,25.0);
  c.text(35.0,213.0,"Choose an option. Back cancels without changing it.",MUTED,16.0);
  for i in 0..bench::dropdown_count(field){let x=35.0+(i/7)as f32*610.0;let y=258.0+(i%7)as f32*60.0;let mut name=Text::new();
   if i==0{let _=write!(name,"{}",if field==4{"Default priority"}else{"Automatic"});}
   else if field==4{let _=write!(name,"Priority {}",i);}
   else if field==3{let _=write!(name,"{} sockets",i-1);}
   else if field==1&&i>=9{let _=write!(name,"G {} stars",i-8);}
   else{let _=write!(name,"{} stars",i-1);}
   c.card(x,y,590.0,50.0,i==s.choice);c.text(x+20.0,y+13.0,name.str(),TEXT,21.0);if i==s.choice{prompt(c,3,x+540.0,y+13.0);}c.hit(x,y,590.0,50.0,Action::Workbench(22,i));
  }
 }else if let Some(group)=s.nav.group{
  let priority=prefs.order.iter().position(|&g|g==group).unwrap_or(0)+1;let mut title=Text::new();let _=write!(title,"Priority {}",priority);c.text(35.0,174.0,title.str(),CYAN,25.0);
  c.text(35.0,211.0,"Select a skill to move it to another priority.",MUTED,17.0);
  let n=bench::group_skills(group,&mut rows);let start=s.row/6*6;
  for(i,&tree)in rows[..n].iter().enumerate().skip(start).take(6){let y=256.0+(i-start)as f32*70.0;c.card(32.0,y,1212.0,62.0,i==s.row);let mut label=Text::new();let _=write!(label,"{}. {}",i+1,data::TREE_NAMES[tree as usize]);c.text(52.0,y+20.0,label.str(),TEXT,21.0);c.hit(32.0,y,730.0,62.0,Action::Workbench(25,i));if i==s.row{action_button(c,850.0,y+14.0,365.0,"Move to priority...",25,i,3);}else{button(c,850.0,y+14.0,365.0,"Move to priority...",25,i,false);}}
 }else if s.nav.priorities{
  c.text(35.0,174.0,"Skill Priorities",CYAN,25.0);c.text(35.0,214.0,"Priority 1 is highest. Open a list to move individual skills between priorities.",MUTED,17.0);
  for(i,&group)in prefs.order.iter().enumerate(){let y=263.0+i as f32*106.0;let n=bench::group_skills(group,&mut rows);c.card(32.0,y,1212.0,92.0,i==s.row);let mut name=Text::new();let _=write!(name,"Priority {}",i+1);c.text(53.0,y+14.0,name.str(),GOLD,23.0);let mut preview=Text::new();for &tree in rows[..n.min(5)].iter(){if !preview.str().is_empty(){let _=write!(preview,", ");}let _=write!(preview,"{}",data::TREE_NAMES[tree as usize]);}c.paragraph(54.0,y+49.0,960.0,preview.str(),MUTED,16.0,0,1);let mut count=Text::new();let _=write!(count,"{} skills",n);c.text(1090.0,y+19.0,count.str(),MUTED,17.0);if i==s.row{prompt(c,3,1195.0,y+14.0);}c.hit(32.0,y,1212.0,92.0,Action::Workbench(23,i));}
 }else if s.nav.requirements{
  c.text(35.0,174.0,"Requirements",CYAN,25.0);c.text(35.0,215.0,"Automatic follows your character. Overrides change the search, not your save.",MUTED,17.0);
  for row in 0..4{let mut value=Text::new();match row{0=>{if prefs.village==255{let _=write!(value,"Automatic");}else{let _=write!(value,"{} stars",prefs.village);}},1=>{if prefs.hub==255{let _=write!(value,"Automatic");}else if prefs.hub>=8{let _=write!(value,"G {} stars",prefs.hub-7);}else{let _=write!(value,"{} stars",prefs.hub);}},2=>{},_=>{if s.slots_override<0{let _=write!(value,"Automatic ({} sockets)",bench::slots());}else{let _=write!(value,"{} sockets",bench::slots());}}}
   let y=265.0+row as f32*89.0;setting_row(c,y,["Village stars","Hub stars","Owned armor only","Weapon slots"][row],value.str(),row,row==s.row,row!=2);
   if row==2{c.frame(1,1180.0,y+18.0,34.0,34.0,if prefs.owned_only{GOLD}else{EDGE});if prefs.owned_only{c.line(1187.0,y+34.0,1195.0,y+43.0,GOLD);c.line(1195.0,y+43.0,1209.0,y+25.0,GOLD);}}
  }
  c.paragraph(40.0,650.0,1170.0,"Star overrides are for planning: special quest and recipe unlocks still need checking. Set both star fields to Automatic to use your character's current unlocks throughout.",MUTED,15.0,0,3);
 }else{
  c.text(35.0,174.0,"Set Builder settings",CYAN,25.0);
  setting_row(c,257.0,"Requirements","Progression, ownership and sockets",0,s.row==0,true);
  setting_row(c,360.0,"Skill Priorities","Edit Priority 1, 2, 3 and 4",1,s.row==1,true);
  c.card(32.0,539.0,1212.0,68.0,s.row==2);c.text(55.0,560.0,"Reset Set Builder Settings",TEXT,22.0);c.hit(32.0,539.0,1212.0,68.0,Action::Workbench(26,0));c.text(42.0,633.0,"Resets search settings and skill priorities. Saved builds are kept.",MUTED,17.0);
 }
 c.text(35.0,729.0,if saved::failed(){"Could not save settings. Previous data retained."}else{s.notice},GOLD,13.0);
}

fn saved(c:&Canvas){let s=bench::state();c.text(32.0,171.0,"Saved builds",CYAN,23.0);if saved::entries().is_empty(){c.text(35.0,235.0,"Save a build from the results screen to keep its complete loadout.",MUTED,18.0);return;}let start=s.row/6*6;
 for(i,b)in saved::entries().iter().enumerate().skip(start).take(6){let y=218.0+(i-start)as f32*73.0;c.card(30.0,y,1214.0,64.0,i==s.row);c.clip(43.0,y+5.0,690.0,54.0);c.text(47.0,y+9.0,b.name(),TEXT,19.0);let mut line=Text::new();let _=write!(line,"{} / {} weapon slots / {} jewel placements",bench::WEAPONS[b.weapon[0]as usize],b.weapon[3],b.jewel_count);c.text(47.0,y+36.0,line.str(),MUTED,13.0);c.endclip();c.hit(30.0,y,715.0,64.0,Action::Workbench(6,i));if i==s.row{action_button(c,758.0,y+15.0,143.0,"Open",6,i,3);action_button(c,912.0,y+15.0,143.0,"Rename",7,i,0);action_button(c,1066.0,y+15.0,160.0,"Delete",8,i,9);}else{button(c,758.0,y+15.0,143.0,"Open",6,i,false);button(c,912.0,y+15.0,143.0,"Rename",7,i,false);button(c,1066.0,y+15.0,160.0,"Delete",8,i,false);}}
 c.paragraph(35.0,692.0,1195.0,"Loadouts retain stable equipment IDs. Opening checks armor type, charm ownership and socket capacity. Run a fresh search to revalidate current unlocks.",MUTED,14.0,0,2);c.text(35.0,734.0,s.notice,GOLD,12.0);
}

fn name_editor(c:&Canvas,u:&Ui){c.text(35.0,170.0,"Rename saved build",CYAN,24.0);let mut name=Text::new();let cursor=u.editor.cursor.min(u.len);let _=write!(name,"{}|{}",core::str::from_utf8(&u.query[..cursor]).unwrap_or(""),core::str::from_utf8(&u.query[cursor..u.len]).unwrap_or(""));c.card(40.0,216.0,1190.0,45.0,true);c.text(55.0,227.0,name.str(),TEXT,21.0);c.hit(40.0,216.0,1190.0,45.0,Action::Search);
 if u.typing{for(i,k)in crate::search::KEYS.iter().enumerate(){let(col,row)=crate::search::position(i);let x=100.0+col as f32*106.0;let y=292.0+row as f32*57.0;let w=if row==4{190.0}else{94.0};c.card(x,y,w,47.0,i==u.key);c.text(x+13.0,y+11.0,k,TEXT,22.0);c.hit(x,y,w,47.0,Action::SearchKey(i));}}
 action_button(c,40.0,658.0,275.0,"Save name",9,0,3);action_button(c,331.0,658.0,275.0,"Controller keyboard",16,0,1);action_button(c,622.0,658.0,275.0,"Cancel",10,0,2);c.text(43.0,719.0,bench::state().notice,MUTED,14.0);
}
fn effect(c:&Canvas,id:usize){let mut buf=[0;4096];c.text(34.0,174.0,data::ACTIVATIONS[id].name,CYAN,24.0);let n=c.paragraph(35.0,229.0,1190.0,skill_effect(id,&mut buf),TEXT,19.0,builder::state().effect_line,builder::EFFECT_PAGE);builder::effect_lines(n);}

fn comparison(c:&Canvas){let Some(first)=bench::state().compare.as_ref()else{return;};let Some(second)=builder::saved_build()else{return;};c.text(33.0,170.0,"Compare complete loadouts",CYAN,24.0);
 for(col,b)in[first,&second].iter().enumerate(){let x=32.0+col as f32*615.0;c.card(x,212.0,596.0,455.0,false);c.text(x+15.0,225.0,if col==0{"Pinned build"}else{"Current result"},GOLD,20.0);for slot in 0..5{let name=crate::armor_data::ARMOR.iter().find(|a|a.key==b.armor[slot]).map_or("Missing catalog entry",|a|a.name);c.text(x+15.0,270.0+slot as f32*35.0,name,TEXT,16.0);}let mut points=Text::new();if let Some(totals)=builder::saved_totals(b){for tree in 1..206{if let Some(id)=builder::activation(tree as u8,totals.points[tree]){if !points.str().is_empty(){let _=write!(points," / ");}let _=write!(points,"{}",data::ACTIVATIONS[id].name);}}}c.paragraph(x+15.0,462.0,565.0,points.str(),GOLD,16.0,0,4);let mut jewels=Text::new();let _=write!(jewels,"{} jewels / {} weapon slots",b.jewel_count,b.weapon[3]);c.text(x+15.0,615.0,jewels.str(),MUTED,15.0);}
 button(c,35.0,691.0,260.0,"Clear comparison",12,0,false);button(c,310.0,691.0,260.0,"Back to result",10,0,false);
}
