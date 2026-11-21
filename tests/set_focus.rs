#![allow(dead_code)]
#[path="../mod/src/set_solver.rs"]mod set_solver;
#[path="../mod/src/set_focus.rs"]mod set_focus;
use set_solver::{Catalog,Gear,Jewel};
struct C;
impl Catalog for C{
 fn armor_count(&self,_:usize)->u32{4}
 fn armor(&self,_:usize,i:u32)->Gear{let mut g=Gear::default();g.points[0]=[1,2,2,10][i as usize];g.slots=if i==2{1}else{0};g}
 fn charm_count(&self)->u32{1}fn charm(&self,_:u32)->Gear{Gear::default()}
 fn jewel_count(&self)->u32{4}fn jewel(&self,i:u32)->Jewel{let mut j=Jewel::default();if i<2{j.points[0]=1;}else if i==2{j.points[1]=1;}else{j.extra[0]=2;}j}
 fn verified(&self,k:usize,i:u32)->bool{k==5||i!=if k==6{1}else{3}}
}
#[test]fn seeds_keep_verified_pareto_candidates_with_bounded_work(){let mut mask=set_focus::Mask::new();let mut ticks=0;while !mask.ready{assert!(mask.tick(&C,1,true,1)<=1);ticks+=1;assert!(ticks<1000);}assert!(ticks>5);for k in 0..5{assert_eq!(mask.count(k),1);assert_eq!(mask.index(k,0),2);}assert_eq!(mask.count(6),2);assert_eq!(mask.index(6,0),0);assert_eq!(mask.index(6,1),3);}
#[test]fn verified_broad_pass_preserves_dominated_candidates(){let mut mask=set_focus::Mask::new();while !mask.ready{mask.tick(&C,1,false,8);}for k in 0..5{assert_eq!(mask.count(k),3);assert_eq!(mask.index(k,1),1);}assert_eq!(mask.count(6),3);assert_eq!(mask.index(6,1),2);}
