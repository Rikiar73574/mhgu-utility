//! Bounded, versioned Set Builder preferences and stable-key saved builds.
//! The complete document is validated before it replaces in-memory state.
#![allow(dead_code)]
use core::fmt::{self, Write};
pub const CAPACITY: usize = 12;
pub const MAX_BYTES: usize = 16 * 1024;
pub const TREE_COUNT: usize = 206;
const HEADER: &str = "utility-set-builder=2";
const DEFAULTS: &str = "utility-set-builder=1\nvillage=255\nhub=255\nowned_only=0\norder=0,1,2,3\ntrees=default\nbuilds=0\nend\n";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Preferences {
    pub village: u8,
    pub hub: u8,
    pub owned_only: bool,
    pub order: [u8; 4],
    pub trees: [u8; TREE_COUNT],
    pub groups: [u8; TREE_COUNT],
}
impl Preferences {
    pub const fn new() -> Self {
        Self {
            village: 255,
            hub: 255,
            owned_only: false,
            order: [0, 1, 2, 3],
            trees: [0; TREE_COUNT],
            groups: [0; TREE_COUNT],
        }
    }
    pub fn validate(&self) -> Result<(), Error> {
        if !(self.village == 255 || self.village <= 10) || !(self.hub == 255 || self.hub <= 11) || self.groups.iter().any(|&g|g>4) {
            return Err(Error::Invalid);
        }
        let mut seen = 0u8;
        for &v in &self.order {
            if v > 3 || seen & (1 << v) != 0 {
                return Err(Error::Invalid);
            }
            seen |= 1 << v;
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SavedJewel {
    pub native: u16,
    pub gear: u8,
}
impl SavedJewel {
    pub const fn new() -> Self {
        Self { native: 0, gear: 0 }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Saved {
    pub name: [u8; 48],
    pub name_len: u8,
    pub armor: [u32; 5],
    pub charm_box: u16,
    pub charm_slots: u8,
    pub charm_trees: [u8; 2],
    pub charm_points: [i8; 2],
    pub jewels: [SavedJewel; 21],
    pub jewel_count: u8,
    pub goal_trees: [u8; 6],
    pub goal_points: [i8; 6],
    pub goal_count: u8,
    pub weapon: [u8; 4],
}
fn valid_name(name: &str) -> bool {
    !name.trim().is_empty() && name.len() <= 48 && !name.chars().any(char::is_control)
}
impl Saved {
    pub const fn new() -> Self {
        Self {
            name: [0; 48],
            name_len: 0,
            armor: [0; 5],
            charm_box: 65535,
            charm_slots: 0,
            charm_trees: [0; 2],
            charm_points: [0; 2],
            jewels: [SavedJewel::new(); 21],
            jewel_count: 0,
            goal_trees: [0; 6],
            goal_points: [0; 6],
            goal_count: 0,
            weapon: [14, 0, 0, 0],
        }
    }
    pub fn name(&self) -> &str {
        core::str::from_utf8(&self.name[..(self.name_len as usize).min(48)]).unwrap_or("")
    }
    pub fn set_name(&mut self, name: &str) -> Result<(), Error> {
        if !valid_name(name) {
            return Err(Error::Invalid);
        }
        self.name = [0; 48];
        self.name[..name.len()].copy_from_slice(name.as_bytes());
        self.name_len = name.len() as u8;
        Ok(())
    }
    pub fn validate(&self) -> Result<(), Error> {
        if self.name_len > 48
            || !valid_name(self.name())
            || self.charm_slots > 3
            || self.jewel_count > 21
            || self.goal_count > 6
            || self.weapon[0] > 14
            || !matches!(self.weapon[1], 0 | 27..=31)
            || !matches!(self.weapon[2], 0..=3 | 92)
            || self.weapon[3] > 3
        {
            return Err(Error::Invalid);
        }
        for i in 0..2 {
            if self.charm_trees[i] as usize >= TREE_COUNT
                || self.charm_points[i] == i8::MIN
                || (self.charm_trees[i] == 0 && self.charm_points[i] != 0)
            {
                return Err(Error::Invalid);
            }
        }
        for j in &self.jewels[..self.jewel_count as usize] {
            if j.gear > 6 {
                return Err(Error::Invalid);
            }
        }
        let mut seen = [false; TREE_COUNT];
        for i in 0..self.goal_count as usize {
            let t = self.goal_trees[i] as usize;
            if t == 0
                || t >= TREE_COUNT
                || seen[t]
                || self.goal_points[i] == 0
                || self.goal_points[i] == i8::MIN
            {
                return Err(Error::Invalid);
            }
            seen[t] = true;
        }
        // Unused fields must be zero so codecs never silently discard information.
        if self.name[self.name_len as usize..].iter().any(|&v| v != 0)
            || self.jewels[self.jewel_count as usize..]
                .iter()
                .any(|j| *j != SavedJewel::new())
            || self.goal_trees[self.goal_count as usize..]
                .iter()
                .any(|&v| v != 0)
            || self.goal_points[self.goal_count as usize..]
                .iter()
                .any(|&v| v != 0)
        {
            return Err(Error::Invalid);
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid,
    TooLarge,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Store {
    pub preferences: Preferences,
    pub entries: [Saved; CAPACITY],
    pub len: usize,
}
impl Store {
    pub const fn new() -> Self {
        Self {
            preferences: Preferences::new(),
            entries: [Saved::new(); CAPACITY],
            len: 0,
        }
    }
    pub fn validate(&self) -> Result<(), Error> {
        self.preferences.validate()?;
        if self.len > CAPACITY {
            return Err(Error::TooLarge);
        }
        for b in &self.entries[..self.len] {
            b.validate()?;
        }
        Ok(())
    }
    /// Publish only after the entire candidate has been encoded and persisted.
    pub fn commit(&mut self, next: Self, save: impl FnOnce(&str) -> bool) -> bool {
        let mut bytes = [0; MAX_BYTES];
        let Ok(text) = encode(&next, &mut bytes) else {
            return false;
        };
        if !save(text) {
            return false;
        }
        *self = next;
        true
    }
}
struct Writer<'a> {
    bytes: &'a mut [u8],
    len: usize,
}
impl Write for Writer<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let end = self.len.checked_add(s.len()).ok_or(fmt::Error)?;
        if end > self.bytes.len() {
            return Err(fmt::Error);
        }
        self.bytes[self.len..end].copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}
fn list<T: fmt::Display>(w: &mut Writer<'_>, key: &str, values: &[T]) -> fmt::Result {
    write!(w, "{key}=")?;
    for (i, v) in values.iter().enumerate() {
        if i > 0 {
            w.write_char(',')?;
        }
        write!(w, "{v}")?;
    }
    w.write_char('\n')
}
pub fn encode<'a>(s: &Store, bytes: &'a mut [u8]) -> Result<&'a str, Error> {
    s.validate()?;
    let mut w = Writer { bytes, len: 0 };
    let result = (|| -> fmt::Result {
        writeln!(w, "{HEADER}")?;
        writeln!(w, "village={}", s.preferences.village)?;
        writeln!(w, "hub={}", s.preferences.hub)?;
        writeln!(w, "owned_only={}", s.preferences.owned_only as u8)?;
        list(&mut w, "order", &s.preferences.order)?;
        if s.preferences.trees.iter().all(|&v| v == 0) {
            w.write_str("trees=default\n")?;
        } else {
            list(&mut w, "trees", &s.preferences.trees)?;
        }
        if s.preferences.groups.iter().all(|&v|v==0){w.write_str("groups=default\n")?;}else{list(&mut w,"groups",&s.preferences.groups)?;}
        writeln!(w, "builds={}", s.len)?;
        for b in &s.entries[..s.len] {
            w.write_str("build\n")?;
            writeln!(w, "name={}", b.name())?;
            list(&mut w, "armor", &b.armor)?;
            writeln!(w, "charm_box={}", b.charm_box)?;
            writeln!(w, "charm_slots={}", b.charm_slots)?;
            list(&mut w, "charm_trees", &b.charm_trees)?;
            list(&mut w, "charm_points", &b.charm_points)?;
            writeln!(w, "jewels={}", b.jewel_count)?;
            for j in &b.jewels[..b.jewel_count as usize] {
                writeln!(w, "jewel={},{}", j.native, j.gear)?;
            }
            writeln!(w, "goals={}", b.goal_count)?;
            for i in 0..b.goal_count as usize {
                writeln!(w, "goal={},{}", b.goal_trees[i], b.goal_points[i])?;
            }
            list(&mut w, "weapon", &b.weapon)?;
            w.write_str("endbuild\n")?;
        }
        w.write_str("end\n")
    })();
    result.map_err(|_| Error::TooLarge)?;
    core::str::from_utf8(&w.bytes[..w.len]).map_err(|_| Error::Invalid)
}
fn number<T: core::str::FromStr>(s: &str) -> Result<T, Error> {
    if s.is_empty() || s.starts_with('+') || s.trim() != s {
        return Err(Error::Invalid);
    }
    s.parse().map_err(|_| Error::Invalid)
}
fn array<T: core::str::FromStr + Copy, const N: usize>(
    s: &str,
    default: T,
) -> Result<[T; N], Error> {
    let mut values = [default; N];
    let mut parts = s.split(',');
    for v in &mut values {
        *v = number(parts.next().ok_or(Error::Invalid)?)?;
    }
    if parts.next().is_some() {
        return Err(Error::Invalid);
    }
    Ok(values)
}
fn field<'a>(lines: &mut core::str::Lines<'a>, key: &str) -> Result<&'a str, Error> {
    let line = lines.next().ok_or(Error::Invalid)?;
    let (k, v) = line.split_once('=').ok_or(Error::Invalid)?;
    if k != key {
        return Err(Error::Invalid);
    }
    Ok(v)
}
fn expect(lines: &mut core::str::Lines<'_>, value: &str) -> Result<(), Error> {
    if lines.next() == Some(value) {
        Ok(())
    } else {
        Err(Error::Invalid)
    }
}
pub fn parse(text: &str) -> Result<Store, Error> {
    if text.len() > MAX_BYTES {
        return Err(Error::TooLarge);
    }
    if !text.ends_with('\n') || text.contains('\r') {
        return Err(Error::Invalid);
    }
    let mut lines = text.lines();
    let version=lines.next().ok_or(Error::Invalid)?;
    if version!=HEADER && version!="utility-set-builder=1"{return Err(Error::Invalid);}
    let mut s = Store::new();
    s.preferences.village = number(field(&mut lines, "village")?)?;
    s.preferences.hub = number(field(&mut lines, "hub")?)?;
    s.preferences.owned_only = match field(&mut lines, "owned_only")? {
        "0" => false,
        "1" => true,
        _ => return Err(Error::Invalid),
    };
    s.preferences.order = array(field(&mut lines, "order")?, 0)?;
    let trees = field(&mut lines, "trees")?;
    if trees != "default" {
        s.preferences.trees = array(trees, 0)?;
    }
    if version==HEADER {let groups=field(&mut lines,"groups")?;if groups!="default"{s.preferences.groups=array(groups,0)?;}}
    s.len = number(field(&mut lines, "builds")?)?;
    if s.len > CAPACITY {
        return Err(Error::TooLarge);
    }
    for b in &mut s.entries[..s.len] {
        expect(&mut lines, "build")?;
        b.set_name(field(&mut lines, "name")?)?;
        b.armor = array(field(&mut lines, "armor")?, 0)?;
        b.charm_box = number(field(&mut lines, "charm_box")?)?;
        b.charm_slots = number(field(&mut lines, "charm_slots")?)?;
        b.charm_trees = array(field(&mut lines, "charm_trees")?, 0)?;
        b.charm_points = array(field(&mut lines, "charm_points")?, 0)?;
        b.jewel_count = number(field(&mut lines, "jewels")?)?;
        if b.jewel_count > 21 {
            return Err(Error::TooLarge);
        }
        for j in &mut b.jewels[..b.jewel_count as usize] {
            let (native, gear) = field(&mut lines, "jewel")?
                .split_once(',')
                .ok_or(Error::Invalid)?;
            j.native = number(native)?;
            j.gear = number(gear)?;
        }
        b.goal_count = number(field(&mut lines, "goals")?)?;
        if b.goal_count > 6 {
            return Err(Error::TooLarge);
        }
        for i in 0..b.goal_count as usize {
            let (tree, points) = field(&mut lines, "goal")?
                .split_once(',')
                .ok_or(Error::Invalid)?;
            b.goal_trees[i] = number(tree)?;
            b.goal_points[i] = number(points)?;
        }
        b.weapon = array(field(&mut lines, "weapon")?, 0)?;
        expect(&mut lines, "endbuild")?;
    }
    expect(&mut lines, "end")?;
    if lines.next().is_some() {
        return Err(Error::Invalid);
    }
    s.validate()?;
    Ok(s)
}

#[cfg(not(test))]
mod live {
    use super::*;
    use core::cell::UnsafeCell;
    const FILE: crate::storage::Location = crate::storage::Location {
        path: c"savedata:/utility_set_builder.txt",
        display: "Utility Set Builder",
        defaults: DEFAULTS,
    };
    struct State {
        store: Store,
        loaded: bool,
        error: bool,
    }
    struct Local(UnsafeCell<State>);
    unsafe impl Sync for Local {}
    // Game UI thread only, matching the other native preference stores.
    static STATE: Local = Local(UnsafeCell::new(State {
        store: Store::new(),
        loaded: false,
        error: false,
    }));
    pub fn load() {
        unsafe {
            let s = &mut *STATE.0.get();
            if s.loaded {
                return;
            }
            let mut bytes = [0; MAX_BYTES];
            match FILE.read(&mut bytes).ok().and_then(|text| parse(text).ok()) {
                Some(store) => {
                    s.store = store;
                    s.loaded = true;
                    s.error = false;
                }
                None => s.error = true,
            }
        }
    }
    pub fn preferences() -> Preferences {
        load();
        unsafe { (*STATE.0.get()).store.preferences }
    }
    pub fn entries() -> &'static [Saved] {
        load();
        unsafe {
            let s = &*STATE.0.get();
            &s.store.entries[..s.store.len]
        }
    }
    pub fn failed() -> bool {
        unsafe { (*STATE.0.get()).error }
    }
    fn change(update: impl FnOnce(&mut Store) -> bool) -> bool {
        load();
        unsafe {
            let s = &mut *STATE.0.get();
            if !s.loaded {
                s.error = true;
                return false;
            }
            let mut next = s.store;
            if !update(&mut next) {
                s.error = true;
                return false;
            }
            s.error = !s.store.commit(next, |text| FILE.save(text).is_ok());
            !s.error
        }
    }
    pub fn set_preferences(value: Preferences) -> bool {
        change(|s| {
            s.preferences = value;
            true
        })
    }
    pub fn add(value: Saved) -> bool {
        change(|s| {
            if s.len == CAPACITY {
                return false;
            }
            s.entries[s.len] = value;
            s.len += 1;
            true
        })
    }
    pub fn remove(index: usize) -> bool {
        change(|s| {
            if index >= s.len {
                return false;
            }
            s.entries.copy_within(index + 1..s.len, index);
            s.len -= 1;
            s.entries[s.len] = Saved::new();
            true
        })
    }
    pub fn rename(index: usize, name: &str) -> bool {
        change(|s| {
            if index >= s.len {
                return false;
            }
            s.entries[index].set_name(name).is_ok()
        })
    }
}
#[cfg(not(test))]
pub use live::{add, entries, failed, load, preferences, remove, rename, set_preferences};
