#![allow(dead_code)]
#[path = "../mod/src/set_saved.rs"]
mod set_saved;
use set_saved::*;
fn sample() -> Saved {
    let mut b = Saved::new();
    b.set_name("火 Build").unwrap();
    b.armor = [100, 200, 300, 400, 500];
    b.charm_box = 37;
    b.charm_slots = 3;
    b.charm_trees = [1, 205];
    b.charm_points = [-10, 13];
    b.jewels[0] = SavedJewel {
        native: 123,
        gear: 6,
    };
    b.jewel_count = 1;
    b.goal_trees[0] = 9;
    b.goal_points[0] = 15;
    b.goal_count = 1;
    b.weapon = [14, 27, 2, 3];
    b
}
fn encoded(s: &Store) -> String {
    let mut bytes = [0; MAX_BYTES];
    encode(s, &mut bytes).unwrap().to_owned()
}
#[test]
fn roundtrip_uses_stable_keys_and_utf8_names() {
    let mut s = Store::new();
    s.preferences.village = 10;
    s.preferences.hub = 11;
    s.entries[0] = sample();
    s.len = 1;
    let text = encoded(&s);
    assert!(text.contains("armor=100,200,300,400,500"));
    assert!(text.contains("name=火 Build"));
    assert_eq!(parse(&text).unwrap(), s);
}
#[test]
fn rejects_truncation_and_unknown_or_duplicate_fields() {
    let mut s = Store::new();
    s.entries[0] = sample();
    s.len = 1;
    let text = encoded(&s);
    for end in 0..text.len() {
        if text.is_char_boundary(end) {
            assert!(parse(&text[..end]).is_err(), "accepted truncation {end}");
        }
    }
    assert!(parse(&(text.clone() + "oops\n")).is_err());
    assert!(parse(&text.replacen("name=火 Build\n", "name=火 Build\nname=Again\n", 1)).is_err());
}
#[test]
fn validates_bounds_and_utf8_names() {
    let mut s = Store::new();
    s.preferences.order = [0, 1, 1, 3];
    assert!(encode(&s, &mut [0; MAX_BYTES]).is_err());
    s = Store::new();
    s.preferences.village = 255;
    s.preferences.hub = 12;
    assert!(encode(&s, &mut [0; MAX_BYTES]).is_err());
    s = Store::new();
    s.entries[0] = sample();
    s.len = 1;
    s.entries[0].weapon[3] = 4;
    assert!(encode(&s, &mut [0; MAX_BYTES]).is_err());
    s.entries[0] = sample();
    s.entries[0].jewels[0].gear = 7;
    assert!(encode(&s, &mut [0; MAX_BYTES]).is_err());
    let mut b = Saved::new();
    assert!(b.set_name("").is_err());
    assert!(b.set_name("bad\nname").is_err());
    assert!(b.set_name(&"é".repeat(25)).is_err());
    b.set_name(&"é".repeat(24)).unwrap();
    assert_eq!(b.name().len(), 48);
    b.name[0] = 255;
    assert!(b.validate().is_err());
}
#[test]
fn rejects_numeric_overflow_and_buffer_overflow() {
    let s = Store::new();
    let text = encoded(&s);
    assert!(parse(&text.replace("village=255", "village=256")).is_err());
    assert!(encode(&s, &mut [0; 16]).is_err());
    let mut s = s;
    for i in 0..CAPACITY {
        s.entries[i] = sample();
    }
    s.len = CAPACITY;
    assert_eq!(parse(&encoded(&s)).unwrap(), s);
    s.len = CAPACITY + 1;
    assert!(encode(&s, &mut [0; MAX_BYTES]).is_err());
}
#[test]
fn failed_persistence_keeps_previous_state() {
    let mut s = Store::new();
    let old = s;
    let mut next = s;
    next.entries[0] = sample();
    next.len = 1;
    assert!(!s.commit(next, |_| false));
    assert_eq!(s, old);
    assert!(s.commit(next, |text| parse(text).is_ok()));
    assert_eq!(s, next);
}
#[test]
fn preserves_full_priority_range_and_full_records() {
    let mut s = Store::new();
    s.preferences.trees = [255; TREE_COUNT];
    s.preferences.owned_only = true;
    s.preferences.order = [3, 2, 1, 0];
    for b in &mut s.entries {
        *b = sample();
        b.set_name(&"龍".repeat(16)).unwrap();
        b.armor = [u32::MAX; 5];
        b.jewel_count = 21;
        b.jewels = [SavedJewel {
            native: u16::MAX,
            gear: 6,
        }; 21];
        b.goal_count = 6;
        b.goal_trees = [1, 2, 3, 4, 5, 205];
        b.goal_points = [-127, -10, 10, 15, 20, 127];
    }
    s.len = CAPACITY;
    let text = encoded(&s);
    assert!(text.len() <= MAX_BYTES);
    assert_eq!(parse(&text).unwrap(), s);
}
#[test]
fn rejects_corrupt_record_counts_ranges_and_duplicate_goals() {
    let mut s = Store::new();
    s.entries[0] = sample();
    s.len = 1;
    let text = encoded(&s);
    for (from, to) in [
        ("builds=1", "builds=13"),
        ("jewels=1", "jewels=22"),
        ("goals=1", "goals=7"),
        ("charm_slots=3", "charm_slots=4"),
        ("charm_trees=1,205", "charm_trees=1,206"),
        ("charm_points=-10,13", "charm_points=-128,13"),
        ("weapon=14,27,2,3", "weapon=15,27,2,3"),
        ("weapon=14,27,2,3", "weapon=14,26,2,3"),
        ("weapon=14,27,2,3", "weapon=14,27,4,3"),
        ("goal=9,15", "goal=0,15"),
        ("goal=9,15", "goal=9,0"),
        ("goal=9,15", "goal=9,128"),
        ("jewel=123,6", "jewel=65536,6"),
    ] {
        assert!(parse(&text.replace(from, to)).is_err(), "accepted {to}");
    }
    assert!(parse(&text.replace("goals=1\ngoal=9,15", "goals=2\ngoal=9,15\ngoal=9,20")).is_err());
    assert!(parse(&"x".repeat(MAX_BYTES + 1)).is_err());
    let old = s;
    let mut invalid = s;
    invalid.entries[0].name_len = 49;
    let mut called = false;
    assert!(!s.commit(invalid, |_| {
        called = true;
        true
    }));
    assert!(!called);
    assert_eq!(s, old);
}

#[test]
fn independent_automatic_stars_and_custom_priority_groups_roundtrip() {
    let mut s = Store::new();
    s.preferences.village = 8;
    assert!(s.preferences.validate().is_ok());
    s.preferences.village = 255;
    s.preferences.hub = 4;
    assert!(s.preferences.validate().is_ok());
    s.preferences.groups[18] = 3;
    assert_eq!(parse(&encoded(&s)).unwrap(), s);
    let legacy="utility-set-builder=1\nvillage=255\nhub=255\nowned_only=0\norder=0,1,2,3\ntrees=default\nbuilds=0\nend\n";
    assert_eq!(parse(legacy).unwrap().preferences.groups,[0;TREE_COUNT]);
    s.preferences.groups[18]=5;
    assert!(s.preferences.validate().is_err());
}
