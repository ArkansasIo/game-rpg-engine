use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::grid2d::{Coord2, Grid2D};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Story,
    Normal,
    Hard,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GameSettings {
    pub difficulty: Difficulty,
    pub encounter_scale_percent: u8,
    pub permadeath: bool,
    pub auto_save_every_turns: u32,
    pub allow_run_from_boss: bool,
    pub day_night_cycle: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            difficulty: Difficulty::Normal,
            encounter_scale_percent: 100,
            permadeath: false,
            auto_save_every_turns: 10,
            allow_run_from_boss: false,
            day_night_cycle: true,
        }
    }
}

impl GameSettings {
    pub fn effective_encounter_rate(&self, base_rate_percent: u8) -> u8 {
        let scaled = u16::from(base_rate_percent) * u16::from(self.encounter_scale_percent) / 100;
        scaled.min(100) as u8
    }

    pub fn physical_damage_with_difficulty(&self, base: i32) -> i32 {
        let adjusted = match self.difficulty {
            Difficulty::Story => base.saturating_mul(80) / 100,
            Difficulty::Normal => base,
            Difficulty::Hard => base.saturating_mul(125) / 100,
        };
        adjusted.max(1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction2D {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Terrain {
    Grass,
    Forest,
    Road,
    Wall,
    Water,
    DoorClosed,
    DoorOpen,
    ChestClosed,
    ChestOpened,
    Trap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClimateBand {
    Temperate,
    Tropical,
    Arid,
    Polar,
    Alpine,
    Coastal,
    Subterranean,
    Volcanic,
    Arcane,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZoneEnvironment {
    Surface,
    Underground,
    Underwater,
    Interior,
    SpiritRealm,
    Infernal,
    Celestial,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZoneClass {
    Overworld,
    Settlement,
    Wilderness,
    Dungeon,
    Landmark,
    StoryInstance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZoneSubtype {
    Plains,
    Forest,
    Desert,
    Swamp,
    Mountain,
    Coast,
    Island,
    Riverlands,
    Tundra,
    Volcano,
    Ruins,
    Cave,
    Castle,
    Crypt,
    Village,
    Town,
    City,
    Harbor,
    Temple,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BiomeType {
    Grassland,
    Woodland,
    Jungle,
    Desert,
    Wetland,
    Taiga,
    Tundra,
    Alpine,
    Canyon,
    Coastal,
    Oceanic,
    River,
    Cavern,
    Volcanic,
    Ruined,
    Arcane,
    Haunted,
    Urban,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BiomeSubtype {
    OpenField,
    DenseForest,
    AncientForest,
    Oasis,
    Mangrove,
    Bog,
    Boreal,
    FrozenWastes,
    Highland,
    Lowland,
    Cliffside,
    CoralReef,
    DeepSea,
    LavaTube,
    CrystalCave,
    Necrotic,
    ArcaneBloom,
    OuterDistrict,
    InnerDistrict,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RectArea {
    pub min: Coord2,
    pub max: Coord2,
}

impl RectArea {
    pub fn contains(&self, c: Coord2) -> bool {
        c.x >= self.min.x && c.y >= self.min.y && c.x <= self.max.x && c.y <= self.max.y
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BiomeProfile {
    pub biome_type: BiomeType,
    pub biome_subtype: BiomeSubtype,
    pub climate: ClimateBand,
    pub danger_level: u8,
    pub encounter_bonus_percent: i8,
    pub move_cost: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SubZone {
    pub id: String,
    pub name: String,
    pub class: ZoneClass,
    pub subtype: ZoneSubtype,
    pub environment: ZoneEnvironment,
    pub area: RectArea,
    pub biome: BiomeProfile,
    pub encounter_multiplier_percent: u8,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Zone {
    pub id: String,
    pub name: String,
    pub class: ZoneClass,
    pub subtype: ZoneSubtype,
    pub environment: ZoneEnvironment,
    pub area: RectArea,
    pub default_biome: BiomeProfile,
    pub encounter_multiplier_percent: u8,
    pub level_min: u8,
    pub level_max: u8,
    pub sub_zones: Vec<SubZone>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChallengeTier {
    Novice,
    Veteran,
    Elite,
    Mythic,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Continent {
    pub id: String,
    pub name: String,
    pub climate: ClimateBand,
    pub country_ids: Vec<String>,
    pub zone_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Country {
    pub id: String,
    pub name: String,
    pub continent_id: String,
    pub capital: String,
    pub zone_ids: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DungeonFloor {
    pub index: u8,
    pub biome: BiomeProfile,
    pub encounter_multiplier_percent: u8,
    pub has_checkpoint: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DungeonBlueprint {
    pub id: String,
    pub name: String,
    pub zone_id: String,
    pub recommended_level_min: u8,
    pub recommended_level_max: u8,
    pub tier: ChallengeTier,
    pub floors: Vec<DungeonFloor>,
    pub boss_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RaidWing {
    pub id: String,
    pub name: String,
    pub encounter_count: u8,
    pub boss_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RaidBlueprint {
    pub id: String,
    pub name: String,
    pub zone_id: String,
    pub tier: ChallengeTier,
    pub recommended_party_size: u8,
    pub wings: Vec<RaidWing>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TrialObjective {
    KillBoss(String),
    SurviveWaves(u8),
    ActivateAltars(u8),
    EscortNpc(String),
    CollectRelic(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrialBlueprint {
    pub id: String,
    pub name: String,
    pub zone_id: String,
    pub tier: ChallengeTier,
    pub time_limit_seconds: Option<u32>,
    pub objectives: Vec<TrialObjective>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TowerStage {
    pub stage: u32,
    pub enemy_pack_id: String,
    pub reward_table_id: String,
    pub modifier_tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TowerBlueprint {
    pub id: String,
    pub name: String,
    pub zone_id: String,
    pub tier: ChallengeTier,
    pub stages: Vec<TowerStage>,
    pub infinite_after_stage: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WorldCreationError {
    DuplicateId(String),
    MissingReference(String),
    InvalidValue(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct WorldAtlas {
    pub continents: HashMap<String, Continent>,
    pub countries: HashMap<String, Country>,
    pub zones: HashMap<String, Zone>,
    pub dungeons: HashMap<String, DungeonBlueprint>,
    pub raids: HashMap<String, RaidBlueprint>,
    pub trials: HashMap<String, TrialBlueprint>,
    pub towers: HashMap<String, TowerBlueprint>,
}

impl WorldAtlas {
    pub fn add_continent(&mut self, continent: Continent) -> Result<(), WorldCreationError> {
        if self.continents.contains_key(&continent.id) {
            return Err(WorldCreationError::DuplicateId(continent.id));
        }
        self.continents.insert(continent.id.clone(), continent);
        Ok(())
    }

    pub fn add_country(&mut self, country: Country) -> Result<(), WorldCreationError> {
        if self.countries.contains_key(&country.id) {
            return Err(WorldCreationError::DuplicateId(country.id));
        }
        let Some(continent) = self.continents.get_mut(&country.continent_id) else {
            return Err(WorldCreationError::MissingReference(format!(
                "continent:{}",
                country.continent_id
            )));
        };
        continent.country_ids.push(country.id.clone());
        self.countries.insert(country.id.clone(), country);
        Ok(())
    }

    pub fn add_zone(
        &mut self,
        zone: Zone,
        continent_id: &str,
        country_id: &str,
    ) -> Result<(), WorldCreationError> {
        if self.zones.contains_key(&zone.id) {
            return Err(WorldCreationError::DuplicateId(zone.id));
        }
        let Some(continent) = self.continents.get_mut(continent_id) else {
            return Err(WorldCreationError::MissingReference(format!(
                "continent:{continent_id}"
            )));
        };
        let Some(country) = self.countries.get_mut(country_id) else {
            return Err(WorldCreationError::MissingReference(format!(
                "country:{country_id}"
            )));
        };
        continent.zone_ids.push(zone.id.clone());
        country.zone_ids.push(zone.id.clone());
        self.zones.insert(zone.id.clone(), zone);
        Ok(())
    }

    pub fn add_sub_zone(
        &mut self,
        zone_id: &str,
        sub_zone: SubZone,
    ) -> Result<(), WorldCreationError> {
        let Some(zone) = self.zones.get_mut(zone_id) else {
            return Err(WorldCreationError::MissingReference(format!(
                "zone:{zone_id}"
            )));
        };
        if zone.sub_zones.iter().any(|z| z.id == sub_zone.id) {
            return Err(WorldCreationError::DuplicateId(sub_zone.id));
        }
        zone.sub_zones.push(sub_zone);
        Ok(())
    }

    pub fn add_dungeon(
        &mut self,
        dungeon: DungeonBlueprint,
    ) -> Result<(), WorldCreationError> {
        if self.dungeons.contains_key(&dungeon.id) {
            return Err(WorldCreationError::DuplicateId(dungeon.id));
        }
        if !self.zones.contains_key(&dungeon.zone_id) {
            return Err(WorldCreationError::MissingReference(format!(
                "zone:{}",
                dungeon.zone_id
            )));
        }
        if dungeon.floors.is_empty() {
            return Err(WorldCreationError::InvalidValue(
                "dungeon requires at least one floor".to_string(),
            ));
        }
        self.dungeons.insert(dungeon.id.clone(), dungeon);
        Ok(())
    }

    pub fn add_raid(&mut self, raid: RaidBlueprint) -> Result<(), WorldCreationError> {
        if self.raids.contains_key(&raid.id) {
            return Err(WorldCreationError::DuplicateId(raid.id));
        }
        if !self.zones.contains_key(&raid.zone_id) {
            return Err(WorldCreationError::MissingReference(format!(
                "zone:{}",
                raid.zone_id
            )));
        }
        if raid.wings.is_empty() {
            return Err(WorldCreationError::InvalidValue(
                "raid requires at least one wing".to_string(),
            ));
        }
        self.raids.insert(raid.id.clone(), raid);
        Ok(())
    }

    pub fn add_trial(&mut self, trial: TrialBlueprint) -> Result<(), WorldCreationError> {
        if self.trials.contains_key(&trial.id) {
            return Err(WorldCreationError::DuplicateId(trial.id));
        }
        if !self.zones.contains_key(&trial.zone_id) {
            return Err(WorldCreationError::MissingReference(format!(
                "zone:{}",
                trial.zone_id
            )));
        }
        if trial.objectives.is_empty() {
            return Err(WorldCreationError::InvalidValue(
                "trial requires at least one objective".to_string(),
            ));
        }
        self.trials.insert(trial.id.clone(), trial);
        Ok(())
    }

    pub fn add_tower(&mut self, tower: TowerBlueprint) -> Result<(), WorldCreationError> {
        if self.towers.contains_key(&tower.id) {
            return Err(WorldCreationError::DuplicateId(tower.id));
        }
        if !self.zones.contains_key(&tower.zone_id) {
            return Err(WorldCreationError::MissingReference(format!(
                "zone:{}",
                tower.zone_id
            )));
        }
        if tower.stages.is_empty() {
            return Err(WorldCreationError::InvalidValue(
                "tower requires at least one stage".to_string(),
            ));
        }
        self.towers.insert(tower.id.clone(), tower);
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TileDef {
    pub terrain: Terrain,
    pub encounter_rate_percent: u8,
}

impl TileDef {
    pub fn new(terrain: Terrain, encounter_rate_percent: u8) -> Self {
        Self {
            terrain,
            encounter_rate_percent: encounter_rate_percent.min(100),
        }
    }

    pub fn is_blocking(&self) -> bool {
        matches!(
            self.terrain,
            Terrain::Wall | Terrain::Water | Terrain::DoorClosed
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ScriptAction {
    SetQuestFlag { key: String, value: bool },
    OpenDoor { at: Coord2 },
    OpenChest { at: Coord2, item_id: String },
    Message { text: String },
    Composite(Vec<ScriptAction>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Trigger {
    pub id: String,
    pub once: bool,
    pub action: ScriptAction,
    pub fired: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WorldEvent {
    Moved { to: Coord2 },
    Blocked { at: Coord2 },
    Triggered { id: String },
    DoorOpened { at: Coord2 },
    ChestLoot { item_id: String },
    Message { text: String },
    QuestUpdated { key: String, value: bool },
    TimeAdvanced { day: u32, minute_of_day: u16 },
    EnvironmentTick {
        fire_level: u8,
        flood_level: u8,
        poison_level: u8,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct EnvironmentState {
    pub fire_level: u8,
    pub flood_level: u8,
    pub poison_level: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct World2D {
    pub map: Grid2D<TileDef>,
    pub triggers: HashMap<Coord2, Trigger>,
    pub quest_flags: HashMap<String, bool>,
    pub turn: u64,
    #[serde(default)]
    pub day: u32,
    #[serde(default)]
    pub minute_of_day: u16,
    #[serde(default)]
    pub environment: EnvironmentState,
}

impl World2D {
    pub fn new(map: Grid2D<TileDef>) -> Self {
        Self {
            map,
            triggers: HashMap::new(),
            quest_flags: HashMap::new(),
            turn: 0,
            day: 1,
            minute_of_day: 8 * 60,
            environment: EnvironmentState::default(),
        }
    }

    pub fn set_tile(&mut self, at: Coord2, tile: TileDef) -> bool {
        self.map.set(at.x, at.y, tile)
    }

    pub fn tile(&self, at: Coord2) -> Option<&TileDef> {
        self.map.get(at.x, at.y)
    }

    pub fn add_trigger(&mut self, at: Coord2, trigger: Trigger) {
        self.triggers.insert(at, trigger);
    }

    pub fn try_move(&mut self, from: Coord2, dir: Direction2D) -> (Coord2, Vec<WorldEvent>) {
        self.turn = self.turn.saturating_add(1);
        let Some(to) = shifted_coord(from, dir, self.map.width(), self.map.height()) else {
            return (from, vec![WorldEvent::Blocked { at: from }]);
        };

        let Some(tile) = self.map.get(to.x, to.y) else {
            return (from, vec![WorldEvent::Blocked { at: to }]);
        };
        if tile.is_blocking() {
            return (from, vec![WorldEvent::Blocked { at: to }]);
        }

        let mut events = vec![WorldEvent::Moved { to }];
        events.extend(self.interact(to));
        (to, events)
    }

    pub fn interact(&mut self, at: Coord2) -> Vec<WorldEvent> {
        let mut events = Vec::new();
        if let Some(trigger) = self.triggers.get_mut(&at) {
            if trigger.once && trigger.fired {
                return events;
            }
            trigger.fired = true;
            events.push(WorldEvent::Triggered {
                id: trigger.id.clone(),
            });
            let action = trigger.action.clone();
            self.apply_action(&action, &mut events);
        }
        events
    }

    pub fn should_start_encounter(&self, at: Coord2, roll_percent: u8) -> bool {
        let Some(tile) = self.tile(at) else {
            return false;
        };
        roll_percent < tile.encounter_rate_percent
    }

    pub fn advance_time(&mut self, minutes: u16) -> WorldEvent {
        let total = u32::from(self.minute_of_day) + u32::from(minutes);
        self.day = self.day.saturating_add(total / (24 * 60));
        self.minute_of_day = (total % (24 * 60)) as u16;
        WorldEvent::TimeAdvanced {
            day: self.day,
            minute_of_day: self.minute_of_day,
        }
    }

    pub fn tick_environment(&mut self) -> WorldEvent {
        self.environment.fire_level = self.environment.fire_level.saturating_sub(1);
        self.environment.flood_level = self.environment.flood_level.saturating_sub(1);
        self.environment.poison_level = self.environment.poison_level.saturating_sub(1);
        WorldEvent::EnvironmentTick {
            fire_level: self.environment.fire_level,
            flood_level: self.environment.flood_level,
            poison_level: self.environment.poison_level,
        }
    }

    fn apply_action(&mut self, action: &ScriptAction, events: &mut Vec<WorldEvent>) {
        match action {
            ScriptAction::SetQuestFlag { key, value } => {
                self.quest_flags.insert(key.clone(), *value);
                events.push(WorldEvent::QuestUpdated {
                    key: key.clone(),
                    value: *value,
                });
            }
            ScriptAction::OpenDoor { at } => {
                if let Some(tile) = self.tile(*at).cloned()
                    && tile.terrain == Terrain::DoorClosed
                {
                    let _ = self.set_tile(*at, TileDef::new(Terrain::DoorOpen, 0));
                    events.push(WorldEvent::DoorOpened { at: *at });
                }
            }
            ScriptAction::OpenChest { at, item_id } => {
                if let Some(tile) = self.tile(*at).cloned()
                    && tile.terrain == Terrain::ChestClosed
                {
                    let _ = self.set_tile(*at, TileDef::new(Terrain::ChestOpened, 0));
                    events.push(WorldEvent::ChestLoot {
                        item_id: item_id.clone(),
                    });
                }
            }
            ScriptAction::Message { text } => {
                events.push(WorldEvent::Message { text: text.clone() });
            }
            ScriptAction::Composite(actions) => {
                for a in actions {
                    self.apply_action(a, events);
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    pub hp: i32,
    pub max_hp: i32,
    pub mp: i32,
    pub max_mp: i32,
    pub str_: i32,
    pub agi: i32,
    pub vit: i32,
    pub int_: i32,
    pub luk: i32,
}

impl Stats {
    pub fn clamp_resources(&mut self) {
        self.hp = self.hp.clamp(0, self.max_hp.max(0));
        self.mp = self.mp.clamp(0, self.max_mp.max(0));
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum StatusEffect {
    Poison { damage: i32, turns_left: u8 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fighter {
    pub id: String,
    pub name: String,
    pub stats: Stats,
    pub statuses: Vec<StatusEffect>,
}

impl Fighter {
    pub fn is_alive(&self) -> bool {
        self.stats.is_alive()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Party {
    pub members: Vec<Fighter>,
    pub inventory: HashMap<String, u32>,
    pub gold: u32,
}

impl Party {
    pub fn first_alive_index(&self) -> Option<usize> {
        self.members.iter().position(Fighter::is_alive)
    }

    pub fn add_item(&mut self, item_id: &str, amount: u32) {
        let e = self.inventory.entry(item_id.to_string()).or_insert(0);
        *e = e.saturating_add(amount);
    }

    pub fn consume_item(&mut self, item_id: &str) -> bool {
        let Some(v) = self.inventory.get_mut(item_id) else {
            return false;
        };
        if *v == 0 {
            return false;
        }
        *v -= 1;
        true
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EncounterPack {
    pub id: String,
    pub terrain: Terrain,
    pub weight: u32,
    pub monsters: Vec<Fighter>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EncounterTable {
    pub packs: Vec<EncounterPack>,
}

impl EncounterTable {
    pub fn pick_for_terrain(&self, terrain: Terrain, roll: u32) -> Option<&EncounterPack> {
        let mut total = 0u32;
        for p in self.packs.iter().filter(|p| p.terrain == terrain) {
            total = total.saturating_add(p.weight);
        }
        if total == 0 {
            return None;
        }
        let mut cursor = roll % total;
        for p in self.packs.iter().filter(|p| p.terrain == terrain) {
            if cursor < p.weight {
                return Some(p);
            }
            cursor -= p.weight;
        }
        None
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SpellKind {
    Damage,
    Heal,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpellDef {
    pub id: String,
    pub mp_cost: i32,
    pub power: i32,
    pub kind: SpellKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ItemDef {
    pub id: String,
    pub heal_hp: i32,
    pub heal_mp: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemCategory {
    Consumable,
    Weapon,
    Armor,
    Accessory,
    Quest,
    Material,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponType {
    Sword,
    Axe,
    Spear,
    Dagger,
    Bow,
    Staff,
    Wand,
    Mace,
    Unarmed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArmorSlot {
    Head,
    Body,
    Legs,
    Feet,
    Hands,
    Shield,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArmorType {
    Cloth,
    Leather,
    Mail,
    Plate,
    Robe,
    Buckler,
    TowerShield,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CharacterRace {
    Human,
    Elf,
    Dwarf,
    Halfling,
    Orc,
    Draconian,
    Undead,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CharacterClassType {
    Warrior,
    Knight,
    Ranger,
    Rogue,
    Cleric,
    Mage,
    Battlemage,
    Necromancer,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct StatModifier {
    pub hp: i32,
    pub mp: i32,
    pub str_: i32,
    pub agi: i32,
    pub vit: i32,
    pub int_: i32,
    pub luk: i32,
}

impl StatModifier {
    pub fn points_spent(&self) -> i32 {
        self.hp + self.mp + self.str_ + self.agi + self.vit + self.int_ + self.luk
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeaponDef {
    pub id: String,
    pub name: String,
    pub weapon_type: WeaponType,
    pub power: i32,
    pub hit_bonus: i32,
    pub crit_bonus: i32,
    pub rarity: ItemRarity,
    pub stat_bonus: StatModifier,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArmorDef {
    pub id: String,
    pub name: String,
    pub armor_type: ArmorType,
    pub slot: ArmorSlot,
    pub defense: i32,
    pub magic_resist: i32,
    pub rarity: ItemRarity,
    pub stat_bonus: StatModifier,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ItemType {
    Consumable(ItemDef),
    Weapon(WeaponDef),
    Armor(ArmorDef),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InventoryEntry {
    pub id: String,
    pub category: ItemCategory,
    pub item: ItemType,
    pub quantity: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct EquipmentLoadout {
    pub weapon: Option<WeaponDef>,
    pub head: Option<ArmorDef>,
    pub body: Option<ArmorDef>,
    pub legs: Option<ArmorDef>,
    pub feet: Option<ArmorDef>,
    pub hands: Option<ArmorDef>,
    pub shield: Option<ArmorDef>,
}

impl EquipmentLoadout {
    pub fn total_stat_bonus(&self) -> StatModifier {
        let mut out = StatModifier::default();
        if let Some(w) = &self.weapon {
            add_modifier(&mut out, &w.stat_bonus);
        }
        for armor in [
            &self.head,
            &self.body,
            &self.legs,
            &self.feet,
            &self.hands,
            &self.shield,
        ]
        .into_iter()
        .flatten()
        {
            add_modifier(&mut out, &armor.stat_bonus);
        }
        out
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CharacterCreationRequest {
    pub id: String,
    pub name: String,
    pub race: CharacterRace,
    pub class_type: CharacterClassType,
    pub bonus_points: i32,
    pub allocation: StatModifier,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CharacterCreationError {
    InvalidName,
    InvalidPointAllocation,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CharacterProfile {
    pub id: String,
    pub name: String,
    pub race: CharacterRace,
    pub class_type: CharacterClassType,
    pub level: u32,
    pub exp: u64,
    pub base_stats: Stats,
    pub derived_stats: Stats,
    pub equipment: EquipmentLoadout,
    pub inventory: Vec<InventoryEntry>,
    pub passives: Vec<String>,
}

impl CharacterProfile {
    pub fn as_fighter(&self) -> Fighter {
        Fighter {
            id: self.id.clone(),
            name: self.name.clone(),
            stats: self.derived_stats.clone(),
            statuses: vec![],
        }
    }
}

pub fn create_character(
    request: CharacterCreationRequest,
) -> Result<CharacterProfile, CharacterCreationError> {
    if request.name.trim().is_empty() {
        return Err(CharacterCreationError::InvalidName);
    }
    if request.allocation.points_spent() != request.bonus_points {
        return Err(CharacterCreationError::InvalidPointAllocation);
    }

    let race_bonus = race_stat_bonus(request.race);
    let class_base = class_base_stats(request.class_type);

    let mut stats = class_base.clone();
    apply_stat_modifier(&mut stats, &race_bonus);
    apply_stat_modifier(&mut stats, &request.allocation);
    stats.str_ = stats.str_.max(1);
    stats.agi = stats.agi.max(1);
    stats.vit = stats.vit.max(1);
    stats.int_ = stats.int_.max(1);
    stats.luk = stats.luk.max(1);
    stats.max_hp = (20 + stats.vit * 3 + stats.str_).max(1);
    stats.hp = stats.max_hp;
    stats.max_mp = (8 + stats.int_ * 3).max(0);
    stats.mp = stats.max_mp;

    let equipment = class_starting_equipment(request.class_type);
    let mut derived_stats = stats.clone();
    let gear_bonus = equipment.total_stat_bonus();
    apply_stat_modifier(&mut derived_stats, &gear_bonus);
    derived_stats.max_hp = (derived_stats.max_hp + gear_bonus.hp).max(1);
    derived_stats.hp = derived_stats.max_hp;
    derived_stats.max_mp = (derived_stats.max_mp + gear_bonus.mp).max(0);
    derived_stats.mp = derived_stats.max_mp;
    derived_stats.clamp_resources();

    let mut inventory = vec![
        InventoryEntry {
            id: "potion_small".to_string(),
            category: ItemCategory::Consumable,
            item: ItemType::Consumable(ItemDef {
                id: "potion_small".to_string(),
                heal_hp: 25,
                heal_mp: 0,
            }),
            quantity: 3,
        },
        InventoryEntry {
            id: "ether_small".to_string(),
            category: ItemCategory::Consumable,
            item: ItemType::Consumable(ItemDef {
                id: "ether_small".to_string(),
                heal_hp: 0,
                heal_mp: 15,
            }),
            quantity: 2,
        },
    ];
    for extra in class_starter_items(request.class_type) {
        inventory.push(extra);
    }

    Ok(CharacterProfile {
        id: request.id,
        name: request.name,
        race: request.race,
        class_type: request.class_type,
        level: 1,
        exp: 0,
        base_stats: stats,
        derived_stats,
        equipment,
        inventory,
        passives: race_passives(request.race),
    })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Side {
    Party,
    Enemy,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BattleAction {
    Attack { actor: usize, target: usize },
    CastSpell {
        actor: usize,
        target: usize,
        spell: SpellDef,
    },
    UseItem {
        actor: usize,
        target: usize,
        item: ItemDef,
    },
    Run { actor: usize, run_roll: u8 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BattleEvent {
    Damage {
        side: Side,
        target: usize,
        amount: i32,
    },
    Heal {
        side: Side,
        target: usize,
        amount: i32,
    },
    Defeated {
        side: Side,
        target: usize,
    },
    Escaped,
    FailedRun,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EngineEvent {
    World(WorldEvent),
    EncounterStarted { pack_id: String },
    Battle(BattleEvent),
    AutoSaved { turn: u64 },
    NoEncounter,
    GameOver,
    MenuAction(MenuAction),
    ToolUsed { tool_id: String },
    ToolFailed { tool_id: String, reason: String },
    ToolChargesChanged { tool_id: String, charges: u32 },
    Audio(AudioEvent),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MenuAction {
    StartNewGame,
    ContinueGame,
    ResumeGame,
    SaveGame,
    LoadGame,
    QuitGame,
    OpenInventory,
    OpenParty,
    OpenSkills,
    OpenEquipment,
    OpenQuestLog,
    OpenSettings,
    OpenTools,
    OpenAudioCreator,
    OpenMap,
    BattleAttack,
    BattleSpell,
    BattleItem,
    BattleRun,
    Confirm,
    Cancel,
    UseTool(String),
    PreviewSfx(String),
    StopAudioPreview,
    Custom(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SoundCategory {
    Ui,
    Ambient,
    Footstep,
    Combat,
    Magic,
    Item,
    Environment,
    Voice,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundEffectDef {
    pub id: String,
    pub name: String,
    pub category: SoundCategory,
    pub file: String,
    pub default_volume_percent: u8,
    pub pitch_variance_percent: u8,
    pub looped: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MusicTrackDef {
    pub id: String,
    pub name: String,
    pub file: String,
    pub default_volume_percent: u8,
    pub looped: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AudioBusState {
    pub master_percent: u8,
    pub music_percent: u8,
    pub sfx_percent: u8,
    pub ui_percent: u8,
    pub muted: bool,
}

impl Default for AudioBusState {
    fn default() -> Self {
        Self {
            master_percent: 80,
            music_percent: 70,
            sfx_percent: 80,
            ui_percent: 85,
            muted: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AudioSystem {
    pub bus: AudioBusState,
    pub sfx_library: HashMap<String, SoundEffectDef>,
    pub music_library: HashMap<String, MusicTrackDef>,
    pub current_music_id: Option<String>,
    pub preview_sfx_id: Option<String>,
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self {
            bus: AudioBusState::default(),
            sfx_library: HashMap::new(),
            music_library: HashMap::new(),
            current_music_id: None,
            preview_sfx_id: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AudioEvent {
    PlaySfx {
        id: String,
        effective_volume_percent: u8,
    },
    StopSfxPreview,
    PlayMusic {
        id: String,
        effective_volume_percent: u8,
    },
    StopMusic,
    SetBus(AudioBusState),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolCategory {
    Traversal,
    Utility,
    Survival,
    Interaction,
    Combat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolType {
    Pickaxe,
    Axe,
    Shovel,
    FishingRod,
    Torch,
    Lockpick,
    Compass,
    Hammer,
    Rope,
    Bomb,
    Scanner,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolClass {
    Basic,
    Advanced,
    Masterwork,
    Artifact,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolTarget {
    SelfTarget,
    Tile,
    Object,
    Enemy,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolDef {
    pub id: String,
    pub name: String,
    pub category: ToolCategory,
    pub tool_type: ToolType,
    pub class: ToolClass,
    pub target: ToolTarget,
    pub power: i32,
    pub cooldown_turns: u32,
    pub max_charges: u32,
    pub menu_visible: bool,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolState {
    pub def: ToolDef,
    pub charges: u32,
    pub last_used_turn: Option<u64>,
}

impl ToolState {
    pub fn new(def: ToolDef) -> Self {
        Self {
            charges: def.max_charges,
            def,
            last_used_turn: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolUseContext {
    pub position: Coord2,
    pub terrain: Option<Terrain>,
    pub in_battle: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolUseResult {
    pub success: bool,
    pub consumed_charge: bool,
    pub cooldown_remaining: u32,
    pub events: Vec<EngineEvent>,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MenuItem {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    pub action: Option<MenuAction>,
    pub children: Vec<MenuItem>,
}

impl MenuItem {
    pub fn leaf(id: &str, label: &str, enabled: bool, action: MenuAction) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            enabled,
            action: Some(action),
            children: vec![],
        }
    }

    pub fn branch(id: &str, label: &str, enabled: bool, children: Vec<MenuItem>) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            enabled,
            action: None,
            children,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MenuState {
    pub root: MenuItem,
    pub open: bool,
    pub path: Vec<usize>,
    pub selected: usize,
}

impl MenuState {
    pub fn new(root: MenuItem) -> Self {
        let mut m = Self {
            root,
            open: true,
            path: vec![],
            selected: 0,
        };
        m.normalize_selection();
        m
    }

    pub fn current_items(&self) -> &[MenuItem] {
        if let Some(node) = node_at_path(&self.root, &self.path) {
            &node.children
        } else {
            &[]
        }
    }

    pub fn current_item(&self) -> Option<&MenuItem> {
        self.current_items().get(self.selected)
    }

    pub fn move_next(&mut self) {
        let len = self.current_items().len();
        if len == 0 {
            return;
        }
        for _ in 0..len {
            self.selected = (self.selected + 1) % len;
            if self.current_items()[self.selected].enabled {
                break;
            }
        }
    }

    pub fn move_prev(&mut self) {
        let len = self.current_items().len();
        if len == 0 {
            return;
        }
        for _ in 0..len {
            self.selected = (self.selected + len - 1) % len;
            if self.current_items()[self.selected].enabled {
                break;
            }
        }
    }

    pub fn enter(&mut self) -> Option<MenuAction> {
        let item = self.current_item()?.clone();
        if !item.enabled {
            return None;
        }
        if !item.children.is_empty() {
            self.path.push(self.selected);
            self.selected = 0;
            self.normalize_selection();
            return None;
        }
        item.action
    }

    pub fn back(&mut self) -> bool {
        if self.path.is_empty() {
            return false;
        }
        self.path.pop();
        self.selected = 0;
        self.normalize_selection();
        true
    }

    pub fn breadcrumbs(&self) -> Vec<String> {
        let mut out = vec![self.root.label.clone()];
        let mut curr = &self.root;
        for idx in &self.path {
            if let Some(next) = curr.children.get(*idx) {
                out.push(next.label.clone());
                curr = next;
            } else {
                break;
            }
        }
        out
    }

    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    fn normalize_selection(&mut self) {
        let len = self.current_items().len();
        if len == 0 {
            self.selected = 0;
            return;
        }
        if self.selected >= len {
            self.selected = 0;
        }
        if self.current_items()[self.selected].enabled {
            return;
        }
        for i in 0..len {
            if self.current_items()[i].enabled {
                self.selected = i;
                return;
            }
        }
    }
}

fn node_at_path<'a>(root: &'a MenuItem, path: &[usize]) -> Option<&'a MenuItem> {
    let mut node = root;
    for idx in path {
        node = node.children.get(*idx)?;
    }
    Some(node)
}

pub fn build_main_menu(has_save: bool) -> MenuState {
    MenuState::new(MenuItem::branch(
        "main",
        "Main Menu",
        true,
        vec![
            MenuItem::leaf("new_game", "New Game", true, MenuAction::StartNewGame),
            MenuItem::leaf("continue", "Continue", has_save, MenuAction::ContinueGame),
            MenuItem::branch(
                "options",
                "Options",
                true,
                vec![
                    MenuItem::leaf(
                        "video",
                        "Video",
                        true,
                        MenuAction::Custom("open_video_options".to_string()),
                    ),
                    MenuItem::leaf(
                        "audio",
                        "Audio",
                        true,
                        MenuAction::OpenAudioCreator,
                    ),
                    MenuItem::leaf(
                        "gameplay",
                        "Gameplay",
                        true,
                        MenuAction::Custom("open_gameplay_options".to_string()),
                    ),
                ],
            ),
            MenuItem::leaf("quit", "Quit", true, MenuAction::QuitGame),
        ],
    ))
}

pub fn build_creator_top_menu() -> MenuState {
    MenuState::new(MenuItem::branch(
        "creator_top",
        "Creator Top Menu",
        true,
        vec![
            MenuItem::branch(
                "file",
                "File",
                true,
                vec![
                    MenuItem::leaf("new_game", "New Project", true, MenuAction::StartNewGame),
                    MenuItem::leaf("save", "Save", true, MenuAction::SaveGame),
                    MenuItem::leaf("load", "Load", true, MenuAction::LoadGame),
                    MenuItem::leaf("quit", "Quit", true, MenuAction::QuitGame),
                ],
            ),
            MenuItem::branch(
                "edit",
                "Edit",
                true,
                vec![
                    MenuItem::leaf(
                        "settings",
                        "Project Settings",
                        true,
                        MenuAction::OpenSettings,
                    ),
                    MenuItem::leaf("tools", "Tools", true, MenuAction::OpenTools),
                ],
            ),
            MenuItem::branch(
                "audio_creator",
                "Audio Creator",
                true,
                vec![
                    MenuItem::leaf(
                        "open_audio",
                        "Open Audio Library",
                        true,
                        MenuAction::OpenAudioCreator,
                    ),
                    MenuItem::leaf(
                        "stop_preview",
                        "Stop Preview",
                        true,
                        MenuAction::StopAudioPreview,
                    ),
                ],
            ),
        ],
    ))
}

pub fn build_pause_menu(can_save: bool) -> MenuState {
    MenuState::new(MenuItem::branch(
        "pause",
        "Pause Menu",
        true,
        vec![
            MenuItem::leaf("resume", "Resume", true, MenuAction::ResumeGame),
            MenuItem::leaf("save", "Save", can_save, MenuAction::SaveGame),
            MenuItem::leaf("load", "Load", true, MenuAction::LoadGame),
            MenuItem::leaf("inventory", "Inventory", true, MenuAction::OpenInventory),
            MenuItem::leaf("party", "Party", true, MenuAction::OpenParty),
            MenuItem::leaf("tools", "Tools", true, MenuAction::OpenTools),
            MenuItem::leaf("quests", "Quest Log", true, MenuAction::OpenQuestLog),
            MenuItem::leaf("map", "Map", true, MenuAction::OpenMap),
            MenuItem::leaf("settings", "Settings", true, MenuAction::OpenSettings),
            MenuItem::leaf("quit", "Quit", true, MenuAction::QuitGame),
        ],
    ))
}

pub fn build_battle_menu(can_run: bool) -> MenuState {
    MenuState::new(MenuItem::branch(
        "battle",
        "Battle Menu",
        true,
        vec![
            MenuItem::leaf("attack", "Attack", true, MenuAction::BattleAttack),
            MenuItem::leaf("spell", "Spell", true, MenuAction::BattleSpell),
            MenuItem::leaf("item", "Item", true, MenuAction::BattleItem),
            MenuItem::leaf("run", "Run", can_run, MenuAction::BattleRun),
        ],
    ))
}

pub fn build_tool_menu(tools: &HashMap<String, ToolState>) -> MenuState {
    let mut items: Vec<MenuItem> = tools
        .values()
        .filter(|t| t.def.menu_visible)
        .map(|t| {
            let label = format!("{} [{}]", t.def.name, t.charges);
            MenuItem::leaf(
                &format!("tool_{}", t.def.id),
                &label,
                t.charges > 0,
                MenuAction::UseTool(t.def.id.clone()),
            )
        })
        .collect();

    items.sort_by(|a, b| a.label.cmp(&b.label));
    MenuState::new(MenuItem::branch("tools", "Tools", true, items))
}

pub fn build_audio_creator_menu(audio: &AudioSystem) -> MenuState {
    let mut items = vec![
        MenuItem::leaf("stop_preview", "Stop Preview", true, MenuAction::StopAudioPreview),
        MenuItem::leaf(
            "audio_bus",
            &format!(
                "Bus M:{} Music:{} SFX:{} UI:{}",
                audio.bus.master_percent, audio.bus.music_percent, audio.bus.sfx_percent, audio.bus.ui_percent
            ),
            false,
            MenuAction::Custom("audio_bus_info".to_string()),
        ),
    ];

    let mut sfx_items: Vec<MenuItem> = audio
        .sfx_library
        .values()
        .map(|sfx| {
            MenuItem::leaf(
                &format!("preview_{}", sfx.id),
                &format!("Preview {}", sfx.name),
                true,
                MenuAction::PreviewSfx(sfx.id.clone()),
            )
        })
        .collect();
    sfx_items.sort_by(|a, b| a.label.cmp(&b.label));
    items.extend(sfx_items);

    MenuState::new(MenuItem::branch("audio_creator", "Audio Creator", true, items))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MenuInput {
    Up,
    Down,
    Confirm,
    Back,
    Close,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MenuViewItem {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    pub selected: bool,
    pub has_children: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MenuViewModel {
    pub title: String,
    pub breadcrumbs: Vec<String>,
    pub items: Vec<MenuViewItem>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct ApiResponse {
    pub events: Vec<EngineEvent>,
    pub menu: Option<MenuViewModel>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BattleResult {
    Ongoing,
    PartyWon,
    EnemyWon,
    Escaped,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BattleState {
    pub party: Party,
    pub enemies: Vec<Fighter>,
    pub result: BattleResult,
    pub turn: u32,
    pub boss_battle: bool,
}

impl BattleState {
    pub fn new(party: Party, enemies: Vec<Fighter>) -> Self {
        Self {
            party,
            enemies,
            result: BattleResult::Ongoing,
            turn: 1,
            boss_battle: false,
        }
    }

    pub fn apply_party_action(&mut self, action: BattleAction) -> Vec<BattleEvent> {
        self.apply_action(Side::Party, action)
    }

    pub fn apply_enemy_action(&mut self, action: BattleAction) -> Vec<BattleEvent> {
        self.apply_action(Side::Enemy, action)
    }

    pub fn end_round_tick(&mut self) {
        tick_statuses(&mut self.party.members);
        tick_statuses(&mut self.enemies);
        self.turn = self.turn.saturating_add(1);
        self.refresh_result();
    }

    fn apply_action(&mut self, side: Side, action: BattleAction) -> Vec<BattleEvent> {
        if self.result != BattleResult::Ongoing {
            return vec![];
        }

        let mut events = Vec::new();
        match action {
            BattleAction::Attack { actor, target } => {
                let (attacker, defender, side_of_defender, target_idx) =
                    self.actor_target_mut(side, actor, target);
                let Some(attacker) = attacker else {
                    return events;
                };
                let Some(defender) = defender else {
                    return events;
                };
                let dmg = physical_damage(&attacker.stats, &defender.stats, 0);
                apply_damage(defender, dmg, side_of_defender, target_idx, &mut events);
            }
            BattleAction::CastSpell {
                actor,
                target,
                spell,
            } => {
                let caster_int = {
                    let Some(caster) = self.side_mut(side.clone()).get_mut(actor) else {
                        return events;
                    };
                    if caster.stats.mp < spell.mp_cost || !caster.is_alive() {
                        return events;
                    }
                    caster.stats.mp -= spell.mp_cost;
                    caster.stats.clamp_resources();
                    caster.stats.int_
                };

                match spell.kind {
                    SpellKind::Damage => {
                        let side_of_defender = opposite(side.clone());
                        let Some(target_ref) =
                            self.side_mut(side_of_defender.clone()).get_mut(target)
                        else {
                            return events;
                        };
                        let dmg = (spell.power + caster_int - target_ref.stats.vit / 3).max(1);
                        apply_damage(target_ref, dmg, side_of_defender, target, &mut events);
                    }
                    SpellKind::Heal => {
                        let Some(target_ref) = self.side_mut(side.clone()).get_mut(target) else {
                            return events;
                        };
                        let amount = (spell.power + caster_int / 2).max(1);
                        apply_heal(target_ref, amount, side, target, &mut events);
                    }
                }
            }
            BattleAction::UseItem {
                actor: _,
                target,
                item,
            } => {
                if !self.party.consume_item(&item.id) {
                    return events;
                }
                let Some(target_ref) = self.party.members.get_mut(target) else {
                    return events;
                };
                if item.heal_hp > 0 {
                    apply_heal(target_ref, item.heal_hp, Side::Party, target, &mut events);
                }
                if item.heal_mp > 0 {
                    target_ref.stats.mp += item.heal_mp;
                    target_ref.stats.clamp_resources();
                }
            }
            BattleAction::Run {
                actor: _,
                run_roll,
            } => {
                if run_roll >= 60 {
                    self.result = BattleResult::Escaped;
                    events.push(BattleEvent::Escaped);
                } else {
                    events.push(BattleEvent::FailedRun);
                }
            }
        }

        self.refresh_result();
        events
    }

    fn actor_target_mut(
        &mut self,
        side: Side,
        actor: usize,
        target: usize,
    ) -> (
        Option<&mut Fighter>,
        Option<&mut Fighter>,
        Side,
        usize,
    ) {
        let side_of_defender = opposite(side.clone());
        match side {
            Side::Party => {
                let attacker = self.party.members.get_mut(actor);
                let defender = self.enemies.get_mut(target);
                (attacker, defender, side_of_defender, target)
            }
            Side::Enemy => {
                let attacker = self.enemies.get_mut(actor);
                let defender = self.party.members.get_mut(target);
                (attacker, defender, side_of_defender, target)
            }
        }
    }

    fn side_mut(&mut self, side: Side) -> &mut Vec<Fighter> {
        match side {
            Side::Party => &mut self.party.members,
            Side::Enemy => &mut self.enemies,
        }
    }

    fn refresh_result(&mut self) {
        if self.result == BattleResult::Escaped {
            return;
        }
        let party_alive = self.party.members.iter().any(Fighter::is_alive);
        let enemies_alive = self.enemies.iter().any(Fighter::is_alive);
        self.result = match (party_alive, enemies_alive) {
            (true, true) => BattleResult::Ongoing,
            (true, false) => BattleResult::PartyWon,
            (false, true) => BattleResult::EnemyWon,
            (false, false) => BattleResult::EnemyWon,
        };
    }
}

pub fn sample_world_20x15() -> World2D {
    let mut map = Grid2D::filled(20, 15, TileDef::new(Terrain::Grass, 8)).unwrap();
    for x in 0..20 {
        let _ = map.set(x, 0, TileDef::new(Terrain::Wall, 0));
        let _ = map.set(x, 14, TileDef::new(Terrain::Wall, 0));
    }
    for y in 0..15 {
        let _ = map.set(0, y, TileDef::new(Terrain::Wall, 0));
        let _ = map.set(19, y, TileDef::new(Terrain::Wall, 0));
    }
    for x in 3..10 {
        let _ = map.set(x, 4, TileDef::new(Terrain::Forest, 20));
    }
    let _ = map.set(12, 8, TileDef::new(Terrain::DoorClosed, 0));
    let _ = map.set(8, 10, TileDef::new(Terrain::ChestClosed, 0));

    let mut world = World2D::new(map);
    world.add_trigger(
        Coord2 { x: 8, y: 10 },
        Trigger {
            id: "chest_8_10".to_string(),
            once: true,
            fired: false,
            action: ScriptAction::Composite(vec![
                ScriptAction::OpenChest {
                    at: Coord2 { x: 8, y: 10 },
                    item_id: "herb".to_string(),
                },
                ScriptAction::SetQuestFlag {
                    key: "found_herb".to_string(),
                    value: true,
                },
            ]),
        },
    );
    world.add_trigger(
        Coord2 { x: 12, y: 8 },
        Trigger {
            id: "door_12_8".to_string(),
            once: true,
            fired: false,
            action: ScriptAction::OpenDoor {
                at: Coord2 { x: 12, y: 8 },
            },
        },
    );
    world
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PlayState {
    Exploring,
    InBattle(BattleState),
    GameOver,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RpgGameEngine {
    pub settings: GameSettings,
    pub world: World2D,
    pub encounter_table: EncounterTable,
    pub party: Party,
    pub player_pos: Coord2,
    pub state: PlayState,
    pub last_auto_save_turn: u64,
    #[serde(default)]
    pub menu_state: Option<MenuState>,
    #[serde(default)]
    pub tools: HashMap<String, ToolState>,
    #[serde(default)]
    pub audio: AudioSystem,
    #[serde(default)]
    pub atlas: WorldAtlas,
}

impl RpgGameEngine {
    pub fn new(
        settings: GameSettings,
        world: World2D,
        encounter_table: EncounterTable,
        party: Party,
        player_pos: Coord2,
    ) -> Self {
        Self {
            settings,
            world,
            encounter_table,
            party,
            player_pos,
            state: PlayState::Exploring,
            last_auto_save_turn: 0,
            menu_state: None,
            tools: HashMap::new(),
            audio: AudioSystem::default(),
            atlas: WorldAtlas::default(),
        }
    }

    pub fn create_continent(
        &mut self,
        id: &str,
        name: &str,
        climate: ClimateBand,
    ) -> Result<(), WorldCreationError> {
        self.atlas.add_continent(Continent {
            id: id.to_string(),
            name: name.to_string(),
            climate,
            country_ids: vec![],
            zone_ids: vec![],
        })
    }

    pub fn create_country(
        &mut self,
        id: &str,
        name: &str,
        continent_id: &str,
        capital: &str,
        tags: Vec<String>,
    ) -> Result<(), WorldCreationError> {
        self.atlas.add_country(Country {
            id: id.to_string(),
            name: name.to_string(),
            continent_id: continent_id.to_string(),
            capital: capital.to_string(),
            zone_ids: vec![],
            tags,
        })
    }

    pub fn create_zone(
        &mut self,
        continent_id: &str,
        country_id: &str,
        zone: Zone,
    ) -> Result<(), WorldCreationError> {
        self.atlas.add_zone(zone, continent_id, country_id)
    }

    pub fn create_sub_zone(
        &mut self,
        zone_id: &str,
        sub_zone: SubZone,
    ) -> Result<(), WorldCreationError> {
        self.atlas.add_sub_zone(zone_id, sub_zone)
    }

    pub fn create_dungeon(
        &mut self,
        dungeon: DungeonBlueprint,
    ) -> Result<(), WorldCreationError> {
        self.atlas.add_dungeon(dungeon)
    }

    pub fn create_raid(&mut self, raid: RaidBlueprint) -> Result<(), WorldCreationError> {
        self.atlas.add_raid(raid)
    }

    pub fn create_trial(&mut self, trial: TrialBlueprint) -> Result<(), WorldCreationError> {
        self.atlas.add_trial(trial)
    }

    pub fn create_tower(&mut self, tower: TowerBlueprint) -> Result<(), WorldCreationError> {
        self.atlas.add_tower(tower)
    }

    pub fn register_tool(&mut self, tool: ToolDef) {
        self.tools.insert(tool.id.clone(), ToolState::new(tool));
    }

    pub fn register_sfx(&mut self, sfx: SoundEffectDef) {
        self.audio.sfx_library.insert(sfx.id.clone(), sfx);
    }

    pub fn register_music(&mut self, track: MusicTrackDef) {
        self.audio.music_library.insert(track.id.clone(), track);
    }

    pub fn set_audio_bus(&mut self, bus: AudioBusState) -> EngineEvent {
        self.audio.bus = bus;
        EngineEvent::Audio(AudioEvent::SetBus(self.audio.bus.clone()))
    }

    pub fn step_move(
        &mut self,
        dir: Direction2D,
        encounter_roll_percent: u8,
        pack_roll: u32,
    ) -> Vec<EngineEvent> {
        if self.state != PlayState::Exploring {
            return vec![];
        }

        let mut events = Vec::new();
        let (new_pos, world_events) = self.world.try_move(self.player_pos, dir);
        self.player_pos = new_pos;
        for e in world_events {
            events.push(EngineEvent::World(e));
        }
        events.push(EngineEvent::World(self.world.advance_time(5)));
        events.push(EngineEvent::World(self.world.tick_environment()));

        let should_auto_save = self.settings.auto_save_every_turns > 0
            && self
                .world
                .turn
                .saturating_sub(self.last_auto_save_turn)
                >= u64::from(self.settings.auto_save_every_turns);
        if should_auto_save {
            self.last_auto_save_turn = self.world.turn;
            events.push(EngineEvent::AutoSaved {
                turn: self.world.turn,
            });
        }

        let Some(tile) = self.world.tile(self.player_pos).cloned() else {
            return events;
        };
        let adjusted_rate = self
            .settings
            .effective_encounter_rate(tile.encounter_rate_percent);
        let should_start = encounter_roll_percent < adjusted_rate;
        if !should_start {
            events.push(EngineEvent::NoEncounter);
            return events;
        }

        let Some(pack) = self.encounter_table.pick_for_terrain(tile.terrain, pack_roll) else {
            events.push(EngineEvent::NoEncounter);
            return events;
        };

        let mut battle = BattleState::new(self.party.clone(), pack.monsters.clone());
        if pack.id.contains("boss") {
            battle.boss_battle = true;
        }
        self.state = PlayState::InBattle(battle);
        events.push(EngineEvent::EncounterStarted {
            pack_id: pack.id.clone(),
        });
        events
    }

    pub fn apply_party_battle_action(
        &mut self,
        action: BattleAction,
        enemy_counter_action: Option<BattleAction>,
    ) -> Vec<EngineEvent> {
        let PlayState::InBattle(battle) = &mut self.state else {
            return vec![];
        };

        let mut out = Vec::new();

        if let BattleAction::Run { .. } = action
            && battle.boss_battle
            && !self.settings.allow_run_from_boss
        {
            out.push(EngineEvent::Battle(BattleEvent::FailedRun));
            return out;
        }

        let party_events = battle.apply_party_action(action);
        for e in party_events {
            out.push(EngineEvent::Battle(e));
        }

        if battle.result == BattleResult::Ongoing
            && let Some(enemy_action) = enemy_counter_action
        {
            let enemy_events = battle.apply_enemy_action(enemy_action);
            for e in enemy_events {
                out.push(EngineEvent::Battle(e));
            }
        }

        battle.end_round_tick();
        match battle.result {
            BattleResult::Ongoing => {}
            BattleResult::PartyWon | BattleResult::Escaped => {
                self.party = battle.party.clone();
                self.state = PlayState::Exploring;
            }
            BattleResult::EnemyWon => {
                self.party = battle.party.clone();
                self.state = PlayState::GameOver;
                out.push(EngineEvent::GameOver);
            }
        }

        out
    }

    pub fn open_pause_menu(&mut self) {
        let can_save = self.state != PlayState::GameOver;
        self.menu_state = Some(build_pause_menu(can_save));
    }

    pub fn open_tool_menu(&mut self) {
        self.menu_state = Some(build_tool_menu(&self.tools));
    }

    pub fn open_audio_creator_menu(&mut self) {
        self.menu_state = Some(build_audio_creator_menu(&self.audio));
    }

    pub fn open_creator_top_menu(&mut self) {
        self.menu_state = Some(build_creator_top_menu());
    }

    pub fn open_battle_menu(&mut self) {
        if let PlayState::InBattle(battle) = &self.state {
            let can_run = !battle.boss_battle || self.settings.allow_run_from_boss;
            self.menu_state = Some(build_battle_menu(can_run));
        }
    }

    pub fn close_menu(&mut self) {
        self.menu_state = None;
    }

    pub fn menu_view(&self) -> Option<MenuViewModel> {
        let menu = self.menu_state.as_ref()?;
        let items = menu
            .current_items()
            .iter()
            .enumerate()
            .map(|(idx, item)| MenuViewItem {
                id: item.id.clone(),
                label: item.label.clone(),
                enabled: item.enabled,
                selected: idx == menu.selected,
                has_children: !item.children.is_empty(),
            })
            .collect();
        Some(MenuViewModel {
            title: menu.root.label.clone(),
            breadcrumbs: menu.breadcrumbs(),
            items,
        })
    }

    pub fn handle_menu_input(&mut self, input: MenuInput) -> ApiResponse {
        let mut events = Vec::new();
        let Some(menu) = self.menu_state.as_mut() else {
            return ApiResponse {
                events,
                menu: None,
            };
        };

        let mut pending_action: Option<MenuAction> = None;
        match input {
            MenuInput::Up => menu.move_prev(),
            MenuInput::Down => menu.move_next(),
            MenuInput::Back => {
                if !menu.back() {
                    self.menu_state = None;
                }
            }
            MenuInput::Close => {
                self.menu_state = None;
            }
            MenuInput::Confirm => {
                pending_action = menu.enter();
            }
        }
        if let Some(action) = pending_action {
            events.extend(self.execute_menu_action(action));
        }
        ApiResponse {
            events,
            menu: self.menu_view(),
        }
    }

    pub fn api_step_move(
        &mut self,
        dir: Direction2D,
        encounter_roll_percent: u8,
        pack_roll: u32,
    ) -> ApiResponse {
        let events = self.step_move(dir, encounter_roll_percent, pack_roll);
        ApiResponse {
            events,
            menu: self.menu_view(),
        }
    }

    pub fn api_battle_action(
        &mut self,
        action: BattleAction,
        enemy_counter_action: Option<BattleAction>,
    ) -> ApiResponse {
        let events = self.apply_party_battle_action(action, enemy_counter_action);
        ApiResponse {
            events,
            menu: self.menu_view(),
        }
    }

    fn execute_menu_action(&mut self, action: MenuAction) -> Vec<EngineEvent> {
        let mut events = vec![EngineEvent::MenuAction(action.clone())];
        match action {
            MenuAction::ResumeGame => {
                self.menu_state = None;
            }
            MenuAction::OpenTools => {
                self.open_tool_menu();
            }
            MenuAction::OpenAudioCreator => {
                self.open_audio_creator_menu();
            }
            MenuAction::PreviewSfx(id) => {
                events.push(self.preview_sfx(&id));
            }
            MenuAction::StopAudioPreview => {
                events.push(EngineEvent::Audio(AudioEvent::StopSfxPreview));
                self.audio.preview_sfx_id = None;
            }
            MenuAction::UseTool(tool_id) => {
                let context = ToolUseContext {
                    position: self.player_pos,
                    terrain: self.world.tile(self.player_pos).map(|t| t.terrain),
                    in_battle: matches!(self.state, PlayState::InBattle(_)),
                };
                let result = self.use_tool_by_id(&tool_id, context);
                events.extend(result.events);
            }
            MenuAction::SaveGame => {
                events.push(EngineEvent::AutoSaved {
                    turn: self.world.turn,
                });
                self.menu_state = None;
            }
            MenuAction::BattleAttack => {
                events.extend(self.apply_party_battle_action(
                    BattleAction::Attack {
                        actor: 0,
                        target: 0,
                    },
                    None,
                ));
            }
            MenuAction::BattleSpell => {
                events.extend(self.apply_party_battle_action(
                    BattleAction::CastSpell {
                        actor: 0,
                        target: 0,
                        spell: SpellDef {
                            id: "menu_spell".to_string(),
                            mp_cost: 1,
                            power: 4,
                            kind: SpellKind::Damage,
                        },
                    },
                    None,
                ));
            }
            MenuAction::BattleItem => {
                events.extend(self.apply_party_battle_action(
                    BattleAction::UseItem {
                        actor: 0,
                        target: 0,
                        item: ItemDef {
                            id: "potion_small".to_string(),
                            heal_hp: 20,
                            heal_mp: 0,
                        },
                    },
                    None,
                ));
            }
            MenuAction::BattleRun => {
                events.extend(self.apply_party_battle_action(
                    BattleAction::Run {
                        actor: 0,
                        run_roll: 99,
                    },
                    None,
                ));
            }
            MenuAction::QuitGame => {
                self.state = PlayState::GameOver;
                events.push(EngineEvent::GameOver);
            }
            _ => {}
        }
        events
    }

    pub fn preview_sfx(&mut self, id: &str) -> EngineEvent {
        let Some(sfx) = self.audio.sfx_library.get(id) else {
            return EngineEvent::ToolFailed {
                tool_id: id.to_string(),
                reason: "sfx_not_found".to_string(),
            };
        };
        self.audio.preview_sfx_id = Some(id.to_string());
        let effective = effective_volume(
            self.audio.bus.master_percent,
            self.audio.bus.sfx_percent,
            sfx.default_volume_percent,
            self.audio.bus.muted,
        );
        EngineEvent::Audio(AudioEvent::PlaySfx {
            id: id.to_string(),
            effective_volume_percent: effective,
        })
    }

    pub fn use_tool_by_id(&mut self, tool_id: &str, ctx: ToolUseContext) -> ToolUseResult {
        let Some(tool) = self.tools.get_mut(tool_id) else {
            return ToolUseResult {
                success: false,
                consumed_charge: false,
                cooldown_remaining: 0,
                events: vec![EngineEvent::ToolFailed {
                    tool_id: tool_id.to_string(),
                    reason: "tool_not_found".to_string(),
                }],
                message: "Tool not found".to_string(),
            };
        };

        if tool.charges == 0 {
            return ToolUseResult {
                success: false,
                consumed_charge: false,
                cooldown_remaining: 0,
                events: vec![EngineEvent::ToolFailed {
                    tool_id: tool_id.to_string(),
                    reason: "no_charges".to_string(),
                }],
                message: "No charges left".to_string(),
            };
        }

        let now_turn = self.world.turn;
        if let Some(last) = tool.last_used_turn {
            let elapsed = now_turn.saturating_sub(last);
            if elapsed < u64::from(tool.def.cooldown_turns) {
                return ToolUseResult {
                    success: false,
                    consumed_charge: false,
                    cooldown_remaining: (u64::from(tool.def.cooldown_turns) - elapsed) as u32,
                    events: vec![EngineEvent::ToolFailed {
                        tool_id: tool_id.to_string(),
                        reason: "cooldown".to_string(),
                    }],
                    message: "Tool is on cooldown".to_string(),
                };
            }
        }

        tool.charges = tool.charges.saturating_sub(1);
        tool.last_used_turn = Some(now_turn);
        let mut events = vec![
            EngineEvent::ToolUsed {
                tool_id: tool_id.to_string(),
            },
            EngineEvent::ToolChargesChanged {
                tool_id: tool_id.to_string(),
                charges: tool.charges,
            },
        ];

        match tool.def.tool_type {
            ToolType::Torch => {
                self.world.environment.fire_level = self.world.environment.fire_level.saturating_add(2);
                events.push(EngineEvent::World(WorldEvent::EnvironmentTick {
                    fire_level: self.world.environment.fire_level,
                    flood_level: self.world.environment.flood_level,
                    poison_level: self.world.environment.poison_level,
                }));
            }
            ToolType::Compass => {
                events.push(EngineEvent::World(WorldEvent::Message {
                    text: format!("Position {}, {}", ctx.position.x, ctx.position.y),
                }));
            }
            ToolType::Lockpick => {
                if ctx.terrain == Some(Terrain::DoorClosed) {
                    let _ = self
                        .world
                        .set_tile(ctx.position, TileDef::new(Terrain::DoorOpen, 0));
                    events.push(EngineEvent::World(WorldEvent::DoorOpened { at: ctx.position }));
                }
            }
            _ => {}
        }

        ToolUseResult {
            success: true,
            consumed_charge: true,
            cooldown_remaining: tool.def.cooldown_turns,
            events,
            message: "Tool used".to_string(),
        }
    }
}

fn shifted_coord(from: Coord2, dir: Direction2D, width: usize, height: usize) -> Option<Coord2> {
    let (dx, dy) = match dir {
        Direction2D::Up => (0isize, -1isize),
        Direction2D::Down => (0, 1),
        Direction2D::Left => (-1, 0),
        Direction2D::Right => (1, 0),
    };
    let nx = from.x as isize + dx;
    let ny = from.y as isize + dy;
    if nx < 0 || ny < 0 || nx >= width as isize || ny >= height as isize {
        return None;
    }
    Some(Coord2 {
        x: nx as usize,
        y: ny as usize,
    })
}

fn opposite(side: Side) -> Side {
    match side {
        Side::Party => Side::Enemy,
        Side::Enemy => Side::Party,
    }
}

fn physical_damage(attacker: &Stats, defender: &Stats, variance: i32) -> i32 {
    (attacker.str_ + variance - defender.vit / 2).max(1)
}

fn effective_volume(master: u8, bus: u8, source: u8, muted: bool) -> u8 {
    if muted {
        return 0;
    }
    let v = u32::from(master) * u32::from(bus) * u32::from(source) / 10_000;
    v.min(100) as u8
}

fn apply_damage(
    target: &mut Fighter,
    damage: i32,
    side: Side,
    target_idx: usize,
    events: &mut Vec<BattleEvent>,
) {
    let amount = damage.max(0);
    target.stats.hp -= amount;
    target.stats.clamp_resources();
    events.push(BattleEvent::Damage {
        side: side.clone(),
        target: target_idx,
        amount,
    });
    if !target.is_alive() {
        events.push(BattleEvent::Defeated {
            side,
            target: target_idx,
        });
    }
}

fn apply_heal(
    target: &mut Fighter,
    amount: i32,
    side: Side,
    target_idx: usize,
    events: &mut Vec<BattleEvent>,
) {
    let heal = amount.max(0);
    target.stats.hp += heal;
    target.stats.clamp_resources();
    events.push(BattleEvent::Heal {
        side,
        target: target_idx,
        amount: heal,
    });
}

fn tick_statuses(team: &mut [Fighter]) {
    for fighter in team {
        if !fighter.is_alive() {
            continue;
        }

        let mut next = Vec::with_capacity(fighter.statuses.len());
        for status in &fighter.statuses {
            match status {
                StatusEffect::Poison { damage, turns_left } => {
                    fighter.stats.hp -= *damage;
                    fighter.stats.clamp_resources();
                    if *turns_left > 1 {
                        next.push(StatusEffect::Poison {
                            damage: *damage,
                            turns_left: turns_left - 1,
                        });
                    }
                }
            }
        }
        fighter.statuses = next;
    }
}

fn add_modifier(dst: &mut StatModifier, src: &StatModifier) {
    dst.hp += src.hp;
    dst.mp += src.mp;
    dst.str_ += src.str_;
    dst.agi += src.agi;
    dst.vit += src.vit;
    dst.int_ += src.int_;
    dst.luk += src.luk;
}

fn apply_stat_modifier(stats: &mut Stats, modifier: &StatModifier) {
    stats.max_hp += modifier.hp;
    stats.max_mp += modifier.mp;
    stats.str_ += modifier.str_;
    stats.agi += modifier.agi;
    stats.vit += modifier.vit;
    stats.int_ += modifier.int_;
    stats.luk += modifier.luk;
    stats.clamp_resources();
}

fn race_stat_bonus(race: CharacterRace) -> StatModifier {
    match race {
        CharacterRace::Human => StatModifier {
            str_: 1,
            int_: 1,
            luk: 1,
            ..Default::default()
        },
        CharacterRace::Elf => StatModifier {
            agi: 2,
            int_: 2,
            vit: -1,
            ..Default::default()
        },
        CharacterRace::Dwarf => StatModifier {
            vit: 3,
            str_: 1,
            agi: -1,
            ..Default::default()
        },
        CharacterRace::Halfling => StatModifier {
            agi: 2,
            luk: 2,
            str_: -1,
            ..Default::default()
        },
        CharacterRace::Orc => StatModifier {
            str_: 3,
            vit: 1,
            int_: -2,
            ..Default::default()
        },
        CharacterRace::Draconian => StatModifier {
            str_: 2,
            vit: 2,
            mp: 2,
            ..Default::default()
        },
        CharacterRace::Undead => StatModifier {
            vit: 2,
            int_: 1,
            luk: -1,
            ..Default::default()
        },
    }
}

fn race_passives(race: CharacterRace) -> Vec<String> {
    match race {
        CharacterRace::Human => vec!["Adaptive".to_string()],
        CharacterRace::Elf => vec!["Keen Sight".to_string(), "Mana Affinity".to_string()],
        CharacterRace::Dwarf => vec!["Stoneblood".to_string()],
        CharacterRace::Halfling => vec!["Lucky Footwork".to_string()],
        CharacterRace::Orc => vec!["Battle Rage".to_string()],
        CharacterRace::Draconian => vec!["Scale Guard".to_string()],
        CharacterRace::Undead => vec!["Deathless Resolve".to_string()],
    }
}

fn class_base_stats(class_type: CharacterClassType) -> Stats {
    match class_type {
        CharacterClassType::Warrior => Stats {
            hp: 0,
            max_hp: 26,
            mp: 0,
            max_mp: 6,
            str_: 10,
            agi: 6,
            vit: 9,
            int_: 3,
            luk: 4,
        },
        CharacterClassType::Knight => Stats {
            hp: 0,
            max_hp: 28,
            mp: 0,
            max_mp: 8,
            str_: 9,
            agi: 5,
            vit: 10,
            int_: 4,
            luk: 4,
        },
        CharacterClassType::Ranger => Stats {
            hp: 0,
            max_hp: 22,
            mp: 0,
            max_mp: 10,
            str_: 7,
            agi: 9,
            vit: 6,
            int_: 5,
            luk: 6,
        },
        CharacterClassType::Rogue => Stats {
            hp: 0,
            max_hp: 20,
            mp: 0,
            max_mp: 8,
            str_: 7,
            agi: 10,
            vit: 5,
            int_: 5,
            luk: 7,
        },
        CharacterClassType::Cleric => Stats {
            hp: 0,
            max_hp: 21,
            mp: 0,
            max_mp: 16,
            str_: 5,
            agi: 5,
            vit: 6,
            int_: 8,
            luk: 5,
        },
        CharacterClassType::Mage => Stats {
            hp: 0,
            max_hp: 16,
            mp: 0,
            max_mp: 22,
            str_: 3,
            agi: 6,
            vit: 4,
            int_: 11,
            luk: 5,
        },
        CharacterClassType::Battlemage => Stats {
            hp: 0,
            max_hp: 22,
            mp: 0,
            max_mp: 16,
            str_: 7,
            agi: 6,
            vit: 6,
            int_: 8,
            luk: 4,
        },
        CharacterClassType::Necromancer => Stats {
            hp: 0,
            max_hp: 17,
            mp: 0,
            max_mp: 20,
            str_: 3,
            agi: 5,
            vit: 4,
            int_: 10,
            luk: 7,
        },
    }
}

fn class_starting_equipment(class_type: CharacterClassType) -> EquipmentLoadout {
    let basic_weapon = match class_type {
        CharacterClassType::Warrior | CharacterClassType::Knight => WeaponDef {
            id: "iron_sword".to_string(),
            name: "Iron Sword".to_string(),
            weapon_type: WeaponType::Sword,
            power: 7,
            hit_bonus: 3,
            crit_bonus: 1,
            rarity: ItemRarity::Common,
            stat_bonus: StatModifier {
                str_: 1,
                ..Default::default()
            },
        },
        CharacterClassType::Ranger => WeaponDef {
            id: "hunter_bow".to_string(),
            name: "Hunter Bow".to_string(),
            weapon_type: WeaponType::Bow,
            power: 6,
            hit_bonus: 4,
            crit_bonus: 2,
            rarity: ItemRarity::Common,
            stat_bonus: StatModifier {
                agi: 1,
                ..Default::default()
            },
        },
        CharacterClassType::Rogue => WeaponDef {
            id: "steel_dagger".to_string(),
            name: "Steel Dagger".to_string(),
            weapon_type: WeaponType::Dagger,
            power: 5,
            hit_bonus: 5,
            crit_bonus: 4,
            rarity: ItemRarity::Common,
            stat_bonus: StatModifier {
                agi: 1,
                luk: 1,
                ..Default::default()
            },
        },
        CharacterClassType::Cleric => WeaponDef {
            id: "oak_mace".to_string(),
            name: "Oak Mace".to_string(),
            weapon_type: WeaponType::Mace,
            power: 5,
            hit_bonus: 2,
            crit_bonus: 0,
            rarity: ItemRarity::Common,
            stat_bonus: StatModifier {
                vit: 1,
                ..Default::default()
            },
        },
        CharacterClassType::Mage | CharacterClassType::Necromancer => WeaponDef {
            id: "apprentice_staff".to_string(),
            name: "Apprentice Staff".to_string(),
            weapon_type: WeaponType::Staff,
            power: 3,
            hit_bonus: 2,
            crit_bonus: 0,
            rarity: ItemRarity::Common,
            stat_bonus: StatModifier {
                int_: 2,
                mp: 2,
                ..Default::default()
            },
        },
        CharacterClassType::Battlemage => WeaponDef {
            id: "spellblade".to_string(),
            name: "Spellblade".to_string(),
            weapon_type: WeaponType::Sword,
            power: 6,
            hit_bonus: 3,
            crit_bonus: 1,
            rarity: ItemRarity::Uncommon,
            stat_bonus: StatModifier {
                str_: 1,
                int_: 1,
                ..Default::default()
            },
        },
    };

    let body = match class_type {
        CharacterClassType::Warrior | CharacterClassType::Knight => ArmorDef {
            id: "chain_mail".to_string(),
            name: "Chain Mail".to_string(),
            armor_type: ArmorType::Mail,
            slot: ArmorSlot::Body,
            defense: 6,
            magic_resist: 1,
            rarity: ItemRarity::Common,
            stat_bonus: StatModifier {
                vit: 1,
                ..Default::default()
            },
        },
        CharacterClassType::Ranger | CharacterClassType::Rogue => ArmorDef {
            id: "leather_vest".to_string(),
            name: "Leather Vest".to_string(),
            armor_type: ArmorType::Leather,
            slot: ArmorSlot::Body,
            defense: 4,
            magic_resist: 1,
            rarity: ItemRarity::Common,
            stat_bonus: StatModifier {
                agi: 1,
                ..Default::default()
            },
        },
        _ => ArmorDef {
            id: "traveler_robe".to_string(),
            name: "Traveler Robe".to_string(),
            armor_type: ArmorType::Robe,
            slot: ArmorSlot::Body,
            defense: 2,
            magic_resist: 3,
            rarity: ItemRarity::Common,
            stat_bonus: StatModifier {
                int_: 1,
                mp: 1,
                ..Default::default()
            },
        },
    };

    EquipmentLoadout {
        weapon: Some(basic_weapon),
        body: Some(body),
        ..Default::default()
    }
}

fn class_starter_items(class_type: CharacterClassType) -> Vec<InventoryEntry> {
    match class_type {
        CharacterClassType::Ranger => vec![InventoryEntry {
            id: "arrow_bundle".to_string(),
            category: ItemCategory::Material,
            item: ItemType::Consumable(ItemDef {
                id: "arrow_bundle".to_string(),
                heal_hp: 0,
                heal_mp: 0,
            }),
            quantity: 1,
        }],
        CharacterClassType::Cleric | CharacterClassType::Mage | CharacterClassType::Necromancer => {
            vec![InventoryEntry {
                id: "mana_tincture".to_string(),
                category: ItemCategory::Consumable,
                item: ItemType::Consumable(ItemDef {
                    id: "mana_tincture".to_string(),
                    heal_hp: 0,
                    heal_mp: 25,
                }),
                quantity: 1,
            }]
        }
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hero() -> Fighter {
        Fighter {
            id: "hero".to_string(),
            name: "Hero".to_string(),
            stats: Stats {
                hp: 30,
                max_hp: 30,
                mp: 10,
                max_mp: 10,
                str_: 12,
                agi: 8,
                vit: 7,
                int_: 6,
                luk: 5,
            },
            statuses: vec![],
        }
    }

    fn slime() -> Fighter {
        Fighter {
            id: "slime".to_string(),
            name: "Slime".to_string(),
            stats: Stats {
                hp: 12,
                max_hp: 12,
                mp: 0,
                max_mp: 0,
                str_: 5,
                agi: 3,
                vit: 3,
                int_: 1,
                luk: 1,
            },
            statuses: vec![],
        }
    }

    #[test]
    fn world_movement_and_trigger() {
        let mut world = sample_world_20x15();
        let from = Coord2 { x: 7, y: 10 };
        let (to, events) = world.try_move(from, Direction2D::Right);
        assert_eq!(to, Coord2 { x: 8, y: 10 });
        assert!(
            events
                .iter()
                .any(|e| matches!(e, WorldEvent::ChestLoot { item_id } if item_id == "herb"))
        );
        assert_eq!(world.quest_flags.get("found_herb"), Some(&true));
    }

    #[test]
    fn encounter_weight_pick() {
        let table = EncounterTable {
            packs: vec![
                EncounterPack {
                    id: "slime_pack".to_string(),
                    terrain: Terrain::Grass,
                    weight: 80,
                    monsters: vec![slime()],
                },
                EncounterPack {
                    id: "wolf_pack".to_string(),
                    terrain: Terrain::Grass,
                    weight: 20,
                    monsters: vec![slime(), slime()],
                },
            ],
        };
        let p = table.pick_for_terrain(Terrain::Grass, 10).unwrap();
        assert_eq!(p.id, "slime_pack");
        let p2 = table.pick_for_terrain(Terrain::Grass, 95).unwrap();
        assert_eq!(p2.id, "wolf_pack");
    }

    #[test]
    fn battle_attack_and_spell() {
        let mut party = Party {
            members: vec![hero()],
            inventory: HashMap::from([(String::from("herb"), 1)]),
            gold: 0,
        };
        party.members[0].stats.hp = 20;
        let mut battle = BattleState::new(party, vec![slime()]);
        let events = battle.apply_party_action(BattleAction::Attack {
            actor: 0,
            target: 0,
        });
        assert!(!events.is_empty());

        let heal = SpellDef {
            id: "heal".to_string(),
            mp_cost: 2,
            power: 5,
            kind: SpellKind::Heal,
        };
        let hp_before = battle.party.members[0].stats.hp;
        let _ = battle.apply_party_action(BattleAction::CastSpell {
            actor: 0,
            target: 0,
            spell: heal,
        });
        assert!(battle.party.members[0].stats.hp >= hp_before);
    }

    #[test]
    fn settings_scale_encounters() {
        let settings = GameSettings {
            encounter_scale_percent: 50,
            ..Default::default()
        };
        assert_eq!(settings.effective_encounter_rate(40), 20);
        assert_eq!(settings.effective_encounter_rate(100), 50);
    }

    #[test]
    fn engine_step_move_and_autosave() {
        let settings = GameSettings {
            auto_save_every_turns: 1,
            encounter_scale_percent: 0,
            ..Default::default()
        };
        let world = sample_world_20x15();
        let encounter_table = EncounterTable { packs: vec![] };
        let party = Party {
            members: vec![hero()],
            inventory: HashMap::new(),
            gold: 0,
        };
        let mut engine = RpgGameEngine::new(
            settings,
            world,
            encounter_table,
            party,
            Coord2 { x: 2, y: 2 },
        );
        let events = engine.step_move(Direction2D::Right, 0, 0);
        assert!(events.iter().any(|e| matches!(e, EngineEvent::AutoSaved { .. })));
        assert!(events.iter().any(|e| matches!(e, EngineEvent::NoEncounter)));
        assert_eq!(engine.player_pos, Coord2 { x: 3, y: 2 });
    }

    #[test]
    fn engine_boss_run_blocked_by_setting() {
        let settings = GameSettings {
            allow_run_from_boss: false,
            ..Default::default()
        };
        let world = sample_world_20x15();
        let encounter_table = EncounterTable {
            packs: vec![EncounterPack {
                id: "boss_king_slime".to_string(),
                terrain: Terrain::Grass,
                weight: 1,
                monsters: vec![slime()],
            }],
        };
        let party = Party {
            members: vec![hero()],
            inventory: HashMap::new(),
            gold: 0,
        };
        let mut engine = RpgGameEngine::new(
            settings,
            world,
            encounter_table,
            party,
            Coord2 { x: 2, y: 2 },
        );

        let _ = engine.step_move(Direction2D::Right, 0, 0);
        let out = engine.apply_party_battle_action(
            BattleAction::Run {
                actor: 0,
                run_roll: 99,
            },
            None,
        );
        assert!(out.iter().any(|e| matches!(
            e,
            EngineEvent::Battle(BattleEvent::FailedRun)
        )));
    }

    #[test]
    fn character_creation_builds_profile_and_fighter() {
        let req = CharacterCreationRequest {
            id: "hero_001".to_string(),
            name: "Arin".to_string(),
            race: CharacterRace::Human,
            class_type: CharacterClassType::Warrior,
            bonus_points: 4,
            allocation: StatModifier {
                str_: 2,
                vit: 2,
                ..Default::default()
            },
        };
        let profile = create_character(req).unwrap();
        assert_eq!(profile.level, 1);
        assert!(profile.derived_stats.str_ >= profile.base_stats.str_);
        assert!(profile.equipment.weapon.is_some());
        let fighter = profile.as_fighter();
        assert_eq!(fighter.name, "Arin");
        assert!(fighter.stats.max_hp > 0);
    }

    #[test]
    fn character_creation_rejects_invalid_points() {
        let req = CharacterCreationRequest {
            id: "bad_001".to_string(),
            name: "Bad".to_string(),
            race: CharacterRace::Elf,
            class_type: CharacterClassType::Mage,
            bonus_points: 5,
            allocation: StatModifier {
                int_: 2,
                agi: 1,
                ..Default::default()
            },
        };
        let err = create_character(req).err().unwrap();
        assert_eq!(err, CharacterCreationError::InvalidPointAllocation);
    }

    #[test]
    fn menu_navigates_submenus_and_breadcrumbs() {
        let mut menu = build_main_menu(true);
        menu.move_next();
        menu.move_next();
        assert_eq!(menu.current_item().map(|m| m.id.as_str()), Some("options"));
        let action = menu.enter();
        assert!(action.is_none());
        let crumbs = menu.breadcrumbs();
        assert_eq!(crumbs, vec!["Main Menu".to_string(), "Options".to_string()]);
        assert_eq!(menu.current_item().map(|m| m.id.as_str()), Some("video"));
        let opened = menu.enter();
        assert_eq!(
            opened,
            Some(MenuAction::Custom("open_video_options".to_string()))
        );
        assert!(menu.back());
        assert_eq!(menu.current_item().map(|m| m.id.as_str()), Some("new_game"));
    }

    #[test]
    fn menu_skips_disabled_items() {
        let mut menu = build_main_menu(false);
        assert_eq!(menu.current_item().map(|m| m.id.as_str()), Some("new_game"));
        menu.move_next();
        assert_eq!(menu.current_item().map(|m| m.id.as_str()), Some("options"));
    }

    #[test]
    fn battle_menu_run_disabled() {
        let mut menu = build_battle_menu(false);
        menu.move_next();
        menu.move_next();
        menu.move_next();
        assert_eq!(menu.current_item().map(|m| m.id.as_str()), Some("attack"));
    }

    #[test]
    fn engine_menu_api_pause_resume() {
        let settings = GameSettings::default();
        let world = sample_world_20x15();
        let party = Party {
            members: vec![hero()],
            inventory: HashMap::new(),
            gold: 0,
        };
        let mut engine = RpgGameEngine::new(
            settings,
            world,
            EncounterTable { packs: vec![] },
            party,
            Coord2 { x: 2, y: 2 },
        );
        engine.open_pause_menu();
        let view = engine.menu_view().unwrap();
        assert_eq!(view.title, "Pause Menu");

        let resp = engine.handle_menu_input(MenuInput::Confirm);
        assert!(resp
            .events
            .iter()
            .any(|e| matches!(e, EngineEvent::MenuAction(MenuAction::ResumeGame))));
        assert!(resp.menu.is_none());
    }

    #[test]
    fn api_step_move_returns_view() {
        let settings = GameSettings {
            encounter_scale_percent: 0,
            ..Default::default()
        };
        let world = sample_world_20x15();
        let party = Party {
            members: vec![hero()],
            inventory: HashMap::new(),
            gold: 0,
        };
        let mut engine = RpgGameEngine::new(
            settings,
            world,
            EncounterTable { packs: vec![] },
            party,
            Coord2 { x: 2, y: 2 },
        );
        engine.open_pause_menu();
        let resp = engine.api_step_move(Direction2D::Right, 0, 0);
        assert!(resp.menu.is_some());
        assert!(resp
            .events
            .iter()
            .any(|e| matches!(e, EngineEvent::NoEncounter)));
    }

    #[test]
    fn tool_menu_shows_registered_tools() {
        let mut engine = RpgGameEngine::new(
            GameSettings::default(),
            sample_world_20x15(),
            EncounterTable { packs: vec![] },
            Party {
                members: vec![hero()],
                inventory: HashMap::new(),
                gold: 0,
            },
            Coord2 { x: 2, y: 2 },
        );
        engine.register_tool(ToolDef {
            id: "torch".to_string(),
            name: "Torch".to_string(),
            category: ToolCategory::Survival,
            tool_type: ToolType::Torch,
            class: ToolClass::Basic,
            target: ToolTarget::SelfTarget,
            power: 1,
            cooldown_turns: 1,
            max_charges: 3,
            menu_visible: true,
            description: "Light source".to_string(),
        });
        engine.open_tool_menu();
        let view = engine.menu_view().unwrap();
        assert_eq!(view.title, "Tools");
        assert!(view.items.iter().any(|i| i.id == "tool_torch"));
    }

    #[test]
    fn tool_use_consumes_charge_and_sets_cooldown() {
        let mut engine = RpgGameEngine::new(
            GameSettings::default(),
            sample_world_20x15(),
            EncounterTable { packs: vec![] },
            Party {
                members: vec![hero()],
                inventory: HashMap::new(),
                gold: 0,
            },
            Coord2 { x: 2, y: 2 },
        );
        engine.world.turn = 10;
        engine.register_tool(ToolDef {
            id: "compass".to_string(),
            name: "Compass".to_string(),
            category: ToolCategory::Utility,
            tool_type: ToolType::Compass,
            class: ToolClass::Basic,
            target: ToolTarget::SelfTarget,
            power: 0,
            cooldown_turns: 2,
            max_charges: 2,
            menu_visible: true,
            description: "Find direction".to_string(),
        });
        let r = engine.use_tool_by_id(
            "compass",
            ToolUseContext {
                position: engine.player_pos,
                terrain: engine.world.tile(engine.player_pos).map(|t| t.terrain),
                in_battle: false,
            },
        );
        assert!(r.success);
        assert!(r
            .events
            .iter()
            .any(|e| matches!(e, EngineEvent::ToolUsed { tool_id } if tool_id == "compass")));

        let r2 = engine.use_tool_by_id(
            "compass",
            ToolUseContext {
                position: engine.player_pos,
                terrain: engine.world.tile(engine.player_pos).map(|t| t.terrain),
                in_battle: false,
            },
        );
        assert!(!r2.success);
        assert_eq!(r2.cooldown_remaining, 2);
    }

    #[test]
    fn audio_creator_menu_and_preview_event() {
        let mut engine = RpgGameEngine::new(
            GameSettings::default(),
            sample_world_20x15(),
            EncounterTable { packs: vec![] },
            Party {
                members: vec![hero()],
                inventory: HashMap::new(),
                gold: 0,
            },
            Coord2 { x: 2, y: 2 },
        );
        engine.register_sfx(SoundEffectDef {
            id: "ui_click".to_string(),
            name: "UI Click".to_string(),
            category: SoundCategory::Ui,
            file: "audio/ui_click.ogg".to_string(),
            default_volume_percent: 90,
            pitch_variance_percent: 0,
            looped: false,
        });
        engine.open_audio_creator_menu();
        let view = engine.menu_view().unwrap();
        assert_eq!(view.title, "Audio Creator");
        assert!(view.items.iter().any(|i| i.id == "preview_ui_click"));

        let ev = engine.preview_sfx("ui_click");
        assert!(matches!(
            ev,
            EngineEvent::Audio(AudioEvent::PlaySfx { id, .. }) if id == "ui_click"
        ));
    }

    #[test]
    fn creator_top_menu_contains_audio_creator() {
        let menu = build_creator_top_menu();
        let labels: Vec<_> = menu.current_items().iter().map(|i| i.label.clone()).collect();
        assert!(labels.contains(&"Audio Creator".to_string()));
    }

    #[test]
    fn world_creation_continent_country_zone_and_challenges() {
        let mut engine = RpgGameEngine::new(
            GameSettings::default(),
            sample_world_20x15(),
            EncounterTable { packs: vec![] },
            Party {
                members: vec![hero()],
                inventory: HashMap::new(),
                gold: 0,
            },
            Coord2 { x: 2, y: 2 },
        );

        let biome = BiomeProfile {
            biome_type: BiomeType::Woodland,
            biome_subtype: BiomeSubtype::DenseForest,
            climate: ClimateBand::Temperate,
            danger_level: 3,
            encounter_bonus_percent: 10,
            move_cost: 2,
        };

        assert!(engine
            .create_continent("cont_auria", "Auria", ClimateBand::Temperate)
            .is_ok());
        assert!(engine
            .create_country(
                "cty_lymere",
                "Lymere",
                "cont_auria",
                "Lymere City",
                vec!["trade".to_string()]
            )
            .is_ok());
        assert!(engine
            .create_zone(
                "cont_auria",
                "cty_lymere",
                Zone {
                    id: "zone_greenglen".to_string(),
                    name: "Greenglen".to_string(),
                    class: ZoneClass::Wilderness,
                    subtype: ZoneSubtype::Forest,
                    environment: ZoneEnvironment::Surface,
                    area: RectArea {
                        min: Coord2 { x: 0, y: 0 },
                        max: Coord2 { x: 100, y: 100 },
                    },
                    default_biome: biome.clone(),
                    encounter_multiplier_percent: 120,
                    level_min: 1,
                    level_max: 8,
                    sub_zones: vec![],
                }
            )
            .is_ok());
        assert!(engine
            .create_sub_zone(
                "zone_greenglen",
                SubZone {
                    id: "sub_ancient_grove".to_string(),
                    name: "Ancient Grove".to_string(),
                    class: ZoneClass::Landmark,
                    subtype: ZoneSubtype::Temple,
                    environment: ZoneEnvironment::Surface,
                    area: RectArea {
                        min: Coord2 { x: 20, y: 20 },
                        max: Coord2 { x: 40, y: 40 },
                    },
                    biome: biome.clone(),
                    encounter_multiplier_percent: 140,
                    tags: vec!["boss_hint".to_string()],
                },
            )
            .is_ok());

        assert!(engine
            .create_dungeon(DungeonBlueprint {
                id: "dng_rootcrypt".to_string(),
                name: "Rootcrypt".to_string(),
                zone_id: "zone_greenglen".to_string(),
                recommended_level_min: 4,
                recommended_level_max: 10,
                tier: ChallengeTier::Veteran,
                floors: vec![DungeonFloor {
                    index: 1,
                    biome: biome.clone(),
                    encounter_multiplier_percent: 130,
                    has_checkpoint: false,
                }],
                boss_id: "boss_rootwarden".to_string(),
            })
            .is_ok());
        assert!(engine
            .create_raid(RaidBlueprint {
                id: "raid_skyaltar".to_string(),
                name: "Sky Altar".to_string(),
                zone_id: "zone_greenglen".to_string(),
                tier: ChallengeTier::Elite,
                recommended_party_size: 4,
                wings: vec![RaidWing {
                    id: "wing_a".to_string(),
                    name: "Approach".to_string(),
                    encounter_count: 3,
                    boss_ids: vec!["boss_a".to_string()],
                }],
            })
            .is_ok());
        assert!(engine
            .create_trial(TrialBlueprint {
                id: "trial_relic".to_string(),
                name: "Relic Trial".to_string(),
                zone_id: "zone_greenglen".to_string(),
                tier: ChallengeTier::Novice,
                time_limit_seconds: Some(600),
                objectives: vec![TrialObjective::CollectRelic("relic_sun".to_string())],
            })
            .is_ok());
        assert!(engine
            .create_tower(TowerBlueprint {
                id: "tower_verdant".to_string(),
                name: "Verdant Tower".to_string(),
                zone_id: "zone_greenglen".to_string(),
                tier: ChallengeTier::Mythic,
                stages: vec![TowerStage {
                    stage: 1,
                    enemy_pack_id: "pack_slimes".to_string(),
                    reward_table_id: "reward_t1".to_string(),
                    modifier_tags: vec!["regen".to_string()],
                }],
                infinite_after_stage: Some(50),
            })
            .is_ok());

        assert!(engine.atlas.continents.contains_key("cont_auria"));
        assert!(engine.atlas.countries.contains_key("cty_lymere"));
        assert!(engine.atlas.zones.contains_key("zone_greenglen"));
        assert!(engine.atlas.dungeons.contains_key("dng_rootcrypt"));
        assert!(engine.atlas.raids.contains_key("raid_skyaltar"));
        assert!(engine.atlas.trials.contains_key("trial_relic"));
        assert!(engine.atlas.towers.contains_key("tower_verdant"));
    }

    #[test]
    fn world_creation_rejects_missing_zone_reference() {
        let mut atlas = WorldAtlas::default();
        let err = atlas
            .add_dungeon(DungeonBlueprint {
                id: "dng_missing".to_string(),
                name: "Missing".to_string(),
                zone_id: "zone_none".to_string(),
                recommended_level_min: 1,
                recommended_level_max: 2,
                tier: ChallengeTier::Novice,
                floors: vec![DungeonFloor {
                    index: 1,
                    biome: BiomeProfile {
                        biome_type: BiomeType::Grassland,
                        biome_subtype: BiomeSubtype::OpenField,
                        climate: ClimateBand::Temperate,
                        danger_level: 1,
                        encounter_bonus_percent: 0,
                        move_cost: 1,
                    },
                    encounter_multiplier_percent: 100,
                    has_checkpoint: false,
                }],
                boss_id: "none".to_string(),
            })
            .err()
            .unwrap();

        assert!(matches!(err, WorldCreationError::MissingReference(_)));
    }
}
