//! Polyphonic synthesizer engine with voice allocation, stealing, and factory presets.

use crate::envelope::AdsrEnvelope;
use crate::filter::MoogLadderFilter;
use crate::math::midi_to_hz;
use crate::osc::Oscillator;

/// Factory preset sound banks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchPreset {
    AnalogBass,
    BladePad,
    Acid303,
    SubPluck,
}

/// Single monophonic synthesizer voice.
#[derive(Debug, Clone)]
pub struct SynthVoice {
    osc1: Oscillator,
    osc2: Oscillator,
    filter: MoogLadderFilter,
    amp_env: AdsrEnvelope,
    filter_env: AdsrEnvelope,
    filter_env_amount: f32,
    base_cutoff: f32,
    active_note: u8,
    age: u64,
}

impl SynthVoice {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            osc1: Oscillator::new(sample_rate),
            osc2: Oscillator::new(sample_rate),
            filter: MoogLadderFilter::new(sample_rate),
            amp_env: AdsrEnvelope::new(sample_rate),
            filter_env: AdsrEnvelope::new(sample_rate),
            filter_env_amount: 1500.0,
            base_cutoff: 800.0,
            active_note: 0,
            age: 0,
        }
    }

    pub fn note_on(&mut self, note: u8, velocity: f32) {
        self.active_note = note;
        let freq = midi_to_hz(note as f32);
        self.osc1.set_frequency(freq);
        self.osc2.set_frequency(freq * 1.002); // Subtle detune
        self.osc1.reset_phase();
        self.osc2.reset_phase();
        self.amp_env.set_sustain(velocity.clamp(0.1, 1.0));
        self.amp_env.gate_on();
        self.filter_env.gate_on();
    }

    pub fn note_off(&mut self) {
        self.amp_env.gate_off();
        self.filter_env.gate_off();
    }

    pub fn is_active(&self) -> bool {
        self.amp_env.is_active()
    }

    pub fn process_sample(&mut self) -> f32 {
        if !self.amp_env.is_active() {
            return 0.0;
        }
        self.age += 1;
        let s1 = self.osc1.next_sample();
        let s2 = self.osc2.next_sample();
        let raw = 0.5 * (s1 + s2);

        let f_env = self.filter_env.next_sample();
        let cutoff = (self.base_cutoff + f_env * self.filter_env_amount).clamp(20.0, 18000.0);
        self.filter.set_cutoff(cutoff);

        let filtered = self.filter.process_sample(raw);
        let amp = self.amp_env.next_sample();
        filtered * amp
    }
}
