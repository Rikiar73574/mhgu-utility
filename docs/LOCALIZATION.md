# Runtime language support

Utility loads item, weapon, armor and skill names from the installed game's GMD
files. It follows `nn::oe::GetDesiredLanguage`; `language.ini` can override this.
No localized game names are included in the new mapping file: it contains only
hashes of the existing canonical English names, table IDs and native string slots.
Existing catalog English text remains the fallback. Descriptions, quest text and
English catalog data are not removed or completely localized by this change.

## Select a language

Edit `romfs/nativeNX/utility/language.ini` inside the Utility mod, then restart.
Use `eng`, `fre`, `ger`, `ita`, `spa`, `jpn`, `kor`, or `auto`.
Names are loaded through Forge's mounted RomFS, including installed language mods.
The loader checks both `nativeNX/<language>/table/` and `nativeNX/table/`.
Missing/malformed tables, empty entries and ambiguous name translations retain
the existing English name. Search accepts both English and localized names. The existing text editor can
paste Unicode queries; native Japanese/Korean IME composition is not added here.

`kor` reads the Japanese table filenames used by the MHXX Korean fan patch.
Set `korean_patch=true` **only** for the patch described in issue #5: its 2,350
KS X 1001 Hangul syllables occupy codepoints starting at U+7A00. Normal Japanese
never gets remapped. This convention needs confirmation against the actual patch;
the supplied mapping is tested against Python's standard EUC-KR codec.

## Translate UI labels

Copy `romfs/nativeNX/utility/locales/eng.tsv` to `<language>.tsv`. Each line is
`English phrase<TAB>translated phrase`; comments start with `#`. Missing labels
remain English. Numeric values and controller prefixes should be preserved.
The template is refreshed by `python3 tools/ui_strings.py`; it covers authored
source phrases, not the game's own GMD text. Runtime name tables supply those.
Full Japanese/Korean UI translations are not bundled; contributor translations
can be added without recompiling the plugin. The small Italian/Japanese/Korean
UI dictionaries used during verification are test fixtures only.

Noto Sans JP/KR fonts are bundled under OFL and merged into the native UI font.
Fonts and name-slot metadata load from runtime files, outside the shared NRO
region. Keep the whole release's `romfs` directory when updating, not just the NRO.

## Build

`tools/prepare.py` now supports omitted `--art-dir`, using existing icon/minimap
fallbacks. Full generation still requires the documented owned English GU dump
and catalog/database. New generated inputs are `name_slots.rs`, `name_slots.bin`
and `field-guide.ttf`; include them when using `--generated-dir`.

## Verification boundary

The pure tests cover malformed/truncated GMD files, UTF-8, phrase boundaries,
Unicode search, missing labels, and opt-in Korean remapping. Font tests verify
Japanese and Hangul coverage. Owned GU Italian tables are checked for matching
slot layouts. Native runtime tests use an isolated copied-save instance; no
inputs or writes go to the user's running game.

A full MHXX 1.5.1 install and the actual Korean patch are not available locally.
Neither MHXX feature parity nor complete Japanese/Korean UI translation is claimed.

### Verified in an isolated GU runtime

- Italian: 18 owned native tables loaded, 10,453 translated names; rendered
  Italian item names and a small test-only UI dictionary.
- Japanese: Japanese labels and mixed Japanese/English item names rendered with
  the bundled font. Item names used a synthetic fixture; native Japanese weapon
  and armor tables were also loaded.
- Korean: synthetic U+7A00/U+7A01 names rendered as `가각`, alongside Korean labels.
- Automatic language selection: English selected through the SDK language API;
  18 tables loaded and unchanged strings retained the normal English fast path.
- The ARM plugin build, 134 pure Rust checks and 77 Python tests passed (7 skipped
  checks require additional native fixtures). Plugin footprint: 9,502,720 bytes,
  within the unchanged budget. The release packager includes fonts, metadata and
  the language configuration with checksums.

These rendering checks do not establish full MHXX gameplay compatibility.
