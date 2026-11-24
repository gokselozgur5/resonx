//! Pure deterministic no_std DSP math primitives and approximations.

/// Mathematical constant PI (32-bit floating point).
pub const PI: f32 = 3.14159265358979323846_f32;
/// Two times PI.
pub const TAU: f32 = 6.28318530717958647692_f32;
/// Half PI.
pub const HALF_PI: f32 = 1.57079632679489661923_f32;

/// Clamps a 32-bit float between `min` and `max`.
#[inline]
pub fn clamp_f32(v: f32, min: f32, max: f32) -> f32 {
    if v < min {
        min
    } else if v > max {
        max
    } else {
        v
    }
}

/// Absolute value of an f32 without standard library or libm.
#[inline]
pub fn abs_f32(x: f32) -> f32 {
    f32::from_bits(x.to_bits() & 0x7FFF_FFFF)
}

/// High-precision polynomial approximation of sine for x in radians.
#[inline]
pub fn fast_sin(x: f32) -> f32 {
    let mut norm = x / TAU;
    norm -= (norm + 0.5).floor_approx();
    let mut y = norm * TAU;
    if y > HALF_PI {
        y = PI - y;
    } else if y < -HALF_PI {
        y = -PI - y;
    }
    let y2 = y * y;
    let p = y * (1.0 + y2 * (-0.16666657 + y2 * (0.008333015 + y2 * (-0.000198409 + y2 * 0.0000027526))));
    clamp_f32(p, -1.0, 1.0)
}

/// Fast approximation of cosine for x in radians.
#[inline]
pub fn fast_cos(x: f32) -> f32 {
    fast_sin(x + HALF_PI)
}

/// Fast hyperbolic tangent approximation using Padé approximation.
#[inline]
pub fn fast_tanh(x: f32) -> f32 {
    let x_clamped = clamp_f32(x, -5.0, 5.0);
    let x2 = x_clamped * x_clamped;
    let num = x_clamped * (135135.0 + x2 * (17325.0 + x2 * (378.0 + x2)));
    let den = 135135.0 + x2 * (62370.0 + x2 * (3150.0 + 28.0 * x2));
    clamp_f32(num / den, -1.0, 1.0)
}

/// Helper extension trait for no_std floating point approximation.
pub trait FloatExt {
    fn floor_approx(self) -> f32;
}

impl FloatExt for f32 {
    #[inline]
    fn floor_approx(self) -> f32 {
        let i = self as i32;
        if self < (i as f32) {
            (i - 1) as f32
        } else {
            i as f32
        }
    }
}
