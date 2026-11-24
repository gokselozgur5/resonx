//! Pure deterministic no_std DSP math primitives and approximations.

#[allow(clippy::approx_constant, clippy::excessive_precision)]
pub const PI: f32 = core::f32::consts::PI;
#[allow(clippy::approx_constant, clippy::excessive_precision)]
pub const TAU: f32 = core::f32::consts::TAU;
#[allow(clippy::approx_constant, clippy::excessive_precision)]
pub const HALF_PI: f32 = core::f32::consts::FRAC_PI_2;

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

/// Converts MIDI note number (0..127) to frequency in Hertz (A4 = 69 -> 440 Hz).
#[inline]
pub fn midi_to_hz(midi: f32) -> f32 {
    let exponent = (midi - 69.0) / 12.0;
    440.0 * fast_exp2(exponent)
}

/// Converts decibels to linear amplitude multiplier.
#[inline]
pub fn db_to_linear(db: f32) -> f32 {
    let exp2_arg = db * (core::f32::consts::LOG2_10 / 20.0);
    fast_exp2(exp2_arg)
}

/// Fast base-2 exponentiation approximation for float values.
#[inline]
pub fn fast_exp2(x: f32) -> f32 {
    if x < -24.0 {
        return 0.0;
    }
    if x > 24.0 {
        return 16777216.0;
    }
    let ipart = x.floor_approx() as i32;
    let fpart = x - (ipart as f32);

    let poly = 1.0 + fpart * (0.6930129 + fpart * (0.2415544 + fpart * (0.0517443 + fpart * 0.0136873)));
    let scale = if ipart >= 0 {
        (1i64 << ipart.min(30)) as f32
    } else {
        1.0 / ((1i64 << (-ipart).min(30)) as f32)
    };
    poly * scale
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp_f32() {
        assert_eq!(clamp_f32(5.0, 0.0, 2.0), 2.0);
        assert_eq!(clamp_f32(-1.0, 0.0, 2.0), 0.0);
        assert_eq!(clamp_f32(1.5, 0.0, 2.0), 1.5);
    }

    #[test]
    fn test_fast_sin_accuracy() {
        for i in 0..360 {
            let rad = (i as f32) * PI / 180.0;
            let approx = fast_sin(rad);
            #[cfg(feature = "std")]
            {
                let diff = abs_f32(rad.sin() - approx);
                assert!(diff < 1e-4, "sin({rad}) diff={diff} exceeds 1e-4");
            }
            assert!((-1.0..=1.0).contains(&approx));
        }
    }

    #[test]
    fn test_fast_tanh_saturation() {
        assert_eq!(fast_tanh(0.0), 0.0);
        assert!((fast_tanh(10.0) - 1.0).abs() < 1e-4);
        assert!((fast_tanh(-10.0) - (-1.0)).abs() < 1e-4);
    }

    #[test]
    fn test_midi_to_hz_a4() {
        let a4 = midi_to_hz(69.0);
        assert!((a4 - 440.0).abs() < 0.1, "A4 must be ~440Hz, got {a4}");
        let a5 = midi_to_hz(81.0);
        assert!((a5 - 880.0).abs() < 0.5, "A5 must be ~880Hz, got {a5}");
    }

    #[test]
    fn test_db_to_linear() {
        assert!((db_to_linear(0.0) - 1.0).abs() < 1e-3);
        assert!((db_to_linear(-6.0) - 0.50118).abs() < 1e-2);
    }
}
