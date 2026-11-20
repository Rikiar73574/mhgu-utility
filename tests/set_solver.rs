#[path = "../mod/src/set_solver.rs"] mod set_solver;
use set_solver::*;
use std::collections::BTreeSet;

#[derive(Default)]
struct Data { armor: [Vec<Gear>; 5], charms: Vec<Gear>, jewels: Vec<Jewel> }
impl Catalog for Data {
    fn armor_count(&self,s:usize)->u32 { self.armor[s].len() as u32 }
    fn armor(&self,s:usize,i:u32)->Gear { self.armor[s][i as usize] }
    fn charm_count(&self)->u32 { self.charms.len() as u32 }
    fn charm(&self,i:u32)->Gear { self.charms[i as usize] }
    fn jewel_count(&self)->u32 { self.jewels.len() as u32 }
    fn jewel(&self,i:u32)->Jewel { self.jewels[i as usize] }
}
fn gear(a:i16,b:i16,slots:u8)->Gear { Gear { points:[a,b,0,0,0,0],slots,..Gear::default() } }
fn jewel(a:i16,b:i16,slots:u8)->Jewel { Jewel { points:[a,b,0,0,0,0],slots,..Jewel::default() } }
fn query(a:i16,b:i16)->Query {
    Query { goals:[Goal{threshold:a,at_least:true},Goal{threshold:b,at_least:true},Goal::default(),Goal::default(),Goal::default(),Goal::default()],goal_count:2,..Query::default() }
}
fn base()->Data { Data { armor:std::array::from_fn(|_|vec![Gear::default()]),charms:vec![Gear::default()],jewels:vec![] } }
type Key = ([u32;5],u32,Vec<(u8,u32)>,[i32;6]);
fn key(s:Solution)->Key {let mut jewels:Vec<_>=s.jewels[..s.jewel_count as usize].iter().map(|p|(p.gear,p.jewel)).collect();jewels.sort();(s.armor,s.charm,jewels,s.totals)}
fn run(data:&Data,q:Query,budget:u32)->BTreeSet<Key> {
    let mut solver=Solver::new(q);let mut results=BTreeSet::new();
    for _ in 0..2_000_000 {
        let before=solver.work_done;
        let status=solver.tick(data,budget);
        assert!(solver.work_done-before<=budget as u64);
        match status { Tick::Found(s)=>{assert!(results.insert(key(s)),"duplicate loadout");},Tick::Complete=>return results,Tick::Working=>{},other=>panic!("{other:?}") }
    }
    panic!("fixture failed to terminate")
}

// Independent fixture oracle: enumerate every per-bin multiset recursively,
// then evaluate arithmetic from the completed loadout rather than DFS state.
fn brute(data:&Data,q:Query)->BTreeSet<Key> {
    fn loadouts(data:&Data,q:Query,chosen:[u32;5],charm:u32,b:usize,min:usize,left:u8,slots:[u8;7],p:&mut Vec<(u8,u32)>,out:&mut BTreeSet<Key>) {
        if b==7 {
            let torso=(0..5).filter(|&s|s!=1&&data.armor[s][chosen[s] as usize].torso_up).count() as i32;
            let mut points=[0i32;6];
            for s in 0..5 { for g in 0..6 {points[g]+=data.armor[s][chosen[s] as usize].points[g] as i32*if s==1 {1+torso}else{1};} }
            for g in 0..6 {points[g]+=data.charms[charm as usize].points[g] as i32;}
            for &(s,j) in p.iter() {for g in 0..6 { points[g]+=data.jewels[j as usize].points[g] as i32*if s==1{1+torso}else{1};}}
            if q.native_wrapping { for point in &mut points { *point = *point as i8 as i32; } }
            if (0..q.goal_count as usize).all(|g| if q.goals[g].at_least {points[g]>=q.goals[g].threshold as i32}else{points[g]<=q.goals[g].threshold as i32}) {
                for g in q.goal_count as usize..6 {points[g]=0;}
                out.insert((chosen,charm,p.clone(),points));
            }
            return;
        }
        loadouts(data,q,chosen,charm,b+1,0,if b+1<7{slots[b+1]}else{0},slots,p,out);
        for j in min..data.jewels.len() {let cost=data.jewels[j].slots;if cost<=left {p.push((b as u8,j as u32));loadouts(data,q,chosen,charm,b,j,left-cost,slots,p,out);p.pop();}}
    }
    fn armor(data:&Data,q:Query,s:usize,c:&mut[u32;5],out:&mut BTreeSet<Key>) {
        if s<5 {for i in 0..data.armor[s].len(){c[s]=i as u32;armor(data,q,s+1,c,out);}return;}
        for charm in 0..data.charms.len() {let mut slots=[0;7];for s in 0..5{slots[s]=data.armor[s][c[s] as usize].slots;}slots[5]=data.charms[charm].slots;slots[6]=q.weapon_slots;loadouts(data,q,*c,charm as u32,0,0,slots[0],slots,&mut vec![],out);}
    }
    let mut out=BTreeSet::new();armor(data,q,0,&mut[0;5],&mut out);out
}

#[test]fn mixed_slots_charms_negatives_match_brute_force() {
    for seed in 0..12i16 {
        let mut d=base();
        for s in 0..5 {d.armor[s]=vec![gear((seed+s as i16)%4-1,2-s as i16,(s%2)as u8),gear(1,(seed+s as i16)%3-1,((s+1)%2)as u8)];}
        d.armor[2][1].torso_up=seed%2==0;
        d.charms=vec![gear(0,1,1),gear(2,-2,0),gear(-2,2,0)];
        d.jewels=vec![jewel(2,-1,1),jewel(-1,2,2)];
        let mut q=query(7,4);q.goals[1].threshold=-2;q.goals[1].at_least=false;
        let expected=brute(&d,q);assert_eq!(run(&d,q,1),expected,"seed {seed}");
        assert_eq!(run(&d,q,47),expected,"budget independence seed {seed}");
    }
}
#[test]fn global_slots_do_not_pack_a_three_slot_jewel() {
    let mut d=base();d.armor[0][0].slots=2;d.armor[1][0].slots=1;d.jewels=vec![jewel(10,0,3)];
    assert!(run(&d,query(10,0),9).is_empty());
    d.armor[0][0].slots=3;assert!(!run(&d,query(10,0),9).is_empty());
}
#[test]fn torso_copies_chest_and_chest_jewels_including_negative_points() {
    let mut d=base();d.armor[1][0]=gear(2,-1,1);d.armor[0][0].torso_up=true;d.armor[3][0].torso_up=true;d.jewels=vec![jewel(1,-1,1)];
    let mut q=query(9,-6);q.goals[1].at_least=false;
    let results=run(&d,q,1);assert_eq!(results.len(),1);assert_eq!(results.first().unwrap().3[..2],[9,-6]);assert_eq!(results,brute(&d,q));
}
#[test]fn no_greedy_armor_or_charm_choice() {
    let mut d=base();d.armor[0]=vec![gear(9,-4,0),gear(5,5,0)];d.charms=vec![gear(5,-4,0),gear(0,5,0),gear(5,5,0)];
    let results=run(&d,query(10,10),1);assert_eq!(results.len(),1);assert_eq!(results.first().unwrap().0[0],1);assert_eq!(results.first().unwrap().1,2);
}
#[test]fn jewels_search_non_greedy_combinations_and_all_empty_bins() {
    let mut d=base();d.armor[0][0].slots=3;d.jewels=vec![jewel(6,0,3),jewel(2,2,1)];
    let results=run(&d,query(6,6),1);assert_eq!(results.len(),1);assert_eq!(results.first().unwrap().2,vec![(0,1);3]);
    assert_eq!(run(&base(),query(0,0),1).len(),1);
}
#[test]fn cancellation_zero_budget_and_exhaustion_are_distinct() {
    let d=base();let mut s=Solver::new(query(0,0));assert_eq!(s.tick(&d,0),Tick::Working);assert_eq!(s.work_done,0);s.cancel();assert_eq!(s.tick(&d,50),Tick::Cancelled);
    let mut s=Solver::new(query(0,0));loop{match s.tick(&d,1){Tick::Found(_)=>break,Tick::Working=>{},x=>panic!("{x:?}")}}
    assert_eq!(s.tick(&d,0),Tick::Working);s.cancel();assert_eq!(s.tick(&d,1),Tick::Cancelled);
    let mut d=base();d.armor[4].clear();assert!(run(&d,query(0,0),1).is_empty());
}
#[test]fn invalid_input_and_memory_bound() {
    println!("solver state: {} bytes", std::mem::size_of::<Solver>());
    assert!(std::mem::size_of::<Solver>()<12*1024);
    let mut d=base();d.jewels.push(jewel(2,0,0));let mut s=Solver::new(query(0,0));assert_eq!(s.tick(&d,100),Tick::InvalidCatalog);
    let mut q=query(0,0);q.goal_count=7;assert_eq!(Solver::new(q).tick(&base(),0),Tick::InvalidCatalog);
}
#[test]fn adapter_can_reject_results_without_losing_completeness() {
    let mut d=base();d.armor[0]=vec![gear(10,0,0),gear(10,0,0)];let mut q=query(10,0);q.goal_count=1;
    assert_eq!(run(&d,q,1).len(),2);
}
#[test]fn full_twenty_one_jewel_capacity() {
    let mut d=base();for s in 0..5{d.armor[s][0].slots=3;}d.charms[0].slots=3;d.jewels=vec![jewel(1,0,1)];let mut q=query(21,0);q.weapon_slots=3;
    let results=run(&d,q,13);assert_eq!(results.len(),1);assert_eq!(results.first().unwrap().2.len(),21);
}

fn totals(d:&Data, native:bool, deviant:u8)->[i32;6] {
    let mut q=query(-128,-128);q.native_wrapping=native;q.boost_nonzero=deviant;
    let results=run(d,q,1);assert_eq!(results.len(),1);results.first().unwrap().3
}
#[test]fn double_charm_includes_jewels_but_not_other_bins() {
    let mut d=base();d.armor[0][0]=gear(1,0,0);d.armor[0][0].extra=[0,10];d.charms[0]=gear(3,0,1);d.jewels=vec![jewel(2,0,1)];
    let results=run(&d,query(11,0),1);assert_eq!(results.len(),1);assert_eq!(results.first().unwrap().3[0],11);
}
#[test]fn sticky_special_activation_in_both_orders() {
    let mut d=base();d.charms[0]=gear(4,0,0);d.charms[0].extra=[3,0];d.armor[0][0].extra=[4,10];
    // Double Talisman promotes Secret Arts 7 -> 10, activating Skill+2.
    assert_eq!(totals(&d,true,0)[0],10);
    d.charms[0].extra=[0,3];d.armor[0][0].extra=[10,5];
    // Skill+2 promotes Talisman Boost 8 -> 10, activating Double Talisman.
    assert_eq!(totals(&d,true,0)[0],10);
}
#[test]fn skill_boost_present_zero_negative_and_deviant_rows() {
    let mut d=base();d.armor[0][0]=gear(-3,-10,0);d.armor[0][0].extra=[10,0];d.armor[1][0]=gear(3,0,0);
    assert_eq!(totals(&d,true,0)[..2],[2,-8]);
    // Deviant aggregate zero still has nonzero per-bin contributions.
    assert_eq!(totals(&d,true,1)[0],2);
    d.armor[0][0].points[0]=0;d.armor[1][0].points[0]=0;d.armor[0][0].present=1;
    assert_eq!(totals(&d,true,0)[0],2);assert_eq!(totals(&d,true,1)[0],0);
    d.armor[0][0].present=0;assert_eq!(totals(&d,true,0)[0],0);
}
#[test]fn native_wrapping_bounds_match_brute_force() {
    for seed in 0..15i16 {
        let mut d=base();d.armor[0]=vec![gear(100+seed,seed-125,1),gear(-110,120,0)];d.armor[1]=vec![gear(30,30,1),gear(0,-30,0)];
        d.armor[2][0].torso_up=seed%2==0;d.charms=vec![gear(0,0,0),gear(-30,20,1)];d.jewels=vec![jewel(25,-25,1),jewel(-50,50,1)];
        let mut q=query(10,-10);q.goals[1].at_least=false;q.native_wrapping=true;
        assert_eq!(run(&d,q,11),brute(&d,q),"seed {seed}");
    }
}

#[test]fn impossible_large_catalog_prunes_before_armor_product() {
    let mut d=base();for slot in &mut d.armor {*slot=vec![gear(1,0,0);1000];}
    let mut s=Solver::new(query(10,0));assert_eq!(s.tick(&d,6000),Tick::Complete);assert!(s.work_done<5100);
}

#[test]fn prioritized_jewel_rotation_still_enumerates_every_multiset() {
    let mut d=base();d.armor[0][0].slots=3;d.jewels=vec![jewel(1,0,1),jewel(3,2,1),jewel(2,1,1)];
    assert_eq!(run(&d,query(2,1),1),brute(&d,query(2,1)));
}

#[test]fn accepted_result_can_skip_jewels_without_losing_equipment_combinations() {
    let mut d=base();
    d.armor[0]=vec![gear(2,0,2),gear(3,0,2),gear(-10,0,0)];
    d.charms=vec![gear(1,0,1),gear(2,0,0),gear(-10,0,0)];
    d.jewels=vec![jewel(1,0,1),jewel(2,0,1)];
    let q=query(5,0);
    let expected:BTreeSet<_>=run(&d,q,11).into_iter().map(|result|(result.0,result.1)).collect();
    assert_eq!(expected.len(),4);
    for budget in [1,2,47] {
        let mut solver=Solver::new(q);
        assert!(!solver.next_equipment());
        let mut actual=BTreeSet::new();
        let mut complete=false;
        for _ in 0..10000 {
            let before=solver.work_done;
            let status=solver.tick(&d,budget);
            assert!(solver.work_done-before<=budget as u64);
            match status {
                Tick::Found(solution)=>{
                    assert!(actual.insert((solution.armor,solution.charm)),"duplicate equipment combination");
                    assert!(solver.next_equipment());
                    assert_eq!(solver.work_done-before<=budget as u64,true);
                    assert!(!solver.next_equipment());
                },
                Tick::Working=>assert!(!solver.next_equipment()),
                Tick::Complete=>{complete=true;break;},
                other=>panic!("{other:?}"),
            }
        }
        assert!(complete);
        assert_eq!(actual,expected);
        assert!(!solver.next_equipment());
    }
}

#[test]fn verified_preference_orders_without_filtering_candidates() {
    struct Preferred { data:Data, prefer:bool }
    impl Catalog for Preferred {
        fn armor_count(&self,s:usize)->u32{self.data.armor_count(s)}
        fn armor(&self,s:usize,i:u32)->Gear{self.data.armor(s,i)}
        fn charm_count(&self)->u32{self.data.charm_count()}
        fn charm(&self,i:u32)->Gear{self.data.charm(i)}
        fn jewel_count(&self)->u32{self.data.jewel_count()}
        fn jewel(&self,i:u32)->Jewel{self.data.jewel(i)}
        fn verified(&self,kind:usize,index:u32)->bool{!self.prefer || ![0,5,6].contains(&kind) || index==1}
    }
    fn collect(c:&Preferred,query:Query)->(Key,BTreeSet<Key>){
        let mut solver=Solver::new(query);let mut first=None;let mut all=BTreeSet::new();
        for _ in 0..10000{match solver.tick(c,1){
            Tick::Found(s)=>{let k=key(s);if first.is_none(){first=Some(k.clone());}assert!(all.insert(k));},
            Tick::Working=>{},Tick::Complete=>return(first.unwrap(),all),other=>panic!("{other:?}"),
        }}panic!("did not complete")
    }
    let mut data=base();data.armor[0]=vec![gear(4,0,1),gear(2,0,1)];data.charms=vec![gear(3,0,0),gear(1,0,0)];data.jewels=vec![jewel(4,0,1),jewel(2,0,1)];
    let mut c=Preferred{data,prefer:false};
    let mut q=query(5,0);q.native_wrapping=true;
    let (unpreferred,expected)=collect(&c,q);assert_eq!(unpreferred.0[0],0);assert_eq!(unpreferred.1,1);
    c.prefer=true;let (preferred,actual)=collect(&c,q);
    assert_eq!(preferred.0[0],1);assert_eq!(preferred.1,1);assert_eq!(preferred.2,vec![(0,1)]);
    assert_eq!(actual,expected);
    let combinations:BTreeSet<_>=actual.iter().map(|k|(k.0,k.1)).collect();assert_eq!(combinations.len(),4);
}

fn first_solution(d: &Data, q: Query) -> Solution {
    let mut solver = Solver::new(q);
    loop {
        match solver.tick(d, 1) {
            Tick::Found(s) => return s,
            Tick::Working => {},
            other => panic!("expected solution, got {other:?}"),
        }
    }
}

#[test]
fn exact_threshold_equipment_precedes_overshoot_in_either_catalog_order() {
    for reverse in [false, true] {
        let mut d = base();
        d.armor[0] = vec![gear(25, 20, 0), gear(20, 20, 0)];
        if reverse { d.armor[0].reverse(); }
        assert_eq!(first_solution(&d, query(20, 20)).totals[..2], [20, 20]);
        assert_eq!(run(&d, query(20, 20), 1), brute(&d, query(20, 20)));
    }
    let mut d = base();
    d.armor[0][0] = gear(25, 20, 0);
    assert_eq!(first_solution(&d, query(20, 20)).totals[..2], [25, 20]);
}

#[test]
fn charm_order_uses_remaining_deficit_without_losing_other_candidates() {
    let mut d = base();
    d.armor[0][0] = gear(15, 20, 0);
    d.charms = vec![gear(10, 0, 0), gear(5, 0, 0)];
    assert_eq!(first_solution(&d, query(20, 20)).totals[..2], [20, 20]);
    assert_eq!(run(&d, query(20, 20), 1), brute(&d, query(20, 20)));
}

#[test]
fn shortlist_remains_complete_beyond_its_capacity() {
    let mut d = base();
    d.armor[0] = (0..19).map(|i| gear(i, 20, 0)).collect();
    d.charms = (0..12).map(|i| gear(i, 0, 0)).collect();
    assert_eq!(run(&d, query(20, 20), 1), brute(&d, query(20, 20)));
}

#[test]
fn continued_layout_search_can_reduce_excess_with_exact_per_piece_packing() {
    let mut d = base();
    d.armor[0][0] = gear(15, 20, 2);
    d.armor[1][0].slots = 1;
    // A three-slot exact jewel cannot fit these bins. The exact alternative
    // uses the two-slot jewel; the stronger one-slot jewel overshoots.
    d.jewels = vec![jewel(10, 0, 1), jewel(5, 0, 2), jewel(5, 0, 3)];
    let q = query(20, 20);
    let all = run(&d, q, 1);
    assert_eq!(all, brute(&d, q));
    assert!(all.iter().any(|k| k.3[..2] == [25, 20]));
    let exact: Vec<_> = all.iter().filter(|k| k.3[..2] == [20, 20]).collect();
    assert_eq!(exact.len(), 1);
    assert_eq!(exact[0].2, vec![(0, 1)]);
}

#[test]
fn refinement_can_finish_after_working_without_skipping_unseen_equipment() {
    let mut d = base();
    d.armor[0] = vec![gear(2, 0, 2), gear(3, 0, 2)];
    d.charms = vec![gear(1, 0, 1), gear(2, 0, 0)];
    d.jewels = vec![jewel(1, 0, 1), jewel(2, 0, 1)];
    let q = query(0, 0);
    let expected: BTreeSet<_> = run(&d, q, 11).into_iter().map(|r| (r.0, r.1)).collect();
    let mut solver = Solver::new(q);
    assert!(!solver.finish_equipment());
    let mut actual = BTreeSet::new();
    let mut complete = false;
    for _ in 0..10000 {
        match solver.tick(&d, 1) {
            Tick::Found(s) => {
                assert!(actual.insert((s.armor, s.charm)));
                assert_eq!(solver.tick(&d, 1), Tick::Working);
                assert!(!solver.next_equipment());
                assert!(solver.finish_equipment());
                assert!(!solver.finish_equipment());
            },
            Tick::Working => assert!(!solver.finish_equipment()),
            Tick::Complete => { complete = true; break; },
            other => panic!("{other:?}"),
        }
    }
    assert!(complete);
    assert_eq!(actual, expected);
    assert!(!solver.finish_equipment());
}

#[test]
fn spare_slot_refinement_is_incremental_uses_all_jewels_and_preserves_minimums() {
    // The exact solver supplies a minimum-valid armor/charm combination. Refining
    // must pass over 80 irrelevant entries, use two jewels to activate the bonus,
    // then use the final slot to upgrade the requested skill beyond its minimum.
    let mut d=base();d.armor[0][0]=gear(10,6,3);
    d.jewels=vec![jewel(0,0,1);80];
    d.jewels.push(jewel(-6,10,1)); // Would break Attack S: reject.
    d.jewels.push(jewel(0,2,1));
    d.jewels.push(jewel(5,0,1));
    let q=query(10,0);
    let mut solver=Solver::new(q);
    let seed=loop{if let Tick::Found(s)=solver.tick(&d,1){break s;}};
    assert_eq!(seed.jewel_count,0);
    fn score(d:&Data,s:&mut Solution)->Option<[u32;10]>{
        let mut attack=d.armor[0][0].points[0]as i32;
        let mut defense=d.armor[0][0].points[1]as i32;
        for p in &s.jewels[..s.jewel_count as usize]{attack+=d.jewels[p.jewel as usize].points[0]as i32;defense+=d.jewels[p.jewel as usize].points[1]as i32;}
        if attack<10{return None;}
        s.totals[0]=attack;s.totals[1]=defense;
        Some([u32::from(defense<10),u32::from(attack<15),100-defense.min(10)as u32,s.jewel_count as u32,0,0,0,0,0,0])
    }
    let mut seed=seed;let initial=score(&d,&mut seed).unwrap();
    let mut refine=DecorationRefiner::new(seed,[3,0,0,0,0,0,0],|j|d.jewels[j as usize].slots,initial).unwrap();
    let mut ticks=0;
    let result=loop{ticks+=1;let before=refine.work_done;
        let (s,done)=refine.tick(d.jewels.len()as u32,|j|d.jewels[j as usize].slots,|s|score(&d,s),3);
        assert!(refine.work_done-before<=3);assert!(s.totals[0]>=10);
        assert!(s.jewel_count<=3);assert!(s.jewels[..s.jewel_count as usize].iter().all(|p|p.gear==0));
        if done{break s;}assert!(ticks<3000);
    };
    assert!(ticks>100);assert_eq!(result.jewel_count,3);
    assert!(result.totals[0]>=15);assert!(result.totals[1]>=10);
    assert!(!result.jewels[..3].iter().any(|p|p.jewel==80));
}

#[test]
fn refinement_validates_each_bin_and_stops_without_useful_improvements(){
    let seed=Solution{armor:[0;5],charm:0,jewels:[Placement::default();MAX_JEWELS],jewel_count:0,totals:[10,0,0,0,0,0]};
    let mut refine=DecorationRefiner::new(seed,[1,1,1,0,0,0,0],|_|2,[0;10]).unwrap();
    let(s,done)=refine.tick(1,|_|2,|_|panic!("two-slot jewel cannot fit separate one-slot bins"),20);
    assert!(done);assert_eq!(s,seed);
    let mut overflow=seed;overflow.jewel_count=1;overflow.jewels[0]=Placement{jewel:0,gear:0};
    assert!(DecorationRefiner::new(overflow,[1;7],|_|2,[0;10]).is_none());
}

#[test]
fn full_slot_first_solution_can_replace_harmful_goal_jewel(){
 let mut d=base();d.armor[0][0].slots=1;
 d.jewels=vec![jewel(10,-10,1),jewel(10,0,1)];
 let mut q=query(10,0);q.goal_count=1;let mut solver=Solver::new(q);
 let seed=loop{if let Tick::Found(s)=solver.tick(&d,1){break s;}};
 assert_eq!(seed.jewels[0].jewel,0);
 let mut refine=DecorationRefiner::new(seed,[1,0,0,0,0,0,0],|_|1,[1,0,0,0,0,0,0,0,0,0]).unwrap();
 let result=loop{let (s,done)=refine.tick(2,|_|1,|s|{
   let j=d.jewels[s.jewels[0].jewel as usize];s.totals[0]=j.points[0]as i32;
   Some([u32::from(j.points[1]<0),0,0,0,0,0,0,0,0,0])
 },2);if done{break s;}};
 assert_eq!(result.jewel_count,1);assert_eq!(result.jewels[0].jewel,1);assert_eq!(result.totals[0],10);
}

#[test]
fn decoration_refinement_yields_equipment_search_within_fixed_work_limit(){
 let seed=Solution{armor:[0;5],charm:0,jewels:[Placement::default();MAX_JEWELS],jewel_count:0,totals:[0;6]};
 let mut refine=DecorationRefiner::new(seed,[3;7],|_|1,[u32::MAX;10]).unwrap();
 for frame in 0..258{
  let(_,done)=refine.tick(256,|_|1,|s|Some([u32::MAX-u32::from(s.jewel_count),0,0,0,0,0,0,0,0,0]),32);
  assert!(refine.work_done<=8192);
  if done{return;}
  assert!(frame<257,"one result must not monopolize subsequent equipment search");
 }
 panic!("refinement did not yield");
}

#[test]
fn refinement_tries_all_seven_bins_after_requested_skill_is_already_met(){
 let seed=Solution{armor:[0;5],charm:0,jewels:[Placement::default();MAX_JEWELS],jewel_count:0,totals:[20,0,0,0,0,0]};
 let mut refiner=DecorationRefiner::new(seed,[1;7],|_|1,[u32::MAX,0,0,0,0,0,0,0,0,0]).unwrap();
 let result=loop{let(s,done)=refiner.tick(1,|_|1,|s|{
  assert_eq!(s.totals[0],20,"requested Attack L must stay intact");
  Some([u32::MAX-u32::from(s.jewel_count),0,0,0,0,0,0,0,0,0])
 },1);if done{break s;}};
 assert_eq!(result.jewel_count,7);
 for gear in 0..7{assert_eq!(result.jewels[..7].iter().filter(|p|p.gear==gear).count(),1);}
}
