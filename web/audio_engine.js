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
  }

  async init(workletCode) {
    if (this.ctx) return;
    const AudioCtx = window.AudioContext || window.webkitAudioContext;
    this.ctx = new AudioCtx();
    this.analyser = this.ctx.createAnalyser();
    this.analyser.fftSize = 2048;
    this.analyser.smoothingTimeConstant = 0.85;

    try {
      const blob = new Blob([workletCode], { type: 'application/javascript' });
      const url = URL.createObjectURL(blob);
      await this.ctx.audioWorklet.addModule(url);
      URL.revokeObjectURL(url);
      this.workletNode = new AudioWorkletNode(this.ctx, 'resonx-worklet-processor', {
        outputChannelCount: [2]
      });
      this.workletNode.connect(this.analyser);
      this.analyser.connect(this.ctx.destination);
      this.isWorklet = true;
    } catch {
      this.analyser.connect(this.ctx.destination);
      this.isWorklet = false;
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
    }
  }

  noteOff(midi) {
    if (this.isWorklet && this.workletNode) {
      this.workletNode.port.postMessage({ type: 'noteOff', note: midi });
    }
  }

  setParam(name, val) {
    if (this.isWorklet && this.workletNode) {
      this.workletNode.port.postMessage({ type: 'param', name, value: val });
    }
  }

  applyPreset(preset) {
    if (this.isWorklet && this.workletNode) {
      this.workletNode.port.postMessage({ type: 'preset', preset });
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



