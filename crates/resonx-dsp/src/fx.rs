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

        // Cross feedback for true ping-pong
        self.buffer_left[self.write_idx] = in_left + delayed_r * self.feedback;
        self.buffer_right[self.write_idx] = in_right + delayed_l * self.feedback;

        self.write_idx = (self.write_idx + 1) % MAX_DELAY_SAMPLES;

        let out_l = in_left * (1.0 - self.wet) + delayed_l * self.wet;
        let out_r = in_right * (1.0 - self.wet) + delayed_r * self.wet;
        (clamp_f32(out_l, -2.0, 2.0), clamp_f32(out_r, -2.0, 2.0))
    }
}
