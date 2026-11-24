//! Click-free ADSR envelope generator and low-frequency modulation oscillator (LFO).

use crate::math::clamp_f32;

/// ADSR envelope generator state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvelopeStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// Zero-allocation, click-free ADSR envelope generator.
#[derive(Debug, Clone)]
pub struct AdsrEnvelope {
    sample_rate: f32,
    stage: EnvelopeStage,
    current_level: f32,
    attack_sec: f32,
    decay_sec: f32,
    sustain_lvl: f32,
    release_sec: f32,
}

impl AdsrEnvelope {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate: sample_rate.max(1.0),
            stage: EnvelopeStage::Idle,
            current_level: 0.0,
            attack_sec: 0.01,
            decay_sec: 0.1,
            sustain_lvl: 0.7,
            release_sec: 0.2,
        }
    }

    pub fn set_attack(&mut self, sec: f32) {
        self.attack_sec = clamp_f32(sec, 0.001, 10.0);
    }

    pub fn set_decay(&mut self, sec: f32) {
        self.decay_sec = clamp_f32(sec, 0.001, 10.0);
    }

    pub fn set_sustain(&mut self, lvl: f32) {
        self.sustain_lvl = clamp_f32(lvl, 0.0, 1.0);
    }

    pub fn set_release(&mut self, sec: f32) {
        self.release_sec = clamp_f32(sec, 0.001, 10.0);
    }

    pub fn gate_on(&mut self) {
        self.stage = EnvelopeStage::Attack;
    }

    pub fn gate_off(&mut self) {
        if self.stage != EnvelopeStage::Idle {
            self.stage = EnvelopeStage::Release;
        }
    }

    pub fn stage(&self) -> EnvelopeStage {
        self.stage
    }

    pub fn is_active(&self) -> bool {
        self.stage != EnvelopeStage::Idle
    }

    pub fn next_sample(&mut self) -> f32 {
        match self.stage {
            EnvelopeStage::Idle => self.current_level = 0.0,
            EnvelopeStage::Attack => {
                let step = 1.0 / (self.attack_sec * self.sample_rate);
                self.current_level += step;
                if self.current_level >= 1.0 {
                    self.current_level = 1.0;
                    self.stage = EnvelopeStage::Decay;
                }
            }
            EnvelopeStage::Decay => {
                let step = 1.0 / (self.decay_sec * self.sample_rate);
                self.current_level -= step;
                if self.current_level <= self.sustain_lvl {
                    self.current_level = self.sustain_lvl;
                    self.stage = EnvelopeStage::Sustain;
                }
            }
            EnvelopeStage::Sustain => self.current_level = self.sustain_lvl,
            EnvelopeStage::Release => {
                let step = 1.0 / (self.release_sec * self.sample_rate);
                self.current_level -= step;
                if self.current_level <= 0.0 {
                    self.current_level = 0.0;
                    self.stage = EnvelopeStage::Idle;
                }
            }
        }
        self.current_level
    }
}

use crate::math::{fast_sin, TAU};

/// Low Frequency Oscillator (LFO) for parameter modulation.
#[derive(Debug, Clone)]
pub struct Lfo {
    sample_rate: f32,
    freq_hz: f32,
    phase: f32,
}

impl Lfo {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate: sample_rate.max(1.0),
            freq_hz: 1.0,
            phase: 0.0,
        }
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.freq_hz = clamp_f32(freq, 0.01, 50.0);
    }

    pub fn next_sample(&mut self) -> f32 {
        let val = fast_sin(self.phase * TAU);
        let dt = self.freq_hz / self.sample_rate;
        self.phase += dt;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        val
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adsr_stages_and_return_to_zero() {
        let mut env = AdsrEnvelope::new(1000.0); // 1ms per sample
        env.set_attack(0.01); // 10 samples
        env.set_decay(0.01); // 10 samples
        env.set_sustain(0.5);
        env.set_release(0.01); // 10 samples

        assert_eq!(env.stage(), EnvelopeStage::Idle);
        assert_eq!(env.next_sample(), 0.0);

        env.gate_on();
        assert_eq!(env.stage(), EnvelopeStage::Attack);

        // Run through attack
        for _ in 0..10 {
            env.next_sample();
        }
        assert_eq!(env.stage(), EnvelopeStage::Decay);

        // Run through decay
        for _ in 0..10 {
            env.next_sample();
        }
        assert_eq!(env.stage(), EnvelopeStage::Sustain);
        assert!((env.next_sample() - 0.5).abs() < 1e-4);

        // Note off -> release
        env.gate_off();
        assert_eq!(env.stage(), EnvelopeStage::Release);
        for _ in 0..12 {
            env.next_sample();
        }
        assert_eq!(env.stage(), EnvelopeStage::Idle);
        assert_eq!(env.next_sample(), 0.0);
    }

    #[test]
    fn test_lfo_bounded() {
        let mut lfo = Lfo::new(44100.0);
        lfo.set_frequency(5.0);
        for _ in 0..1000 {
            let s = lfo.next_sample();
            assert!((-1.0..=1.0).contains(&s));
        }
    }
}
