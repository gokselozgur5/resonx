//! PolyBLEP anti-aliased oscillator primitives and noise generator.

use crate::math::{clamp_f32, fast_sin, FloatExt, TAU};

/// Oscillator waveform selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waveform {
    Sine,
    Saw,
    Square,
    Triangle,
    Noise,
}

/// Anti-aliased multi-waveform oscillator using PolyBLEP residual smoothing.
#[derive(Debug, Clone)]
pub struct Oscillator {
    waveform: Waveform,
    sample_rate: f32,
    freq_hz: f32,
    phase: f32,
    pulse_width: f32,
    rng_state: u32,
}

impl Oscillator {
    /// Creates a new oscillator at the specified sample rate.
    pub fn new(sample_rate: f32) -> Self {
        Self {
            waveform: Waveform::Saw,
            sample_rate: sample_rate.max(1.0),
            freq_hz: 440.0,
            phase: 0.0,
            pulse_width: 0.5,
            rng_state: 0x1234_5678,
        }
    }

    /// Sets the active waveform.
    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    /// Sets frequency in Hertz.
    pub fn set_frequency(&mut self, freq: f32) {
        self.freq_hz = clamp_f32(freq, 0.1, self.sample_rate * 0.49);
    }

    /// Sets pulse width for square waveform in range `0.01..0.99`.
    pub fn set_pulse_width(&mut self, pw: f32) {
        self.pulse_width = clamp_f32(pw, 0.01, 0.99);
    }

    /// Resets oscillator phase.
    pub fn reset_phase(&mut self) {
        self.phase = 0.0;
    }

    /// Generates next sample in range `[-1.0, 1.0]`.
    pub fn next_sample(&mut self) -> f32 {
        let dt = self.freq_hz / self.sample_rate;
        let t = self.phase;

        let sample = match self.waveform {
            Waveform::Sine => fast_sin(t * TAU),
            Waveform::Saw => (2.0 * t - 1.0) - poly_blep(t, dt),
            Waveform::Square => {
                let pw = self.pulse_width;
                let naive = if t < pw { 1.0 } else { -1.0 };
                let phase_offset = (t - pw + 1.0) - (t - pw + 1.0).floor_approx();
                naive + poly_blep(t, dt) - poly_blep(phase_offset, dt)
            }
            Waveform::Triangle => {
                if t < 0.5 {
                    4.0 * t - 1.0
                } else {
                    3.0 - 4.0 * t
                }
            }
            Waveform::Noise => {
                self.rng_state ^= self.rng_state << 13;
                self.rng_state ^= self.rng_state >> 17;
                self.rng_state ^= self.rng_state << 5;
                let val = (self.rng_state as f32) / (u32::MAX as f32);
                2.0 * val - 1.0
            }
        };

        self.phase += dt;
        if self.phase >= 1.0 {
            self.phase -= self.phase.floor_approx();
        }

        clamp_f32(sample, -1.0, 1.0)
    }
}

/// PolyBLEP residual function for quasi-bandlimited step discontinuities.
#[inline]
fn poly_blep(t: f32, dt: f32) -> f32 {
    if dt <= 0.0 {
        return 0.0;
    }
    if t < dt {
        let t_norm = t / dt;
        2.0 * t_norm - t_norm * t_norm - 1.0
    } else if t > 1.0 - dt {
        let t_norm = (t - 1.0) / dt;
        t_norm * t_norm + 2.0 * t_norm + 1.0
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oscillator_bounded_output() {
        let waveforms = [
            Waveform::Sine,
            Waveform::Saw,
            Waveform::Square,
            Waveform::Triangle,
            Waveform::Noise,
        ];
        for wf in waveforms {
            let mut osc = Oscillator::new(44100.0);
            osc.set_waveform(wf);
            osc.set_frequency(440.0);
            for _ in 0..1000 {
                let s = osc.next_sample();
                assert!(!s.is_nan(), "Sample must not be NaN for {wf:?}");
                assert!((-1.05..=1.05).contains(&s), "Sample out of bounds: {s}");
            }
        }
    }

    #[test]
    fn test_saw_period_frequency() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::Saw);
        osc.set_frequency(100.0);
        let mut zero_crossings: i32 = 0;
        let mut prev = osc.next_sample();
        for _ in 0..44100 {
            let cur = osc.next_sample();
            if prev < 0.0 && cur >= 0.0 {
                zero_crossings += 1;
            }
            prev = cur;
        }
        assert!((zero_crossings - 100).abs() <= 2);
    }
}
