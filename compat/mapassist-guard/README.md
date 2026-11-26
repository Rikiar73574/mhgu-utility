# MapAssist Role Guard

**Utility v0.5.1 embeds this guard in its existing plugin.** The normal Utility
release ZIP works with existing mod updaters. The standalone build below is
optional for installations without Utility; install only one copy of the guard.

Compatibility fix for Katt404's MapAssist `4c218` binary, MHGU 1.4.0 and
MHGURecomp's Forge hook chaining. Independent of MHGU Utility.

MapAssist recognises map sprites using animation tags and a learned texture
rectangle. Native companion markers can match both, get hidden, and receive a
monster icon. This reproduces in a native two-player local quest with only
MapAssist and KatLib installed, without Utility or replacement UI/textures.

The guard admits a sprite to MapAssist's shared classifier only when its exact
animation instance and sprite belong to a native type-3 enemy map record. It
also validates the current enemy actor association and clears its records when
the owning layout is destroyed. Original MapAssist matching still decides how
to display valid enemy markers. Players, companions and gathering sprites are
left to their existing renderers. No original MapAssist code/assets are included.

## Build and install

```
MHGU_TOOLCHAIN=/path/to/toolchain ./build.sh
```

Copy `../../build/mapassist_role_guard.nro` into a new mod folder:

```
mods/MapAssist-Role-Guard/romfs/nativeNX/plugins/mapassist_role_guard.nro
```

Keep MapAssist and KatLib installed. Add `MapAssist-Role-Guard` to an explicit
`mods/load_order.txt` if one is used. The guard installs after plugin initialisers.
Restart the game when ready; it does not attach to an already running game.
Remove the new mod folder (and its load-order entry) to undo the fix.
The guard leaves other MapAssist versions untouched and logs a warning.

## Verification

- Native two-client quest 10303 with only KatLib + MapAssist: baseline accepts
  companion animation tags 11/13, hides them at 0.24 px and associates monster
  icons. Actual hunters are separate tags 10/12. These tags are layout-local.
- Corrected minimal multiplayer run: all four hunter/companion markers remain
  24 px, actual enemy tag 18 is still replaced; second-client movement tested.
- Corrected full-stack two-client quest with Utility, HuntLog, KatSmithy and
  HD UI/textures: restored native party markers and preserved real monster icon;
  second-client movement and Utility mining markers verified concurrently.
- Host Rust tests check native role, actor and index admission.
- Unicorn tests execute the compiled ARM plugin's hooks: companion rejection,
  valid enemy admission, sprite identity, stale actor rejection, layout
  destruction/recreation and unsupported-binary refusal.

Run the ARM tests with `unicorn` and `pyelftools` installed:

```
python3 tests/arm_roles.py
python3 tests/arm_roles.py --unsupported
# Test the same runtime embedded in a built Utility:
python3 tests/arm_roles.py --utility
python3 tests/arm_roles.py --utility --unsupported
```
