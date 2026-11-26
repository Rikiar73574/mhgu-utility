//! Role guard for MapAssist 4c218. No original mod code/assets redistributed.
#[path = "roles.rs"]
mod roles;
use core::{arch::asm, mem, ptr};
use roles::Marker;
#[repr(C)]
struct Hook([u32; 19]);
static mut HOOKS: [Hook; 3] = [Hook([0; 19]), Hook([0; 19]), Hook([0; 19])];
static mut ORIGINAL: [u32; 3] = [0; 3];
#[derive(Clone, Copy)]
struct Map {
    layout: u32,
    markers: [Marker; 32],
}
const EMPTY_MARKER: Marker = Marker {
    node: 0,
    tag: 0,
    kind: 0,
    index: 0,
    actor: 0,
    sprite: 0,
};
const EMPTY: Map = Map {
    layout: 0,
    markers: [EMPTY_MARKER; 32],
};
static mut MAPS: [Map; 4] = [EMPTY; 4];
static mut WAIT: u32 = 0;
unsafe extern "C" {
    #[link_name = "forge_hook_create"]
    fn hook(out: *mut Hook, target: u32, detour: u32, original: *mut u32) -> *mut Hook;
}
fn addr(pc: u32) -> u32 {
    forge::mem::text_addr() + pc - 0x200000
}
fn heap(p: u32, n: u32) -> bool {
    p >= 0x8000000 && p & 3 == 0 && p.checked_add(n).is_some_and(|e| e <= 0xa8000000)
}
unsafe fn word(p: u32) -> u32 {
    unsafe { ptr::read_unaligned(p as *const u32) }
}
unsafe fn node(layout: u32, id: u32) -> u32 {
    unsafe {
        let split = word(layout + 0x298);
        let (array, ix, count) = if id < split {
            (word(layout + 0xf8), id, split)
        } else {
            (
                word(layout + 0x294),
                id.wrapping_sub(split),
                word(layout + 0x288),
            )
        };
        if count > 2048 || ix >= count || !heap(array, count * 4) {
            return 0;
        }
        let n = word(array + ix * 4);
        if heap(n, 0xdc) { n } else { 0 }
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mapassist_guard_update(ui: u32) {
    unsafe {
        if heap(ui, 0xe0) && word(ui) == addr(0x1974c14) {
            let layout = word(ui + 0x24);
            let table = word(ui + 0xd8);
            if heap(layout, 0x29c) && heap(table, 32 * 32 + 12) {
                let maps = &mut *(&raw mut MAPS);
                if let Some(map) = maps.iter_mut().find(|m| m.layout == layout) {
                    capture(map, ui, table);
                } else if let Some(map) = maps.iter_mut().find(|m| m.layout == 0) {
                    *map = EMPTY;
                    map.layout = layout;
                    capture(map, ui, table);
                }
            }
        }
        let f: unsafe extern "C" fn(u32) = mem::transmute(ORIGINAL[0]);
        f(ui);
    }
}
unsafe fn capture(map: &mut Map, ui: u32, table: u32) {
    unsafe {
        map.markers.fill(EMPTY_MARKER);
        let manager = word(addr(0x1a51fe0));
        if !heap(manager, 0x44) {
            return;
        }
        for i in 0..32 {
            let kind = word(table + 0x18 + i * 32);
            let index = word(table + 0x1c + i * 32);
            if kind != 3 || index >= 10 {
                continue;
            }
            let actor = word(manager + 0x1c + index * 4);
            if !heap(actor, 4) {
                continue;
            }
            let n = node(map.layout, word(ui + 0x4c + i * 4));
            if n == 0 {
                continue;
            }
            let parts = word(n + 0xd8);
            if !heap(parts, 8) {
                continue;
            }
            let sprite = word(parts + 4);
            if !heap(sprite, 0x11c) {
                continue;
            }
            map.markers[i as usize] = Marker {
                node: n,
                tag: word(n + 4),
                kind,
                index,
                actor,
                sprite,
            };
        }
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mapassist_guard_drop(layout: u32) {
    unsafe {
        for m in &mut *(&raw mut MAPS) {
            if m.layout == layout {
                *m = EMPTY;
            }
        }
        let f: unsafe extern "C" fn(u32) = mem::transmute(ORIGINAL[1]);
        f(layout);
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mapassist_guard_classify(sprite: u32, animation: u32) -> u32 {
    unsafe {
        // Identity comes from the native enemy record, never a reused tag or UV.
        let manager = word(addr(0x1a51fe0));
        if !heap(manager, 0x44) {
            return 0;
        }
        for map in &*(&raw const MAPS) {
            for m in &map.markers {
                if roles::enemy(m)
                    && m.node == animation
                    && m.sprite == sprite
                    && word(animation + 4) == m.tag
                    && word(manager + 0x1c + m.index * 4) == m.actor
                {
                    let f: unsafe extern "C" fn(u32, u32) -> u32 = mem::transmute(ORIGINAL[2]);
                    return f(sprite, animation);
                }
            }
        }
        0
    }
}
unsafe fn readable(at: u32, n: u32) -> bool {
    unsafe {
        let mut info = [0u32; 10];
        let mut rc = info.as_mut_ptr() as u32;
        asm!("svc #6",inout("r0")rc,in("r1")0u32,in("r2")at,lateout("r3")_,options(nostack));
        rc == 0
            && info[4] != 0
            && info[6] & 1 != 0
            && (at as u64) >= ((info[1] as u64) << 32 | info[0] as u64)
            && (at as u64 + n as u64)
                <= ((info[1] as u64) << 32 | info[0] as u64)
                    .saturating_add((info[3] as u64) << 32 | info[2] as u64)
    }
}
unsafe fn module() -> u32 {
    unsafe {
        // Fixed Forge arenas; query before touching a page, including unmapped spill.
        for (start, end) in [(0x03200000u32, 0x03e00000u32), (0xf0100000, 0xf8100000)] {
            let mut p = start;
            while p < end {
                if readable(p, 0x80)
                    && word(p + 0x10) == 0x304f524e
                    && word(p + 0x18) == 0x12000
                    && readable(p, 0xb000)
                {
                    let fingerprints = [
                        (0x3e1c, 0xe280c0b4),
                        (0x3e20, 0xe16d41f8),
                        (0x3e2c, 0xe1a05001),
                        (0x6e98, 0xe59f33ac),
                        (0xa820, 0xe59fc404),
                        (0xace4, 0xe16d41f4),
                    ];
                    if fingerprints.iter().all(|&(o, w)| word(p + o) == w) {
                        return p;
                    }
                }
                p += 0x1000;
            }
        }
        0
    }
}
unsafe fn compatible(pc: u32, expected: u32) -> bool {
    unsafe {
        let w = word(addr(pc));
        w == expected
            || w == 0xe51ff004 && {
                let p = word(addr(pc) + 4);
                ((0x03200000..0x03e00000).contains(&p) || (0xf0100000..0xf8100000).contains(&p))
                    && p & 3 == 0
            }
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn mapassist_guard_tick(_dt: f32) {
    unsafe {
        if WAIT > 1 {
            return;
        }
        WAIT += 1;
        if WAIT < 2 {
            return;
        }
        let base = module();
        if base == 0 {
            forge::log::prelude::warn!(
                "MapAssist guard: supported 4c218 binary absent; no hooks installed"
            );
            return;
        }
        if !compatible(0x819494, 0xe92d4ff0) || !compatible(0xb1b284, 0xe92d4830) {
            forge::log::prelude::error!("MapAssist guard: native version mismatch");
            return;
        }
        hook(
            &raw mut HOOKS[0],
            addr(0x819494),
            mapassist_guard_update as *const () as u32,
            &raw mut ORIGINAL[0],
        );
        hook(
            &raw mut HOOKS[1],
            addr(0xb1b284),
            mapassist_guard_drop as *const () as u32,
            &raw mut ORIGINAL[1],
        );
        if ORIGINAL[0] != 0 && ORIGINAL[1] != 0 {
            hook(
                &raw mut HOOKS[2],
                base + 0x3e1c,
                mapassist_guard_classify as *const () as u32,
                &raw mut ORIGINAL[2],
            );
        }
        forge::log::prelude::info!(
            "MapAssist guard: binary {:#x}, role hooks {:#x}/{:#x}/{:#x}",
            base,
            ORIGINAL[0],
            ORIGINAL[1],
            ORIGINAL[2]
        );
    }
}
