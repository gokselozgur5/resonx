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
}

