# Smithy plugin compatibility investigation

Issue [#2](https://github.com/Rikiar73574/mhgu-utility/issues/2) reports that
MHGU Utility v0.3.0 and either Modern Weapon Trees or KatSmithy depend on load
order on MHGURecomp 0.16. The following source change addresses the smithy
update hook's initialization order. It does not establish full compatibility or
resolve the separate plugin memory limit.

## Hook order

Utility already recognizes the stock smithy update entry at `main+0x6f1ae8`
and Forge's aligned literal jump into the plugin arena. However, installing
that hook during `forge_onInit` changes the prologue before a later plugin can
check it. A plugin that accepts only the stock prologue then declines to hook.
Accepting an earlier Forge hook on Utility's side alone does not fix that order.

The inspected Modern Weapon Trees v0.59 binary explicitly reports that another
plugin's patch disables the affected hook. KatSmithy 1.5.0's embedded default
probe configuration guards `smithy_win` at `0x6f1ae8` with the two stock words
`e92d41f0 e24dd018` and registers hooks through KatLib. Binary inspection is
supporting evidence, not execution of those third-party plugins.

Utility now installs only this hook during its first `forge_onUpdate`, once,
when its input integration is ready. In the inspected MHGURecomp source (`06fa605`),
`runtime/forge.c`'s `app_run` finishes all plugin `forge_onInit` calls before
entering the native application; `frame_update` invokes update callbacks after
the native frame. This allows peers to inspect the stock entry first in either
initialization order. Forge's `hook_install` records the previous detour as the
new original trampoline, so Utility calls that peer before capturing selection.
A failed install or an unrecognized entry is not retried each frame. Existing
foreign-jump guards and native hint menu checks remain in place.

## Regression evidence

`tests/arm_smithy_chain.py` executes the compiled ARM Utility with an isolated
stock-prologue-only peer and a mocked Forge hook adapter. It reproduces the
previous early-install failure and checks both initialization orders, execution
through the peer and native original with selection capture and the native hint,
the stock-only path, disabled-input behavior, one-shot hook failure,
and accepted/refused entry signatures. It uses fresh plugin state for each case.
`tests/arm_native_hint.py` checks selection capture and the native wishlist hint
after the deferred install; `tests/arm_smithy_categories.py` checks equipment
categories, row flags and menu guards.

```sh
MHGU_GAME_DIR=/path/to/owned-game/data python3 tests/arm_smithy_chain.py
MHGU_GAME_DIR=/path/to/owned-game/data python3 tests/arm_native_hint.py
MHGU_GAME_DIR=/path/to/owned-game/data python3 tests/arm_smithy_categories.py
```

These tests use the built ARM ELF and owned game metadata. They do not launch
the game or execute Modern Weapon Trees, KatSmithy or KatLib. No live game
compatibility check was performed for this change. Peers that install their
hooks later than initialization, altered menus/cursors, and other shared hooks
need separate validation.

## Plugin capacity remains a separate blocker

The inspected stock Recomp loader reserves `0x03200000..0x03e00000` (12 MiB)
for all plugin images and BSS, including alignment and loader headroom. A plugin
that does not fit is skipped before its initializer runs, so changing hook order
cannot make that plugin load. Check `log.txt` for
`skipped, no room left in the plugin memory region`.

Measured artifacts on 24 November 2026:

| Artifact | Image bytes | BSS bytes | Aligned footprint |
| --- | ---: | ---: | ---: |
| Utility normal build | 9,388,032 | 49,152 | 9,437,184 |
| Modern Weapon Trees v0.59 | 5,836,800 | 16,384 | 5,898,240 |
| KatSmithy 1.5.0 | 606,208 | 23,109,632 | 23,724,032 |

Utility and Modern v0.59 alone exceed that arena by 2,752,512 bytes before
additional loader headroom. KatSmithy 1.5.0 alone exceeds it; its included README
requires Recomp 0.18.0 or newer and KatLib. These figures do not prove the
footprints of the exact third-party versions in the original 0.16 report.
An optional-artwork Utility build saves approximately 256 KiB, which does not
close the Modern v0.59 capacity gap. Use a runtime with sufficient capacity
before testing coexistence. This change does not expand Recomp's arena.

Inspected downloads: [Modern Weapon Trees v0.59](https://gamebanana.com/dl/1883049)
(NRO SHA-256 `3ab995217390d1e80e985926f6531a623ee8bd8a1ddaa515823b7bca7d00cec7`)
and [KatSmithy 1.5.0](https://gamebanana.com/dl/1900478)
(NRO SHA-256 `e6c198034f29da6145ba67fed191c43f3e0f9fb564c62fc99f4a22a89948526b`).
