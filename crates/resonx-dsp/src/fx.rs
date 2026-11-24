//! Zero-runtime-allocation stereo effects: ping-pong delay and Schroeder reverb.

use crate::math::clamp_f32;

/// Maximum buffer length for delay lines (approx 1 second at 48 kHz).
pub const MAX_DELAY_SAMPLES: usize = 48000;

/// Fixed-buffer zero-allocation stereo ping-pong delay effect.
#[derive(Debug, Clone)]
pub struct StereoPingPongDelay {
    sample_rate: f32,
    buffer_left: [f32; MAX_DELAY_SAMPLES],
    buffer_right: [f32; MAX_DELAY_SAMPLES],
    write_idx: usize,
    delay_samples: usize,
    feedback: f32,
    wet: f32,
}

impl StereoPingPongDelay {
    pub fn new(sample_rate: f32) -> Self {
        let sr = sample_rate.max(1.0);
        let default_delay = (sr * 0.25) as usize; // 250ms
        Self {
            sample_rate: sr,
            buffer_left: [0.0; MAX_DELAY_SAMPLES],
            buffer_right: [0.0; MAX_DELAY_SAMPLES],
            write_idx: 0,
            delay_samples: default_delay.min(MAX_DELAY_SAMPLES - 1),
            feedback: 0.4,
            wet: 0.3,
        }
    }

    pub fn set_delay_time(&mut self, seconds: f32) {
        let samples = (seconds * self.sample_rate) as usize;
        self.delay_samples = samples.clamp(1, MAX_DELAY_SAMPLES - 1);
    }

    pub fn set_feedback(&mut self, fb: f32) {
        self.feedback = clamp_f32(fb, 0.0, 0.95);
    }

    pub fn set_wet(&mut self, wet: f32) {
        self.wet = clamp_f32(wet, 0.0, 1.0);
    }

    pub fn reset(&mut self) {
        self.buffer_left = [0.0; MAX_DELAY_SAMPLES];
        self.buffer_right = [0.0; MAX_DELAY_SAMPLES];
        self.write_idx = 0;
    }

    pub fn process_sample(&mut self, in_left: f32, in_right: f32) -> (f32, f32) {
        let read_idx = if self.write_idx >= self.delay_samples {
            self.write_idx - self.delay_samples
        } else {
            MAX_DELAY_SAMPLES + self.write_idx - self.delay_samples
        };

        let delayed_l = self.buffer_left[read_idx];
        let delayed_r = self.buffer_right[read_idx];

        // Ping-pong cross-feed: mono input sums into left tap; cross-feed L -> R -> L
        let mono_in = 0.5 * (in_left + in_right);
        self.buffer_left[self.write_idx] = mono_in + delayed_r * self.feedback;
        self.buffer_right[self.write_idx] = delayed_l * self.feedback;

        self.write_idx = (self.write_idx + 1) % MAX_DELAY_SAMPLES;

        let out_l = in_left * (1.0 - self.wet) + delayed_l * self.wet;
        let out_r = in_right * (1.0 - self.wet) + delayed_r * self.wet;
        (clamp_f32(out_l, -2.0, 2.0), clamp_f32(out_r, -2.0, 2.0))
    }
}

/// Maximum buffer size for comb filters in Schroeder reverb.
const COMB_BUF_MAX: usize = 2048;
/// Maximum buffer size for allpass filters in Schroeder reverb.
const ALLPASS_BUF_MAX: usize = 512;

/// Single comb filter unit with feedback and damping.
#[derive(Debug, Clone, Copy)]
struct CombFilter {
    buffer: [f32; COMB_BUF_MAX],
    len: usize,
    idx: usize,
    filter_store: f32,
    feedback: f32,
    damp: f32,
}

impl CombFilter {
    fn new(len: usize, feedback: f32, damp: f32) -> Self {
        Self {
            buffer: [0.0; COMB_BUF_MAX],
            len: len.clamp(1, COMB_BUF_MAX - 1),
            idx: 0,
            filter_store: 0.0,
            feedback,
            damp,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.buffer[self.idx];
        self.filter_store = output * (1.0 - self.damp) + self.filter_store * self.damp;
        self.buffer[self.idx] = input + self.filter_store * self.feedback;
        self.idx = (self.idx + 1) % self.len;
        output
    }
}

/// Single allpass filter unit.
#[derive(Debug, Clone, Copy)]
struct AllpassFilter {
    buffer: [f32; ALLPASS_BUF_MAX],
    len: usize,
    idx: usize,
    feedback: f32,
}

impl AllpassFilter {
    fn new(len: usize, feedback: f32) -> Self {
        Self {
            buffer: [0.0; ALLPASS_BUF_MAX],
            len: len.clamp(1, ALLPASS_BUF_MAX - 1),
            idx: 0,
            feedback,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let buf_out = self.buffer[self.idx];
        let output = -input + buf_out;
        self.buffer[self.idx] = input + buf_out * self.feedback;
        self.idx = (self.idx + 1) % self.len;
        output
    }
}

/// Zero-allocation Schroeder/Freeverb-style algorithmic reverb.
#[derive(Debug, Clone)]
pub struct SchroederReverb {
    combs: [CombFilter; 4],
    allpasses: [AllpassFilter; 2],
    wet: f32,
}

impl SchroederReverb {
    pub fn new() -> Self {
        Self {
            combs: [
                CombFilter::new(1116, 0.84, 0.2),
                CombFilter::new(1188, 0.84, 0.2),
                CombFilter::new(1277, 0.84, 0.2),
                CombFilter::new(1356, 0.84, 0.2),
            ],
            allpasses: [AllpassFilter::new(225, 0.5), AllpassFilter::new(341, 0.5)],
            wet: 0.3,
        }
    }

    pub fn set_room_size(&mut self, room_size: f32) {
        let fb = clamp_f32(room_size * 0.28 + 0.7, 0.7, 0.98);
        for c in &mut self.combs {
            c.feedback = fb;
        }
    }

    pub fn set_wet(&mut self, wet: f32) {
        self.wet = clamp_f32(wet, 0.0, 1.0);
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        let mut sum = 0.0;
        for c in &mut self.combs {
            sum += c.process(input);
        }
        let mut ap_out = sum * 0.25;
        for ap in &mut self.allpasses {
            ap_out = ap.process(ap_out);
        }
        let out = input * (1.0 - self.wet) + ap_out * self.wet;
        clamp_f32(out, -2.0, 2.0)
    }
}

impl Default for SchroederReverb {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_pong_mono_input_alternates_stereo_channels() {
        let mut delay = StereoPingPongDelay::new(44100.0);
        delay.set_delay_time(0.001); // ~44 samples
        delay.set_feedback(0.5);
        delay.set_wet(1.0);

        // Feed mono input (L = R = 1.0) on sample 0
        let (out_l0, out_r0) = delay.process_sample(1.0, 1.0);
        assert_eq!(out_l0, 0.0);
        assert_eq!(out_r0, 0.0);

        // Advance to first echo: L should carry delayed signal, R should still be zero
        let mut first_echo_l = 0.0;
        let mut first_echo_r = 0.0;
        for _ in 0..44 {
            let (l, r) = delay.process_sample(0.0, 0.0);
            if l.abs() > first_echo_l {
                first_echo_l = l.abs();
            }
            if r.abs() > first_echo_r {
                first_echo_r = r.abs();
            }
        }
        assert!(
            first_echo_l > 0.5 && first_echo_r < 0.01,
            "Ping-pong delay must alternate: first echo in L ({first_echo_l}), zero in R ({first_echo_r})"
        );
    }

    #[test]
    fn test_ping_pong_decay_and_finite() {
        let mut delay = StereoPingPongDelay::new(44100.0);
        delay.set_delay_time(0.01); // 10ms
        delay.set_feedback(0.5);
        delay.set_wet(0.5);

        // Send a single impulse into left channel
        let (out_l0, out_r0) = delay.process_sample(1.0, 0.0);
        assert!(!out_l0.is_nan() && !out_r0.is_nan());

        let mut energy = 0.0_f32;
        for _ in 0..5000 {
            let (l, r) = delay.process_sample(0.0, 0.0);
            assert!(l.is_finite() && r.is_finite());
            energy += l * l + r * r;
        }
        assert!(energy > 0.01, "Echoes should produce sustained energy");
    }

    #[test]
    fn test_schroeder_reverb_stability() {
        let mut verb = SchroederReverb::new();
        verb.set_room_size(0.8);
        verb.set_wet(0.5);

        // Feed impulse
        let _ = verb.process_sample(1.0);
        for _ in 0..5000 {
            let out = verb.process_sample(0.0);
            assert!(out.is_finite());
            assert!((-2.0..=2.0).contains(&out));
        }
    }
}
