//! Allocation-free, resumable mixed-set search. Catalog indices must remain stable
//! for the life of a search. The caller filters armor for progression/type/gender,
//! supplies every owned charm (and an empty charm if desired), and validates any
//! unrequested skill trees in `Found` results. Discarding a result is safe: the
//! next tick continues enumeration. No candidate or jewel loadout is truncated.
//!
//! Each budget unit scans one catalog record or visits one bounded DFS edge.
//! Exact search is exponential; `Working` never means no solution exists.

pub const MAX_GOALS: usize = 6;
pub const MAX_JEWELS: usize = 21;
const BINS: usize = 7;
// A bounded ordering hint, never a filter on the catalog.
const PREFERRED: usize = 8;
const FRAMES: usize = MAX_JEWELS + BINS + 1;
pub type Points = [i16; MAX_GOALS];

#[derive(Clone, Copy, Debug, Default)]
pub struct Gear {
    pub points: Points,
    /// Requested tree rows present, including rows whose aggregate is zero.
    pub present: u8,
    pub slots: u8,
    /// Secret Arts and Talisman Boost tree points, before any multipliers.
    pub extra: [i16; 2],
    pub torso_up: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Jewel {
    pub points: Points,
    /// Requested tree rows present, including rows whose aggregate is zero.
    pub present: u8,
    pub slots: u8,
    pub extra: [i16; 2],
}

/// Armor slots are head, chest, arms, waist, legs, in that order.
pub trait Catalog {
    fn armor_count(&self, slot: usize) -> u32;
    fn armor(&self, slot: usize, index: u32) -> Gear;
    fn charm_count(&self) -> u32;
    fn charm(&self, index: u32) -> Gear;
    fn jewel_count(&self) -> u32;
    fn jewel(&self, index: u32) -> Jewel;
    /// Ordering preference only. Unverified records still participate fully.
    /// Kinds 0..4 are armor slots, 5 charms, and 6 jewels.
    fn verified(&self, _kind: usize, _index: u32) -> bool { true }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Goal {
    pub threshold: i16,
    /// false requests a negative activation, e.g. points <= -10.
    pub at_least: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Query {
    pub goals: [Goal; MAX_GOALS],
    pub goal_count: u8,
    pub weapon_slots: u8,
    /// Use the game's signed-byte point arithmetic.
    pub native_wrapping: bool,
    /// Deviant tree goal bits: boost only if some per-gear or aggregate points remain.
    pub boost_nonzero: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Placement {
    pub jewel: u32,
    /// 0..4 armor, 5 charm, 6 weapon. Chest jewels are multiplied by Torso Up.
    pub gear: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Solution {
    pub armor: [u32; 5],
    pub charm: u32,
    pub jewels: [Placement; MAX_JEWELS],
    pub jewel_count: u8,
    pub totals: [i32; MAX_GOALS],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tick {
    Working,
    Found(Solution),
    /// All candidates and legal per-gear jewel multisets have been considered.
    Complete,
    Cancelled,
    InvalidCatalog,
}

#[derive(Clone, Copy, Default)]
struct Frame {
    bin: u8,
    remaining: u8,
    count: u8,
    minimum: u32,
    // zero = advance to next bin; 1..=jewel_count = try that jewel.
    next: u64,
    totals: [i32; MAX_GOALS + 2],
    charm: [i32; MAX_GOALS + 2],
    per_bin: [[i32; MAX_GOALS]; BINS],
    present: u8,
}

pub struct Solver {
    query: Query,
    // 0 scanning, 1 armor DFS, 2 jewel DFS, 3 complete, 4 cancelled, 5 invalid.
    phase: u8,
    scan_kind: usize,
    scan_index: u32,
    counts: [u32; 7],
    maxima: [[i32; MAX_GOALS]; 6],
    minima: [[i32; MAX_GOALS]; 6],
    max_slots: [u8; 6],
    can_torso: [bool; 6],
    efficiency: [i32; MAX_GOALS],
    negative_efficiency: [i32; MAX_GOALS],
    preferred: [[u32; PREFERRED]; 6],
    preferred_score: [[i64; PREFERRED]; 6],
    preferred_verified: [[bool; PREFERRED]; 6],
    preferred_count: [u8; 6],
    preferred_used: [u8; 6],
    best: [u32; 6],
    best_verified: [bool; 6],
    best_jewel: u32,
    best_jewel_verified: bool,
    best_jewel_score: i64,
    best_score: [i64; 6],
    depth: usize,
    next: [u64; 6],
    chosen: [u32; 6],
    gear: [Gear; 6],
    partial: [[i32; MAX_GOALS + 2]; 7],
    specials: bool,
    bins: [u8; BINS],
    multiplier: u8,
    frames: [Frame; FRAMES],
    frame_count: usize,
    placements: [Placement; MAX_JEWELS],
    after_found: bool,
    equipment_found: bool,
    pub work_done: u64,
}

impl Solver {
    pub fn new(query: Query) -> Self {
        Self {
            phase: if query.goal_count as usize > MAX_GOALS || query.weapon_slots > 3 { 5 } else { 0 },
            query, scan_kind: 0, scan_index: 0, counts: [0; 7],
            maxima: [[i32::MIN; MAX_GOALS]; 6], minima: [[i32::MAX; MAX_GOALS]; 6], max_slots: [0; 6],
            can_torso: [false; 6], efficiency: [0; MAX_GOALS], negative_efficiency: [0; MAX_GOALS],
            preferred: [[0; PREFERRED]; 6], preferred_score: [[i64::MIN; PREFERRED]; 6],
            preferred_verified: [[false; PREFERRED]; 6], preferred_count: [0; 6], preferred_used: [0; 6],
            best: [0; 6], best_verified: [false; 6], best_jewel: 0, best_jewel_verified: false, best_jewel_score: i64::MIN, best_score: [i64::MIN; 6], depth: 0,
            next: [0; 6], chosen: [0; 6], gear: [Gear::default(); 6],
            partial: [[0; MAX_GOALS + 2]; 7], specials: false, bins: [0; BINS], multiplier: 1,
            frames: [Frame::default(); FRAMES], frame_count: 0,
            placements: [Placement::default(); MAX_JEWELS], after_found: false, equipment_found: false, work_done: 0,
        }
    }

    pub fn cancel(&mut self) { if self.phase < 3 { self.phase = 4; } }

    /// After accepting `Found`, skip the remaining decoration layouts for that
    /// armor/charm combination. Returns false unless the last tick found a
    /// solution. Call only after full adapter validation: rejected layouts may
    /// have acceptable alternatives. `Complete` then proves equipment-combination
    /// coverage, rather than enumeration of every decoration layout.
    pub fn next_equipment(&mut self) -> bool {
        if self.phase != 2 || !self.after_found { return false; }
        self.finish_equipment()
    }

    /// End bounded decoration refinement for the current armor/charm combination,
    /// even after a `Working` tick. Allowed only after this equipment has emitted
    /// `Found`; the marker resets when a new combination starts. Callers must
    /// have validated and retained a solution before skipping its alternatives.
    /// As with `next_equipment`, completion covers equipment combinations, not
    /// every decoration layout. Returns false outside that equipment's jewel DFS.
    pub fn finish_equipment(&mut self) -> bool {
        if self.phase != 2 || !self.equipment_found { return false; }
        self.frame_count = 0;
        self.phase = 1;
        self.after_found = false;
        self.equipment_found = false;
        true
    }

    fn orient(&self, value: i32, goal: usize) -> i32 {
        if self.query.goals[goal].at_least { value } else { -value }
    }

    pub fn tick<C: Catalog>(&mut self, catalog: &C, budget: u32) -> Tick {
        self.after_found = false;
        for _ in 0..budget {
            if self.phase >= 3 { break; }
            self.work_done = self.work_done.saturating_add(1);
            match self.phase {
                0 => self.scan(catalog),
                1 => self.armor_step(catalog),
                2 => if let Some(solution) = self.jewel_step(catalog) {
                    self.after_found = true;
                    self.equipment_found = true;
                    return Tick::Found(solution);
                },
                _ => {},
            }
        }
        match self.phase {
            3 => Tick::Complete, 4 => Tick::Cancelled, 5 => Tick::InvalidCatalog,
            _ => Tick::Working,
        }
    }

    fn scan<C: Catalog>(&mut self, catalog: &C) {
        let kind = self.scan_kind;
        if self.scan_index == 0 {
            self.counts[kind] = match kind {
                0..=4 => catalog.armor_count(kind),
                5 => catalog.charm_count(), _ => catalog.jewel_count(),
            };
        }
        if self.scan_index == self.counts[kind] {
            if kind < 6 && self.counts[kind] == 0 { self.phase = 3; return; }
            self.scan_kind += 1;
            self.scan_index = 0;
            if self.scan_kind == 7 {
                self.phase = if self.armor_possible(0) { 1 } else { 3 };
            }
            return;
        }
        let index = self.scan_index;
        let verified = catalog.verified(kind, index);
        self.scan_index += 1;
        if kind == 6 {
            let jewel = catalog.jewel(index);
            self.specials |= jewel.extra != [0; 2];
            if jewel.slots == 0 || jewel.slots > 3 { self.phase = 5; return; }
            let mut score = 0i64;
            for g in 0..self.query.goal_count as usize {
                let target = self.orient(self.query.goals[g].threshold as i32, g).max(0);
                score += self.orient(jewel.points[g] as i32, g).min(target) as i64 * 16 / jewel.slots as i64;
                let negative = (-self.orient(jewel.points[g] as i32, g)).max(0);
                self.negative_efficiency[g] = self.negative_efficiency[g].max((negative + jewel.slots as i32 - 1) / jewel.slots as i32);
                let points = self.orient(jewel.points[g] as i32, g).max(0);
                self.efficiency[g] = self.efficiency[g].max((points + jewel.slots as i32 - 1) / jewel.slots as i32);
            }
            if (verified, score) > (self.best_jewel_verified, self.best_jewel_score) {
                self.best_jewel_verified = verified;
                self.best_jewel_score = score;
                self.best_jewel = index;
            }
        } else {
            let gear = if kind == 5 { catalog.charm(index) } else { catalog.armor(kind, index) };
            self.specials |= gear.extra != [0; 2];
            if gear.slots > 3 || (kind == 5 && gear.torso_up) { self.phase = 5; return; }
            self.max_slots[kind] = self.max_slots[kind].max(gear.slots);
            self.can_torso[kind] |= gear.torso_up && kind != 1;
            let mut score = gear.slots as i64 * 4 + if gear.torso_up { 8 } else { 0 };
            for g in 0..self.query.goal_count as usize {
                let points = self.orient(gear.points[g] as i32, g);
                self.maxima[kind][g] = self.maxima[kind][g].max(points);
                self.minima[kind][g] = self.minima[kind][g].min(points);
                let target = self.orient(self.query.goals[g].threshold as i32, g);
                score += points.min(target.max(0)) as i64 * 16
                    - (points - target.max(0)).max(0) as i64 * 16;
            }
            // Retain a few promising records while doing the required scan.
            // Equal scores preserve catalog order. All omitted records are still
            // visited by the fallback pass, including unverified candidates.
            let count = self.preferred_count[kind] as usize;
            let mut position = 0;
            while position < count && (self.preferred_verified[kind][position], self.preferred_score[kind][position]) >= (verified, score) {
                position += 1;
            }
            if position < PREFERRED {
                for at in (position + 1..(count + 1).min(PREFERRED)).rev() {
                    self.preferred[kind][at] = self.preferred[kind][at - 1];
                    self.preferred_score[kind][at] = self.preferred_score[kind][at - 1];
                    self.preferred_verified[kind][at] = self.preferred_verified[kind][at - 1];
                }
                self.preferred[kind][position] = index;
                self.preferred_score[kind][position] = score;
                self.preferred_verified[kind][position] = verified;
                self.preferred_count[kind] = (count + 1).min(PREFERRED) as u8;
            }
            if (verified, score) > (self.best_verified[kind], self.best_score[kind]) {
                self.best_verified[kind] = verified;
                self.best_score[kind] = score;
                self.best[kind] = index;
            }
        }
    }

    // Optimistic independent skill and slot bounds; never discard a candidate
    // because it is dominated on requested trees (other trees may matter).
    fn armor_possible(&self, depth: usize) -> bool {
        let mut torso = 0i32;
        let mut slots = self.query.weapon_slots as i32;
        for s in 0..6 {
            if s < depth {
                slots += self.gear[s].slots as i32;
                if s != 1 && self.gear[s].torso_up { torso += 1; }
            } else {
                slots += self.max_slots[s] as i32;
                torso += self.can_torso[s] as i32;
            }
        }
        let chest_slots = if depth > 1 { self.gear[1].slots } else { self.max_slots[1] } as i32;
        for g in 0..self.query.goal_count as usize {
            let mut bound = self.orient(self.partial[depth][g], g);
            let mut lower = bound;
            for s in depth..6 { bound += self.maxima[s][g]; lower += self.minima[s][g]; }
            let chest = if depth > 1 { self.orient(self.gear[1].points[g] as i32, g) } else { self.maxima[1][g] };
            let low_chest = if depth > 1 { self.orient(self.gear[1].points[g] as i32, g) } else { self.minima[1][g] };
            let jewel_slots = slots + chest_slots * torso;
            bound += chest.max(0) * torso + jewel_slots * self.efficiency[g];
            lower += low_chest.min(0) * torso - jewel_slots * self.negative_efficiency[g];
            if self.specials {
                let charm = if depth == 6 { self.orient(self.gear[5].points[g] as i32, g) } else { self.maxima[5][g] };
                bound += charm.max(0) + self.max_slots[5] as i32 * self.efficiency[g] + 2;
                let low_charm = if depth == 6 { self.orient(self.gear[5].points[g] as i32, g) } else { self.minima[5][g] };
                lower += low_charm.min(0) - self.max_slots[5] as i32 * self.negative_efficiency[g] - 2;
            }
            if !self.range_possible(lower, bound, g) { return false; }
        }
        true
    }

    fn armor_step<C: Catalog>(&mut self, catalog: &C) {
        let d = self.depth;
        let preferred_count = self.preferred_count[d] as u32;
        if self.next[d] == self.counts[d] as u64 + preferred_count as u64 {
            self.next[d] = 0;
            self.preferred_used[d] = 0;
            if d == 0 { self.phase = 3; } else { self.depth -= 1; }
            return;
        }
        let index;
        if self.next[d] < preferred_count as u64 {
            // Re-rank only the bounded shortlist for this partial set. Useful
            // contribution stops at the remaining deficit; excess is penalized.
            // This is a soft preference even under wrapping/special arithmetic:
            // neither candidate eligibility nor feasibility bounds change.
            let mut selected = 0;
            let mut best = (false, i64::MIN);
            for at in 0..preferred_count as usize {
                if self.preferred_used[d] & (1 << at) != 0 { continue; }
                let candidate = self.preferred[d][at];
                let gear = if d == 5 { catalog.charm(candidate) } else { catalog.armor(d, candidate) };
                let mut score = gear.slots as i64 * 4 + if gear.torso_up { 8 } else { 0 };
                for g in 0..self.query.goal_count as usize {
                    let deficit = (self.orient(self.query.goals[g].threshold as i32 - self.partial[d][g], g)).max(0);
                    let points = self.orient(gear.points[g] as i32, g);
                    score += points.min(deficit) as i64 * 16 - (points - deficit).max(0) as i64 * 16;
                }
                let rank = (self.preferred_verified[d][at], score);
                if rank > best { best = rank; selected = at; }
            }
            self.preferred_used[d] |= 1 << selected;
            index = self.preferred[d][selected];
        } else {
            let ordinal = self.next[d] - preferred_count as u64;
            index = ((self.best[d] as u64 + ordinal as u64) % self.counts[d] as u64) as u32;
            self.next[d] += 1;
            if self.preferred[d][..preferred_count as usize].contains(&index) { return; }
            let gear = if d == 5 { catalog.charm(index) } else { catalog.armor(d, index) };
            self.visit_armor(d, index, gear);
            return;
        }
        self.next[d] += 1;
        let gear = if d == 5 { catalog.charm(index) } else { catalog.armor(d, index) };
        self.visit_armor(d, index, gear);
    }

    fn visit_armor(&mut self, d: usize, index: u32, gear: Gear) {
        self.gear[d] = gear;
        self.chosen[d] = index;
        for g in 0..self.query.goal_count as usize {
            self.partial[d + 1][g] = self.partial[d][g] + gear.points[g] as i32;
        }
        for g in 0..2 { self.partial[d + 1][MAX_GOALS + g] = self.partial[d][MAX_GOALS + g] + gear.extra[g] as i32; }
        if !self.armor_possible(d + 1) { return; }
        if d < 5 { self.depth += 1; return; }
        self.multiplier = 1;
        for s in 0..6 {
            self.bins[s] = self.gear[s].slots;
            if s != 1 && self.gear[s].torso_up { self.multiplier += 1; }
        }
        self.bins[6] = self.query.weapon_slots;
        let mut totals = self.partial[6];
        for g in 0..self.query.goal_count as usize {
            totals[g] += self.gear[1].points[g] as i32 * (self.multiplier as i32 - 1);
        }
        for g in 0..2 { totals[MAX_GOALS + g] += self.gear[1].extra[g] as i32 * (self.multiplier as i32 - 1); }
        let mut charm = [0; MAX_GOALS + 2];
        for g in 0..self.query.goal_count as usize { charm[g] = self.gear[5].points[g] as i32; }
        for g in 0..2 { charm[MAX_GOALS + g] = self.gear[5].extra[g] as i32; }
        let mut per_bin = [[0; MAX_GOALS]; BINS];
        let mut present = 0;
        for b in 0..6 {
            present |= self.gear[b].present;
            for g in 0..self.query.goal_count as usize {
                per_bin[b][g] = self.gear[b].points[g] as i32 * if b == 1 { self.multiplier as i32 } else { 1 };
                if self.gear[b].points[g] != 0 { present |= 1 << g; }
            }
        }
        self.frames[0] = Frame { charm, per_bin, present, remaining: self.bins[0], totals, ..Frame::default() };
        self.frame_count = 1;
        self.equipment_found = false;
        self.phase = 2;
    }

    fn jewel_possible(&self, frame: Frame) -> bool {
        let mut slots = 0i32;
        for b in frame.bin as usize..BINS {
            let remaining = if b == frame.bin as usize { frame.remaining } else { self.bins[b] };
            slots += remaining as i32 * if b == 1 { self.multiplier as i32 } else { 1 };
        }
        for g in 0..self.query.goal_count as usize {
            let mut upper = self.orient(frame.totals[g], g) + slots * self.efficiency[g];
            let mut lower = self.orient(frame.totals[g], g) - slots * self.negative_efficiency[g];
            if self.specials {
                let charm_slots = if frame.bin < 5 { self.bins[5] } else if frame.bin == 5 { frame.remaining } else { 0 } as i32;
                upper += self.orient(frame.charm[g], g).max(0) + charm_slots * self.efficiency[g] + 2;
                lower += self.orient(frame.charm[g], g).min(0) - charm_slots * self.negative_efficiency[g] - 2;
            }
            if !self.range_possible(lower, upper, g) { return false; }
        }
        true
    }

    // For native arithmetic, test the optimistic mathematical interval against
    // every signed-byte residue interval that can activate the requested skill.
    // This preserves pruning without assuming monotonic wrapping arithmetic.
    fn range_possible(&self, lower: i32, upper: i32, g: usize) -> bool {
        if !self.query.native_wrapping {
            return upper >= self.orient(self.query.goals[g].threshold as i32, g);
        }
        let goal = self.query.goals[g];
        let (lo, hi, wanted_lo, wanted_hi) = if goal.at_least {
            (lower, upper, (goal.threshold as i32).max(-128), 127)
        } else {
            (-upper, -lower, -128, (goal.threshold as i32).min(127))
        };
        if wanted_lo > wanted_hi { return false; }
        let difference = lo - wanted_hi;
        let k = difference.div_euclid(256) + if difference.rem_euclid(256) != 0 { 1 } else { 0 };
        wanted_lo + k * 256 <= hi
    }

    fn arithmetic(&self, points: i32) -> i32 {
        if self.query.native_wrapping { points as i8 as i32 } else { points }
    }

    fn final_totals(&self, frame: Frame) -> [i32; MAX_GOALS] {
        let mut values = frame.totals;
        for value in &mut values { *value = self.arithmetic(*value); }
        let double = values[MAX_GOALS + 1] >= 10;
        let mut boost = values[MAX_GOALS] >= 10;
        if double {
            for g in 0..MAX_GOALS + 2 { values[g] = self.arithmetic(values[g] + frame.charm[g]); }
            boost |= values[MAX_GOALS] >= 10;
        }
        if boost {
            for g in 0..self.query.goal_count as usize {
                let bit = 1 << g;
                let mut nonzero = values[g] != 0;
                for b in 0..BINS {
                    let points = frame.per_bin[b][g] * if b == 5 && double { 2 } else { 1 };
                    nonzero |= self.arithmetic(points) != 0;
                }
                if frame.present & bit != 0 && (self.query.boost_nonzero & bit == 0 || nonzero) {
                    values[g] = self.arithmetic(values[g] + 2);
                }
            }
            // Both special trees use normal row semantics. Absent special rows
            // can never be near +10, so their presence cannot change activation.
            for g in MAX_GOALS..MAX_GOALS + 2 { values[g] = self.arithmetic(values[g] + 2); }
            if !double && values[MAX_GOALS + 1] >= 10 {
                for g in 0..MAX_GOALS + 2 { values[g] = self.arithmetic(values[g] + frame.charm[g]); }
            }
        }
        let mut totals = [0; MAX_GOALS];
        totals[..self.query.goal_count as usize].copy_from_slice(&values[..self.query.goal_count as usize]);
        totals
    }

    fn jewel_step<C: Catalog>(&mut self, catalog: &C) -> Option<Solution> {
        if self.frame_count == 0 { self.phase = 1; return None; }
        let top = self.frame_count - 1;
        let frame = self.frames[top];
        if frame.bin as usize == BINS {
            self.frame_count -= 1;
            let totals = self.final_totals(frame);
            for g in 0..self.query.goal_count as usize {
                if self.orient(totals[g], g) < self.orient(self.query.goals[g].threshold as i32, g) { return None; }
            }
            return Some(Solution {
                armor: [self.chosen[0], self.chosen[1], self.chosen[2], self.chosen[3], self.chosen[4]],
                charm: self.chosen[5], jewels: self.placements,
                jewel_count: frame.count, totals,
            });
        }
        if !self.jewel_possible(frame) || frame.next > self.counts[6] as u64 {
            self.frame_count -= 1;
            return None;
        }
        self.frames[top].next += 1;
        if frame.next == 0 {
            self.frames[top].next = frame.minimum as u64 + 1;
            let bin = frame.bin + 1;
            self.frames[self.frame_count] = Frame {
                bin, remaining: if bin as usize == BINS { 0 } else { self.bins[bin as usize] },
                count: frame.count, totals: frame.totals, charm: frame.charm, per_bin: frame.per_bin, present: frame.present, ..Frame::default()
            };
            self.frame_count += 1;
        } else {
            let ordinal = (frame.next - 1) as u32;
            if ordinal < frame.minimum { return None; }
            let index = ((self.best_jewel as u64 + ordinal as u64) % self.counts[6] as u64) as u32;
            let jewel = catalog.jewel(index);
            if jewel.slots > frame.remaining { return None; }
            let mut totals = frame.totals;
            for g in 0..self.query.goal_count as usize {
                totals[g] += jewel.points[g] as i32 * if frame.bin == 1 { self.multiplier as i32 } else { 1 };
            }
            let mut per_bin = frame.per_bin;
            let mut present = frame.present | jewel.present;
            for g in 0..self.query.goal_count as usize {
                per_bin[frame.bin as usize][g] += jewel.points[g] as i32 * if frame.bin == 1 { self.multiplier as i32 } else { 1 };
                if jewel.points[g] != 0 { present |= 1 << g; }
            }
            let mut charm = frame.charm;
            for g in 0..2 { totals[MAX_GOALS + g] += jewel.extra[g] as i32 * if frame.bin == 1 { self.multiplier as i32 } else { 1 }; }
            if frame.bin == 5 {
                for g in 0..self.query.goal_count as usize { charm[g] += jewel.points[g] as i32; }
                for g in 0..2 { charm[MAX_GOALS + g] += jewel.extra[g] as i32; }
            }
            self.placements[frame.count as usize] = Placement { jewel: index, gear: frame.bin };
            self.frames[self.frame_count] = Frame {
                remaining: frame.remaining - jewel.slots, minimum: ordinal,
                next: 0, count: frame.count + 1, totals, charm, per_bin, present, ..frame
            };
            self.frame_count += 1;
        }
        None
    }
}

/// Bounded, allocation-free spare-slot and decoration-swap improvement. Each tick evaluates at most
/// `budget` moves; each pass scans jewels, single placements/swaps and bundles
/// across free sockets. Bundles can cross an activation threshold in one move.
/// The caller evaluates every skill tree and rejects penalties or broken goals.
/// Each result uses at most 8192 edges and 42 improving passes, returning search
/// time to other equipment combinations. This is not an optimality proof.
pub struct DecorationRefiner {
    current: Solution,
    score: [u32; 10],
    best: Solution,
    best_score: [u32; 10],
    remaining: [u8; BINS],
    capacity: [u8; BINS],
    passes: u8,
    jewel: u32,
    bin: usize,
    complete: bool,
    pub work_done: u64,
}
impl DecorationRefiner {
    pub fn new(current: Solution, capacity: [u8; BINS], costs: impl Fn(u32) -> u8, score: [u32; 10]) -> Option<Self> {
        if current.jewel_count as usize > MAX_JEWELS { return None; }
        let mut remaining = capacity;
        for placement in &current.jewels[..current.jewel_count as usize] {
            let bin = placement.gear as usize;
            if bin >= BINS { return None; }
            let cost = costs(placement.jewel);
            if cost == 0 || cost > remaining[bin] { return None; }
            remaining[bin] -= cost;
        }
        Some(Self { current, score, best: current, best_score: score, remaining, capacity, passes: 0, jewel: 0, bin: 0, complete: false, work_done: 0 })
    }
    /// Returns the latest committed improvement and whether refinement is done.
    /// `evaluate` may update requested totals after full native arithmetic.
    pub fn tick(&mut self, jewel_count: u32, costs: impl Fn(u32) -> u8, mut evaluate: impl FnMut(&mut Solution) -> Option<[u32; 10]>, budget: u32) -> (Solution, bool) {
        for _ in 0..budget {
            if self.complete { break; }
            if self.work_done >= 8192 {
                self.current = self.best;
                self.complete = true;
                break;
            }
            self.work_done += 1;
            if self.jewel == jewel_count {
                if self.best_score >= self.score || self.passes as usize == MAX_JEWELS * 2 {
                    self.complete = true;
                    break;
                }
                self.current = self.best;
                self.score = self.best_score;
                self.passes += 1;
                self.remaining = self.capacity;
                for p in &self.current.jewels[..self.current.jewel_count as usize] { self.remaining[p.gear as usize] -= costs(p.jewel); }
                self.jewel = 0;
                self.bin = 0;
                continue;
            }
            let jewel = self.jewel;
            let bin = self.bin;
            let cost = costs(jewel);
            let packs = if cost == 0 { 0 } else {
                self.remaining.iter().map(|&slots| usize::from(slots / cost)).sum::<usize>()
                    .min(MAX_JEWELS - self.current.jewel_count as usize)
            };
            let singles = BINS + self.current.jewel_count as usize;
            self.bin += 1;
            if self.bin == singles + packs.saturating_sub(1) { self.bin = 0; self.jewel += 1; }
            if cost == 0 { continue; }
            let mut candidate = self.current;
            if bin < BINS {
                if cost > self.remaining[bin] || candidate.jewel_count as usize == MAX_JEWELS { continue; }
                candidate.jewels[candidate.jewel_count as usize] = Placement { jewel, gear: bin as u8 };
                candidate.jewel_count += 1;
            } else if bin < singles {
                let at = bin - BINS;
                let old = candidate.jewels[at];
                if cost > self.remaining[old.gear as usize] + costs(old.jewel) { continue; }
                candidate.jewels[at] = Placement { jewel, gear: old.gear };
            } else {
                // Evaluate every useful prefix, not only a full-socket fill:
                // reaching one threshold can leave room for a second skill.
                let mut left = bin - singles + 2;
                for gear in 0..BINS {
                    let count = usize::from(self.remaining[gear] / cost).min(left);
                    for _ in 0..count {
                        candidate.jewels[candidate.jewel_count as usize] = Placement { jewel, gear: gear as u8 };
                        candidate.jewel_count += 1;
                    }
                    left -= count;
                }
            }
            if let Some(score) = evaluate(&mut candidate) {
                if score < self.best_score { self.best = candidate; self.best_score = score; }
            }
        }
        (self.current, self.complete)
    }
}
