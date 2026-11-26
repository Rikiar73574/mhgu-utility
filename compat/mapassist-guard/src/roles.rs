#[derive(Clone, Copy, Default)]
pub struct Marker {
    pub node: u32,
    pub tag: u32,
    pub kind: u32,
    pub index: u32,
    pub actor: u32,
    pub sprite: u32,
}
pub fn enemy(m: &Marker) -> bool {
    m.kind == 3 && m.index < 10 && m.actor >= 0x08000000 && m.actor < 0xa8000000 && m.actor & 3 == 0
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_multiplayer_records() {
        // Native type1/subtype1 companion records had UV96,0,120,24 and were
        // accepted and hidden by MapAssist. Tags are NOT native layout IDs.
        for tag in [11, 13] {
            assert!(!enemy(&Marker {
                node: 0xc40c960,
                tag,
                kind: 1,
                index: 0,
                actor: 0xc250060,
                sprite: 0xc400000
            }));
        }
        assert!(enemy(&Marker {
            node: 0xc400000,
            tag: 18,
            kind: 3,
            index: 0,
            actor: 0xc2d00b0,
            sprite: 0xc410000
        }));
    }
    #[test]
    fn tag_is_not_identity() {
        assert!(!enemy(&Marker {
            tag: 18,
            kind: 2,
            ..Marker::default()
        }));
        assert!(enemy(&Marker {
            tag: 11,
            kind: 3,
            actor: 0xc000000,
            ..Marker::default()
        }));
    }
    #[test]
    fn stale_or_unknown_actor() {
        for (kind, index, actor) in [
            (0, 0, 0xc000000),
            (3, 10, 0xc000000),
            (3, 0, 0),
            (3, 0, 0xc000001),
        ] {
            assert!(!enemy(&Marker {
                kind,
                index,
                actor,
                ..Marker::default()
            }));
        }
    }
}
