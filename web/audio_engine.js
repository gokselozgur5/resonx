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
}
