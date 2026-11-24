//! Polyphonic synthesizer engine with voice allocation, stealing, and factory presets.

use crate::envelope::AdsrEnvelope;
use crate::filter::MoogLadderFilter;
use crate::fx::{SchroederReverb, StereoPingPongDelay};
use crate::math::midi_to_hz;
use crate::osc::Oscillator;

/// Maximum polyphonic voice count.
pub const MAX_VOICES: usize = 8;

/// Single monophonic synthesizer voice.
#[derive(Debug, Clone)]
pub struct SynthVoice {
    pub osc1: Oscillator,
    pub osc2: Oscillator,
    pub filter: MoogLadderFilter,
    pub amp_env: AdsrEnvelope,
    pub filter_env: AdsrEnvelope,
    pub filter_env_amount: f32,
    pub base_cutoff: f32,
    pub active_note: u8,
    pub velocity: f32,
    pub age: u64,
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
            velocity: 0.8,
            age: 0,
        }
    }

    pub fn note_on(&mut self, note: u8, velocity: f32) {
        self.active_note = note;
        self.velocity = velocity.clamp(0.0, 1.0);
        self.age = 0;
        let freq = midi_to_hz(note as f32);
        self.osc1.set_frequency(freq);
        self.osc2.set_frequency(freq * 1.002);
        self.osc1.reset_phase();
        self.osc2.reset_phase();
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
        filtered * amp * self.velocity
    }
}

/// 8-Voice polyphonic synthesizer with voice stealing and effects rack.
#[derive(Debug, Clone)]
pub struct PolySynth {
    pub voices: [SynthVoice; MAX_VOICES],
    pub delay: StereoPingPongDelay,
    pub reverb: SchroederReverb,
    pub master_gain: f32,
}

impl PolySynth {
    pub fn new(sample_rate: f32) -> Self {
        let sr = sample_rate.max(1.0);
        Self {
            voices: [
                SynthVoice::new(sr),
                SynthVoice::new(sr),
                SynthVoice::new(sr),
                SynthVoice::new(sr),
                SynthVoice::new(sr),
                SynthVoice::new(sr),
                SynthVoice::new(sr),
                SynthVoice::new(sr),
            ],
            delay: StereoPingPongDelay::new(sr),
            reverb: SchroederReverb::new(),
            master_gain: 0.8,
        }
    }

    pub fn note_on(&mut self, note: u8, velocity: f32) {
        for v in &mut self.voices {
            if !v.is_active() {
                v.note_on(note, velocity);
                return;
            }
        }
        let mut oldest_idx = 0;
        let mut max_age = 0;
        for (i, v) in self.voices.iter().enumerate() {
            if v.age > max_age {
                max_age = v.age;
                oldest_idx = i;
            }
        }
        self.voices[oldest_idx].note_on(note, velocity);
    }

    pub fn note_off(&mut self, note: u8) {
        for v in &mut self.voices {
            if v.active_note == note && v.is_active() {
                v.note_off();
            }
        }
    }

    pub fn render_stereo(&mut self, out_left: &mut [f32], out_right: &mut [f32]) {
        let frames = out_left.len().min(out_right.len());
        for i in 0..frames {
            let mut mixed = 0.0;
            for v in &mut self.voices {
                mixed += v.process_sample();
            }
            let scaled = mixed * 0.25 * self.master_gain;
            let (dl, dr) = self.delay.process_sample(scaled, scaled);
            let rev_l = self.reverb.process_sample(dl);
            let rev_r = self.reverb.process_sample(dr);

            out_left[i] = rev_l.clamp(-1.0, 1.0);
            out_right[i] = rev_r.clamp(-1.0, 1.0);
        }
    }
}

use crate::osc::Waveform;

/// Factory preset sound banks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchPreset {
    AnalogBass,
    BladePad,
    Acid303,
    SubPluck,
}

impl PolySynth {
    pub fn set_preset(&mut self, preset: PatchPreset) {
        for v in &mut self.voices {
            match preset {
                PatchPreset::AnalogBass => {
                    v.osc1.set_waveform(Waveform::Saw);
                    v.osc2.set_waveform(Waveform::Square);
                    v.osc2.set_pulse_width(0.5);
                    v.base_cutoff = 400.0;
                    v.filter_env_amount = 2000.0;
                    v.filter.set_resonance(0.6);
                    v.filter.set_drive(1.8);
                    v.amp_env.set_attack(0.005);
                    v.amp_env.set_decay(0.15);
                    v.amp_env.set_sustain(0.4);
                    v.amp_env.set_release(0.1);
                    v.filter_env.set_attack(0.005);
                    v.filter_env.set_decay(0.12);
                    v.filter_env.set_sustain(0.1);
                    v.filter_env.set_release(0.1);
                }
                PatchPreset::BladePad => {
                    v.osc1.set_waveform(Waveform::Saw);
                    v.osc2.set_waveform(Waveform::Saw);
                    v.base_cutoff = 1200.0;
                    v.filter_env_amount = 800.0;
                    v.filter.set_resonance(0.3);
                    v.filter.set_drive(1.1);
                    v.amp_env.set_attack(0.3);
                    v.amp_env.set_decay(0.5);
                    v.amp_env.set_sustain(0.8);
                    v.amp_env.set_release(0.6);
                    v.filter_env.set_attack(0.4);
                    v.filter_env.set_decay(0.4);
                    v.filter_env.set_sustain(0.6);
                    v.filter_env.set_release(0.5);
                }
                PatchPreset::Acid303 => {
                    v.osc1.set_waveform(Waveform::Saw);
                    v.osc2.set_waveform(Waveform::Saw);
                    v.base_cutoff = 300.0;
                    v.filter_env_amount = 3500.0;
                    v.filter.set_resonance(0.9);
                    v.filter.set_drive(2.2);
                    v.amp_env.set_attack(0.002);
                    v.amp_env.set_decay(0.2);
                    v.amp_env.set_sustain(0.0);
                    v.amp_env.set_release(0.05);
                    v.filter_env.set_attack(0.002);
                    v.filter_env.set_decay(0.18);
                    v.filter_env.set_sustain(0.0);
                    v.filter_env.set_release(0.05);
                }
                PatchPreset::SubPluck => {
                    v.osc1.set_waveform(Waveform::Triangle);
                    v.osc2.set_waveform(Waveform::Sine);
                    v.base_cutoff = 800.0;
                    v.filter_env_amount = 1200.0;
                    v.filter.set_resonance(0.2);
                    v.filter.set_drive(1.0);
                    v.amp_env.set_attack(0.001);
                    v.amp_env.set_decay(0.08);
                    v.amp_env.set_sustain(0.0);
                    v.amp_env.set_release(0.08);
                    v.filter_env.set_attack(0.001);
                    v.filter_env.set_decay(0.06);
                    v.filter_env.set_sustain(0.0);
                    v.filter_env.set_release(0.06);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polysynth_steals_oldest_voice() {
        let mut synth = PolySynth::new(44100.0);
        // Trigger 8 notes
        for note in 60..68 {
            synth.note_on(note, 0.8);
            let mut l = [0.0; 10];
            let mut r = [0.0; 10];
            synth.render_stereo(&mut l, &mut r);
        }
        for v in &synth.voices {
            assert!(v.is_active());
        }

        // Trigger 9th and 10th notes to test continuous saturation and stealing oldest voice
        synth.note_on(68, 0.8);
        assert_eq!(synth.voices[0].active_note, 68);

        let mut l = [0.0; 10];
        let mut r = [0.0; 10];
        synth.render_stereo(&mut l, &mut r);

        synth.note_on(69, 0.8);
        assert_eq!(synth.voices[1].active_note, 69);
    }

    #[test]
    fn test_polysynth_subpluck_decays_near_silence_while_held() {
        let mut synth = PolySynth::new(44100.0);
        synth.set_preset(PatchPreset::SubPluck);
        synth.note_on(40, 0.9);

        // Process samples beyond attack + decay time (e.g. 0.25 seconds = ~11000 samples)
        let mut left = [0.0; 1000];
        let mut right = [0.0; 1000];
        for _ in 0..15 {
            synth.render_stereo(&mut left, &mut right);
        }

        // Voice envelope sustain is 0.0, so while held it must decay near silence
        synth.render_stereo(&mut left, &mut right);
        let max_amp = left
            .iter()
            .chain(right.iter())
            .fold(0.0_f32, |acc, &s| acc.max(s.abs()));
        assert!(
            max_amp < 0.01,
            "SubPluck must decay near silence while held, got max amplitude {max_amp}"
        );
    }

    #[test]
    fn test_polysynth_render_preset() {
        let mut synth = PolySynth::new(44100.0);
        synth.set_preset(PatchPreset::AnalogBass);
        synth.note_on(36, 0.9); // C2

        let mut left = [0.0; 1000];
        let mut right = [0.0; 1000];
        synth.render_stereo(&mut left, &mut right);

        let sum_energy: f32 = left
            .iter()
            .zip(right.iter())
            .map(|(l, r)| l * l + r * r)
            .sum();
        assert!(sum_energy > 0.01, "Synth must produce audio energy");
        for (l, r) in left.iter().zip(right.iter()) {
            assert!(l.is_finite() && r.is_finite());
            assert!((-1.0..=1.0).contains(l));
            assert!((-1.0..=1.0).contains(r));
        }
    }
}
