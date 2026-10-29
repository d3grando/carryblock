# Carry Block

A **native** Horizon mod (horizon-mixins source mixin): sneak and right click
with an empty hand to pick up a chest, a furnace, a barrel... with everything
in it, or a small animal, and carry it around. Right click a block to put it
down again, exactly as it was.

Inspired by Carry On (https://modrinth.com/mod/carry-on); not affiliated, no code from it.

> **Native mods are not sandboxed.** Unlike `.hmod` (Wasm) mods, a native mod is
> Rust source compiled into your own Horizon executable and runs with your user
> rights. Read the source (it is small: `mixins/carryblock/`) before building.

## What's new in 0.1.2

- **The HUD box keeps off the hotbar.** In a bottom corner, a box that would
  cover the column over the hotbar (the hotbar, the off hand's slot, the XP
  bar, hearts, armour, food and the action bar) goes up over the action bar
  instead, at every GUI scale and window size; a box narrow enough to fit
  beside the hotbar stays in the corner.
- **The HUD box goes with the load.** It disappears the moment the load is put
  down or left behind at death, and never shows on the death screen (it used
  to linger for up to 1.5 seconds).
- **Deny-listed blocks say so.** With the default `blocks: "withData"`,
  bedrock (and anything else on `denyBlocks`) answers "That can't be carried"
  instead of nothing. A plain block the setting leaves out (dirt, stone) stays
  silent, as vanilla's right click on it is.
- **Adventure mode says why.** A sneaking empty-handed right click on
  something a survival player would pick up says "You can't build here" (as
  putting down already did); the click still does what it does in vanilla (a
  chest opens).

## Behaviour

**Picking up.** Sneak, empty main hand, right click:

- a **block with a block entity** (chest, barrel, furnace, hopper, dispenser,
  shulker box, brewing stand, beehive, lectern, sign, banner, jukebox,
  decorated pot, campfire...): the block and its whole block entity
  (contents, custom name, cooking progress, bees, the book, the record...) are
  carried. With `blocks: "any"`, any single block that has an item;
- a **mob** that is not a monster and at most `maxEntitySize` (default 1.5)
  blocks wide and tall: pigs, sheep, cows, chickens, wolves, cats, foxes,
  rabbits, baby villagers... with all its data (name, owner, age, health,
  equipment, UUID).

It works in survival and creative (each can be turned off), never in adventure
or spectator (in adventure, what a survival player would pick up says "You
can't build here", then the click does what it would have done). Things that
are never picked up:

- what the deny lists name (by default bedrock, end portal/frame/gateway,
  nether portal, barrier, light, structure/jigsaw/command/test blocks,
  reinforced deepslate, spawners, trial spawners, vaults, budding amethyst;
  players, the ender dragon, the wither, the warden, the elder guardian);
- half of a two-block thing (beds, doors, tall plants) and extended pistons:
  the other half would break and drop an item;
- blocks without an item (portals, fire, piston heads, moving pistons);
- unbreakable blocks in survival;
- locked containers (a `lock` on the block entity);
- monsters (unless `mobs: "any"`), mobs bigger than `maxEntitySize`, mobs that
  ride or are ridden, leashed mobs, mobs holding other mobs' leads (a wandering
  trader's llamas), mobs with a saddle, chest or inventory (horses, donkeys,
  llamas), pets tamed by someone else, a mob someone is trading with, a
  villager asleep in its bed, `Invulnerable` mobs (except in creative);
- what the player could not break or use there: see
  [Permissions and protection](#permissions-and-protection).

A refused pick up says why in the action bar, then the click does what it
would have done (a deny-listed chest still opens). A deny-listed block says
so whatever `blocks` is set to (bedrock: "That can't be carried"); a block
that only the `blocks` setting leaves out (dirt with the default "with data")
is left to the game without a word, so sneaking around a build never fills the
action bar.

**Carrying.** One thing at a time. The hand slot stays as it was: what is
carried is **not an item**, so it cannot be dropped, stored, thrown into a
hopper or crafted. While carrying, the player has **Slowness I** (setting:
none to IV; a stronger slowness from a potion is left alone), the window shows
a box "Carrying: Chest (12 stacks) / Right click a block to put it down"
(setting: which corner, or off; see [The HUD box](#the-hud-box)) and other
players (LAN guests, a server's
players) get the same reminder in their action bar every 2 seconds. Every right
click goes to putting down: no item is used and nothing is opened while
carrying.

**Putting down.** Right click a block face. A block is placed by vanilla's own
placement rules for its item (there must be room, it must be able to stand
there, nothing may be in the way, it faces as a placed one would and a chest
joins a chest next to it), then gets back its own other properties (a lit
furnace stays lit, a hive keeps its honey, a lectern its book) and its block
entity. A natural sculk shrieker or creaking heart is put down as a placed
one (it summons no warden, spawns no creaking), as silk touch would give it.
A mob is let go on that face, facing you, if that block has room for it.
Putting down works even when picking up is turned off or the settings file is
broken, wherever the player may place a block (see
[Permissions and protection](#permissions-and-protection)). In adventure mode
nothing can be put down, block or mob (vanilla: no building, no spawn egg);
switch mode or see "Death".

**Death.** When a carrier dies, what they carry is left where they died: a
block on the nearest free spot (air, water, grass...) within 2 blocks of their
feet where it can stand (never in another block entity's place, under their
spawn protection or outside the world border); if there is none, it is
dropped as its item holding all its data (the Ctrl + Pick Block form, e.g. a
chest item with its contents in it), as a dead player's inventory is. A mob is let go at their
feet. Respawning, the player carries nothing.

**Disconnecting, quitting, crashing.** What a player carries is part of their
own saved player data (a `carryblock:carried` compound in the player NBT), so
it is saved, loaded and kept exactly like their inventory: a LAN guest who
leaves still carries it when they come back; single player keeps carrying it
after quitting and reopening the world.

### Permissions and protection

A pick up is the game's own break, a put down its own placement: Carry Block
asks what the game asks a player who breaks or places that block (vanilla
`ServerPlayerGameMode.handleBlockBreakAction`/`destroyBlock`,
`ServerGamePacketListenerImpl.handleUseItemOn`/`handleInteract`,
`ServerLevel.mayInteract`; Horizon's `lan_guest_packet`), and refuses with
the reason in the action bar:

| Check | Picking up a block | Picking up a mob | Putting down |
|---|---|---|---|
| In a loaded chunk, within the level's height | the block | the mob's block | the block it goes into |
| Within reach (`isWithinBlockInteractionRange(pos, 1.0)`) | the block | the mob ray's 3 blocks (5 in creative) | the clicked block |
| Spawn protection (`DedicatedServer.isUnderSpawnProtection`) | the block | the mob's block | the clicked block and the block it goes into |
| Inside the world border (`WorldBorder.isWithinBounds`) | the block | the mob's block | the clicked block and the block it goes into |
| May build (`Abilities.mayBuild`: survival, creative) | yes | yes | yes |
| Game master blocks (command, structure, jigsaw, test blocks) | creative and permission level 2 | | creative and permission level 2 |

- **Spawn protection** is a dedicated server's, as vanilla's: `spawn-protection`
  blocks around the world spawn (a square, any height, the Overworld), not
  for operators, for nobody while `ops.json` is empty or the radius is 0.
  Single player and LAN never have it (an integrated server). A refused
  player gets the game's own red "spawn protection" line. Vanilla checks only
  the clicked block when placing; Carry Block also checks the block it puts
  down into, and protects mobs too (vanilla lets anyone use a mob at spawn;
  taking it away is more than using it).
- **Adventure mode** (vanilla's `CanBreak`/`CanPlaceOn`) never lets a player
  break or place with an empty hand, and Carry Block never has an item in it:
  nothing is picked up or put down in adventure mode. Both say "You can't
  build here": putting down on any click, picking up on a click at what a
  survival player would pick up (a chest, a pig; the chest still opens, as it
  does in vanilla). Sneaking at dirt or a button says nothing, and a held
  button says it once.
- **Unbreakable blocks** (bedrock, end portal frames, a map's barrier) are not
  picked up in survival, as they cannot be broken.
- **A container someone has open** (a chest, barrel, furnace, hopper...,
  either half of an open double chest) is not picked up: "Someone is using
  that".
- **Pets and mobs**: a pet tamed by someone else, a mob someone is trading
  with, a villager asleep in its bed (it stays claimed), a mob holding leads,
  an `Invulnerable` mob (a map's: only creative players can hurt it) are not
  picked up.
- **Death** never leaves a block under the dead player's spawn protection or
  outside the border: it is dropped as its item instead.
- **Held or repeated clicks**: a use within 5 ticks of the player's previous
  one does nothing (see below); a refusal is said once per click.

In single player nothing of this gets in the way: there is no spawn
protection, the owner may use game master blocks with cheats on, and the
world border is the only limit (as vanilla's integrated server has it).

Deny-listed by default because no tool gets them in survival, silk touch
included: spawners, trial spawners, vaults and budding amethyst (carrying a
dungeon's spawner next to another's, or a vault out of a trial chamber, is
what vanilla only lets a player destroy). A server that wants them movable
takes them off `denyBlocks`.

### How duplication is prevented

Every copy of the data exists in exactly one place at any moment, and every
move is done in one go on the game's own thread:

- **Never an item.** While carried, the thing exists only in the carrier's
  player compound. There is no stack to copy with creative middle click,
  shift-click into a container, drop, or put in a bundle.
- **Pick up removes before it carries.** The block entity is read, then the
  block is removed by the game's own `setBlock` without the block entity's
  side effects (nothing spills, no item pops); only if that removal happened is
  the carried compound written. What Horizon also remembers for that position
  (the saved compound of a block entity it keeps as loaded, the compound the
  block entity was loaded from, a pending loot roll) is removed too, so a
  block placed there later cannot inherit it (no loot re-roll).
- **Put down places before it forgets, and forgets as soon as it placed.** The
  carried compound is cleared the moment the block is written to the world,
  whatever happens to the block next (a falling block that turns into an
  entity is not "still carried"). If the write does not happen, nothing
  changed and it stays carried.
- **Open containers.** A player who has the chest open while it is picked up
  cannot take anything out: Horizon's container menus read and write the
  block entity in the world, which is gone (the menu closes). Two halves of a
  double chest: the open double menu needs both.
- **Mobs** are removed with the game's `remove` (no death, no drops, no
  experience); a mob already removed this tick (picked up by someone else,
  despawned) is not picked up again. Mobs with an inventory (saddle, chest)
  are refused, because the mount menu works on a copy of it.
- **Two players, one block.** Everything runs on the server's thread; the
  second click finds air.
- **Held button.** A use within 5 ticks of the player's previous one (the use
  button held down, or the second packet of the same click) neither picks up
  nor puts down.
- **Death.** The load is released first and only then cleared from the dead
  player; the release never fails half way (a block that finds no spot is
  dropped as an item instead).
- **Saving.** The carried compound is saved with the player and the world's
  chunks without it: the owner of a single player world (and an in-frame
  world's player) is saved in the same save as the chunks, so a crash never
  finds both or neither. A LAN guest is saved when they leave, as Horizon
  saves every guest's inventory: a crash between a guest leaving and the next
  autosave can duplicate or lose what they carried exactly as it can their
  inventory or a chest they emptied (vanilla behaves the same).

## Commands

| Command | Effect |
|---|---|
| `/carryblock` or `/carryblock status` | Says what you carry |
| `/carryblock reload` | Re-read `config.json` (permission level 2) |

They run where the game runs commands: in single player, on a server whose
players may use commands (a LAN guest without command rights gets "unknown
command"; Carry Block itself needs no command).

## Settings

**Options > Mods** lists Carry Block with its **Settings** button. The row's own
button turns the mod off (it then does nothing at all, the launcher's next
build leaves it out; what players carry stays in their saved data until it is
back) and on again. One button per setting; a click moves it to its next
value, and it applies at once, for every player, no restart:

| Button | Key | Values a click cycles through |
|---|---|---|
| Picking up: ON | `enabled` | ON, OFF (putting down always works) |
| Survival players may carry: ON | `survival` | ON, OFF |
| Creative players may carry: ON | `creative` | ON, OFF |
| Creative: same rules as survival | `creativeIgnoresRules` | same rules, ignores lists/size/monster rule |
| Blocks: with contents or data only | `blocks` | with data (`withData`), any single block (`any`), none (`off`) |
| Mobs: animals, no monsters | `mobs` | `passive`, monsters too (`any`), none (`off`) |
| Largest mob: 1.5 blocks | `maxEntitySize` | 0.5, 0.75, 1, 1.5, 2, 3, 5 |
| Slowness while carrying: I | `slowness` | none, I, II, III, IV |
| HUD box: bottom right | `hud` | off, top left, top right, bottom left, bottom right |
| Action bar for other players: ON | `actionBar` | ON, OFF |
| Deny lists: 20 blocks, 5 mobs (config.json, click to reload) | `denyBlocks`, `denyEntities` | edited in the file; a click reads it again |
| Reset to defaults (deny lists kept) | | every setting above back to its default |

"Creative ignores rules" never lifts the rules that keep the world
consistent or a server's permissions: two-block things, blocks without an
item, locks, open containers, riders, leashes, mob inventories, other players'
pets, trading and sleeping mobs, game master blocks, spawn protection and the
world border.

### The HUD box

The HUD box is drawn by the window from what the game simulating the world
says (single player and LAN hosts: same process). On another computer's
server, the action bar shows it instead. It goes the moment the load is put
down or left behind at death, and doesn't show on the death screen.

In a bottom corner (bottom right by default), a box too wide for the room
beside the hotbar would cover its end: there it goes up instead, a margin over
the action bar (74 GUI pixels from the bottom), clear of the hotbar, the off
hand's slot, the XP bar, two rows of hearts, armour, food, air and the held
item's name. The room beside the hotbar is half the screen's GUI width less
120 (the hotbar's half, 91, and the off hand's slot, 29), so it is worked out
for every GUI scale and window size: at 1280x720 and GUI scale 3 (427 GUI
pixels wide) about 90 pixels, so "Carrying: Chest (8 stacks)" goes up; on a
wide screen at a small GUI scale it stays in the corner. Boxes other mods put
in the same corner first (the minimap) still stack under it.

## Configuration file

The settings page writes, and the mod reads, one JSON file in the mod's
settings folder, beside the game's `options.txt`:

```
horizon-mods/.horizon-mod-data/carryblock/config.json
```

A missing file is written with the defaults. Edit it by hand for the deny
lists, then click the Deny lists button or run `/carryblock reload`. Every key
is optional; unknown keys or out-of-range values stop picking up (putting down
still works; the page shows the error and a reset, stderr says
`Carry Block: no picking up: ...`) until the file is fixed. Defaults:

```json
{
  "enabled": true,
  "survival": true,
  "creative": true,
  "creativeIgnoresRules": false,
  "blocks": "withData",
  "mobs": "passive",
  "maxEntitySize": 1.5,
  "slowness": 1,
  "hud": "bottomRight",
  "actionBar": true,
  "denyBlocks": ["minecraft:bedrock", "minecraft:end_portal", "..."],
  "denyEntities": ["minecraft:player", "minecraft:ender_dragon", "minecraft:wither", "minecraft:warden", "minecraft:elder_guardian"]
}
```

Deny list entries: ids (`minecraft:spawner`, or `spawner`), `*` globs
(`minecraft:*_shulker_box`, `mymod:*`) and, for blocks, block tags
(`#minecraft:beds`).

## Compatibility

Horizon's native mods can use **hook points** (`game/native_hooks.rs`,
`NATIVE_HOOKS_API` 1), kept stable from release to release. Carry Block uses
them for everything they cover:

- `native_tick`: slowness, the HUD/action bar, releasing a dead carrier's
  load; the window passes Options > Mods' on/off to single player's server
  thread;
- `native_hud`: the HUD box (placed with `Corners`, lifted over the hotbar in
  a bottom corner, drawn with `native_fill` and `native_text`, hidden where
  `native_corner_shown` says F3 is; the death screen is the game's `Screen`);
- `native_command`: `/carryblock`;
- `native_mods`, `native_settings`, `native_settings_click`: the row in
  Options > Mods and the settings page; `native_setting`/`native_set_setting`
  for the file, `native_on`/`native_window` for the on/off.

There is no hook point for a right click, so Carry Block hooks one game
function directly and is **pinned** to one Horizon commit:

| Hook | Kind | Why it is not a hook point |
|---|---|---|
| `Game::use_item` | HEAD, cancellable | Picking up and putting down replace a right click: the in-frame player's use key and a LAN guest's `UseItemOn` and `Interact` packets all come through it, with the guest's state swapped in. There is no use/interact hook point |

Beyond hooks, `native.rs` uses the pinned game's internals: block entities
(`encode_block_entity`/`decode_block_entity`, the saved compounds of the ones
Horizon keeps as loaded), the block behaviour library (`set_block`,
`placement_state`, `can_survive`), mobs' saved compounds (`mob_tree_nbt`,
`load_mob`), the player compound (`Level::player_extra`, `Guest::saved`), the
hub's players and their effects, and for permissions the hub's
`spawn_protection`, operators, permission levels, open menus and trades (read
every tick: a guest's turn runs with the hub taken out of the game), the
world border and the world spawn.

Pinned: Horizon `142388e1c136e622652ec7505af15d8ad195e041`, horizon-mixins
`0d6b103c70633d8fd9443d8d0acd3aadaacedb06`, game 26.3: the same revisions as
TreeFall 0.3.0 and Mason 0.2.0, so the three build into one game (checked:
planned and applied together, 23 injections, 14 modules; 0.1.2 also with TreeFall,
BiomeCompass 0.2.2, LookAt, EnchantmentHints, ItemNew, MouseTricks and
SmartF3: 55 injections, 17 modules, and `cargo check -p horizon-client
--features mods` of the applied source). Mason also hooks
`Game::use_item`, for its wand (a wooden axe, creative only); Carry Block's
priority is 250, below Mason's 300, so a creative player holding the wand
selects with it even while carrying. A pick up always needs an empty hand, so
the two never compete for the same click.

## Build and try

Carry Block is a source bundle; the launcher compiles it into a custom game:

```sh
horizon-launcher native-build <this folder> --version v0.2.97 --trust-native-code
```

`--version` is the installed release whose game files the build reuses; the
code is the pinned commit's whatever the release. `<this folder>` may also be
a folder holding several native mods (with TreeFall and Mason, for example).

## Development

`mixins/carryblock/`:

- `horizon-mixin.toml`: two support modules and seven HEAD hooks (above).
- `runtime.rs`: every decision as pure code with unit tests: the settings and
  their page, deny lists, what may be picked up, who may pick up or put down
  where (spawn protection, the world border, reach, building), the state a
  block is put down with, where a dead carrier's block goes, held-button
  detection, slowness, where the HUD box goes and when it shows,
  texts, commands. It is both this repository's Cargo library and the injected
  `crate::game::carryblock` module.
- `native.rs`: the adapter to the game (`crate::game::carryblock_native`).
- `use.rs`, `tick.rs`, `hud.rs`, `command.rs`, `list.rs`, `settings.rs`,
  `click.rs`: the hook helpers.

```sh
cargo test
cargo fmt --check                                   # runtime.rs
rustfmt --edition 2024 --check mixins/carryblock/*.rs
cargo clippy --all-targets -- -D warnings
python tools/pack_hnmod.py   # dist/carryblock-<version>.hnmod
```

Not affiliated with Carry On or its authors; original code, written from the
mod's public description only. MIT licensed.
