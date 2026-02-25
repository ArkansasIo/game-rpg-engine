use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::grid2d::{Coord2, Grid2D};

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
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct World2D {
    pub map: Grid2D<TileDef>,
    pub triggers: HashMap<Coord2, Trigger>,
    pub quest_flags: HashMap<String, bool>,
    pub turn: u64,
}

impl World2D {
    pub fn new(map: Grid2D<TileDef>) -> Self {
        Self {
            map,
            triggers: HashMap::new(),
            quest_flags: HashMap::new(),
            turn: 0,
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
}

impl BattleState {
    pub fn new(party: Party, enemies: Vec<Fighter>) -> Self {
        Self {
            party,
            enemies,
            result: BattleResult::Ongoing,
            turn: 1,
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
}
