// Application entry point wiring audio engine, keyboard, presets, and controls
import { PRESETS, DEMO_SEQUENCE } from './presets.js';
import { RESONX_WORKLET_CODE } from './worklet_dsp.js';
import { ResonxAudioEngine } from './audio_engine.js';
import { encodeWav16, downloadWav } from './wav_export.js';
import { ResonxVisualizer } from './visualizer.js';
import { ResonxKeyboard } from './keyboard.js';

window.addEventListener('DOMContentLoaded', async () => {
  const engine = new ResonxAudioEngine();
  const canvas = document.getElementById('visualizer-canvas');
  const visualizer = new ResonxVisualizer(canvas, engine);
  const statusBadge = document.getElementById('status-badge');
  const presetSelect = document.getElementById('preset-select');
  const demoBtn = document.getElementById('demo-btn');
  const recordBtn = document.getElementById('record-btn');
  let demoPlaying = false;
  let demoTimer = null;

  async function ensureInit() {
    if (!engine.ctx) {
      await engine.init(RESONX_WORKLET_CODE);
      visualizer.start();
      statusBadge.textContent = engine.isWorklet ? 'DSP Worklet 48kHz' : 'WebAudio Fallback';
      statusBadge.classList.add('live');
    }
    await engine.resume();
  }

  const kbContainer = document.getElementById('keyboard-container');
  let keyboard;
  keyboard = new ResonxKeyboard(
    kbContainer,
    async (note, vel) => {
      await ensureInit();
      if (!keyboard.activeNotes.has(note)) return;
      engine.noteOn(note, vel);
    },
    (note) => { engine.noteOff(note); }
  );

  const paramInputs = [
    'cutoff', 'resonance', 'drive', 'envAmount',
    'attack', 'decay', 'sustain', 'release',
    'lfoRate', 'lfoDepth', 'delayMix', 'reverbMix', 'detune'
  ];

  function syncControls(preset) {
    paramInputs.forEach(key => {
      const input = document.getElementById(`param-${key}`);
      const valDisplay = document.getElementById(`val-${key}`);
      if (input && preset[key] !== undefined) {
        input.value = preset[key];
        if (valDisplay) valDisplay.textContent = preset[key];
      }
    });
  }

  function bindInputEvents() {
    paramInputs.forEach(key => {
      const input = document.getElementById(`param-${key}`);
      const valDisplay = document.getElementById(`val-${key}`);
      if (input) {
        input.addEventListener('input', async (e) => {
          await ensureInit();
          const val = parseFloat(e.target.value);
          if (valDisplay) valDisplay.textContent = val;
          engine.setParam(key, val);
        });
      }
    });
  }
  bindInputEvents();

  presetSelect.addEventListener('change', async (e) => {
    await ensureInit();
    const preset = PRESETS[e.target.value];
    if (preset) {
      engine.applyPreset(preset);
      syncControls(preset);
    }
  });
  syncControls(PRESETS['analog-bass']);

  demoBtn.addEventListener('click', async () => {
    await ensureInit();
    if (demoPlaying) {
      demoPlaying = false;
      clearTimeout(demoTimer);
      demoBtn.textContent = 'Play Demo Arp';
      demoBtn.classList.remove('active');
      return;
    }
    demoPlaying = true;
    demoBtn.textContent = 'Stop Demo Arp';
    demoBtn.classList.add('active');

    let step = 0;
    const playStep = () => {
      if (!demoPlaying) return;
      const { note, duration } = DEMO_SEQUENCE[step % DEMO_SEQUENCE.length];
      if (note > 0) {
        keyboard.triggerNoteOn(note);
        setTimeout(() => keyboard.triggerNoteOff(note), duration * 800);
      }
      step++;
      demoTimer = setTimeout(playStep, duration * 1000);
    };
    playStep();
  });

  recordBtn.addEventListener('click', async () => {
    await ensureInit();
    if (!engine.recording) {
      engine.startRecording();
      recordBtn.textContent = 'Stop & Save WAV';
      recordBtn.classList.add('recording');
    } else {
      const { left, right, sampleRate } = engine.stopRecording();
      recordBtn.textContent = 'Record Audio to WAV';
      recordBtn.classList.remove('recording');
      if (left.length > 0) {
        const wavBlob = encodeWav16(left, right, sampleRate);
        const activePreset = presetSelect.value || 'preset';
        downloadWav(wavBlob, `resonx_${activePreset}_${Date.now()}.wav`);
      }
    }
  });
});
