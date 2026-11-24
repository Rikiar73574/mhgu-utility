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

Utility leaves its first `forge_onUpdate` untouched and installs this hook
once on the following update, when input integration is ready. In the inspected
MHGURecomp source (`06fa605`), `app_run` finishes all initializers before
entering the application. One complete update-callback round also lets peers
that install on their first update inspect the stock entry in either order.
KatSmithy 1.5.0 does exactly that: real testing caught a guard mismatch with the
earlier first-update implementation. The second-update implementation preserves
its smithy hook. This is a lifecycle boundary, not a wall-clock delay.

Forge records the previous detour as the new original trampoline, so Utility
calls that peer before capturing selection. A failed install or unrecognized
entry is not retried each frame. Foreign-jump and native-menu guards remain.
Peers that install later than their first update need separate validation.

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
the game or execute Modern Weapon Trees, KatSmithy or KatLib. The real-plugin matrix below supplements these isolated checks. Altered
menus/cursors and other shared hooks still need their own coverage.

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

## Real-plugin and multiplayer checks

On 24 November, the final candidate NRO
`2e869f39ab573e2c0c64f2a214364009b0691317c61c98c58bdd602c769b15e1`
passed these actual third-party runs:

| Peer | Utility first | Peer first |
| --- | --- | --- |
| Modern Weapon Trees 0.59 | Pass | Pass |
| Modern Weapon Trees 0.90 | Pass | Pass |
| KatSmithy 1.5 + KatLib | Pass | Pass |

Peer-only baselines were tested first. Checks exercised real chained detours,
peer trees, native selected-recipe capture, persisted R3 wishlist add/remove,
and Utility opening/closing over those menus. Both KatSmithy orders completed
its direct tree mapping and rendered the weapon preview. The earlier build
failed its real Utility-first smithy guard and fell back without that preview;
the second-update build passed. KatSmithy's own free-tree cursor is distinct
from the native upgrade selection captured by Utility.

Modern 0.90 also passed Hammer, Hunting Horn and Lance selections, exact recipe
quantities, material sources/back, and saved wishlist removal after a process
restart. An existing lazy-load limitation remains: a saved entry initially has
an Add hint until the wishlist loads; R3 still removes the existing entry.

These runs used Linux source `06fa605`, owned game data, copied saves, Xvfb,
dummy audio, rejected hardware controllers and isolated networking. The private
runtime expanded the plugin arena to 44 MiB and moved its adjacent trampoline,
stub and HLE regions; hook behavior and game-library identity checks were
unchanged. This does **not** establish stock-capacity support. Modern 0.90 uses
9,830,400 aligned bytes; its combination with Utility also exceeds 12 MiB.
KatSmithy targets Windows Recomp 0.18+, so its Linux result is exploratory.
KatSmithy and Modern were kept separate as instructed by KatSmithy's author.
The Modern 0.59 missing-rodata-export warning and 0.90 scan warning also occurred
in their peer-only baselines; neither stopped the exercised flow.

A separate two-client loopback session loaded this candidate together with
Armor Fusion on the host and no plugins on the peer. Both entered the same
native Hub quest 10101, stage 1, camp, in distinct slots. Utility's gathering
menu opened and closed during the shared quest. All seven replicated equipment
records, appearances, decorations, skill points, active skills and identity
matched. Health, resistances and base attack/defense matched in both directions
once native owner-only pouch charm bonuses (+6 attack/+12 defense here) were
accounted for. The raw captures were retained, not rewritten to force parity.

Build output, per-case plugin hashes, logs, snapshots and screenshots remain in
ignored `build/pr-ready-lab/`. This covers the tested native Linux configuration,
not every weapon/language, later hook installers, Windows execution or WAN play.

Screenshots: [KatSmithy tree](screenshots/compatibility/kat-tree.webp),
[native wishlist hint with KatSmithy loaded](screenshots/compatibility/kat-native-wishlist.webp).

## Installed Windows 0.19.2 stack

The user's Windows 0.19.2 executable was also tested under Wine on Linux, in a
separate prefix/display/network namespace with copied saves and the complete
installed mod stack. Unlike the earlier inspected Linux loader, this executable
has an additional 128 MiB plugin arena `[0xf0100000,0xf8100000)`. KatSmithy and
Fusion load there. Utility now recognizes aligned Forge entry jumps into that
verified arena, retaining the same signature and foreign-entry guards.
Compiled boundary regressions failed before this change and pass afterward.

Candidate Utility NRO:
`f2b9aa68f9dbd8e7f3b35f6ba882ee0b74f5e8146c01fe111cf2b21e7906a8f2`.
Fusion also needed to defer its own smithy hook: its installed onInit hook caused
KatSmithy's stock-prologue probe to fail independently of Utility. With both
fixes, the actual chain is Fusion `f17a4798` -> Utility `033b177c` -> KatSmithy
`f01536f4` -> native original. All nine plugins loaded, and KatSmithy accepted the
stock entry during its first update. The new Windows artifact is distinct from
the earlier Linux/multiplayer artifact recorded above; this startup change
was not relabeled as another multiplayer run.

The exact Windows executable under Wine passed KatSmithy's full Great Sword
tree and 3D preview, native Upgrade Equipment wishlist add/remove, and Fusion's
native selector open/cancel with unchanged equipment, pouch, box and zenny.
The wishlist test restored all three existing copied-save entries byte-for-byte.
R3 in KatSmithy's custom tree is not a Utility wishlist integration; the verified
shortcut belongs to the normal Upgrade Equipment screen. The screenshots above
now show this Windows-under-Wine full-stack pass. Tests used a private software
Vulkan renderer to avoid the user's GPU session; the running game was not restarted.
