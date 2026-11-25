# Set Builder web preview

Run from the repository root after preparing the private generated data and
`build/web-set-preview/snapshot.json`:

```sh
MHGU_TOOLCHAIN=/path/to/toolchain python3 tools/set_preview.py
python3 tools/set_preview_web/server.py
```

Open `http://127.0.0.1:8780/`. The server binds only to loopback. The local
`build/web-set-preview/ui-effects.json` contains the previously reviewed
`SKILL_EFFECTS`, `SKILL_SUMMARIES` and `SKILL_FULL` captions from the existing
preview. Generated fonts, factual inputs, character captures and saves are
not included in this directory.

Required skills are optional. Leave the list empty to recommend equipment using
the weapon profile and skill priorities; specifying a skill adds a minimum
constraint. Armor, charm and jewel checks remain active in both modes.

Search, Settings and Saved Builds are dedicated views. Back and Escape return
through the views to the Utility landing page. A build displays armor
ownership, all seven socket containers, per-piece jewel placements, active
effects and remaining skill points together. Category and skill ordering
affect the production search adapter, focused seeds and jewel refinement.
The native workspace uses the same weapon-aware priority profile and ranking.

Saved builds, settings and wishlist plans use browser local storage. Export
them before clearing browser data. They are not synchronized to the game.
Saved builds retain their character/search fingerprints and are labelled for
revalidation after either changes. Search settings can be restored from a
saved build without treating its old ownership or availability as current.

Normal searches reuse the captured native candidate list. Explicit Village
and Hub overrides are planning scenarios: indexed quest material sources
filter candidates, but event/request/forge flags are not recreated. Every
non-owned candidate is marked unverified. Owned armor is retained. Missing
source coverage remains provisional. Hub 1–7 is separate from G 1–4 (encoded
as 8–11). These scenarios must not be presented as proof of native unlocks.

Checks:

```sh
python3 -m unittest discover -s tests -p 'test_set_preview*.py' -v
rustc +nightly --edition 2024 --test tests/set_skills.rs -o build/set-skills-tests
build/set-skills-tests
node --check tools/set_preview_web/app.js
```

No server route writes to a game save, installed mod or in-game wishlist.
