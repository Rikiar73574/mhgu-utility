#[path = "../mod/src/set_skills.rs"] mod set_skills;
use set_skills::*;

#[test]
fn every_native_tree_and_unrequested_penalties_are_preserved() {
    let mut e = Evaluator::new();
    e.add(0, 35, 12); e.add(1, 25, -8); e.add(6, 25, -3);
    e.add(2, 202, 4); e.add(3, 91, 0);
    let t = e.finish();
    assert_eq!(t.points[35], 12); assert_eq!(t.points[25], -11);
    assert_eq!(t.points[202], 4); assert!(t.present[91]);
    assert!(!t.present[90]);
}

#[test]
fn torso_multiplies_chest_and_its_jewels_including_penalties() {
    let mut e = Evaluator::new();
    e.set_torso_count(2); e.add(0, TORSO_UP, 0);
    e.add(1, 35, 2); e.add(1, 35, 1);
    e.add(1, 25, -1); e.add(1, 25, -2);
    e.add(6, 35, 1); e.add(0, 35, 1);
    let t = e.finish();
    assert_eq!(t.points[35], 11); assert_eq!(t.points[25], -9);
}

#[test]
fn double_talisman_doubles_charm_jewels_only() {
    let mut e = Evaluator::new();
    e.add(0, TALISMAN_BOOST, 10); e.add(0, 35, 1);
    e.add(5, 35, 3); e.add(5, 35, 2); e.add(6, 35, 1);
    e.add(5, 25, -6); e.add(5, 25, -1);
    let t = e.finish();
    assert_eq!(t.points[35], 12); assert_eq!(t.points[25], -14);
}

#[test]
fn sticky_special_activation_in_both_orders() {
    let mut e = Evaluator::new();
    e.add(5, 35, 4); e.add(5, SKILL_PLUS, 3);
    e.add(0, SKILL_PLUS, 4); e.add(0, TALISMAN_BOOST, 10);
    assert_eq!(e.finish().points[35], 10);
    let mut e = Evaluator::new();
    e.add(5, 35, 4); e.add(5, TALISMAN_BOOST, 3);
    e.add(0, SKILL_PLUS, 10); e.add(0, TALISMAN_BOOST, 5);
    assert_eq!(e.finish().points[35], 10);
}

#[test]
fn boost_keeps_zero_rows_but_deviants_require_nonzero_contributions() {
    let mut e = Evaluator::new();
    e.add(0, SKILL_PLUS, 10);
    e.add(0, 35, 0); e.add(0, 25, -10);
    e.add(0, 144, 0); e.add(0, 179, 3); e.add(1, 179, -3);
    e.add(0, 180, 0);
    let t = e.finish();
    assert_eq!(t.points[35], 2); assert_eq!(t.points[25], -8);
    assert_eq!(t.points[144], 0); assert!(t.present[144]);
    assert_eq!(t.points[179], 2); assert_eq!(t.points[180], 2);
    assert_eq!(t.points[36], 0); assert!(!t.present[36]);
}

#[test]
fn native_wrapping_happens_before_activation_and_after_special_changes() {
    let mut e = Evaluator::new();
    e.add(0, SKILL_PLUS, 100); e.add(1, SKILL_PLUS, 30);
    e.add(0, 35, 127); e.add(1, 35, 3);
    assert_eq!(e.finish().points[35], -126);
    let mut e = Evaluator::new();
    e.add(0, SKILL_PLUS, 10); e.add(0, TALISMAN_BOOST, 10);
    e.add(5, 35, 64); e.add(0, 35, -1);
    assert_eq!(e.finish().points[35], -127);
    // Boost activation stays true even when charm doubling wraps Secret Arts.
    let mut e = Evaluator::new();
    e.add(0, SKILL_PLUS, -110); e.add(5, SKILL_PLUS, 120);
    e.add(0, TALISMAN_BOOST, 10); e.add(0, 35, 0);
    assert_eq!(e.finish().points[35], 2);
}

#[test]
fn deviant_nonzero_uses_wrapped_per_gear_points_after_charm_doubling() {
    let mut e = Evaluator::new();
    e.add(0, SKILL_PLUS, 10); e.add(0, TALISMAN_BOOST, 10);
    e.add(5, 144, 128);
    assert_eq!(e.finish().points[144], 0);
    // Same-bin cancellation is zero, unlike equal opposite points on two bins.
    e.add(0, 179, 3); e.add(0, 179, -3);
    assert_eq!(e.finish().points[179], 0);
}

#[test]
fn malformed_rows_are_rejected_and_state_stays_small() {
    let mut e = Evaluator::new();
    assert!(!e.add(7, 35, 10)); assert!(!e.add(0, 206, 10));
    assert!(e.add(0, 205, 0));
    assert!(e.finish().present[205]);
    assert!(std::mem::size_of::<Evaluator>() < 4096);
}

#[allow(dead_code)]
#[path = "../mod/src/set_solver.rs"] mod set_solver;

#[test]
fn full_evaluation_matches_solver_native_totals_across_wrapping_and_specials() {
    use set_solver::{Catalog, Gear, Goal, Jewel, Query, Solver, Tick};
    struct Loadout([Gear; 6]);
    impl Catalog for Loadout {
        fn armor_count(&self, _: usize) -> u32 { 1 }
        fn armor(&self, slot: usize, _: u32) -> Gear { self.0[slot] }
        fn charm_count(&self) -> u32 { 1 }
        fn charm(&self, _: u32) -> Gear { self.0[5] }
        fn jewel_count(&self) -> u32 { 0 }
        fn jewel(&self, _: u32) -> Jewel { unreachable!() }
    }
    let trees = [35, 25, 144, 179, 180, 202];
    let mut random = 0x843ae91du32;
    for case in 0..512 {
        let mut e = Evaluator::new();
        let mut data = Loadout([Gear::default(); 6]);
        let mut torso = 0;
        for gear in 0..6 {
            if gear < 5 && gear != 1 && case & (1 << gear) != 0 {
                data.0[gear].torso_up = true; torso += 1;
            }
            for (goal, &tree) in trees.iter().enumerate() {
                random = random.wrapping_mul(1664525).wrapping_add(1013904223);
                if random & 3 == 0 { continue; }
                let points = (random >> 24) as i8 as i16;
                data.0[gear].points[goal] = points;
                data.0[gear].present |= 1 << goal;
                e.add(gear, tree, points);
            }
            for (special, tree) in [SKILL_PLUS, TALISMAN_BOOST].into_iter().enumerate() {
                random = random.wrapping_mul(1664525).wrapping_add(1013904223);
                let points = match case % 4 {
                    0 => 0,
                    1 => (random % 7) as i16,
                    _ => (random >> 24) as i8 as i16,
                };
                data.0[gear].extra[special] = points;
                e.add(gear, tree, points);
            }
        }
        e.set_torso_count(torso);
        let query = Query {
            goals: [Goal { threshold: -128, at_least: true }; 6], goal_count: 6,
            native_wrapping: true, boost_nonzero: 0b001100, ..Query::default()
        };
        let mut solver = Solver::new(query);
        let expected = loop {
            match solver.tick(&data, 100) {
                Tick::Working => {}, Tick::Found(solution) => break solution.totals,
                other => panic!("fixture {case}: {other:?}"),
            }
        };
        let actual = e.finish();
        for (goal, tree) in trees.into_iter().enumerate() {
            assert_eq!(i32::from(actual.points[tree as usize]), expected[goal], "fixture {case}, tree {tree}");
        }
    }
}

#[test]
fn higher_activated_tiers_are_useful_not_wasted_points() {
    let tiers = [10, 15, 20];
    assert_eq!(tier_score(10, 10, tiers.into_iter()), (0, 0));
    assert_eq!(tier_score(15, 10, tiers.into_iter()), (5, 0));
    assert_eq!(tier_score(20, 10, tiers.into_iter()), (10, 0));
    assert_eq!(tier_score(25, 10, tiers.into_iter()), (10, 5));
    assert_eq!(tier_score(20, 20, tiers.into_iter()), (0, 0));
}

#[test]
fn fixed_ranking_puts_more_activated_skills_before_other_preferences(){
    assert!(skill_rank(3,1,30)<skill_rank(2,0,40),"three positive skills outrank two, regardless of tier strength");
    assert!(skill_rank(3,0,30)<skill_rank(3,1,45),"equal skill counts prefer fewer penalties");
    assert!(skill_rank(3,0,45)<skill_rank(3,0,30),"equal counts and penalties prefer stronger tiers");
}

#[test]
fn search_stops_after_five_skill_variations_not_five_loadouts() {
    let mut search = SearchBudget::new();
    let mut signature = [0; TREE_COUNT]; signature[18] = 20;
    assert_eq!(search.poll(100.0), None);
    for _ in 0..30 { search.observe(signature); }
    assert_eq!(search.variations, 1);
    assert_eq!(search.poll(159.999), None);
    assert_eq!(search.poll(160.0), Some(SearchStop::Time));
    search = SearchBudget::new();
    search.poll(5.0);
    search.observe(signature);
    for tree in [85, 89, 34] { let mut s = signature; s[tree] = 10; search.observe(s); }
    assert_eq!(search.variations, 4);
    assert_eq!(search.poll(6.0), None);
    let mut s = signature; s[19] = 10; search.observe(s);
    assert_eq!(search.poll(6.0), Some(SearchStop::Variations));
}

#[test]
fn active_skill_signature_ignores_raw_points_and_negative_penalties() {
    let activations = [(18, 10), (18, 15), (18, 20), (85, 10), (85, 15), (19, -10)];
    let mut points = [0; TREE_COUNT]; points[18] = 21; points[85] = 9; points[19] = -11;
    let first = skill_signature(&points, activations.into_iter());
    assert_eq!(first[18], 20); assert_eq!(first[85], 0); assert_eq!(first[19], 0);
    points[18] = 25; points[85] = 5;
    assert_eq!(first, skill_signature(&points, activations.into_iter()));
    points[85] = 10;
    assert_ne!(first, skill_signature(&points, activations.into_iter()));
}

#[test]
fn weapon_priorities_prefer_combat_to_unrelated_skill_count(){
 let p=PriorityProfile::new(0,0,0);
 let mut combat=Totals{points:[0;206],present:[false;206]};combat.points[18]=20;combat.points[71]=10;
 let mut utility=Totals{points:[0;206],present:[false;206]};utility.points[18]=20;for t in [131,132,133,134]{utility.points[t]=10;}
 let activations=[(18,10),(18,15),(18,20),(71,10),(131,10),(132,10),(133,10),(134,10)];
 assert!(p.rank(&combat,activations.into_iter())<p.rank(&utility,activations.into_iter()));
 let only_l=p.rank(&combat,[(18,20),(71,10)].into_iter());
 assert_eq!(p.rank(&combat,activations.into_iter()),only_l,"lower tiers must not be counted twice");
}
#[test]
fn weapon_priorities_require_matching_element_status_and_weapon(){
 let gs=PriorityProfile::new(0,27,0);
 assert_eq!(gs.preference(28).0,0);assert_eq!(gs.preference(33).0,0);
 assert_eq!(gs.preference(27).0,4);assert_eq!(gs.preference(32).0,4);
 assert_eq!(gs.preference(89).0,0);assert_eq!(gs.preference(85).0,2);
 let lance=PriorityProfile{weapon:6,..gs};assert_eq!(lance.preference(85).0,2);assert!(lance.preference(85).1>gs.preference(85).1);
 let horn=PriorityProfile{weapon:5,..gs};assert_eq!(horn.preference(89).0,3);
 let raw=PriorityProfile{element:0,..gs};assert_eq!(raw.preference(27).0,0);assert_eq!(raw.preference(74).0,0);
 let poison=PriorityProfile{status:1,..raw};assert_eq!(poison.preference(33).0,4);
 let blast=PriorityProfile{status:92,..raw};assert_eq!(blast.preference(33).0,0);assert_eq!(blast.preference(92).0,4);
 assert!(gs.preference(40).1>PriorityProfile{weapon:3,..gs}.preference(40).1);
}

#[test]
fn weapon_priority_refinement_rewards_matching_progress_and_rejects_penalties(){
 let p=PriorityProfile::new(3,27,0);
 let mut fire=Totals{points:[0;206],present:[false;206]};fire.points[27]=9;
 let mut water=Totals{points:[0;206],present:[false;206]};water.points[28]=9;
 let rows=[(27,10),(28,10),(19,-10)];
 assert!(p.progress(&fire,rows.into_iter())>p.progress(&water,rows.into_iter()));
 fire.points[27]=10;water.points[28]=10;
 assert!(p.rank(&fire,rows.into_iter())<p.rank(&water,rows.into_iter()));
 let clean=p.rank(&fire,rows.into_iter());fire.points[19]=-10;
 assert!(clean<p.rank(&fire,rows.into_iter()),"same offense must prefer no negative activation");
}

#[test]
fn category_order_and_skill_order_change_build_preference(){
 let mut p=PriorityProfile::new(0,0,0);
 let mut attack=Totals{points:[0;206],present:[false;206]};attack.points[18]=10;
 let mut defense=Totals{points:[0;206],present:[false;206]};defense.points[19]=10;
 let rows=[(18,10),(19,10)];assert!(p.rank(&attack,rows.into_iter())<p.rank(&defense,rows.into_iter()));
 p.order=[2,0,1,3];assert!(p.rank(&defense,rows.into_iter())<p.rank(&attack,rows.into_iter()));
 p.order=[0,1,2,3];p.trees[70]=200;
 let mut expert=Totals{points:[0;206],present:[false;206]};expert.points[70]=10;
 assert!(p.rank(&expert,[(18,10),(70,10)].into_iter())<p.rank(&attack,[(18,10),(70,10)].into_iter()));
 assert_eq!(p.preference(77).0,3,"Focus has its own weapon-support category");
}

#[test]
fn adrenaline_one_is_defense_and_only_tier_two_counts_as_damage(){
 let p=PriorityProfile::new(6,0,0);let mut all=Totals{points:[0;206],present:[false;206]};all.points[98]=10;
 let rows=[(98,10),(98,15)];let first=p.scores(&all,rows.into_iter());assert_eq!(first[0],0);assert!(first[2]>0);
 all.points[98]=15;let second=p.scores(&all,rows.into_iter());assert!(second[0]>0);assert_eq!(second[2],0);
}

#[test]
fn priority_groups_are_user_assignments_not_fixed_category_names(){
 let mut p=PriorityProfile::new(0,0,0);
 p.groups[18]=3;
 assert_eq!(p.preference(18).0,2);
 p.groups[98]=4;
 assert_eq!(p.activation_preference(98,10).0,1);
 assert_eq!(p.activation_preference(98,15).0,1);
 // An incompatible weapon skill remains irrelevant even with a custom group.
 p.groups[89]=1;assert_eq!(p.preference(89).0,0);
}
