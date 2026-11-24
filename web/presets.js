// Factory preset sound banks for resonx
export const PRESETS = {
  'analog-bass': {
    name: 'Analog Bass', osc1: 'saw', osc2: 'square', detune: 4,
    cutoff: 400, resonance: 0.6, drive: 1.8, envAmount: 2000,
    attack: 0.005, decay: 0.15, sustain: 0.4, release: 0.1,
    lfoRate: 1.5, lfoDepth: 80, delayMix: 0.15, delayTime: 0.25,
    delayFeedback: 0.3, reverbMix: 0.1
  },
  'blade-pad': {
    name: 'Blade Runner Pad', osc1: 'saw', osc2: 'saw', detune: 8,
    cutoff: 1200, resonance: 0.3, drive: 1.1, envAmount: 800,
    attack: 0.35, decay: 0.5, sustain: 0.8, release: 0.65,
    lfoRate: 0.4, lfoDepth: 350, delayMix: 0.4, delayTime: 0.375,
    delayFeedback: 0.5, reverbMix: 0.45
  },
  'acid-303': {
    name: 'Acid 303', osc1: 'saw', osc2: 'square', detune: 2,
    cutoff: 320, resonance: 0.88, drive: 2.2, envAmount: 3400,
    attack: 0.002, decay: 0.22, sustain: 0.05, release: 0.1,
    lfoRate: 4.0, lfoDepth: 200, delayMix: 0.25, delayTime: 0.2,
    delayFeedback: 0.4, reverbMix: 0.15
  },
  'sub-pluck': {
    name: 'Sub Pluck', osc1: 'triangle', osc2: 'sine', detune: 0,
    cutoff: 800, resonance: 0.2, drive: 1.0, envAmount: 1400,
    attack: 0.001, decay: 0.18, sustain: 0.0, release: 0.12,
    lfoRate: 0.2, lfoDepth: 0, delayMix: 0.2, delayTime: 0.25,
    delayFeedback: 0.25, reverbMix: 0.1
  },
  'fm-bell': {
    name: 'FM Bell', osc1: 'sine', osc2: 'triangle', detune: 14,
    cutoff: 2800, resonance: 0.45, drive: 1.2, envAmount: 1800,
    attack: 0.002, decay: 0.6, sustain: 0.15, release: 0.8,
    lfoRate: 3.5, lfoDepth: 120, delayMix: 0.35, delayTime: 0.33,
    delayFeedback: 0.45, reverbMix: 0.35
  }
};
