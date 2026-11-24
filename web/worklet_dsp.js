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
}
`;
