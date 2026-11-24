// ResonxAudioEngine orchestrates WebAudio context and audio processing
export class ResonxAudioEngine {
  constructor() {
    this.ctx = null;
    this.workletNode = null;
    this.analyser = null;
    this.recording = false;
    this.recordedL = [];
    this.recordedR = [];
    this.isWorklet = false;
    this.activeNodes = new Map();
    this.masterGainNode = null;
    this.pannerNode = null;
  }

  async init(workletCode) {
    if (this.ctx) return;
    const AudioCtx = window.AudioContext || window.webkitAudioContext;
    this.ctx = new AudioCtx();
    this.analyser = this.ctx.createAnalyser();
    this.analyser.fftSize = 2048;
    this.analyser.smoothingTimeConstant = 0.85;

    // Stereo panner and master gain stage for fallback and visualizer tap
    if (this.ctx.createStereoPanner) {
      this.pannerNode = this.ctx.createStereoPanner();
    }
    this.masterGainNode = this.ctx.createGain();
    this.masterGainNode.gain.setValueAtTime(0.8, this.ctx.currentTime);

    let url = null;
    try {
      const blob = new Blob([workletCode], { type: 'application/javascript' });
      url = URL.createObjectURL(blob);
      await this.ctx.audioWorklet.addModule(url);
      this.workletNode = new AudioWorkletNode(this.ctx, 'resonx-worklet-processor', {
        outputChannelCount: [2]
      });
      this.workletNode.connect(this.analyser);
      this.analyser.connect(this.ctx.destination);
      this.isWorklet = true;
    } catch {
      if (this.pannerNode) {
        this.pannerNode.connect(this.masterGainNode);
        this.masterGainNode.connect(this.analyser);
      } else {
        this.masterGainNode.connect(this.analyser);
      }
      this.analyser.connect(this.ctx.destination);
      this.isWorklet = false;
    } finally {
      if (url) URL.revokeObjectURL(url);
    }
  }

  async resume() {
    if (this.ctx && this.ctx.state === 'suspended') {
      await this.ctx.resume();
    }
  }

  noteOn(midi, velocity = 0.8) {
    this.resume();
    if (this.isWorklet && this.workletNode) {
      this.workletNode.port.postMessage({ type: 'noteOn', note: midi, velocity });
      return;
    }
    if (!this.ctx) return;
    const now = this.ctx.currentTime;
    if (this.activeNodes.has(midi)) this.noteOff(midi);
    const osc = this.ctx.createOscillator();
    const filter = this.ctx.createBiquadFilter();
    const gain = this.ctx.createGain();
    const freq = 440 * Math.pow(2, (midi - 69) / 12);
    osc.type = 'sawtooth';
    osc.frequency.setValueAtTime(freq, now);
    filter.type = 'lowpass';
    filter.frequency.setValueAtTime(this.fallbackParams?.cutoff || 600, now);
    gain.gain.setValueAtTime(0.0001, now);
    gain.gain.linearRampToValueAtTime(velocity * 0.4, now + 0.01);
    osc.connect(filter);
    filter.connect(gain);
    if (this.pannerNode) {
      gain.connect(this.pannerNode);
    } else if (this.masterGainNode) {
      gain.connect(this.masterGainNode);
    } else {
      gain.connect(this.analyser);
    }
    osc.start(now);
    this.activeNodes.set(midi, { osc, filter, gain });
  }

  noteOff(midi) {
    if (this.isWorklet && this.workletNode) {
      this.workletNode.port.postMessage({ type: 'noteOff', note: midi });
      return;
    }
    const nodes = this.activeNodes.get(midi);
    if (!nodes || !this.ctx) return;
    const now = this.ctx.currentTime;
    nodes.gain.gain.cancelScheduledValues(now);
    nodes.gain.gain.setValueAtTime(nodes.gain.gain.value, now);
    nodes.gain.gain.exponentialRampToValueAtTime(0.0001, now + 0.15);
    nodes.osc.stop(now + 0.16);
    this.activeNodes.delete(midi);
  }

  setParam(name, val) {
    if (this.isWorklet && this.workletNode) {
      this.workletNode.port.postMessage({ type: 'param', name, value: val });
      return;
    }
    if (!this.fallbackParams) this.fallbackParams = {};
    this.fallbackParams[name] = val;
    if (this.ctx) {
      const now = this.ctx.currentTime;
      if (name === 'cutoff') {
        for (const nodes of this.activeNodes.values()) {
          nodes.filter.frequency.setValueAtTime(val, now);
        }
      } else if (name === 'masterGain' && this.masterGainNode) {
        this.masterGainNode.gain.setValueAtTime(val, now);
      } else if (name === 'pan' && this.pannerNode) {
        this.pannerNode.pan.setValueAtTime(val, now);
      }
    }
  }

  applyPreset(preset) {
    if (this.isWorklet && this.workletNode) {
      this.workletNode.port.postMessage({ type: 'preset', preset });
      return;
    }
    if (preset && typeof preset === 'object') {
      for (const [k, v] of Object.entries(preset)) {
        this.setParam(k, v);
      }
    }
  }

  getTimeDomainData(array) {
    if (this.analyser) {
      this.analyser.getByteTimeDomainData(array);
    }
  }

  getFrequencyData(array) {
    if (this.analyser) {
      this.analyser.getByteFrequencyData(array);
    }
  }

  startRecording() {
    this.recordedL = [];
    this.recordedR = [];
    this.recording = true;
    if (this.ctx && !this.recorderNode) {
      this.recorderNode = this.ctx.createScriptProcessor(4096, 2, 2);
      this.recorderNode.onaudioprocess = (e) => {
        if (!this.recording) return;
        const inL = e.inputBuffer.getChannelData(0);
        const inR = e.inputBuffer.getChannelData(1);
        this.recordedL.push(new Float32Array(inL));
        this.recordedR.push(new Float32Array(inR));
      };
      this.analyser.connect(this.recorderNode);
      this.recorderNode.connect(this.ctx.destination);
    }
  }

  stopRecording() {
    this.recording = false;
    const totalSamples = this.recordedL.reduce((acc, buf) => acc + buf.length, 0);
    const left = new Float32Array(totalSamples);
    const right = new Float32Array(totalSamples);
    let offset = 0;
    for (let i = 0; i < this.recordedL.length; i++) {
      left.set(this.recordedL[i], offset);
      right.set(this.recordedR[i], offset);
      offset += this.recordedL[i].length;
    }
    return { left, right, sampleRate: this.ctx ? this.ctx.sampleRate : 44100 };
  }
}



