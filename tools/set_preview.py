"""Build a host harness around the production Set Builder, using private captured inputs.

Only the clock and game-data adapters differ. Search, task scheduling, candidate
filtering, refinement, ranking and result limits come from the mod sources.
"""
from pathlib import Path
import hashlib,json,subprocess,os
R=Path(__file__).resolve().parents[1];P=R/'build/web-set-preview';d=json.loads((P/'snapshot.json').read_text())
INPUTS=['mod/src/set_builder.rs','mod/src/set_solver.rs','mod/src/set_focus.rs','mod/src/set_skills.rs','build/generated/set_data.rs','build/generated/armor_data.rs','build/generated/catalog.rs','build/web-set-preview/snapshot.json']
proof={name:hashlib.sha256((R/name).read_bytes()).hexdigest()for name in INPUTS}
proof['revision']=subprocess.check_output(['git','rev-parse','--short','HEAD'],cwd=R,text=True).strip()
proof['source_hash']=hashlib.sha256(json.dumps(proof,sort_keys=True).encode()).hexdigest()

def arr(v):return '['+','.join(map(str,v))+']'
modules=f'''#![allow(dead_code)]
#[path="{R}/mod/src/set_solver.rs"] mod set_solver;
#[path="{R}/mod/src/set_skills.rs"] mod set_skills;
#[path="{R}/mod/src/set_focus.rs"] mod set_focus;
mod armor_data{{include!("{R}/build/generated/armor_data.rs");}}
mod set_data{{include!("{R}/build/generated/set_data.rs");}}
pub struct Entry{{pub name:&'static str,pub kind:u8,pub body:&'static str,pub links:&'static [(usize,&'static str)]}}
mod catalog{{include!("{R}/build/generated/catalog.rs");}}
pub struct Ui{{tab:usize,builder:bool,selected:usize,len:usize,detail:usize,dirty:bool,count:usize,results:[u16;512]}}
fn enter(_: &mut Ui,_:usize){{}}
mod input{{pub fn close(){{}}}}
mod wishlist{{pub const ARMOR_BASE:usize=0;pub fn contains(_:usize,_:usize)->bool{{false}}pub fn toggle(_:usize,_:usize){{}}pub fn failed()->bool{{false}}}}
// Preparation is bypassed: the native game already captured each candidate's availability.
mod set_unlock{{pub fn begin()->bool{{true}}pub fn ready()->bool{{true}}pub fn tick()->bool{{true}}pub fn armor(_:u8,_:u16)->Option<bool>{{unreachable!()}}pub fn decoration(_:u16)->Option<bool>{{unreachable!()}}}}
mod set_player{{
 pub const MAX_EQUIPMENT:usize=2000;
 #[derive(Clone,Copy)]pub struct Skill{{pub tree:u8,pub points:i8}}
 pub struct Charm{{pub box_index:u16,pub slots:u8,pub skills:[Skill;2]}}
 pub struct Owned{{pub slot:u8,pub native:u16}}
 pub struct Snapshot{{pub hunter:Option<u8>,pub gender:Option<u8>,pub weapon_slots:Option<u8>,pub owned_count:usize,pub owned:[Owned;{len(d['owned'])}],pub charm_count:usize,pub charms:[Charm;{len(d['charms'])}]}}
 pub fn configure(slots:u8){{unsafe{{DATA.weapon_slots=Some(slots);}}}}
 pub fn refresh()->bool{{true}}pub fn snapshot()->&'static Snapshot{{unsafe{{&*(&raw const DATA)}}}}
 static mut DATA:Snapshot=Snapshot{{hunter:Some({d['hunter']}),gender:Some({d['gender']}),weapon_slots:Some({d['weapon_slots']}),owned_count:{len(d['owned'])},owned:[{','.join(f'Owned{{slot:{a},native:{b}}}'for a,b in d['owned'])}],charm_count:{len(d['charms'])},charms:[{','.join('Charm{box_index:%s,slots:%s,skills:[Skill{tree:%s,points:%s},Skill{tree:%s,points:%s}]}'%(c['box_index'],c['slots'],c['trees'][0],c['points'][0],c['trees'][1],c['points'][1])for c in d['charms'])}]}};
}}
'''
source=(R/'mod/src/set_builder.rs').read_text().split('\n',1)[1]
source=source.replace('forge::imgui::get_time()', 'crate::preview_time()')
setup='''
pub fn preview_prepare(goals: &[usize],profile:skills::PriorityProfile,slots:u8,scenario:Option<&str>,owned_only:bool){
 player::configure(slots);
 let s=mutable();s.weapon_slots=slots;s.priority=Some(profile);s.goals.fill(usize::MAX);s.goal_count=goals.len();s.goals[..goals.len()].copy_from_slice(goals);
 let p=player::snapshot();s.owned_count=p.owned_count;for(i,a)in p.owned.iter().enumerate(){s.owned[i]=(u32::from(a.slot)<<16)|u32::from(a.native);}s.owned[..s.owned_count].sort_unstable();
'''
for slot,ids in enumerate(d['armor']):setup+=f' s.armor_count[{slot}]={len(ids)};s.armor_ids[{slot}][..{len(ids)}].copy_from_slice(&{arr(ids)});\n'
setup+=f' s.jewel_count={len(d["jewels"])};s.jewel_ids[..s.jewel_count].copy_from_slice(&{arr(d["jewels"])});\n'
setup+='''
 if let Some(path)=scenario{let text=std::fs::read_to_string(path).expect("scenario candidates");let rows:Vec<&str>=text.lines().collect();assert_eq!(rows.len(),6);
 for k in 0..5{let ids:Vec<u16>=rows[k].split(',').filter(|v|!v.is_empty()).map(|v|v.parse().unwrap()).collect();assert!(ids.len()<=1200);s.armor_count[k]=ids.len()as u16;s.armor_ids[k][..ids.len()].copy_from_slice(&ids);}
 let ids:Vec<u16>=rows[5].split(',').filter(|v|!v.is_empty()).map(|v|v.parse().unwrap()).collect();assert!(ids.len()<=256);s.jewel_count=ids.len();s.jewel_ids[..ids.len()].copy_from_slice(&ids);}
 if owned_only{for k in 0..5{let mut n=0;for j in 0..s.armor_count[k]as usize{let id=s.armor_ids[k][j];if owns_in(s,(id&0x7fff)as usize){s.armor_ids[k][n]=id;n+=1;}}s.armor_count[k]=n as u16;}}
 if s.armor_count.contains(&0){s.phase=Phase::Complete;s.show_results=true;return;}
 let mut q=solver::Query::default();q.goal_count=goals.len()as u8;q.weapon_slots=p.weapon_slots.unwrap();q.native_wrapping=true;
 for(i,&id)in goals.iter().enumerate(){let a=&data::ACTIVATIONS[id];q.goals[i]=solver::Goal{threshold:i16::from(a.points),at_least:true};if(144..=179).contains(&a.tree){q.boost_nonzero|=1<<i;}}
 s.progress=armor::ARMOR.len()+data::DECO.len();prepare_focus(s);start_focus(s,0);s.phase=Phase::Searching;s.show_results=true;s.search=skills::SearchBudget::new();
}
pub fn preview_json(phase:&str)->String{
 use std::fmt::Write;let s=mutable();let mut o=String::new();write!(o,"{{\\"phase\\":{},\\"visited\\":{},\\"count\\":{},\\"builds\\":[",crate::quote(phase),s.visited,s.count).unwrap();
 for i in 0..s.count{if i>0{o.push(',');}let r=s.results[i].unwrap();let all=evaluate(s,&r);let(cap,used)=slots_for(s,&r);let(neg,act,_,_,_,_)=quality(&all);write!(o,"{{\\"rank\\":{},\\"active_count\\":{},\\"penalties\\":{},\\"verified\\":{},\\"capacity\\":{:?},\\"used\\":{:?},\\"armor\\":[",i+1,act,neg,!(0..5).any(|k|s.armor_ids[k][r.armor[k]as usize]&0x8000!=0)&&!r.jewels[..r.jewel_count as usize].iter().any(|p|s.jewel_ids[p.jewel as usize]&0x8000!=0),cap,used).unwrap();
 let priority=s.priority.unwrap();let scores=priority.scores(&all,data::ACTIVATIONS.iter().filter(|a|applicable(a.tree)).map(|a|(a.tree,a.points)));let key=rank_with_totals(s,&r,&all);
 let mut defense=0;
 for k in 0..5{if k>0{o.push(',');}let a=&armor::ARMOR[id(s,&r,k)];defense+=a.defense as u32;write!(o,"{{\\"name\\":{},\\"native\\":{},\\"slots\\":{},\\"owned\\":{}}}",crate::quote(a.name),a.native,a.slots,owns_in(s,id(s,&r,k))).unwrap();}
 write!(o,"],\\"defense\\":{},\\"charm\\":",defense).unwrap();
 if r.charm==0{o.push_str("null");}else{let c=&player::snapshot().charms[r.charm as usize-1];write!(o,"{{\\"box_index\\":{},\\"slots\\":{},\\"skills\\":[",c.box_index+1,c.slots).unwrap();let mut n=0;for p in c.skills{if p.tree==0{continue;}if n>0{o.push(',');}n+=1;write!(o,"{{\\"name\\":{},\\"points\\":{}}}",crate::quote(data::TREE_NAMES[p.tree as usize]),p.points).unwrap();}o.push_str("]}");}
 o.push_str(",\\"jewels\\":[");for(j,p)in r.jewels[..r.jewel_count as usize].iter().enumerate(){if j>0{o.push(',');}let d=&data::DECO[(s.jewel_ids[p.jewel as usize]&0x7fff)as usize];write!(o,"{{\\"gear\\":{},\\"name\\":{},\\"slots\\":{}}}",p.gear,crate::quote(crate::catalog::ENTRIES[d.entry as usize].name),d.slots).unwrap();}
 o.push_str("],\\"skills\\":[");let mut n=0;
 for tree in 1..206{let p=all.points[tree];if !all.present[tree]||p==0{continue;}if n>0{o.push(',');}n+=1;let a=activation(tree as u8,p).map(|id|&data::ACTIVATIONS[id]);write!(o,"{{\\"tree\\":{},\\"name\\":{},\\"points\\":{},\\"active\\":{},\\"activation\\":{},\\"threshold\\":{},\\"requested\\":{},\\"category\\":{}}}",tree,crate::quote(data::TREE_NAMES[tree]),p,p>0&&a.is_some()&&applicable(tree as u8),a.map(|a|crate::quote(a.name)).unwrap_or("null".into()),a.map_or(10,|a|a.points),s.goals[..s.goal_count].iter().any(|&id|data::ACTIVATIONS[id].tree as usize==tree),s.priority.unwrap().activation_preference(tree as u8,a.map_or(10,|a|i16::from(a.points))).0).unwrap();}
 write!(o,"],\\\"priority_scores\\\":{:?},\\\"ranking_key\\\":{:?}}}",scores,key).unwrap();}
 o.push_str("]}");o
}
'''
# The exporter is generated locally, while solver/evaluation code remains a direct source copy.
main=r'''

fn preview_time()->f64{use std::sync::OnceLock;static START:OnceLock<std::time::Instant>=OnceLock::new();START.get_or_init(std::time::Instant::now).elapsed().as_secs_f64()}
fn quote(s:&str)->String{
 let mut o=String::from("\"");
 for c in s.chars(){match c{
  '"'=>o.push_str("\\\""),'\\'=>o.push_str("\\\\"),'\n'=>o.push_str("\\n"),'\r'=>o.push_str("\\r"),'\t'=>o.push_str("\\t"),
  c if c<' '=>{use std::fmt::Write;write!(o,"\\u{:04x}",c as u32).unwrap();},c=>o.push(c)
 }}o.push('"');o
}
fn main(){
 use std::io::Write;let args:Vec<String>=std::env::args().collect();
 let number=|i:usize,default:u8|args.get(i).map(|s|s.parse::<u8>().expect("profile number")).unwrap_or(default);
 let mut profile=set_skills::PriorityProfile::new(number(2,14),number(3,0),number(4,0));
 let slots=number(5,set_player::snapshot().weapon_slots.unwrap());assert!(profile.weapon<=14&&slots<=3&&(profile.element==0||(27..=31).contains(&profile.element))&&[0,1,2,3,92].contains(&profile.status));
 if let Some(order)=args.get(6){let values:Vec<u8>=order.split(',').map(|s|s.parse().unwrap()).collect();assert_eq!(values.len(),4);let mut sorted=values.clone();sorted.sort();assert_eq!(sorted,vec![0,1,2,3]);profile.order.copy_from_slice(&values);}
 if let Some(boosts)=args.get(7){for pair in boosts.split(',').filter(|s|!s.is_empty()){let(t,v)=pair.split_once(':').unwrap();let tree=t.parse::<usize>().unwrap();assert!((1..206).contains(&tree));profile.trees[tree]=v.parse().unwrap();}}
 if args.get(1).map(String::as_str)==Some("--meta"){
  print!("{{\"skills\":[");let mut n=0;
  for(i,a)in set_data::ACTIVATIONS.iter().enumerate(){if a.points<=0||!set_builder::applicable(a.tree){continue;}if n>0{print!(",");}n+=1;print!("{{\"id\":{},\"tree\":{},\"name\":{},\"points\":{},\"tree_name\":{},\"category\":{},\"weight\":{}}}",i,a.tree,quote(a.name),a.points,quote(set_data::TREE_NAMES[a.tree as usize]),profile.activation_preference(a.tree,i16::from(a.points)).0,profile.activation_preference(a.tree,i16::from(a.points)).1);}println!("],\"source_hash\":{},\"snapshot_hash\":{},\"revision\":{},\"fps\":60}}",quote(env!("PREVIEW_SOURCE_HASH")),quote(env!("PREVIEW_SNAPSHOT_HASH")),quote(env!("PREVIEW_REVISION")));return;
 }
 let goals:Vec<usize>=args.get(1).expect("skill ids").split(',').filter(|x|!x.is_empty()).map(|x|x.parse().unwrap()).collect();assert!(goals.len()<=6);let mut trees=std::collections::HashSet::new();for &g in &goals{let a=&set_data::ACTIVATIONS[g];assert!(a.points>0&&trees.insert(a.tree));}
 set_builder::preview_prepare(&goals,profile,slots,args.get(8).filter(|s|!s.is_empty()).map(String::as_str),args.get(9).is_some_and(|s|s=="1"));
 let emit=|phase:&str|{let mut d=set_builder::preview_json(phase);d.pop();let s=set_builder::state();d.push_str(&format!(",\"elapsed\":{},\"search_version\":\"builder-ui-v1\",\"variations\":{},\"batch_variations\":{},\"time_limit\":60,\"variation_target\":5,\"source_hash\":{}}}",s.search.elapsed,s.count,s.search.variations,quote(env!("PREVIEW_SOURCE_HASH"))));println!("{}",d);std::io::stdout().flush().unwrap();};
 let testing=std::env::var("MHGU_PREVIEW_TEST_FRAMES").ok().and_then(|s|s.parse::<usize>().ok());
 let mut frames=0usize;let mut last=0usize;
 loop{
  let frame_started=std::time::Instant::now();let now=if testing.is_some(){frames as f64/60.0}else{preview_time()};
  set_builder::tick_at(now);frames+=1;
  let state=set_builder::state();
  if state.count!=last||frames%30==0{emit("searching");last=state.count;}
  if state.phase==set_builder::Phase::Complete{emit("complete");break;}
  if state.phase==set_builder::Phase::Paused{
   emit(if state.search.variations>=5{"variations"}else{"timeout"});
   if testing.is_some(){break;}
   let mut command=String::new();if std::io::stdin().read_line(&mut command).unwrap_or(0)==0||command.trim()!="resume"{break;}
   set_builder::run_toggle();emit("searching");
  }
  if testing.is_some_and(|n|frames>=n){emit("checkpoint");break;}
  if testing.is_none(){let target=frame_started+std::time::Duration::from_secs_f64(1.0/60.0);if let Some(delay)=target.checked_duration_since(std::time::Instant::now()){std::thread::sleep(delay);}}
 }
}

'''
(P/'host.rs').write_text(modules+'mod set_builder{\n'+source+setup+'\n}\n'+main)
env=os.environ.copy()
if env.get('MHGU_TOOLCHAIN'):
 env.update(RUSTUP_HOME=str(Path(env['MHGU_TOOLCHAIN'])/'rustup'),CARGO_HOME=str(Path(env['MHGU_TOOLCHAIN'])/'cargo'))
env.update(PREVIEW_SOURCE_HASH=proof['source_hash'],PREVIEW_SNAPSHOT_HASH=proof['build/web-set-preview/snapshot.json'],PREVIEW_REVISION=proof['revision'])
subprocess.run(['rustc','+nightly','--edition','2024','-O',str(P/'host.rs'),'-o',str(P/'solver')],env=env,check=True)
print('Native-snapshot host solver built')
(P/'build-info.json').write_text(json.dumps(proof,indent=2)+'\n')
