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

/// Higher activated tiers count as upgrades; only points past that tier are excess.
pub fn tier_score(points:i16,minimum:i16,thresholds:impl Iterator<Item=i16>)->(u16,u16){
 let achieved=thresholds.filter(|&p|p>0&&p<=points).max().unwrap_or(minimum);
 ((achieved-minimum).max(0)as u16,(points-achieved).max(0)as u16)
}

/// Fixed result order: useful activations, penalties, then activation strength.
/// Lower keys rank first. Each tree contributes at most one positive activation.
pub fn skill_rank(active:u32,penalties:u32,strength:u32)->[u32;3]{
    [u32::MAX-active,penalties,u32::MAX-strength]
}

/// A variation is a set of positive activated tiers, not an armor/charm loadout.
pub fn skill_signature(points:&[i16;TREE_COUNT],activations:impl Iterator<Item=(u8,i8)>)->[i8;TREE_COUNT]{
 let mut signature=[0;TREE_COUNT];
 for(tree,threshold)in activations{let t=tree as usize;if threshold>0&&points[t]>=i16::from(threshold){signature[t]=signature[t].max(threshold);}}
 signature
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum SearchStop{Time,Variations}
pub const SEARCH_SECONDS:f64=60.0;
pub const SEARCH_VARIATIONS:usize=5;
pub struct SearchBudget{started:Option<f64>,signatures:[[i8;TREE_COUNT];SEARCH_VARIATIONS],pub variations:usize,pub elapsed:f64}
impl SearchBudget{
 pub const fn new()->Self{Self{started:None,signatures:[[0;TREE_COUNT];SEARCH_VARIATIONS],variations:0,elapsed:0.0}}
 pub fn observe(&mut self,signature:[i8;TREE_COUNT]){
  if self.variations<SEARCH_VARIATIONS&&!self.signatures[..self.variations].contains(&signature){self.signatures[self.variations]=signature;self.variations+=1;}
 }
 pub fn poll(&mut self,now:f64)->Option<SearchStop>{
  let started=*self.started.get_or_insert(now);self.elapsed=(now-started).max(self.elapsed);
  if self.variations>=SEARCH_VARIATIONS{Some(SearchStop::Variations)}else if self.elapsed>=SEARCH_SECONDS{Some(SearchStop::Time)}else{None}
 }
}
