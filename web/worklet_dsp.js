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
}
`;
