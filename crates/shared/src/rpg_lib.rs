use crate::rpg2d::{GameSettings, Stats};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageType {
    Physical,
    Magical,
    TrueDamage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HitOutcome {
    pub hit: bool,
    pub crit: bool,
    pub damage: i32,
}

pub fn compute_hit_chance(attacker_agi: i32, defender_agi: i32, bonus: i32) -> u8 {
    let base = 78 + (attacker_agi - defender_agi) / 2 + bonus;
    base.clamp(5, 99) as u8
}

pub fn compute_crit_chance(attacker_luk: i32, weapon_crit_bonus: i32) -> u8 {
    (3 + attacker_luk / 4 + weapon_crit_bonus).clamp(0, 60) as u8
}

pub fn resolve_attack(
    attacker: &Stats,
    defender: &mut Stats,
    power: i32,
    damage_type: DamageType,
    settings: &GameSettings,
    roll_hit: u32,
    roll_crit: u32,
) -> HitOutcome {
    let hit_chance = compute_hit_chance(attacker.agi, defender.agi, 0);
    let hit = (roll_hit % 100) < u32::from(hit_chance);
    if !hit {
        return HitOutcome {
            hit: false,
            crit: false,
            damage: 0,
        };
    }

    let crit_chance = compute_crit_chance(attacker.luk, 0);
    let crit = (roll_crit % 100) < u32::from(crit_chance);
    let mut damage = match damage_type {
        DamageType::Physical => (power + attacker.str_ - defender.vit / 2).max(1),
        DamageType::Magical => (power + attacker.int_ - defender.int_ / 3).max(1),
        DamageType::TrueDamage => power.max(1),
    };
    if crit {
        damage = damage.saturating_mul(150) / 100;
    }
    damage = settings.physical_damage_with_difficulty(damage);
    defender.hp = defender.hp.saturating_sub(damage);
    defender.clamp_resources();

    HitOutcome {
        hit: true,
        crit,
        damage,
    }
}

pub fn xp_to_next_level(level: u32) -> u32 {
    let l = level.max(1);
    100u32
        .saturating_add(l.saturating_mul(l).saturating_mul(25))
        .saturating_add(l.saturating_mul(35))
}

pub fn level_from_total_xp(total_xp: u64) -> u32 {
    let mut level = 1u32;
    let mut remaining = total_xp;
    loop {
        let need = u64::from(xp_to_next_level(level));
        if remaining < need {
            return level;
        }
        remaining -= need;
        level = level.saturating_add(1);
        if level >= 99 {
            return 99;
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct CooldownBook {
    pub turns_left: HashMap<String, u32>,
}

impl CooldownBook {
    pub fn set(&mut self, id: impl Into<String>, turns: u32) {
        self.turns_left.insert(id.into(), turns);
    }

    pub fn is_ready(&self, id: &str) -> bool {
        self.turns_left.get(id).copied().unwrap_or(0) == 0
    }

    pub fn tick(&mut self) {
        for turns in self.turns_left.values_mut() {
            *turns = turns.saturating_sub(1);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct ThreatTable {
    pub threat: HashMap<String, i32>,
}

impl ThreatTable {
    pub fn add_threat(&mut self, id: impl Into<String>, amount: i32) {
        let e = self.threat.entry(id.into()).or_insert(0);
        *e = e.saturating_add(amount);
    }

    pub fn highest_target(&self) -> Option<&str> {
        self.threat
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, _)| k.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LootEntry {
    pub item_id: String,
    pub weight: u32,
    pub min: u32,
    pub max: u32,
}

pub fn roll_loot(entries: &[LootEntry], roll_pick: u32, roll_qty: u32) -> Option<(String, u32)> {
    let total: u32 = entries.iter().map(|e| e.weight).sum();
    if total == 0 {
        return None;
    }
    let mut cursor = roll_pick % total;
    let picked = entries.iter().find_map(|e| {
        if cursor < e.weight {
            Some(e)
        } else {
            cursor -= e.weight;
            None
        }
    })?;
    let span = picked.max.saturating_sub(picked.min).saturating_add(1);
    let qty = picked.min.saturating_add(roll_qty % span);
    Some((picked.item_id.clone(), qty.max(1)))
}

pub fn encounter_roll(
    steps_since_last: u32,
    base_rate_percent: u8,
    settings: &GameSettings,
    roll: u32,
) -> bool {
    let mut chance = settings.effective_encounter_rate(base_rate_percent);
    let ramp = (steps_since_last / 12) as u8;
    chance = chance.saturating_add(ramp).min(95);
    (roll % 100) < u32::from(chance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpg2d::Difficulty;

    #[test]
    fn damage_and_level_helpers_work() {
        let settings = GameSettings {
            difficulty: Difficulty::Normal,
            ..Default::default()
        };
        let atk = Stats {
            hp: 20,
            max_hp: 20,
            mp: 5,
            max_mp: 5,
            str_: 12,
            agi: 10,
            vit: 8,
            int_: 4,
            luk: 7,
        };
        let mut def = Stats {
            hp: 20,
            max_hp: 20,
            mp: 0,
            max_mp: 0,
            str_: 6,
            agi: 8,
            vit: 10,
            int_: 3,
            luk: 2,
        };
        let out = resolve_attack(&atk, &mut def, 5, DamageType::Physical, &settings, 0, 99);
        assert!(out.hit);
        assert!(out.damage > 0);
        assert!(def.hp < 20);
        assert!(xp_to_next_level(2) > xp_to_next_level(1));
        assert!(level_from_total_xp(0) >= 1);
    }

    #[test]
    fn loot_and_cooldown_helpers_work() {
        let mut cd = CooldownBook::default();
        cd.set("slash", 2);
        assert!(!cd.is_ready("slash"));
        cd.tick();
        cd.tick();
        assert!(cd.is_ready("slash"));

        let loot = vec![
            LootEntry {
                item_id: "herb".to_string(),
                weight: 10,
                min: 1,
                max: 2,
            },
            LootEntry {
                item_id: "ether".to_string(),
                weight: 1,
                min: 1,
                max: 1,
            },
        ];
        let rolled = roll_loot(&loot, 0, 1);
        assert!(rolled.is_some());
    }
}
