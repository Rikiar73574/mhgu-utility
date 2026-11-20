//! Complete, allocation-free native skill arithmetic for a finished loadout.

pub const TREE_COUNT: usize = 206;
pub const TORSO_UP: u8 = 203;
pub const SKILL_PLUS: u8 = 204;
pub const TALISMAN_BOOST: u8 = 205;
const GEARS: usize = 7;

pub struct Totals {
    /// Signed-byte native totals, widened for display and threshold comparison.
    pub points: [i16; TREE_COUNT],
    /// A source row existed, even if its aggregate points are zero.
    pub present: [bool; TREE_COUNT],
}

pub struct Evaluator {
    per_gear: [[i16; TREE_COUNT]; GEARS],
    present: [bool; TREE_COUNT],
    torso_count: u8,
}

impl Evaluator {
    pub const fn new() -> Self {
        Self { per_gear: [[0; TREE_COUNT]; GEARS], present: [false; TREE_COUNT], torso_count: 0 }
    }

    /// Add armor/charm rows and each installed jewel's rows to its own gear.
    /// Gear indices: head, chest, arms, waist, legs, charm, weapon jewels.
    /// Callers must omit sentinel/no-skill rows. Invalid indices return false.
    pub fn add(&mut self, gear: usize, tree: u8, points: i16) -> bool {
        if gear >= GEARS || tree as usize >= TREE_COUNT { return false; }
        self.per_gear[gear][tree as usize] = self.per_gear[gear][tree as usize].wrapping_add(points);
        self.present[tree as usize] = true;
        true
    }

    /// Number of Torso Up armor pieces outside the chest slot.
    pub fn set_torso_count(&mut self, count: u8) { self.torso_count = count; }

    pub fn finish(&self) -> Totals {
        let mut points = [0i16; TREE_COUNT];
        let multiplier = i16::from(self.torso_count) + 1;
        for (tree, value) in points.iter_mut().enumerate() {
            for gear in 0..GEARS {
                let contribution = self.per_gear[gear][tree].wrapping_mul(if gear == 1 { multiplier } else { 1 });
                *value = value.wrapping_add(contribution);
            }
            *value = native(*value);
        }
        let double = points[TALISMAN_BOOST as usize] >= 10;
        let mut boost = points[SKILL_PLUS as usize] >= 10;
        if double {
            self.add_charm(&mut points);
            boost |= points[SKILL_PLUS as usize] >= 10;
        }
        if boost {
            for (tree, value) in points.iter_mut().enumerate() {
                if tree >= SKILL_PLUS as usize { continue; }
                let nonzero = *value != 0 || (0..GEARS).any(|gear| {
                    let factor = if gear == 1 { multiplier } else if gear == 5 && double { 2 } else { 1 };
                    native(self.per_gear[gear][tree].wrapping_mul(factor)) != 0
                });
                if self.present[tree] && (!(144..=179).contains(&tree) || nonzero) {
                    *value = native(value.wrapping_add(2));
                }
            }
            // Match Solver::final_totals: boost both special accumulators before
            // checking whether Skill+2 has newly activated Double Talisman.
            for tree in [SKILL_PLUS, TALISMAN_BOOST] {
                points[tree as usize] = native(points[tree as usize].wrapping_add(2));
            }
            if !double && points[TALISMAN_BOOST as usize] >= 10 {
                self.add_charm(&mut points);
            }
        }
        Totals { points, present: self.present }
    }

    fn add_charm(&self, points: &mut [i16; TREE_COUNT]) {
        for (tree, value) in points.iter_mut().enumerate() {
            *value = native(value.wrapping_add(self.per_gear[5][tree]));
        }
    }
}

fn native(points: i16) -> i16 { points as i8 as i16 }
