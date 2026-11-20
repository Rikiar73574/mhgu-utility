# Screenshots

## Current interface

Captured on 20 November 2026 in an isolated test game with a copied save,
using the Windows executable under Wine on Linux. The main views are native
2560 × 1440 captures; the Set Builder preview captures are 1216 × 760.
HD textures, the Hunter Font and other installed mods are visible, so their
appearance can differ from a stock installation. These are actual game captures,
not mockups or the emulated preview renderer.

Images are encoded as lossless WebP. Decoded RGB pixels were checked against
the original PNG captures. [Capture dimensions and checksums](screenshots/current/captures.json).
Click any image to view it at its original resolution.

| Item categories | Item list |
| --- | --- |
| ![Categorized item browser](screenshots/current/items-categories.webp) | ![Items with native icons and MHGU-style frames](screenshots/current/items.webp) |

| Weapon tree | Weapon details |
| --- | --- |
| ![Connected Lance upgrade tree](screenshots/current/weapon-tree.webp) | ![Weapon image, miniature tree, stats and crafting materials](screenshots/current/weapon-details.webp) |

| Material sources | Wishlist |
| --- | --- |
| ![Bealite Ore sources grouped by map with rank and drop rates](screenshots/current/item-sources.webp) | ![Armor wishlist with owned, needed and total material counts](screenshots/current/wishlist.webp) |

| Locations | Gathering by area |
| --- | --- |
| ![Location artwork, map previews and shared rank selector](screenshots/current/locations.webp) | ![Items grouped by gathering type within the selected area](screenshots/current/area-gathering.webp) |

| Quest suggestions | Gathering checklist |
| --- | --- |
| ![Unlocked quest route covering wishlist requirements](screenshots/current/quest-planner.webp) | ![Mining category and item checkboxes with chance ranges](screenshots/current/gathering-filters.webp) |

### Gathering markers alongside Map Assist

Grey and gold pickaxes mark selected mining points. The monster icon is drawn
by the separately installed Map Assist mod; the lower-left HP display is HuntLog.

![Selected mining points on the quest map alongside Map Assist](screenshots/current/gathering-map.webp)

## Set Builder preview

This feature is on the [Set Builder branch](https://github.com/Rikiar73574/mhgu-utility/tree/feat/set-builder)
and is not included in the latest published release. The result shown here uses
verified equipment availability and a charm read from the copied character's box.

| Skill selection | Build results |
| --- | --- |
| ![Choose activated skills using the detected character profile](screenshots/current/set-builder-skills.webp) | ![Armor ownership, charm skills, skill totals and add-missing-armor action](screenshots/current/set-builder-results.webp) |

<details>
<summary>Earlier screenshots and validation records</summary>

### Earlier game captures

Captured on Linux with stock MHGURecomp and the complete installed mod stack,
using the standalone Compendium build identified in [validation](validation.md).
These are actual game captures, not the emulated preview renderer.

| Items | Weapon types | Weapon tree |
| --- | --- | --- |
| ![Items](screenshots/items.png) | ![Weapon classes](screenshots/weapon-types.png) | ![Weapon tree](screenshots/weapon-tree.png) |

| Weapon details | Material sources | Wishlist |
| --- | --- | --- |
| ![Weapon details](screenshots/weapon-details.png) | ![Direct material source link](screenshots/item-sources.png) | ![Wishlist](screenshots/wishlist.png) |

| Controller search | Locations | Passive quest tracker |
| --- | --- | --- |
| ![Controller search](screenshots/search.png) | ![Grouped gathering sources](screenshots/locations.png) | ![Passive material tracker](screenshots/quest-tracker.png) |

### Version 0.3.0

Captured in the separate hidden test game with copied save and HD textures/font.
See [UX validation](ux-validation.md) for artifact hashes and platform coverage.

| Item list | Controller search | Armor preview |
| --- | --- | --- |
| ![Native frames and game font](screenshots/native-style-items.png) | ![Controller search](screenshots/controller-search.png) | ![Armor preview](screenshots/armor-preview.png) |

| Keyboard editing (Windows executable under Wine) | Exact byproduct level | Point requirements without an eligible route |
| --- | --- | --- |
| ![Inline clipboard replacement](screenshots/keyboard-editing.png) | ![Wyvern Lovers level 5](screenshots/weapon-byproduct-source.png) | ![Known requirements stay visible](screenshots/point-requirements.png) |

</details>
