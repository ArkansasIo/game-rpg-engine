use crate::grid2d::Coord2;
use crate::rpg2d::BiomeType;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PropArchetype {
    Tree,
    Rock,
    Crate,
    Torch,
    Fence,
    Column,
    Ruin,
    Mushroom,
    Banner,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProceduralPropDef {
    pub id: String,
    pub archetype: PropArchetype,
    pub size: (u8, u8),
    pub passable: bool,
    pub durability: i32,
    pub light_radius: u8,
    pub value: u32,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlacedProp {
    pub coord: Coord2,
    pub def: ProceduralPropDef,
    pub rotation_quadrants: u8,
    pub variation: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropGeneratorConfig {
    pub seed: u64,
    pub density_percent: u8,
    pub max_props: usize,
    pub allow_blocking: bool,
}

impl Default for PropGeneratorConfig {
    fn default() -> Self {
        Self {
            seed: 1,
            density_percent: 8,
            max_props: 512,
            allow_blocking: true,
        }
    }
}

pub struct ProceduralPropGenerator {
    pub config: PropGeneratorConfig,
}

impl ProceduralPropGenerator {
    pub fn new(config: PropGeneratorConfig) -> Self {
        Self { config }
    }

    pub fn generate_for_zone(
        &self,
        width: usize,
        height: usize,
        biome: BiomeType,
        blocked_cells: &HashSet<Coord2>,
    ) -> Vec<PlacedProp> {
        let mut out = Vec::new();
        let table = biome_prop_table(biome);
        if width == 0 || height == 0 || table.is_empty() {
            return out;
        }

        for y in 0..height as i32 {
            for x in 0..width as i32 {
                if out.len() >= self.config.max_props {
                    return out;
                }
                let c = Coord2 {
                    x: x as usize,
                    y: y as usize,
                };
                if blocked_cells.contains(&c) {
                    continue;
                }
                let roll = hash01(self.config.seed, x, y);
                if roll > f32::from(self.config.density_percent) / 100.0 {
                    continue;
                }
                let idx = ((hash32(self.config.seed, x, y) as usize) % table.len()).min(table.len() - 1);
                let mut def = table[idx].clone();
                if !self.config.allow_blocking {
                    def.passable = true;
                }
                out.push(PlacedProp {
                    coord: c,
                    rotation_quadrants: (hash32(self.config.seed ^ 0xA5A5, x, y) % 4) as u8,
                    variation: (hash32(self.config.seed ^ 0x5A5A, x, y) % 8) as u8,
                    def,
                });
            }
        }
        out
    }
}

pub fn biome_prop_table(biome: BiomeType) -> Vec<ProceduralPropDef> {
    let mut out = vec![];
    match biome {
        BiomeType::Grassland | BiomeType::Woodland => {
            out.push(prop("prop_tree_small", PropArchetype::Tree, false, 24, 0, 8));
            out.push(prop("prop_rock_round", PropArchetype::Rock, false, 40, 0, 5));
            out.push(prop("prop_fence", PropArchetype::Fence, false, 30, 0, 3));
        }
        BiomeType::Desert | BiomeType::Canyon => {
            out.push(prop("prop_rock_tall", PropArchetype::Rock, false, 55, 0, 7));
            out.push(prop("prop_banner_torn", PropArchetype::Banner, true, 10, 0, 2));
        }
        BiomeType::Cavern | BiomeType::Volcanic => {
            out.push(prop("prop_torch_wall", PropArchetype::Torch, true, 12, 4, 9));
            out.push(prop("prop_column_cracked", PropArchetype::Column, false, 70, 0, 10));
            out.push(prop("prop_ruin_shard", PropArchetype::Ruin, true, 18, 0, 6));
        }
        _ => {
            out.push(prop("prop_crate", PropArchetype::Crate, false, 20, 0, 4));
            out.push(prop("prop_mushroom", PropArchetype::Mushroom, true, 8, 0, 1));
        }
    }
    out
}

fn prop(
    id: &str,
    archetype: PropArchetype,
    passable: bool,
    durability: i32,
    light_radius: u8,
    value: u32,
) -> ProceduralPropDef {
    ProceduralPropDef {
        id: id.to_string(),
        archetype,
        size: (1, 1),
        passable,
        durability,
        light_radius,
        value,
        tags: vec![],
    }
}

fn hash32(seed: u64, x: i32, y: i32) -> u32 {
    let mut v = seed
        ^ (x as u64).wrapping_mul(0x9E3779B185EBCA87)
        ^ (y as u64).wrapping_mul(0xC2B2AE3D27D4EB4F);
    v ^= v >> 33;
    v = v.wrapping_mul(0x62A9D9ED799705F5);
    v ^= v >> 28;
    v as u32
}

fn hash01(seed: u64, x: i32, y: i32) -> f32 {
    let h = hash32(seed, x, y);
    (h as f32) / (u32::MAX as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_generation() {
        let generator = ProceduralPropGenerator::new(PropGeneratorConfig {
            seed: 1337,
            density_percent: 100,
            max_props: 10,
            allow_blocking: true,
        });
        let a = generator.generate_for_zone(4, 4, BiomeType::Grassland, &HashSet::new());
        let b = generator.generate_for_zone(4, 4, BiomeType::Grassland, &HashSet::new());
        assert_eq!(a, b);
        assert!(!a.is_empty());
    }
}
