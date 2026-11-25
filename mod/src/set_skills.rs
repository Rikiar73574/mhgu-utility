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

/// Preview policy, not estimated DPS. Weapon order matches the preview selector:
/// GS, LS, SnS, DB, Hammer, HH, Lance, GL, SA, CB, IG, LBG, HBG, Bow, unknown.
/// Element/status use native skill-tree IDs; missing weapon data is never guessed.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct PriorityProfile { pub weapon:u8, pub element:u8, pub status:u8, pub order:[u8;4], pub trees:[u8;TREE_COUNT], pub groups:[u8;TREE_COUNT] }
impl PriorityProfile {
    pub const fn new(weapon:u8,element:u8,status:u8)->Self{Self{weapon,element,status,order:[0,1,2,3],trees:[0;TREE_COUNT],groups:[0;TREE_COUNT]}}
    /// Category: 0 irrelevant, 1 utility, 2 comfort, 3 weapon support, 4 damage.
    /// Small ordinal weights express recommendations, not damage percentages.
    pub fn preference(self,tree:u8)->(u8,u32) {
        let melee=self.weapon<11;let known=self.weapon<14;
        let (category,weight)=match tree {
            18|70|71=> (4,24),
            72=> (4,if matches!(self.weapon,2|3|10|11|12|13){26}else{18}),
            75=> (4,18),
            27..=31=> if self.element==tree{(4,22)}else{(0,0)},
            32|74=> if self.element!=0{(4,18)}else{(0,0)},
            33|73=> if matches!(self.status,1..=3){(4,16)}else{(0,0)},
            92=> if self.status==92{(4,16)}else{(1,1)},
            35|36|38=> if melee{(3,18)}else{(0,0)},
            // Draw bonuses are a modest GS recommendation, not a selectable playstyle.
            40|41=> if self.weapon==0{(4,18)}else{(0,0)},
            77=> if matches!(self.weapon,0|1|3|4|8|9|13){(3,24)}else{(0,0)},
            89=> if self.weapon==5{(3,28)}else{(0,0)},
            // Phial/ammo data is not supplied: do not assume impact CB or a shot type.
            90=> if self.weapon==7{(3,24)}else{(1,1)},
            48..=64|66=> (0,0),
            65|68|69=> if matches!(self.weapon,11|12){(3,18)}else{(0,0)},
            67=> if matches!(self.weapon,7|9|11|12|13){(3,18)}else{(0,0)},
            94|95|100=> (4,18),
            96|98|99|101=> (4,10), // Conditional offense is a recommendation, not assumed uptime.
            85|86=> if matches!(self.weapon,2|6|7|9){(2,18)}else if self.weapon==0{(2,3)}else{(0,0)},
            42=> if known{(3,if matches!(self.weapon,0|1|7|12){20}else{8})}else{(0,0)},
            34|37|43|44=> if melee{(2,8)}else{(0,0)},
            45..=47=> if matches!(self.weapon,11|12){(2,12)}else{(0,0)},
            78..=80=> if matches!(self.weapon,3|13){(2,16)}else{(2,6)},
            87|88=> if matches!(self.weapon,4|5){(2,12)}else{(0,0)},
            1..=17|19..=26|81..=84|102|115..=127|201|202=> (2,6),
            0|203..=205=> (0,0),
            _=> (1,1),
        };let group=self.groups[tree as usize];let category=if category>0&&(1..=4).contains(&group){5-group}else{category};(category,if self.trees[tree as usize]>0{u32::from(self.trees[tree as usize])}else{weight})
    }
    pub fn activation_preference(self,tree:u8,threshold:i16)->(u8,u32){if tree==98&&threshold<15&&self.groups[tree as usize]==0{(2,6)}else{self.preference(tree)}}
    pub fn scores(self,all:&Totals,activations:impl Iterator<Item=(u8,i8)>)->[u32;5] {
        let mut strongest=[0i16;TREE_COUNT];
        for(tree,threshold)in activations{if threshold>0&&all.points[tree as usize]>=i16::from(threshold){strongest[tree as usize]=strongest[tree as usize].max(i16::from(threshold));}}
        let mut out=[0;5];
        for(tree,&threshold)in strongest.iter().enumerate(){if threshold==0{continue;}
            let(category,weight)=self.activation_preference(tree as u8,threshold);
            if category>0{out[4-category as usize]+=weight+weight*(threshold-10).max(0)as u32/20;}
            out[4]+=1;
        }out
    }
    pub fn rank(self,all:&Totals,activations:impl Iterator<Item=(u8,i8)>)->[u32;6] {
        let mut strongest=[0i8;TREE_COUNT];let mut negative=[false;TREE_COUNT];
        for(tree,p)in activations{if p>0&&all.points[tree as usize]>=i16::from(p){strongest[tree as usize]=strongest[tree as usize].max(p);}if p<0&&all.points[tree as usize]<=i16::from(p){negative[tree as usize]=true;}}
        let scores=self.scores(all,strongest.iter().enumerate().map(|(i,&p)|(i as u8,p)));
        [u32::MAX-scores[self.order[0]as usize],negative.iter().filter(|&&v|v).count()as u32,u32::MAX-scores[self.order[1]as usize],u32::MAX-scores[self.order[2]as usize],u32::MAX-scores[self.order[3]as usize],u32::MAX-scores[4]]
    }
    pub fn progress(self,all:&Totals,activations:impl Iterator<Item=(u8,i8)>)->u32 {
        let mut next=[i16::MAX;TREE_COUNT];
        for(tree,p)in activations{if p>0&&all.points[tree as usize]>0&&all.points[tree as usize]<i16::from(p){next[tree as usize]=next[tree as usize].min(i16::from(p));}}
        next.iter().enumerate().filter(|(_,p)|**p!=i16::MAX).map(|(tree,&p)|{let(c,w)=self.activation_preference(tree as u8,p);let scale=if c==0{0}else{let at=self.order.iter().position(|&v|v==4-c).unwrap_or(3);[100000,1000,10,1][at]};scale*w*all.points[tree].max(0)as u32/p as u32}).sum()
    }
}
