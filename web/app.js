// Application entry point wiring audio engine, tactile knobs, keyboard, and presets
import { PRESETS, DEMO_SEQUENCE } from './presets.js';
import { RESONX_WORKLET_CODE } from './worklet_dsp.js';
import { ResonxAudioEngine } from './audio_engine.js';
import { encodeWav16, downloadWav } from './wav_export.js';
import { ResonxVisualizer } from './visualizer.js';
import { ResonxKeyboard } from './keyboard.js';
import { ResonxKnob } from './knob.js';

window.addEventListener('DOMContentLoaded', async () => {
  const engine = new ResonxAudioEngine();
  const canvas = document.getElementById('visualizer-canvas');
  const visualizer = new ResonxVisualizer(canvas, engine);
  const statusBadge = document.getElementById('status-badge');
  const presetSelect = document.getElementById('preset-select');
  const demoBtn = document.getElementById('demo-btn');
  const recordBtn = document.getElementById('record-btn');
  const demoText = document.getElementById('demo-text');
  const recordText = document.getElementById('record-text');

  let demoPlaying = false;
  let demoTimer = null;

  async function ensureInit() {
    if (!engine.ctx) {
      await engine.init(RESONX_WORKLET_CODE);
      visualizer.start();
      statusBadge.textContent = engine.isWorklet ? 'DSP WORKLET 48kHz [ACTIVE]' : 'WEBAUDIO CORE [ACTIVE]';
      statusBadge.classList.add('live');
    }
    await engine.resume();
  }

  // First interaction handler to unlock WebAudio
  window.addEventListener('pointerdown', ensureInit, { once: true });
  window.addEventListener('keydown', ensureInit, { once: true });

  const kbContainer = document.getElementById('keyboard-container');
  const keyboard = new ResonxKeyboard(
    kbContainer,
    async (note, vel) => {
      await ensureInit();
      engine.noteOn(note, vel);
    },
    (note) => {
      engine.noteOff(note);
    }
  );

  // Knob configuration map for all parameters
  const KNOB_CONFIGS = [
    // Module 1: Oscillators
    { id: 'detune', containerId: 'knob-detune', label: 'Detune', unit: 'ct', min: 0, max: 30, step: 1, value: 4, color: '#00e5ff' },
    { id: 'envAmount', containerId: 'knob-envAmount', label: 'Env Mod', unit: '%', min: 0, max: 100, step: 1, value: 50, color: '#00e5ff' },

    // Module 2: Moog Filter
    { id: 'cutoff', containerId: 'knob-cutoff', label: 'Cutoff', unit: 'Hz', min: 40, max: 12000, step: 10, value: 400, color: '#ff9100' },
    { id: 'resonance', containerId: 'knob-resonance', label: 'Resonance', unit: '', min: 0.0, max: 0.95, step: 0.01, value: 0.6, color: '#ff9100' },
    { id: 'drive', containerId: 'knob-drive', label: 'Drive', unit: 'x', min: 0.5, max: 3.0, step: 0.1, value: 1.8, color: '#ff9100' },

    // Module 3: ADSR Envelope
    { id: 'attack', containerId: 'knob-attack', label: 'Attack', unit: 's', min: 0.001, max: 1.5, step: 0.005, value: 0.005, color: '#00e676' },
    { id: 'decay', containerId: 'knob-decay', label: 'Decay', unit: 's', min: 0.01, max: 2.0, step: 0.01, value: 0.15, color: '#00e676' },
    { id: 'sustain', containerId: 'knob-sustain', label: 'Sustain', unit: '', min: 0.0, max: 1.0, step: 0.02, value: 0.4, color: '#00e676' },
    { id: 'release', containerId: 'knob-release', label: 'Release', unit: 's', min: 0.01, max: 3.0, step: 0.02, value: 0.1, color: '#00e676' },

    // Module 4: LFO & Effects
    { id: 'lfoRate', containerId: 'knob-lfoRate', label: 'LFO Rate', unit: 'Hz', min: 0.1, max: 15.0, step: 0.1, value: 1.5, color: '#00e5ff' },
    { id: 'lfoDepth', containerId: 'knob-lfoDepth', label: 'LFO Depth', unit: '', min: 0, max: 600, step: 10, value: 80, color: '#00e5ff' },
    { id: 'delayMix', containerId: 'knob-delayMix', label: 'Delay Wet', unit: '%', min: 0.0, max: 0.8, step: 0.02, value: 0.15, color: '#e040fb' },
    { id: 'reverbMix', containerId: 'knob-reverbMix', label: 'Space Wet', unit: '%', min: 0.0, max: 0.8, step: 0.02, value: 0.2, color: '#e040fb' }
  ];

  const knobInstances = new Map();

  KNOB_CONFIGS.forEach(cfg => {
    const el = document.getElementById(cfg.containerId);
    if (!el) return;
    const knob = new ResonxKnob(el, {
      id: cfg.id,
      label: cfg.label,
      unit: cfg.unit,
      min: cfg.min,
      max: cfg.max,
      step: cfg.step,
      value: cfg.value,
      color: cfg.color,
      size: 58,
      onChange: async (val) => {
        await ensureInit();
        engine.setParam(cfg.id, val);
      }
    });
    knobInstances.set(cfg.id, knob);
  });

  function syncPresetKnobs(preset) {
    Object.entries(preset).forEach(([key, val]) => {
      const knob = knobInstances.get(key);
      if (knob) {
        knob.setValue(val, false);
      }
    });
  }

  presetSelect.addEventListener('change', async (e) => {
    await ensureInit();
    const preset = PRESETS[e.target.value];
    if (preset) {
      engine.applyPreset(preset);
      syncPresetKnobs(preset);
    }
  });

  // Apply default preset
  if (PRESETS['analog-bass']) {
    syncPresetKnobs(PRESETS['analog-bass']);
  }

  // Demo Arpeggiator
  demoBtn.addEventListener('click', async () => {
    await ensureInit();
    if (demoPlaying) {
      demoPlaying = false;
      clearTimeout(demoTimer);
      demoBtn.classList.remove('active');
      demoText.textContent = 'PLAY DEMO ARP';
      return;
    }
    demoPlaying = true;
    demoBtn.classList.add('active');
    demoText.textContent = 'STOP ARP';

    let step = 0;
    const playStep = () => {
      if (!demoPlaying) return;
      const { note, duration } = DEMO_SEQUENCE[step % DEMO_SEQUENCE.length];
      if (note > 0) {
        keyboard.triggerNoteOn(note);
        setTimeout(() => keyboard.triggerNoteOff(note), duration * 750);
      }
      step++;
      demoTimer = setTimeout(playStep, duration * 1000);
    };
    playStep();
  });

  // WAV Recording
  recordBtn.addEventListener('click', async () => {
    await ensureInit();
    if (!engine.recording) {
      engine.startRecording();
      recordBtn.classList.add('recording');
      recordText.textContent = 'STOP & EXPORT';
    } else {
      const { left, right, sampleRate } = engine.stopRecording();
      recordBtn.classList.remove('recording');
      recordText.textContent = 'RECORD WAV';
      if (left.length > 0) {
        const wavBlob = encodeWav16(left, right, sampleRate);
        const activePreset = presetSelect.value || 'preset';
        downloadWav(wavBlob, `resonx_${activePreset}_${Date.now()}.wav`);
      }
    }
  });
});
