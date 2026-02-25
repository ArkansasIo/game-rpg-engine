use crate::rpg2d::Direction2D;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RigPart {
    Head,
    Torso,
    Arms,
    Legs,
    Hair,
    Accessory,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PixelFrame {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

impl PixelFrame {
    pub fn new_filled(width: usize, height: usize, palette_index: u8) -> Self {
        Self {
            width,
            height,
            pixels: vec![palette_index; width.saturating_mul(height)],
        }
    }

    fn index(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.width && y < self.height {
            Some(y * self.width + x)
        } else {
            None
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<u8> {
        let i = self.index(x, y)?;
        self.pixels.get(i).copied()
    }

    pub fn set(&mut self, x: usize, y: usize, palette_index: u8) -> bool {
        let Some(i) = self.index(x, y) else {
            return false;
        };
        if let Some(px) = self.pixels.get_mut(i) {
            *px = palette_index;
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectionFrames {
    pub up: PixelFrame,
    pub down: PixelFrame,
    pub left: PixelFrame,
    pub right: PixelFrame,
}

impl DirectionFrames {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            up: PixelFrame::new_filled(width, height, 0),
            down: PixelFrame::new_filled(width, height, 0),
            left: PixelFrame::new_filled(width, height, 0),
            right: PixelFrame::new_filled(width, height, 0),
        }
    }

    pub fn get(&self, direction: Direction2D) -> &PixelFrame {
        match direction {
            Direction2D::Up => &self.up,
            Direction2D::Down => &self.down,
            Direction2D::Left => &self.left,
            Direction2D::Right => &self.right,
        }
    }

    pub fn get_mut(&mut self, direction: Direction2D) -> &mut PixelFrame {
        match direction {
            Direction2D::Up => &mut self.up,
            Direction2D::Down => &mut self.down,
            Direction2D::Left => &mut self.left,
            Direction2D::Right => &mut self.right,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RigLayer {
    pub part: RigPart,
    pub z_index: i32,
    pub locked: bool,
    pub frames: DirectionFrames,
}

impl RigLayer {
    pub fn new(part: RigPart, z_index: i32, width: usize, height: usize) -> Self {
        Self {
            part,
            z_index,
            locked: false,
            frames: DirectionFrames::new(width, height),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DirectionalOffsets {
    pub up: (i32, i32),
    pub down: (i32, i32),
    pub left: (i32, i32),
    pub right: (i32, i32),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RigJoint {
    pub part: RigPart,
    pub pivot: (i32, i32),
    pub directional_offset: DirectionalOffsets,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PixelAvatarRig {
    pub id: String,
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub layers: Vec<RigLayer>,
    pub joints: Vec<RigJoint>,
}

impl PixelAvatarRig {
    pub fn new(id: impl Into<String>, name: impl Into<String>, width: usize, height: usize) -> Self {
        let layers = vec![
            RigLayer::new(RigPart::Legs, 10, width, height),
            RigLayer::new(RigPart::Torso, 20, width, height),
            RigLayer::new(RigPart::Arms, 30, width, height),
            RigLayer::new(RigPart::Head, 40, width, height),
            RigLayer::new(RigPart::Hair, 50, width, height),
            RigLayer::new(RigPart::Accessory, 60, width, height),
        ];
        Self {
            id: id.into(),
            name: name.into(),
            width,
            height,
            layers,
            joints: default_humanoid_joints(),
        }
    }

    pub fn set_locked(&mut self, part: RigPart, locked: bool) -> bool {
        let Some(layer) = self.layers.iter_mut().find(|l| l.part == part) else {
            return false;
        };
        layer.locked = locked;
        true
    }

    pub fn paint(
        &mut self,
        part: RigPart,
        direction: Direction2D,
        x: usize,
        y: usize,
        palette_index: u8,
    ) -> bool {
        let Some(layer) = self.layers.iter_mut().find(|l| l.part == part) else {
            return false;
        };
        if layer.locked {
            return false;
        }
        layer.frames.get_mut(direction).set(x, y, palette_index)
    }

    pub fn erase(&mut self, part: RigPart, direction: Direction2D, x: usize, y: usize) -> bool {
        self.paint(part, direction, x, y, 0)
    }

    pub fn mirror_left_to_right(&mut self, part: RigPart) -> bool {
        let Some(layer) = self.layers.iter_mut().find(|l| l.part == part) else {
            return false;
        };
        let left = layer.frames.left.clone();
        let right = &mut layer.frames.right;
        for y in 0..left.height {
            for x in 0..left.width {
                if let Some(v) = left.get(x, y) {
                    let rx = left.width - 1 - x;
                    let _ = right.set(rx, y, v);
                }
            }
        }
        true
    }

    pub fn compose(&self, direction: Direction2D) -> PixelFrame {
        let mut out = PixelFrame::new_filled(self.width, self.height, 0);
        let mut order: Vec<&RigLayer> = self.layers.iter().collect();
        order.sort_by_key(|l| l.z_index);

        for layer in order {
            let frame = layer.frames.get(direction);
            for y in 0..self.height {
                for x in 0..self.width {
                    if let Some(src) = frame.get(x, y)
                        && src != 0
                    {
                        let _ = out.set(x, y, src);
                    }
                }
            }
        }
        out
    }
}

pub fn default_humanoid_joints() -> Vec<RigJoint> {
    let walk_offsets = DirectionalOffsets {
        up: (0, -1),
        down: (0, 1),
        left: (-1, 0),
        right: (1, 0),
    };

    vec![
        RigJoint {
            part: RigPart::Torso,
            pivot: (8, 9),
            directional_offset: DirectionalOffsets::default(),
        },
        RigJoint {
            part: RigPart::Head,
            pivot: (8, 5),
            directional_offset: DirectionalOffsets::default(),
        },
        RigJoint {
            part: RigPart::Arms,
            pivot: (8, 10),
            directional_offset: walk_offsets,
        },
        RigJoint {
            part: RigPart::Legs,
            pivot: (8, 14),
            directional_offset: walk_offsets,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paint_and_compose_four_directions() {
        let mut rig = PixelAvatarRig::new("a", "Hero", 16, 16);
        assert!(rig.paint(RigPart::Head, Direction2D::Down, 8, 4, 3));
        assert!(rig.paint(RigPart::Torso, Direction2D::Down, 8, 8, 6));
        let out = rig.compose(Direction2D::Down);
        assert_eq!(out.get(8, 4), Some(3));
        assert_eq!(out.get(8, 8), Some(6));
    }

    #[test]
    fn mirror_left_to_right() {
        let mut rig = PixelAvatarRig::new("b", "Hero2", 8, 8);
        assert!(rig.paint(RigPart::Accessory, Direction2D::Left, 0, 0, 9));
        assert!(rig.mirror_left_to_right(RigPart::Accessory));
        let right = rig.compose(Direction2D::Right);
        assert_eq!(right.get(7, 0), Some(9));
    }
}
