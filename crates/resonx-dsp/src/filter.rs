//! Virtual analog zero-delay feedback (ZDF) and state-variable filters.

use crate::math::{clamp_f32, fast_tanh, PI};

/// 4-pole Zero-Delay Feedback (ZDF) Moog Ladder Filter.
///
/// Features 24dB/octave low-pass response with non-linear saturation drive
/// and bounded resonance feedback up to self-oscillation.
#[derive(Debug, Clone)]
pub struct MoogLadderFilter {
    sample_rate: f32,
    cutoff_hz: f32,
    resonance: f32,
    drive: f32,
    s: [f32; 4],
}

impl MoogLadderFilter {
    /// Creates a new Moog ladder filter at specified sample rate.
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate: sample_rate.max(1.0),
            cutoff_hz: 1000.0,
            resonance: 0.1,
            drive: 1.0,
            s: [0.0; 4],
        }
    }

    /// Sets cutoff frequency in Hertz clamped between 10 Hz and Nyquist.
    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff_hz = clamp_f32(cutoff, 10.0, self.sample_rate * 0.49);
    }

    /// Sets filter resonance in range `0.0..0.98`.
    pub fn set_resonance(&mut self, res: f32) {
        self.resonance = clamp_f32(res, 0.0, 0.98);
    }

    /// Sets drive amount (`1.0` = normal clean, `> 1.0` = warm saturation).
    pub fn set_drive(&mut self, drive: f32) {
        self.drive = clamp_f32(drive, 0.1, 10.0);
    }

    /// Resets internal filter state.
    pub fn reset(&mut self) {
        self.s = [0.0; 4];
    }

    /// Processes one audio sample through the 4-pole ladder filter.
    pub fn process_sample(&mut self, input: f32) -> f32 {
        let driven_input = fast_tanh(input * self.drive);
        let g = fast_tanh(PI * self.cutoff_hz / self.sample_rate);
        let g_pow = g * g * g * g;
        let k = 4.0 * self.resonance;

        // Feedback estimation with linear solver
        let s_sum = self.s[0] * (g * g * g) + self.s[1] * (g * g) + self.s[2] * g + self.s[3];
        let u = (driven_input - k * s_sum) / (1.0 + k * g_pow);

        let mut current_in = u;
        for i in 0..4 {
            let v = (current_in - self.s[i]) * g;
            let y = v + self.s[i];
            self.s[i] = y + v;
            current_in = y;
        }

        clamp_f32(current_in, -2.0, 2.0)
    }
}
