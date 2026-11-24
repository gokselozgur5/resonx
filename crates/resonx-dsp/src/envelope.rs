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
