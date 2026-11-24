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
});
