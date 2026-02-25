use crate::unreal_editor::{UeVec2, UeVec3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct UeMathLib;

impl UeMathLib {
    pub fn clamp(x: f32, min: f32, max: f32) -> f32 {
        x.clamp(min, max)
    }

    pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

    pub fn inverse_lerp(a: f32, b: f32, v: f32) -> f32 {
        let d = b - a;
        if d.abs() < f32::EPSILON {
            0.0
        } else {
            (v - a) / d
        }
    }

    pub fn map_range(value: f32, in_a: f32, in_b: f32, out_a: f32, out_b: f32) -> f32 {
        let t = Self::inverse_lerp(in_a, in_b, value);
        Self::lerp(out_a, out_b, t)
    }

    pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
        let t = Self::clamp(Self::inverse_lerp(edge0, edge1, x), 0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    pub fn sin_deg(d: f32) -> f32 {
        d.to_radians().sin()
    }
    pub fn cos_deg(d: f32) -> f32 {
        d.to_radians().cos()
    }
    pub fn tan_deg(d: f32) -> f32 {
        d.to_radians().tan()
    }
    pub fn atan2_deg(y: f32, x: f32) -> f32 {
        y.atan2(x).to_degrees()
    }

    pub fn vec2_add(a: UeVec2, b: UeVec2) -> UeVec2 {
        UeVec2 {
            x: a.x + b.x,
            y: a.y + b.y,
        }
    }
    pub fn vec2_sub(a: UeVec2, b: UeVec2) -> UeVec2 {
        UeVec2 {
            x: a.x - b.x,
            y: a.y - b.y,
        }
    }
    pub fn vec2_scale(a: UeVec2, s: f32) -> UeVec2 {
        UeVec2 {
            x: a.x * s,
            y: a.y * s,
        }
    }
    pub fn vec2_dot(a: UeVec2, b: UeVec2) -> f32 {
        a.x * b.x + a.y * b.y
    }
    pub fn vec2_length(a: UeVec2) -> f32 {
        (a.x * a.x + a.y * a.y).sqrt()
    }
    pub fn vec2_normalize(a: UeVec2) -> UeVec2 {
        let len = Self::vec2_length(a);
        if len <= f32::EPSILON {
            UeVec2::default()
        } else {
            Self::vec2_scale(a, 1.0 / len)
        }
    }
    pub fn vec2_rotate_deg(v: UeVec2, deg: f32) -> UeVec2 {
        let r = deg.to_radians();
        let c = r.cos();
        let s = r.sin();
        UeVec2 {
            x: v.x * c - v.y * s,
            y: v.x * s + v.y * c,
        }
    }

    pub fn vec3_add(a: UeVec3, b: UeVec3) -> UeVec3 {
        UeVec3 {
            x: a.x + b.x,
            y: a.y + b.y,
            z: a.z + b.z,
        }
    }
    pub fn vec3_sub(a: UeVec3, b: UeVec3) -> UeVec3 {
        UeVec3 {
            x: a.x - b.x,
            y: a.y - b.y,
            z: a.z - b.z,
        }
    }
    pub fn vec3_scale(a: UeVec3, s: f32) -> UeVec3 {
        UeVec3 {
            x: a.x * s,
            y: a.y * s,
            z: a.z * s,
        }
    }
    pub fn vec3_dot(a: UeVec3, b: UeVec3) -> f32 {
        a.x * b.x + a.y * b.y + a.z * b.z
    }
    pub fn vec3_cross(a: UeVec3, b: UeVec3) -> UeVec3 {
        UeVec3 {
            x: a.y * b.z - a.z * b.y,
            y: a.z * b.x - a.x * b.z,
            z: a.x * b.y - a.y * b.x,
        }
    }
    pub fn vec3_length(a: UeVec3) -> f32 {
        (a.x * a.x + a.y * a.y + a.z * a.z).sqrt()
    }
    pub fn vec3_normalize(a: UeVec3) -> UeVec3 {
        let len = Self::vec3_length(a);
        if len <= f32::EPSILON {
            UeVec3::default()
        } else {
            Self::vec3_scale(a, 1.0 / len)
        }
    }
    pub fn vec3_reflect(v: UeVec3, n: UeVec3) -> UeVec3 {
        let nn = Self::vec3_normalize(n);
        let d = Self::vec3_dot(v, nn);
        Self::vec3_sub(v, Self::vec3_scale(nn, 2.0 * d))
    }

    pub fn hash_noise_2d(seed: u32, x: f32, y: f32) -> f32 {
        let xi = x.floor() as i32;
        let yi = y.floor() as i32;
        let mut v = (seed as u64)
            ^ (xi as u64).wrapping_mul(0x9E3779B185EBCA87)
            ^ (yi as u64).wrapping_mul(0xC2B2AE3D27D4EB4F);
        v ^= v >> 33;
        v = v.wrapping_mul(0x62A9D9ED799705F5);
        ((v as u32) as f32) / (u32::MAX as f32)
    }

    pub fn fbm_noise_2d(seed: u32, x: f32, y: f32, octaves: u8) -> f32 {
        let mut freq = 1.0f32;
        let mut amp = 0.5f32;
        let mut sum = 0.0f32;
        let mut norm = 0.0f32;
        for i in 0..octaves.max(1) {
            sum += Self::hash_noise_2d(seed.wrapping_add(u32::from(i)), x * freq, y * freq) * amp;
            norm += amp;
            freq *= 2.0;
            amp *= 0.5;
        }
        if norm <= f32::EPSILON { 0.0 } else { sum / norm }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UeMathUnaryOp {
    Negate,
    Abs,
    Floor,
    Ceil,
    Round,
    Sqrt,
    SinDeg,
    CosDeg,
    TanDeg,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UeMathBinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Min,
    Max,
    Mod,
    Atan2Deg,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UeMathExpr {
    Const(f32),
    Var(String),
    Unary {
        op: UeMathUnaryOp,
        arg: Box<UeMathExpr>,
    },
    Binary {
        op: UeMathBinaryOp,
        lhs: Box<UeMathExpr>,
        rhs: Box<UeMathExpr>,
    },
    Select {
        cond: String,
        on_true: Box<UeMathExpr>,
        on_false: Box<UeMathExpr>,
    },
}

pub fn eval_math_expr(expr: &UeMathExpr, scalars: &HashMap<String, f32>, bools: &HashMap<String, bool>) -> Option<f32> {
    match expr {
        UeMathExpr::Const(v) => Some(*v),
        UeMathExpr::Var(name) => scalars.get(name).copied(),
        UeMathExpr::Unary { op, arg } => {
            let v = eval_math_expr(arg, scalars, bools)?;
            Some(match op {
                UeMathUnaryOp::Negate => -v,
                UeMathUnaryOp::Abs => v.abs(),
                UeMathUnaryOp::Floor => v.floor(),
                UeMathUnaryOp::Ceil => v.ceil(),
                UeMathUnaryOp::Round => v.round(),
                UeMathUnaryOp::Sqrt => v.max(0.0).sqrt(),
                UeMathUnaryOp::SinDeg => UeMathLib::sin_deg(v),
                UeMathUnaryOp::CosDeg => UeMathLib::cos_deg(v),
                UeMathUnaryOp::TanDeg => UeMathLib::tan_deg(v),
            })
        }
        UeMathExpr::Binary { op, lhs, rhs } => {
            let a = eval_math_expr(lhs, scalars, bools)?;
            let b = eval_math_expr(rhs, scalars, bools)?;
            Some(match op {
                UeMathBinaryOp::Add => a + b,
                UeMathBinaryOp::Sub => a - b,
                UeMathBinaryOp::Mul => a * b,
                UeMathBinaryOp::Div => {
                    if b.abs() <= f32::EPSILON { 0.0 } else { a / b }
                }
                UeMathBinaryOp::Pow => a.powf(b),
                UeMathBinaryOp::Min => a.min(b),
                UeMathBinaryOp::Max => a.max(b),
                UeMathBinaryOp::Mod => {
                    if b.abs() <= f32::EPSILON { 0.0 } else { a % b }
                }
                UeMathBinaryOp::Atan2Deg => UeMathLib::atan2_deg(a, b),
            })
        }
        UeMathExpr::Select {
            cond,
            on_true,
            on_false,
        } => {
            if bools.get(cond).copied().unwrap_or(false) {
                eval_math_expr(on_true, scalars, bools)
            } else {
                eval_math_expr(on_false, scalars, bools)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_and_noise_math_works() {
        let v = UeMathLib::vec2_rotate_deg(UeVec2 { x: 1.0, y: 0.0 }, 90.0);
        assert!(v.x.abs() < 0.0001);
        assert!((v.y - 1.0).abs() < 0.0001);

        let n1 = UeMathLib::fbm_noise_2d(42, 1.2, 4.6, 4);
        let n2 = UeMathLib::fbm_noise_2d(42, 1.2, 4.6, 4);
        assert!((n1 - n2).abs() < 0.000001);
    }

    #[test]
    fn blueprint_expr_eval_works() {
        let expr = UeMathExpr::Binary {
            op: UeMathBinaryOp::Mul,
            lhs: Box::new(UeMathExpr::Binary {
                op: UeMathBinaryOp::Add,
                lhs: Box::new(UeMathExpr::Var("a".to_string())),
                rhs: Box::new(UeMathExpr::Const(2.0)),
            }),
            rhs: Box::new(UeMathExpr::Const(3.0)),
        };
        let vars = HashMap::from([(String::from("a"), 4.0)]);
        let out = eval_math_expr(&expr, &vars, &HashMap::new());
        assert_eq!(out, Some(18.0));
    }
}
