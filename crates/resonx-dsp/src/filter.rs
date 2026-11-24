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
        let k = 4.0 * self.resonance;

        // Feedback estimation with linear solver
        let g1 = g / (1.0 + g);
        let g1_4 = g1 * g1 * g1 * g1;
        let s_sum = self.s[0] * (g1 * g1 * g1) + self.s[1] * (g1 * g1) + self.s[2] * g1 + self.s[3];
        let u = (driven_input - k * s_sum) / (1.0 + k * g1_4);

        let mut current_in = u;
        for stage in &mut self.s {
            let v = (current_in - *stage) * g / (1.0 + g);
            let y = v + *stage;
            *stage = y + v;
            current_in = y;
        }

        clamp_f32(current_in, -2.0, 2.0)
    }
}

/// Multi-mode State Variable Filter (SVF) outputs.
#[derive(Debug, Clone, Copy, Default)]
pub struct SvfOutput {
    pub lowpass: f32,
    pub bandpass: f32,
    pub highpass: f32,
    pub notch: f32,
}

/// Zero-delay feedback State Variable Filter (SVF).
#[derive(Debug, Clone)]
pub struct StateVariableFilter {
    sample_rate: f32,
    cutoff_hz: f32,
    resonance: f32,
    ic1eq: f32,
    ic2eq: f32,
}

impl StateVariableFilter {
    /// Creates a new SVF at specified sample rate.
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate: sample_rate.max(1.0),
            cutoff_hz: 1000.0,
            resonance: 0.5,
            ic1eq: 0.0,
            ic2eq: 0.0,
        }
    }

    /// Sets cutoff frequency in Hertz.
    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff_hz = clamp_f32(cutoff, 10.0, self.sample_rate * 0.49);
    }

    /// Sets Q/resonance factor (0.0 = critically damped, 1.0 = sharp peak).
    pub fn set_resonance(&mut self, res: f32) {
        self.resonance = clamp_f32(res, 0.0, 0.99);
    }

    /// Resets internal integrator state.
    pub fn reset(&mut self) {
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
    }

    /// Processes an input sample and computes LP, BP, HP, and Notch outputs.
    pub fn process_sample(&mut self, input: f32) -> SvfOutput {
        let g = fast_tanh(PI * self.cutoff_hz / self.sample_rate);
        let k = 2.0 * (1.0 - self.resonance);
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;

        let v3 = input - self.ic2eq;
        let v1 = a1 * self.ic1eq + a2 * v3;
        let v2 = self.ic2eq + a2 * self.ic1eq + a3 * v3;

        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;

        SvfOutput {
            lowpass: v2,
            bandpass: v1,
            highpass: input - k * v1 - v2,
            notch: input - k * v1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moog_attenuation_and_finite() {
        let mut filter = MoogLadderFilter::new(44100.0);
        filter.set_cutoff(500.0);
        filter.set_resonance(0.8);
        filter.set_drive(1.2);

        // Process a low frequency sine wave (100 Hz)
        let mut low_energy = 0.0_f32;
        for i in 0..1000 {
            let t = (i as f32) / 44100.0;
            let low_in = (t * 100.0 * 2.0 * PI).sin();
            let out_low = filter.process_sample(low_in);
            assert!(!out_low.is_nan() && out_low.is_finite());
            low_energy += out_low * out_low;
        }

        filter.reset();

        // Process a high frequency sine wave (5000 Hz)
        let mut high_energy = 0.0_f32;
        for i in 0..1000 {
            let t = (i as f32) / 44100.0;
            let high_in = (t * 5000.0 * 2.0 * PI).sin();
            let out_high = filter.process_sample(high_in);
            assert!(!out_high.is_nan() && out_high.is_finite());
            high_energy += out_high * out_high;
        }

        assert!(
            low_energy > high_energy * 2.0,
            "Low-pass filter must pass low frequencies ({low_energy}) significantly more than high ({high_energy})"
        );
    }

    #[test]
    fn test_svf_multi_outputs() {
        let mut svf = StateVariableFilter::new(44100.0);
        svf.set_cutoff(1000.0);
        svf.set_resonance(0.5);

        for _ in 0..100 {
            let out = svf.process_sample(0.5);
            assert!(!out.lowpass.is_nan());
            assert!(!out.bandpass.is_nan());
            assert!(!out.highpass.is_nan());
            assert!(!out.notch.is_nan());
        }
    }
}
