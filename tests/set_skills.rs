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
