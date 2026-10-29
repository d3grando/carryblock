// Carry Block: sneak and right click with an empty hand to pick up a block with
// everything in it (a chest, a furnace...) or a small mob, carry it around and
// right click to put it down again.
//
// Original implementation for Horizon (a horizon-mixins native mod). This file
// is both the standalone, unit-tested Cargo library and the injected
// `crate::game::carryblock` module. It has no Horizon dependency: it decides
// (what may be picked up, the state a block is put down with, the settings
// page, the commands, the HUD text), and `native.rs` does what it decides in
// the running game.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// A player's UUID as the game keeps it.
pub type Uuid = [i32; 4];

/// The mod's id: its row in Options > Mods and its settings folder
/// (`horizon-mods/.horizon-mod-data/carryblock/`).
pub const ID: &str = "carryblock";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "Sneak and right click with an empty hand to pick up a chest, a \
furnace or a small animal with everything in it; right click a block to put it down.";
/// The configuration, in the mod's settings folder: what the settings page
/// writes, and where the deny lists are edited by hand.
pub const CONFIG_FILE: &str = "config.json";
/// The chat command (without its slash).
pub const COMMAND: &str = "carryblock";
pub const USAGE: &str = "Usage: /carryblock [status|reload]";

/// The key of the player's saved compound that holds what they carry. It is
/// saved and loaded with the player, exactly like their inventory.
pub const CARRY_KEY: &str = "carryblock:carried";
/// The keys of the carried compound.
pub const KEY_FORMAT: &str = "format";
pub const KEY_KIND: &str = "kind";
/// A block: its state (`minecraft:chest[facing=north,...]`).
pub const KEY_STATE: &str = "state";
/// A block: its block entity as the game saves it, when it has one.
pub const KEY_BLOCK_ENTITY: &str = "block_entity";
/// A mob: its entity compound as the game saves it.
pub const KEY_ENTITY: &str = "entity";
/// What the HUD and `/carryblock` call it.
pub const KEY_NAME: &str = "name";
pub const KIND_BLOCK: &str = "block";
pub const KIND_MOB: &str = "mob";
/// The carried compound's layout.
pub const FORMAT: i32 = 1;

/// A use this many ticks or less after the player's previous one is the use
/// button held down (it repeats every 4 ticks): it neither picks up nor puts
/// down, so holding the button never flips between the two.
pub const REPEAT_TICKS: u64 = 5;
/// The slowness a carrier gets lasts this long and is renewed once it has
/// `SLOWNESS_REFRESH` ticks left, so it wears off soon after putting down.
pub const SLOWNESS_TICKS: i32 = 40;
pub const SLOWNESS_REFRESH: i32 = 10;
/// How often a remote carrier is reminded in their action bar (ticks).
pub const ACTION_BAR_EVERY: u64 = 40;
/// How long the HUD box shows after the server last said what is carried.
pub const HUD_FRESH_MS: u64 = 1500;
/// How far from a dead carrier's feet a block is put down.
pub const RELEASE_RADIUS: i32 = 2;

/// What may be picked up.
pub const DEFAULT_DENY_BLOCKS: &[&str] = &[
    "minecraft:bedrock",
    "minecraft:end_portal",
    "minecraft:end_portal_frame",
    "minecraft:end_gateway",
    "minecraft:nether_portal",
    "minecraft:barrier",
    "minecraft:light",
    "minecraft:structure_void",
    "minecraft:structure_block",
    "minecraft:jigsaw",
    "minecraft:command_block",
    "minecraft:chain_command_block",
    "minecraft:repeating_command_block",
    "minecraft:test_block",
    "minecraft:test_instance_block",
    "minecraft:reinforced_deepslate",
];
pub const DEFAULT_DENY_ENTITIES: &[&str] = &[
    "minecraft:player",
    "minecraft:ender_dragon",
    "minecraft:wither",
    "minecraft:warden",
    "minecraft:elder_guardian",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BlockRule {
    /// Blocks with a block entity: chests, furnaces, barrels, signs...
    WithData,
    /// Any single block that has an item.
    Any,
    Off,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MobRule {
    /// Mobs that are not monsters.
    Passive,
    Any,
    Off,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HudCorner {
    Off,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct Config {
    /// Picking up at all (putting down always works).
    pub enabled: bool,
    /// Survival players may pick up.
    pub survival: bool,
    /// Creative players may pick up.
    pub creative: bool,
    /// Creative players ignore the deny lists, the mob size and the monster
    /// rule (never the rules that keep the world consistent).
    pub creative_ignores_rules: bool,
    pub blocks: BlockRule,
    pub mobs: MobRule,
    /// The largest mob (its width or height, in blocks).
    pub max_entity_size: f32,
    /// Slowness level while carrying, 0 for none.
    pub slowness: u8,
    /// Where the window shows what its player carries.
    pub hud: HudCorner,
    /// Remind other players what they carry in their action bar.
    pub action_bar: bool,
    /// Blocks never picked up: ids (`minecraft:spawner`, `spawner`), `*`
    /// globs (`minecraft:*_shulker_box`) and block tags (`#minecraft:beds`).
    pub deny_blocks: Vec<String>,
    /// Mobs never picked up: ids and `*` globs.
    pub deny_entities: Vec<String>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            survival: true,
            creative: true,
            creative_ignores_rules: false,
            blocks: BlockRule::WithData,
            mobs: MobRule::Passive,
            max_entity_size: 1.5,
            slowness: 1,
            hud: HudCorner::BottomRight,
            action_bar: true,
            deny_blocks: DEFAULT_DENY_BLOCKS
                .iter()
                .map(|s| (*s).to_owned())
                .collect(),
            deny_entities: DEFAULT_DENY_ENTITIES
                .iter()
                .map(|s| (*s).to_owned())
                .collect(),
        }
    }
}
impl Config {
    pub fn from_json(source: &str) -> Result<Self, String> {
        let config: Self = serde_json::from_str(source).map_err(|e| e.to_string())?;
        config.validate()?;
        Ok(config)
    }
    /// The file the settings page writes: every key, readable by `from_json`.
    pub fn to_json(&self) -> String {
        let mut json = serde_json::to_string_pretty(self).unwrap_or_default();
        json.push('\n');
        json
    }
    pub fn validate(&self) -> Result<(), String> {
        if !(self.max_entity_size.is_finite() && (0.1..=16.0).contains(&self.max_entity_size)) {
            return Err("maxEntitySize must be 0.1..=16".into());
        }
        if self.slowness > 4 {
            return Err("slowness must be 0..=4".into());
        }
        for (key, list, tags) in [
            ("denyBlocks", &self.deny_blocks, true),
            ("denyEntities", &self.deny_entities, false),
        ] {
            if let Some(bad) = list.iter().find(|p| !valid_pattern(p, tags)) {
                return Err(format!("{key}: bad entry {bad:?}"));
            }
        }
        Ok(())
    }
}

/// `ns:path` ids, `*` globs and (for blocks) `#tag` entries.
fn valid_pattern(pattern: &str, tags: bool) -> bool {
    let body = match pattern.strip_prefix('#') {
        Some(rest) if tags => rest,
        Some(_) => return false,
        None => pattern,
    };
    !body.is_empty()
        && body.len() <= 128
        && body.matches(':').count() <= 1
        && body.chars().all(|c| {
            c.is_ascii_lowercase()
                || c.is_ascii_digit()
                || matches!(c, '_' | '-' | '.' | '/' | ':')
                || c == '*' && !pattern.starts_with('#')
        })
}

/// `id` with the `minecraft:` namespace when it has none.
pub fn namespaced(id: &str) -> String {
    if id.contains(':') {
        id.to_owned()
    } else {
        format!("minecraft:{id}")
    }
}

/// Whether `text` matches `pattern`, where `*` stands for any run of characters.
pub fn glob(pattern: &str, text: &str) -> bool {
    let (p, t): (Vec<char>, Vec<char>) = (pattern.chars().collect(), text.chars().collect());
    let (mut pi, mut ti) = (0, 0);
    let mut star: Option<(usize, usize)> = None;
    while ti < t.len() {
        if pi < p.len() && p[pi] == '*' {
            star = Some((pi, ti));
            pi += 1;
        } else if pi < p.len() && p[pi] == t[ti] {
            pi += 1;
            ti += 1;
        } else if let Some((sp, st)) = star {
            pi = sp + 1;
            ti = st + 1;
            star = Some((sp, st + 1));
        } else {
            return false;
        }
    }
    p[pi..].iter().all(|&c| c == '*')
}

/// Whether `id` is on a deny list. `in_tag` answers for `#tag` entries
/// (namespaced tag names).
pub fn denied(patterns: &[String], id: &str, in_tag: &dyn Fn(&str) -> bool) -> bool {
    let id = namespaced(id);
    patterns.iter().any(|p| match p.strip_prefix('#') {
        Some(tag) => in_tag(&namespaced(tag)),
        None => glob(&namespaced(p), &id),
    })
}

/// The game mode of the player acting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

/// What to do with a sneaking empty-handed right click.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// Pick it up.
    Allow,
    /// Not ours: the game uses the click as usual.
    Pass,
    /// Tell the player why not, then let the game use the click.
    Refuse(Refusal),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Denied,
    Unbreakable,
    TwoBlocks,
    NoItem,
    Locked,
    Hostile,
    TooBig,
    Riding,
    Leashed,
    Inventory,
    NotYours,
}
impl Refusal {
    pub fn message(self) -> &'static str {
        match self {
            Self::Denied => "That can't be carried",
            Self::Unbreakable => "That can't be moved",
            Self::TwoBlocks => "Only single blocks can be carried",
            Self::NoItem => "That can't be carried",
            Self::Locked => "It's locked",
            Self::Hostile => "Monsters can't be carried",
            Self::TooBig => "That's too big to carry",
            Self::Riding => "Something is riding it, or it rides something",
            Self::Leashed => "Unleash it first",
            Self::Inventory => "Take its saddle and chest off first",
            Self::NotYours => "That pet isn't yours",
        }
    }
}

/// Whether this player may pick something up with this click at all:
/// sneaking, the main hand empty, and their game mode allowed.
pub fn player_may_pick_up(config: &Config, mode: Mode, sneaking: bool, hand_empty: bool) -> bool {
    config.enabled
        && sneaking
        && hand_empty
        && match mode {
            Mode::Survival => config.survival,
            Mode::Creative => config.creative,
            Mode::Adventure | Mode::Spectator => false,
        }
}

fn ignores_rules(config: &Config, mode: Mode) -> bool {
    mode == Mode::Creative && config.creative_ignores_rules
}

/// A block as the game describes it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlockInfo {
    pub name: String,
    pub properties: BTreeMap<String, String>,
    /// It has a block entity (`BlockState.hasBlockEntity`).
    pub block_entity: bool,
    /// `destroyTime`: below 0 is unbreakable.
    pub hardness: f32,
    /// Its item (`minecraft:air` is none).
    pub item: Option<String>,
    /// Air or a fluid: nothing to pick up.
    pub empty: bool,
    /// Its block entity has a lock (`lock`).
    pub locked: bool,
}

/// Half of a block that is two blocks (beds, doors, tall plants), or an
/// extended piston: the other half would break on its own and drop an item.
pub fn is_two_blocks(properties: &BTreeMap<String, String>) -> bool {
    let is = |k: &str, values: &[&str]| {
        properties
            .get(k)
            .is_some_and(|v| values.contains(&v.as_str()))
    };
    is("part", &["head", "foot"]) || is("half", &["upper", "lower"]) || is("extended", &["true"])
}

/// Whether `block` may be picked up. `in_tag` answers block tags for the
/// deny list.
pub fn block_verdict(
    config: &Config,
    mode: Mode,
    block: &BlockInfo,
    in_tag: &dyn Fn(&str) -> bool,
) -> Verdict {
    if block.empty {
        return Verdict::Pass;
    }
    match config.blocks {
        BlockRule::Off => return Verdict::Pass,
        BlockRule::WithData if !block.block_entity => return Verdict::Pass,
        BlockRule::WithData | BlockRule::Any => {}
    }
    let bypass = ignores_rules(config, mode);
    if !bypass && denied(&config.deny_blocks, &block.name, in_tag) {
        return Verdict::Refuse(Refusal::Denied);
    }
    if block.item.is_none() {
        return Verdict::Refuse(Refusal::NoItem);
    }
    if is_two_blocks(&block.properties) {
        return Verdict::Refuse(Refusal::TwoBlocks);
    }
    if block.hardness < 0.0 && mode != Mode::Creative {
        return Verdict::Refuse(Refusal::Unbreakable);
    }
    if block.locked {
        return Verdict::Refuse(Refusal::Locked);
    }
    Verdict::Allow
}

/// A mob as the game describes it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MobInfo {
    pub id: String,
    pub width: f32,
    pub height: f32,
    /// In the `monster` category.
    pub hostile: bool,
    pub alive: bool,
    /// Already gone (picked up, despawned) this tick.
    pub removed: bool,
    pub riding: bool,
    pub ridden: bool,
    pub leashed: bool,
    /// A saddle, a chest or an inventory (horses, llamas...).
    pub inventory: bool,
    /// Tamed by another player.
    pub owned_by_other: bool,
}

pub fn mob_verdict(config: &Config, mode: Mode, mob: &MobInfo) -> Verdict {
    if mob.removed || !mob.alive || config.mobs == MobRule::Off {
        return Verdict::Pass;
    }
    let bypass = ignores_rules(config, mode);
    if !bypass {
        if denied(&config.deny_entities, &mob.id, &|_| false) {
            return Verdict::Refuse(Refusal::Denied);
        }
        if config.mobs == MobRule::Passive && mob.hostile {
            return Verdict::Refuse(Refusal::Hostile);
        }
        if mob.width.max(mob.height) > config.max_entity_size + 1e-4 {
            return Verdict::Refuse(Refusal::TooBig);
        }
    }
    if mob.riding || mob.ridden {
        return Verdict::Refuse(Refusal::Riding);
    }
    if mob.leashed {
        return Verdict::Refuse(Refusal::Leashed);
    }
    if mob.inventory {
        return Verdict::Refuse(Refusal::Inventory);
    }
    if mob.owned_by_other {
        return Verdict::Refuse(Refusal::NotYours);
    }
    Verdict::Allow
}

/// A block state: its name and properties (`name[k=v,...]` as text).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub name: String,
    pub properties: BTreeMap<String, String>,
}
impl State {
    pub fn parse(text: &str) -> Option<Self> {
        let (name, rest) = match text.split_once('[') {
            Some((n, rest)) => (n, Some(rest.strip_suffix(']')?)),
            None => (text, None),
        };
        if name.is_empty() || name.contains([']', ',', '=']) {
            return None;
        }
        let mut properties = BTreeMap::new();
        for pair in rest.unwrap_or("").split(',').filter(|p| !p.is_empty()) {
            let (k, v) = pair.split_once('=')?;
            if k.is_empty() || v.is_empty() {
                return None;
            }
            properties.insert(k.to_owned(), v.to_owned());
        }
        Some(Self {
            name: namespaced(name),
            properties,
        })
    }
    pub fn text(&self) -> String {
        if self.properties.is_empty() {
            return self.name.clone();
        }
        let props: Vec<String> = self
            .properties
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        format!("{}[{}]", self.name, props.join(","))
    }
}

/// Properties the spot and the player decide when a block is put down
/// (which way it faces, what it connects to, water, redstone power): every
/// other property (a furnace's `lit`, a hive's `honey_level`, a lectern's
/// `has_book`...) is the carried block's own, so it matches its contents.
pub const PLACEMENT_KEYS: &[&str] = &[
    "facing",
    "axis",
    "rotation",
    "type",
    "waterlogged",
    "face",
    "attachment",
    "hanging",
    "half",
    "shape",
    "north",
    "south",
    "east",
    "west",
    "up",
    "down",
    "attached",
    "in_wall",
    "hinge",
    "open",
    "powered",
    "triggered",
    "enabled",
];

/// The state to put a carried block down with: `placed` (what placing its
/// item there gives) with the carried block's other properties. A double
/// slab stays double.
pub fn placed_state(carried: &State, placed: &State) -> State {
    let mut state = placed.clone();
    for (k, v) in &carried.properties {
        let decided_here = PLACEMENT_KEYS.contains(&k.as_str())
            && !(k == "type" && v == "double" && carried.name == placed.name);
        if !decided_here && state.properties.contains_key(k) {
            state.properties.insert(k.clone(), v.clone());
        }
    }
    state
}

/// The state to leave a carried block in where its carrier died: as carried,
/// alone (no chest half), closed, and holding water when it is put in water.
pub fn released_state(carried: &State, water: bool) -> State {
    let mut state = carried.clone();
    let p = &mut state.properties;
    if p.get("type").is_some_and(|t| t == "left" || t == "right") {
        p.insert("type".into(), "single".into());
    }
    for key in ["open", "powered"] {
        if p.contains_key(key) {
            p.insert(key.into(), "false".into());
        }
    }
    if p.contains_key("waterlogged") {
        p.insert("waterlogged".into(), water.to_string());
    }
    state
}

/// `minecraft:oak_wall_sign` as `Oak Wall Sign`.
pub fn display_name(id: &str) -> String {
    let path = id.rsplit(':').next().unwrap_or(id);
    path.split(['_', '/'])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            c.next().map_or_else(String::new, |f| {
                f.to_uppercase().chain(c).collect::<String>()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// What the HUD calls a carried thing: its name and, for a container, how
/// many stacks are in it.
pub fn summary(name: &str, stacks: Option<usize>) -> String {
    match stacks {
        Some(1) => format!("{name} (1 stack)"),
        Some(n) if n > 0 => format!("{name} ({n} stacks)"),
        _ => name.to_owned(),
    }
}
/// The window's HUD box.
pub fn hud_lines(summary: &str) -> [String; 2] {
    [
        format!("Carrying: {summary}"),
        "Right click a block to put it down".into(),
    ]
}
/// A remote player's action bar.
pub fn action_bar_text(summary: &str) -> String {
    format!("Carrying {summary} - right click a block to put it down")
}
/// `/carryblock status`.
pub fn status_text(summary: Option<&str>) -> String {
    match summary {
        Some(s) => format!("Carry Block {VERSION}: you are carrying {s}"),
        None => format!("Carry Block {VERSION}: you are not carrying anything"),
    }
}

/// Where around a dead carrier's feet their block goes, nearest first (never
/// more than one block below the feet).
pub fn release_spots(radius: i32) -> Vec<[i32; 3]> {
    let mut spots = Vec::new();
    for dy in -1..=radius {
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                spots.push([dx, dy, dz]);
            }
        }
    }
    spots.sort_by_key(|&[dx, dy, dz]| (dx * dx + dy * dy + dz * dz, dy < 0, dy.abs(), dx, dz));
    spots
}

/// Each player's last use, to tell a click from the use button held down.
#[derive(Clone, Debug, Default)]
pub struct Clicks {
    last: HashMap<Uuid, u64>,
}
impl Clicks {
    /// Notes a use by `who` at tick `now`: true when it is a new click.
    pub fn fresh(&mut self, who: Uuid, now: u64) -> bool {
        let previous = self.last.insert(who, now);
        if self.last.len() > 256 {
            self.last
                .retain(|_, t| now.saturating_sub(*t) <= REPEAT_TICKS);
        }
        previous.is_none_or(|p| now < p || now - p > REPEAT_TICKS)
    }
}

/// The slowness amplifier carrying gives, if any.
pub fn slowness_amplifier(config: &Config) -> Option<i32> {
    (config.slowness > 0).then(|| i32::from(config.slowness) - 1)
}
/// Whether to (re)give a carrier slowness `amplifier`, given the slowness
/// they have (amplifier, ticks left; -1 forever). A stronger or longer one
/// from elsewhere is left alone.
pub fn slowness_wanted(current: Option<(i32, i32)>, amplifier: i32) -> bool {
    match current {
        None => true,
        Some((a, _)) if a > amplifier => false,
        Some((a, _)) if a < amplifier => true,
        Some((_, d)) => d != -1 && d <= SLOWNESS_REFRESH,
    }
}
/// Whether an effect instance is the slowness carrying gave (so putting
/// down may take it away): faint, without particles, nothing under it, and
/// no longer than carrying gives it.
pub fn is_carry_slowness(duration: i32, ambient: bool, visible: bool, hides_other: bool) -> bool {
    ambient && !visible && !hides_other && (0..=SLOWNESS_TICKS).contains(&duration)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Status,
    Reload,
}
pub fn is_command(line: &str) -> bool {
    let line = line.trim().trim_start_matches('/');
    line == COMMAND || line.starts_with(&format!("{COMMAND} "))
}
/// `/carryblock ...`: None when the line is not Carry Block's.
pub fn parse_command(line: &str) -> Option<Result<Command, &'static str>> {
    if !is_command(line) {
        return None;
    }
    let mut words = line
        .trim()
        .trim_start_matches('/')
        .split_whitespace()
        .skip(1);
    let command = match (words.next(), words.next()) {
        (None | Some("status"), None) => Ok(Command::Status),
        (Some("reload"), None) => Ok(Command::Reload),
        _ => Err(USAGE),
    };
    Some(command)
}

/// The configuration as read from [`CONFIG_FILE`]: Err when the file is
/// broken (no new pick ups until it is fixed or reset).
pub type Loaded = Result<Config, String>;

/// A button of the settings page (Options > Mods > Carry Block > Settings).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setting {
    /// The file is broken: its error. A click reads it again.
    Broken,
    Enabled,
    Survival,
    Creative,
    CreativeRules,
    Blocks,
    Mobs,
    MaxSize,
    Slowness,
    Hud,
    ActionBar,
    /// The deny lists, edited in the file. A click reads it again.
    Lists,
    /// Every setting back to its default (the lists kept).
    Defaults,
}
/// What a click on the settings page asks of the game.
#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    /// Write this configuration to the file; it applies at once.
    Save(Config),
    /// Read the file again (after editing it by hand).
    Reload,
}

/// The steps `maxEntitySize` cycles through (the default among them).
pub const MAX_ENTITY_SIZE_STEPS: [f32; 7] = [0.5, 0.75, 1.0, 1.5, 2.0, 3.0, 5.0];
/// Longest error shown on the page.
const ERROR_WIDTH: usize = 40;

/// The next step above `now`, back to the first after the last.
fn step<T: Copy + PartialOrd>(steps: &[T], now: T) -> T {
    steps.iter().copied().find(|&s| s > now).unwrap_or(steps[0])
}
fn on_off(on: bool) -> &'static str {
    if on { "ON" } else { "OFF" }
}
fn roman(level: u8) -> &'static str {
    ["none", "I", "II", "III", "IV"][usize::from(level.min(4))]
}

/// The page's buttons, in order.
pub fn page(loaded: &Loaded) -> Vec<Setting> {
    if loaded.is_err() {
        return vec![Setting::Broken, Setting::Defaults];
    }
    vec![
        Setting::Enabled,
        Setting::Survival,
        Setting::Creative,
        Setting::CreativeRules,
        Setting::Blocks,
        Setting::Mobs,
        Setting::MaxSize,
        Setting::Slowness,
        Setting::Hud,
        Setting::ActionBar,
        Setting::Lists,
        Setting::Defaults,
    ]
}

impl Setting {
    /// Its button's label.
    pub fn label(self, loaded: &Loaded) -> String {
        let c = match loaded {
            Ok(c) => c,
            Err(error) => {
                return match self {
                    Self::Defaults => "Reset to defaults".into(),
                    _ => {
                        let mut short: String = error.chars().take(ERROR_WIDTH).collect();
                        if short.len() < error.len() {
                            short = format!("{}...", short.trim_end());
                        }
                        format!("{CONFIG_FILE}: {short} (click to reload)")
                    }
                };
            }
        };
        match self {
            Self::Broken => String::new(),
            Self::Enabled => format!("Picking up: {}", on_off(c.enabled)),
            Self::Survival => format!("Survival players may carry: {}", on_off(c.survival)),
            Self::Creative => format!("Creative players may carry: {}", on_off(c.creative)),
            Self::CreativeRules => if c.creative_ignores_rules {
                "Creative: ignores lists, size and monster rule"
            } else {
                "Creative: same rules as survival"
            }
            .into(),
            Self::Blocks => match c.blocks {
                BlockRule::WithData => "Blocks: with contents or data only",
                BlockRule::Any => "Blocks: any single block",
                BlockRule::Off => "Blocks: none",
            }
            .into(),
            Self::Mobs => match c.mobs {
                MobRule::Passive => "Mobs: animals, no monsters",
                MobRule::Any => "Mobs: monsters too",
                MobRule::Off => "Mobs: none",
            }
            .into(),
            Self::MaxSize => format!("Largest mob: {} blocks", c.max_entity_size),
            Self::Slowness => format!("Slowness while carrying: {}", roman(c.slowness)),
            Self::Hud => match c.hud {
                HudCorner::Off => "HUD box: off",
                HudCorner::TopLeft => "HUD box: top left",
                HudCorner::TopRight => "HUD box: top right",
                HudCorner::BottomLeft => "HUD box: bottom left",
                HudCorner::BottomRight => "HUD box: bottom right",
            }
            .into(),
            Self::ActionBar => format!("Action bar for other players: {}", on_off(c.action_bar)),
            Self::Lists => format!(
                "Deny lists: {} blocks, {} mobs ({CONFIG_FILE}, click to reload)",
                c.deny_blocks.len(),
                c.deny_entities.len()
            ),
            Self::Defaults => "Reset to defaults (deny lists kept)".into(),
        }
    }

    /// What a click on it does to `loaded`.
    pub fn click(self, loaded: &Loaded) -> Effect {
        let Ok(c) = loaded else {
            return match self {
                Self::Defaults => Effect::Save(Config::default()),
                _ => Effect::Reload,
            };
        };
        let mut c = c.clone();
        match self {
            Self::Broken | Self::Lists => return Effect::Reload,
            Self::Enabled => c.enabled = !c.enabled,
            Self::Survival => c.survival = !c.survival,
            Self::Creative => c.creative = !c.creative,
            Self::CreativeRules => c.creative_ignores_rules = !c.creative_ignores_rules,
            Self::Blocks => {
                c.blocks = match c.blocks {
                    BlockRule::WithData => BlockRule::Any,
                    BlockRule::Any => BlockRule::Off,
                    BlockRule::Off => BlockRule::WithData,
                }
            }
            Self::Mobs => {
                c.mobs = match c.mobs {
                    MobRule::Passive => MobRule::Any,
                    MobRule::Any => MobRule::Off,
                    MobRule::Off => MobRule::Passive,
                }
            }
            Self::MaxSize => c.max_entity_size = step(&MAX_ENTITY_SIZE_STEPS, c.max_entity_size),
            Self::Slowness => c.slowness = (c.slowness + 1) % 5,
            Self::Hud => {
                c.hud = match c.hud {
                    HudCorner::Off => HudCorner::TopLeft,
                    HudCorner::TopLeft => HudCorner::TopRight,
                    HudCorner::TopRight => HudCorner::BottomLeft,
                    HudCorner::BottomLeft => HudCorner::BottomRight,
                    HudCorner::BottomRight => HudCorner::Off,
                }
            }
            Self::ActionBar => c.action_bar = !c.action_bar,
            Self::Defaults => {
                c = Config {
                    deny_blocks: c.deny_blocks,
                    deny_entities: c.deny_entities,
                    ..Config::default()
                }
            }
        }
        Effect::Save(c)
    }
}

/// The settings page's labels (`native_settings`).
pub fn labels(loaded: &Loaded) -> Vec<String> {
    page(loaded).into_iter().map(|s| s.label(loaded)).collect()
}
/// A click on button `button` of the page `labels` gave (`native_settings_click`).
pub fn click(loaded: &Loaded, button: usize) -> Option<Effect> {
    page(loaded).get(button).map(|s| s.click(loaded))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn props(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect()
    }
    fn chest() -> BlockInfo {
        BlockInfo {
            name: "minecraft:chest".into(),
            properties: props(&[("facing", "north"), ("type", "single")]),
            block_entity: true,
            hardness: 2.5,
            item: Some("minecraft:chest".into()),
            empty: false,
            locked: false,
        }
    }
    fn dirt() -> BlockInfo {
        BlockInfo {
            name: "minecraft:dirt".into(),
            block_entity: false,
            hardness: 0.5,
            item: Some("minecraft:dirt".into()),
            ..BlockInfo::default()
        }
    }
    fn pig() -> MobInfo {
        MobInfo {
            id: "minecraft:pig".into(),
            width: 0.9,
            height: 0.9,
            alive: true,
            ..MobInfo::default()
        }
    }
    fn no_tags(_: &str) -> bool {
        false
    }

    #[test]
    fn default_config_is_valid_and_round_trips() {
        let c = Config::default();
        c.validate().unwrap();
        assert_eq!(Config::from_json(&c.to_json()).unwrap(), c);
        assert_eq!(Config::from_json("{}").unwrap(), c);
        assert!(c.deny_blocks.contains(&"minecraft:bedrock".to_owned()));
        assert!(c.deny_entities.contains(&"minecraft:player".to_owned()));
        assert!(MAX_ENTITY_SIZE_STEPS.contains(&c.max_entity_size));
    }

    #[test]
    fn broken_configs_are_refused() {
        assert!(Config::from_json("{\"nope\": 1}").is_err());
        assert!(Config::from_json("{\"maxEntitySize\": 0}").is_err());
        assert!(Config::from_json("{\"maxEntitySize\": 99}").is_err());
        assert!(Config::from_json("{\"slowness\": 5}").is_err());
        assert!(Config::from_json("{\"blocks\": \"everything\"}").is_err());
        assert!(Config::from_json("{\"denyBlocks\": [\"\"]}").is_err());
        assert!(Config::from_json("{\"denyBlocks\": [\"Minecraft:Stone\"]}").is_err());
        assert!(Config::from_json("{\"denyEntities\": [\"#minecraft:raiders\"]}").is_err());
        assert!(Config::from_json("{\"denyBlocks\": [\"#minecraft:beds\"]}").is_ok());
        assert!(Config::from_json("{\"denyBlocks\": [\"minecraft:*_shulker_box\"]}").is_ok());
        assert!(Config::from_json("not json").is_err());
    }

    #[test]
    fn globs_and_namespaces() {
        assert!(glob("minecraft:*_bed", "minecraft:red_bed"));
        assert!(glob("*", "anything"));
        assert!(glob("a*b*c", "aXbYc"));
        assert!(!glob("a*b*c", "aXbY"));
        assert!(!glob("minecraft:chest", "minecraft:trapped_chest"));
        assert_eq!(namespaced("chest"), "minecraft:chest");
        assert_eq!(namespaced("mod:thing"), "mod:thing");
        let list = vec!["spawner".to_owned(), "#minecraft:beds".to_owned()];
        assert!(denied(&list, "minecraft:spawner", &no_tags));
        assert!(denied(&list, "spawner", &no_tags));
        assert!(!denied(&list, "minecraft:red_bed", &no_tags));
        assert!(denied(&list, "minecraft:red_bed", &|t| t == "minecraft:beds"));
    }

    #[test]
    fn who_may_pick_up() {
        let c = Config::default();
        assert!(player_may_pick_up(&c, Mode::Survival, true, true));
        assert!(player_may_pick_up(&c, Mode::Creative, true, true));
        assert!(!player_may_pick_up(&c, Mode::Survival, false, true));
        assert!(!player_may_pick_up(&c, Mode::Survival, true, false));
        assert!(!player_may_pick_up(&c, Mode::Adventure, true, true));
        assert!(!player_may_pick_up(&c, Mode::Spectator, true, true));
        let off = Config {
            survival: false,
            ..Config::default()
        };
        assert!(!player_may_pick_up(&off, Mode::Survival, true, true));
        assert!(player_may_pick_up(&off, Mode::Creative, true, true));
        let disabled = Config {
            enabled: false,
            ..Config::default()
        };
        assert!(!player_may_pick_up(&disabled, Mode::Creative, true, true));
    }

    #[test]
    fn blocks_with_data_by_default() {
        let c = Config::default();
        assert_eq!(
            block_verdict(&c, Mode::Survival, &chest(), &no_tags),
            Verdict::Allow
        );
        assert_eq!(
            block_verdict(&c, Mode::Survival, &dirt(), &no_tags),
            Verdict::Pass
        );
        let any = Config {
            blocks: BlockRule::Any,
            ..Config::default()
        };
        assert_eq!(
            block_verdict(&any, Mode::Survival, &dirt(), &no_tags),
            Verdict::Allow
        );
        let none = Config {
            blocks: BlockRule::Off,
            ..Config::default()
        };
        assert_eq!(
            block_verdict(&none, Mode::Survival, &chest(), &no_tags),
            Verdict::Pass
        );
        let air = BlockInfo {
            empty: true,
            ..chest()
        };
        assert_eq!(
            block_verdict(&any, Mode::Survival, &air, &no_tags),
            Verdict::Pass
        );
    }

    #[test]
    fn denied_and_unsafe_blocks() {
        let c = Config::default();
        let portal = BlockInfo {
            name: "minecraft:end_portal".into(),
            block_entity: true,
            hardness: -1.0,
            item: None,
            ..BlockInfo::default()
        };
        assert_eq!(
            block_verdict(&c, Mode::Survival, &portal, &no_tags),
            Verdict::Refuse(Refusal::Denied)
        );
        let bypass = Config {
            creative_ignores_rules: true,
            ..Config::default()
        };
        // Even ignoring the lists, a block without an item stays put.
        assert_eq!(
            block_verdict(&bypass, Mode::Creative, &portal, &no_tags),
            Verdict::Refuse(Refusal::NoItem)
        );
        let bed = BlockInfo {
            name: "minecraft:red_bed".into(),
            properties: props(&[("part", "foot"), ("facing", "east")]),
            item: Some("minecraft:red_bed".into()),
            ..chest()
        };
        assert_eq!(
            block_verdict(&c, Mode::Creative, &bed, &no_tags),
            Verdict::Refuse(Refusal::TwoBlocks)
        );
        let any = Config {
            blocks: BlockRule::Any,
            ..Config::default()
        };
        let door = BlockInfo {
            name: "minecraft:oak_door".into(),
            properties: props(&[("half", "lower")]),
            ..dirt()
        };
        assert_eq!(
            block_verdict(&any, Mode::Survival, &door, &no_tags),
            Verdict::Refuse(Refusal::TwoBlocks)
        );
        let stairs = BlockInfo {
            name: "minecraft:oak_stairs".into(),
            properties: props(&[("half", "top")]),
            ..dirt()
        };
        assert_eq!(
            block_verdict(&any, Mode::Survival, &stairs, &no_tags),
            Verdict::Allow
        );
        let unbreakable = BlockInfo {
            name: "mod:vault".into(),
            hardness: -1.0,
            ..chest()
        };
        assert_eq!(
            block_verdict(&c, Mode::Survival, &unbreakable, &no_tags),
            Verdict::Refuse(Refusal::Unbreakable)
        );
        assert_eq!(
            block_verdict(&c, Mode::Creative, &unbreakable, &no_tags),
            Verdict::Allow
        );
        let locked = BlockInfo {
            locked: true,
            ..chest()
        };
        assert_eq!(
            block_verdict(&c, Mode::Creative, &locked, &no_tags),
            Verdict::Refuse(Refusal::Locked)
        );
        let bedrock = BlockInfo {
            name: "minecraft:bedrock".into(),
            hardness: -1.0,
            item: Some("minecraft:bedrock".into()),
            ..dirt()
        };
        assert_eq!(
            block_verdict(&any, Mode::Creative, &bedrock, &no_tags),
            Verdict::Refuse(Refusal::Denied)
        );
        let any_bypass = Config {
            creative_ignores_rules: true,
            ..any
        };
        assert_eq!(
            block_verdict(&any_bypass, Mode::Creative, &bedrock, &no_tags),
            Verdict::Allow
        );
        assert_eq!(
            block_verdict(&any_bypass, Mode::Survival, &bedrock, &no_tags),
            Verdict::Refuse(Refusal::Denied)
        );
    }

    #[test]
    fn mob_rules() {
        let c = Config::default();
        assert_eq!(mob_verdict(&c, Mode::Survival, &pig()), Verdict::Allow);
        let zombie = MobInfo {
            id: "minecraft:zombie".into(),
            width: 0.6,
            height: 1.95,
            hostile: true,
            ..pig()
        };
        assert_eq!(
            mob_verdict(&c, Mode::Survival, &zombie),
            Verdict::Refuse(Refusal::Hostile)
        );
        let any = Config {
            mobs: MobRule::Any,
            max_entity_size: 2.0,
            ..Config::default()
        };
        assert_eq!(mob_verdict(&any, Mode::Survival, &zombie), Verdict::Allow);
        let cow = MobInfo {
            id: "minecraft:cow".into(),
            width: 0.9,
            height: 1.4,
            ..pig()
        };
        assert_eq!(mob_verdict(&c, Mode::Survival, &cow), Verdict::Allow);
        let villager = MobInfo {
            id: "minecraft:villager".into(),
            width: 0.6,
            height: 1.95,
            ..pig()
        };
        assert_eq!(
            mob_verdict(&c, Mode::Survival, &villager),
            Verdict::Refuse(Refusal::TooBig)
        );
        let wither = MobInfo {
            id: "minecraft:wither".into(),
            hostile: true,
            ..pig()
        };
        assert_eq!(
            mob_verdict(&any, Mode::Creative, &wither),
            Verdict::Refuse(Refusal::Denied)
        );
        let bypass = Config {
            creative_ignores_rules: true,
            ..Config::default()
        };
        assert_eq!(
            mob_verdict(&bypass, Mode::Creative, &villager),
            Verdict::Allow
        );
        assert_eq!(
            mob_verdict(&bypass, Mode::Survival, &villager),
            Verdict::Refuse(Refusal::TooBig)
        );
        for (mob, refusal) in [
            (
                MobInfo {
                    riding: true,
                    ..pig()
                },
                Refusal::Riding,
            ),
            (
                MobInfo {
                    ridden: true,
                    ..pig()
                },
                Refusal::Riding,
            ),
            (
                MobInfo {
                    leashed: true,
                    ..pig()
                },
                Refusal::Leashed,
            ),
            (
                MobInfo {
                    inventory: true,
                    ..pig()
                },
                Refusal::Inventory,
            ),
            (
                MobInfo {
                    owned_by_other: true,
                    ..pig()
                },
                Refusal::NotYours,
            ),
        ] {
            // These hold in creative too, whatever the settings.
            assert_eq!(
                mob_verdict(&bypass, Mode::Creative, &mob),
                Verdict::Refuse(refusal)
            );
        }
        let gone = MobInfo {
            removed: true,
            ..pig()
        };
        assert_eq!(mob_verdict(&c, Mode::Survival, &gone), Verdict::Pass);
        let dead = MobInfo {
            alive: false,
            ..pig()
        };
        assert_eq!(mob_verdict(&c, Mode::Survival, &dead), Verdict::Pass);
        let off = Config {
            mobs: MobRule::Off,
            ..Config::default()
        };
        assert_eq!(mob_verdict(&off, Mode::Survival, &pig()), Verdict::Pass);
    }

    #[test]
    fn states_parse_and_print() {
        let s = State::parse("minecraft:chest[facing=north,type=left,waterlogged=false]").unwrap();
        assert_eq!(s.name, "minecraft:chest");
        assert_eq!(s.properties.get("type").map(String::as_str), Some("left"));
        assert_eq!(
            s.text(),
            "minecraft:chest[facing=north,type=left,waterlogged=false]"
        );
        assert_eq!(State::parse("stone").unwrap().text(), "minecraft:stone");
        assert!(State::parse("").is_none());
        assert!(State::parse("minecraft:chest[facing]").is_none());
        assert!(State::parse("minecraft:chest[facing=north").is_none());
        assert!(State::parse("minecraft:chest[=north]").is_none());
    }

    #[test]
    fn put_down_state_keeps_contents_properties() {
        let carried = State::parse("minecraft:furnace[facing=north,lit=true]").unwrap();
        let placed = State::parse("minecraft:furnace[facing=east,lit=false]").unwrap();
        assert_eq!(
            placed_state(&carried, &placed).text(),
            "minecraft:furnace[facing=east,lit=true]"
        );
        let hive = State::parse("minecraft:beehive[facing=north,honey_level=5]").unwrap();
        let placed = State::parse("minecraft:beehive[facing=south,honey_level=0]").unwrap();
        assert_eq!(
            placed_state(&hive, &placed).text(),
            "minecraft:beehive[facing=south,honey_level=5]"
        );
        let chest =
            State::parse("minecraft:chest[facing=north,type=left,waterlogged=true]").unwrap();
        let placed =
            State::parse("minecraft:chest[facing=west,type=single,waterlogged=false]").unwrap();
        assert_eq!(placed_state(&chest, &placed), placed);
        // A wall sign put down on top of a block becomes a standing sign.
        let wall = State::parse("minecraft:oak_wall_sign[facing=north,waterlogged=false]").unwrap();
        let standing = State::parse("minecraft:oak_sign[rotation=4,waterlogged=false]").unwrap();
        assert_eq!(placed_state(&wall, &standing), standing);
        let double = State::parse("minecraft:stone_slab[type=double,waterlogged=false]").unwrap();
        let single = State::parse("minecraft:stone_slab[type=bottom,waterlogged=false]").unwrap();
        assert_eq!(
            placed_state(&double, &single).text(),
            "minecraft:stone_slab[type=double,waterlogged=false]"
        );
    }

    #[test]
    fn released_state_is_alone_and_closed() {
        let chest =
            State::parse("minecraft:chest[facing=north,type=right,waterlogged=false]").unwrap();
        assert_eq!(
            released_state(&chest, true).text(),
            "minecraft:chest[facing=north,type=single,waterlogged=true]"
        );
        let barrel = State::parse("minecraft:barrel[facing=up,open=true]").unwrap();
        assert_eq!(
            released_state(&barrel, true).text(),
            "minecraft:barrel[facing=up,open=false]"
        );
    }

    #[test]
    fn names_and_texts() {
        assert_eq!(display_name("minecraft:oak_wall_sign"), "Oak Wall Sign");
        assert_eq!(display_name("chest"), "Chest");
        assert_eq!(display_name("mod:big/thing_x"), "Big Thing X");
        assert_eq!(summary("Chest", Some(5)), "Chest (5 stacks)");
        assert_eq!(summary("Chest", Some(1)), "Chest (1 stack)");
        assert_eq!(summary("Chest", Some(0)), "Chest");
        assert_eq!(summary("Pig", None), "Pig");
        assert_eq!(hud_lines("Pig")[0], "Carrying: Pig");
        assert!(action_bar_text("Pig").starts_with("Carrying Pig"));
        assert!(status_text(Some("Pig")).ends_with("carrying Pig"));
        assert!(status_text(None).contains("not carrying"));
        for r in [Refusal::Denied, Refusal::TooBig, Refusal::Locked] {
            assert!(!r.message().is_empty());
        }
    }

    #[test]
    fn release_spots_start_at_the_feet() {
        let spots = release_spots(RELEASE_RADIUS);
        assert_eq!(spots[0], [0, 0, 0]);
        assert_eq!(spots.len(), 5 * 5 * 4);
        let mut unique = spots.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), spots.len());
        assert!(spots.iter().all(|s| s[1] >= -1));
        // Above the feet before below them, at the same distance.
        let up = spots.iter().position(|s| *s == [0, 1, 0]).unwrap();
        let down = spots.iter().position(|s| *s == [0, -1, 0]).unwrap();
        assert!(up < down);
    }

    #[test]
    fn held_use_button_is_not_a_click() {
        let mut clicks = Clicks::default();
        let me = [1, 2, 3, 4];
        assert!(clicks.fresh(me, 100));
        assert!(!clicks.fresh(me, 100)); // the second packet of the same click
        assert!(!clicks.fresh(me, 104)); // held: repeats every 4 ticks
        assert!(!clicks.fresh(me, 108));
        assert!(clicks.fresh(me, 120));
        assert!(clicks.fresh([9, 9, 9, 9], 121));
        assert!(clicks.fresh(me, 50)); // a new world counts again
        for i in 0..300 {
            clicks.fresh([i, 0, 0, 0], 1000);
        }
        assert!(clicks.last.len() <= 301);
        clicks.fresh([0, 0, 0, 1], 2000);
        assert!(clicks.last.len() < 10);
    }

    #[test]
    fn slowness_rules() {
        let c = Config::default();
        assert_eq!(slowness_amplifier(&c), Some(0));
        let none = Config {
            slowness: 0,
            ..Config::default()
        };
        assert_eq!(slowness_amplifier(&none), None);
        assert!(slowness_wanted(None, 0));
        assert!(slowness_wanted(Some((0, 5)), 0));
        assert!(!slowness_wanted(Some((0, 30)), 0));
        assert!(!slowness_wanted(Some((0, -1)), 0));
        assert!(!slowness_wanted(Some((2, 5)), 0));
        assert!(slowness_wanted(Some((0, 600)), 1));
        assert!(is_carry_slowness(30, true, false, false));
        assert!(!is_carry_slowness(30, false, true, false));
        assert!(!is_carry_slowness(600, true, false, false));
        assert!(!is_carry_slowness(30, true, false, true));
        assert!(!is_carry_slowness(-1, true, false, false));
    }

    #[test]
    fn commands() {
        assert_eq!(parse_command("carryblock"), Some(Ok(Command::Status)));
        assert_eq!(
            parse_command("/carryblock status"),
            Some(Ok(Command::Status))
        );
        assert_eq!(
            parse_command("carryblock reload"),
            Some(Ok(Command::Reload))
        );
        assert_eq!(parse_command("carryblock drop"), Some(Err(USAGE)));
        assert_eq!(parse_command("carryblock reload now"), Some(Err(USAGE)));
        assert_eq!(parse_command("carryblockx"), None);
        assert_eq!(parse_command("give @s stone"), None);
    }

    #[test]
    fn settings_page() {
        let loaded: Loaded = Ok(Config::default());
        let labels = labels(&loaded);
        assert_eq!(labels.len(), page(&loaded).len());
        assert_eq!(labels[0], "Picking up: ON");
        assert!(labels.iter().all(|l| !l.is_empty()));
        assert_eq!(click(&loaded, 99), None);
        let Some(Effect::Save(c)) = click(&loaded, 0) else {
            panic!("a save");
        };
        assert!(!c.enabled);
        // Each cycling button comes back to its default.
        for (button, steps) in [(4, 3), (5, 3), (6, 7), (7, 5), (8, 5)] {
            let mut now: Loaded = Ok(Config::default());
            for _ in 0..steps {
                let Some(Effect::Save(c)) = click(&now, button) else {
                    panic!("a save");
                };
                c.validate().unwrap();
                now = Ok(c);
            }
            assert_eq!(now, Ok(Config::default()), "button {button}");
        }
        let custom = Config {
            deny_blocks: vec!["minecraft:spawner".into()],
            slowness: 3,
            ..Config::default()
        };
        let lists = page(&Ok(custom.clone()))
            .iter()
            .position(|s| *s == Setting::Lists)
            .unwrap();
        assert_eq!(click(&Ok(custom.clone()), lists), Some(Effect::Reload));
        let defaults = page(&Ok(custom.clone())).len() - 1;
        let Some(Effect::Save(reset)) = click(&Ok(custom.clone()), defaults) else {
            panic!("a save");
        };
        assert_eq!(reset.slowness, 1);
        assert_eq!(reset.deny_blocks, custom.deny_blocks);
    }

    #[test]
    fn broken_file_page() {
        let loaded: Loaded = Err("expected value at line 1 column 1 and some more words".into());
        let labels = labels(&loaded);
        assert_eq!(labels.len(), 2);
        assert!(labels[0].contains("click to reload"));
        assert!(labels[0].contains("..."));
        assert_eq!(click(&loaded, 0), Some(Effect::Reload));
        assert_eq!(click(&loaded, 1), Some(Effect::Save(Config::default())));
    }
}
