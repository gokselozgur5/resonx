// Self-contained AudioWorklet processor code for resonx
export const RESONX_WORKLET_CODE = `
class ResonxVoice {
  constructor(sampleRate) {
    this.sampleRate = sampleRate;
    this.active = false;
    this.note = 0;
    this.velocity = 0;
    this.phase1 = 0;
    this.phase2 = 0;
    this.envLevel = 0;
    this.envStage = 0; // 0=idle, 1=att, 2=dec, 3=sus, 4=rel
    this.filterEnvLevel = 0;
    this.filterEnvStage = 0;
    this.s1 = 0; this.s2 = 0; this.s3 = 0; this.s4 = 0;
    this.age = 0;
  }

  polyBlep(t, dt) {
    if (t < dt) { const x = t / dt; return x + x - x * x - 1.0; }
    if (t > 1.0 - dt) { const x = (t - 1.0) / dt; return x * x + x + x + 1.0; }
    return 0.0;
  }

  oscSample(type, phase, dt) {
    if (type === 'saw') return (2.0 * phase - 1.0) - this.polyBlep(phase, dt);
    if (type === 'square') {
      let v = phase < 0.5 ? 1.0 : -1.0;
      v += this.polyBlep(phase, dt);
      v -= this.polyBlep((phase + 0.5) % 1.0, dt);
      return v;
    }
    if (type === 'triangle') return 2.0 * Math.abs(2.0 * phase - 1.0) - 1.0;
    if (type === 'sine') return Math.sin(2.0 * Math.PI * phase);
    return Math.random() * 2.0 - 1.0; // noise
  }

  processLadderFilter(input, cutoff, resonance, drive) {
    const wc = 2.0 * Math.PI * Math.min(cutoff, this.sampleRate * 0.45);
    const g = Math.tan((wc * 0.5) / this.sampleRate);
    const g1 = g / (1.0 + g);
    const drivenInput = Math.tanh(input * drive);
    const resBounded = Math.min(resonance * 3.8, 3.85);
    const fb = this.s4 * resBounded;
    const u = (drivenInput - fb) / (1.0 + resBounded * g1 * g1 * g1 * g1);

    const v1 = (u - this.s1) * g1; const y1 = v1 + this.s1; this.s1 = y1 + v1;
    const v2 = (y1 - this.s2) * g1; const y2 = v2 + this.s2; this.s2 = y2 + v2;
    const v3 = (y2 - this.s3) * g1; const y3 = v3 + this.s3; this.s3 = y3 + v3;
    const v4 = (y3 - this.s4) * g1; const y4 = v4 + this.s4; this.s4 = y4 + v4;

    return Number.isFinite(y4) ? y4 : 0.0;
  }

  processEnvelope(stage, level, a, d, s, r) {
    if (stage === 1) { // attack
      const rate = 1.0 / (Math.max(0.001, a) * this.sampleRate);
      level += rate;
      if (level >= 1.0) { level = 1.0; stage = 2; }
    } else if (stage === 2) { // decay
      const rate = (1.0 - s) / (Math.max(0.001, d) * this.sampleRate);
      level -= rate;
      if (level <= s) { level = s; stage = 3; }
    } else if (stage === 3) { // sustain
      level = s;
    } else if (stage === 4) { // release
      const rate = 1.0 / (Math.max(0.001, r) * this.sampleRate);
      level -= rate;
      if (level <= 0.0001) { level = 0.0; stage = 0; }
    }
    return [stage, level];
  }

  processSample(params, lfoVal) {
    if (this.envStage === 0) { this.active = false; return 0.0; }
    this.age++;

    const freq = 440.0 * Math.pow(2.0, (this.note - 69) / 12.0);
    const dt1 = freq / this.sampleRate;
    const dt2 = (freq * (1.0 + params.detune * 0.001)) / this.sampleRate;

    this.phase1 = (this.phase1 + dt1) % 1.0;
    this.phase2 = (this.phase2 + dt2) % 1.0;

    const s1 = this.oscSample(params.osc1, this.phase1, dt1);
    const s2 = this.oscSample(params.osc2, this.phase2, dt2);
    const oscMix = 0.5 * (s1 + s2);

    const [fStage, fLvl] = this.processEnvelope(
      this.filterEnvStage, this.filterEnvLevel,
      params.attack, params.decay, params.sustain, params.release
    );
    this.filterEnvStage = fStage; this.filterEnvLevel = fLvl;

    const modCutoff = Math.max(20.0, Math.min(18000.0,
      params.cutoff + fLvl * params.envAmount + lfoVal * params.lfoDepth
    ));

    const filtered = this.processLadderFilter(oscMix, modCutoff, params.resonance, params.drive);

    const [aStage, aLvl] = this.processEnvelope(
      this.envStage, this.envLevel,
      params.attack, params.decay, params.sustain, params.release
    );
    this.envStage = aStage; this.envLevel = aLvl;

    return filtered * aLvl * this.velocity;
  }
}

class ResonxWorkletProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.voices = Array.from({ length: 8 }, () => new ResonxVoice(sampleRate));
    this.lfoPhase = 0;
    this.params = {
      osc1: 'saw', osc2: 'square', detune: 4,
      cutoff: 400, resonance: 0.6, drive: 1.8, envAmount: 2000,
      attack: 0.005, decay: 0.15, sustain: 0.4, release: 0.1,
      lfoRate: 1.5, lfoDepth: 80, delayMix: 0.2, delayTime: 0.25,
      delayFeedback: 0.35, pan: 0.0, masterGain: 0.8
    };
    this.delayBufferL = new Float32Array(sampleRate * 2);
    this.delayBufferR = new Float32Array(sampleRate * 2);
    this.delayWriteIdx = 0;

    this.port.onmessage = (e) => this.handleMessage(e.data);
  }

  handleMessage(msg) {
    if (msg.type === 'noteOn') {
      let voice = this.voices.find(v => !v.active);
      if (!voice) {
        voice = this.voices.reduce((oldest, v) => (v.age > oldest.age ? v : oldest), this.voices[0]);
      }
      voice.note = msg.note; voice.velocity = msg.velocity || 0.8;
      voice.phase1 = 0; voice.phase2 = 0;
      voice.envStage = 1; voice.filterEnvStage = 1;
      voice.active = true; voice.age = 0;
    } else if (msg.type === 'noteOff') {
      this.voices.forEach(v => {
        if (v.note === msg.note && v.active) {
          v.envStage = 4; v.filterEnvStage = 4;
        }
      });
    } else if (msg.type === 'param') {
      this.params[msg.name] = msg.value;
    } else if (msg.type === 'preset') {
      Object.assign(this.params, msg.preset);
    }
  }

  process(inputs, outputs) {
    const outL = outputs[0][0]; const outR = outputs[0][1] || outputs[0][0];
    const frames = outL.length;
    const lfoDt = (this.params.lfoRate || 1.0) / sampleRate;
    const delaySamples = Math.max(1, Math.min(this.delayBufferL.length - 1,
      Math.floor((this.params.delayTime || 0.25) * sampleRate)
    ));
    const dMix = this.params.delayMix || 0.0;
    const dFb = Math.min(0.85, this.params.delayFeedback || 0.3);
    const pan = Math.max(-1.0, Math.min(1.0, this.params.pan || 0.0));
    const panL = Math.cos((pan + 1.0) * Math.PI * 0.25);
    const panR = Math.sin((pan + 1.0) * Math.PI * 0.25);

    for (let i = 0; i < frames; i++) {
      this.lfoPhase = (this.lfoPhase + lfoDt) % 1.0;
      const lfoVal = Math.sin(2.0 * Math.PI * this.lfoPhase);
      let mix = 0.0;
      for (let v = 0; v < 8; v++) {
        if (this.voices[v].active) mix += this.voices[v].processSample(this.params, lfoVal);
      }
      const direct = mix * 0.2 * (this.params.masterGain !== undefined ? this.params.masterGain : 0.8);
      const directL = direct * panL * 1.414;
      const directR = direct * panR * 1.414;
      const readIdx = (this.delayWriteIdx - delaySamples + this.delayBufferL.length) % this.delayBufferL.length;
      const dL = this.delayBufferL[readIdx]; const dR = this.delayBufferR[readIdx];
      this.delayBufferL[this.delayWriteIdx] = directL + dR * dFb;
      this.delayBufferR[this.delayWriteIdx] = directR + dL * dFb;
      this.delayWriteIdx = (this.delayWriteIdx + 1) % this.delayBufferL.length;

      outL[i] = Math.max(-1.0, Math.min(1.0, directL * (1.0 - dMix) + dL * dMix));
      outR[i] = Math.max(-1.0, Math.min(1.0, directR * (1.0 - dMix) + dR * dMix));
    }
    return true;
  }
}
registerProcessor('resonx-worklet-processor', ResonxWorkletProcessor);
`;
