//! Production adapter exercised against deterministic progression/player catalogs.
#![allow(dead_code)]
#[path="../mod/src/set_solver.rs"]mod set_solver;
#[path="../mod/src/set_skills.rs"]mod set_skills;
#[path="../mod/src/set_builder.rs"]mod set_builder;
#[derive(Default)]pub struct Ui{tab:usize,builder:bool,selected:usize,len:usize,detail:usize,dirty:bool,count:usize,results:[u16;8]}
fn enter(_: &mut Ui,_:usize){}
mod input{pub fn close(){}}
mod catalog{pub const ENTRIES:[u8;0]=[];}
mod wishlist{pub const ARMOR_BASE:usize=0;pub fn contains(_:usize,_:usize)->bool{false}pub fn toggle(_:usize,_:usize){}pub fn failed()->bool{false}}
mod set_unlock{pub fn begin()->bool{true}pub fn ready()->bool{true}pub fn tick()->bool{true}pub fn armor(_:u8,_:u16)->Option<bool>{Some(true)}pub fn decoration(_:u16)->Option<bool>{Some(true)}}
mod armor_data{
 pub struct Armor{pub native:u16,pub slot:u8,pub hunter:u8,pub gender:u8,pub slots:u8,pub skill:u16,pub skill_count:u8,pub defense:u16}
 pub static ARMOR:[Armor;5]=[Armor{native:0,slot:0,hunter:2,gender:2,slots:3,skill:0,skill_count:2,defense:10},
Armor{native:1,slot:1,hunter:2,gender:2,slots:0,skill:0,skill_count:0,defense:10},
Armor{native:2,slot:2,hunter:2,gender:2,slots:0,skill:0,skill_count:0,defense:10},
Armor{native:3,slot:3,hunter:2,gender:2,slots:0,skill:0,skill_count:0,defense:10},
Armor{native:4,slot:4,hunter:2,gender:2,slots:0,skill:0,skill_count:0,defense:10}];
 pub const SKILLS:[(u8,i8);2]=[(0,10),(1,6)];
}
mod set_data{
 pub const SKILL_PLUS:u8=204;pub const TALISMAN_BOOST:u8=205;pub const TORSO_UP:u8=203;
 pub const ARMOR_TREES:[u8;2]=[1,2];
 pub struct Activation{pub tree:u8,pub points:i8,pub name:&'static str}
 pub const ACTIVATIONS:[Activation;7]=[
  Activation{tree:1,points:10,name:"Attack S"},Activation{tree:1,points:15,name:"Attack M"},Activation{tree:1,points:20,name:"Attack L"},
  Activation{tree:2,points:10,name:"Defense S"},Activation{tree:2,points:15,name:"Defense M"},Activation{tree:2,points:20,name:"Defense L"},Activation{tree:3,points:-10,name:"Penalty"}];
 pub struct Deco{pub native:u16,pub slots:u8,pub skill:u16,pub skill_count:u8,pub entry:u16}
 pub const DECO:[Deco;5]=[
  Deco{native:0,slots:1,skill:0,skill_count:1,entry:0},Deco{native:1,slots:1,skill:1,skill_count:1,entry:1},
  Deco{native:2,slots:1,skill:2,skill_count:1,entry:2},Deco{native:3,slots:1,skill:3,skill_count:2,entry:3},Deco{native:4,slots:1,skill:5,skill_count:2,entry:4}];
 pub const POINTS:[(u8,i8);7]=[(4,1),(2,2),(1,5),(1,-20),(2,10),(2,10),(3,-10)];
}
mod set_player{
 pub const MAX_EQUIPMENT:usize=1;
 #[derive(Clone,Copy)]pub struct Skill{pub tree:u8,pub points:i8}
 pub struct Charm{pub slots:u8,pub skills:[Skill;2]}
 pub struct Owned{pub slot:u8,pub native:u16}
 pub struct Snapshot{pub hunter:Option<u8>,pub gender:Option<u8>,pub weapon_slots:Option<u8>,pub owned_count:usize,pub owned:[Owned;0],pub charm_count:usize,pub charms:[Charm;2]}
 pub fn refresh()->bool{true}
 pub fn snapshot()->&'static Snapshot{&DATA}
 static DATA:Snapshot=Snapshot{hunter:Some(0),gender:Some(0),weapon_slots:Some(0),owned_count:0,owned:[],charm_count:2,charms:[
  Charm{slots:0,skills:[Skill{tree:4,points:1};2]},Charm{slots:1,skills:[Skill{tree:2,points:4},Skill{tree:0,points:0}]}]};
}
#[test]
fn production_adapter_fills_spare_slots_keeps_goals_charms_and_exposes_l3(){
 let mut ui=Ui::default();set_builder::open(&mut ui);set_builder::toggle(0);set_builder::start();
 for _ in 0..20_000{set_builder::tick();if matches!(set_builder::state().phase,set_builder::Phase::Complete){break;}}
 let s=set_builder::state();assert!(matches!(s.phase,set_builder::Phase::Complete));assert!(s.count>=3);
 // This checks actual adapter quality, native evaluation and result retention.
 let mut saw_bonus=false;let mut saw_late_charm=false;let mut saw_upgrade=false;
 for result in 0..s.count{set_builder::select_result(result);set_builder::refresh_summary();let s=set_builder::state();let r=s.results[s.result].unwrap();
  assert!(s.totals.points[1]>=10);assert!(s.totals.points[3]>-10);
  saw_bonus|=s.totals.points[2]>=10;saw_upgrade|=s.totals.points[1]>=15;saw_late_charm|=r.charm==2;
  let(c,u)=set_builder::slots();assert!((0..7).all(|i|u[i]<=c[i]));
 }
 assert!(saw_bonus&&saw_upgrade&&saw_late_charm);
 // Requesting both small tiers also accepts medium/large activations.
 set_builder::toggle(3);set_builder::start();
 for _ in 0..20_000{set_builder::tick();if matches!(set_builder::state().phase,set_builder::Phase::Complete){break;}}
 assert!(set_builder::state().count>0);
 for result in 0..set_builder::state().count{set_builder::select_result(result);set_builder::refresh_summary();let s=set_builder::state();assert!(s.totals.points[1]>=10&&s.totals.points[2]>=10);}
 set_builder::select_result(0);set_builder::input(&mut ui,1<<4);assert!(set_builder::state().skills);
}
