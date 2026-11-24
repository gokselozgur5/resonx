// Interactive 2-octave piano keyboard builder and input mapper
export const KEYBOARD_MAP = {
  KeyZ: 48, KeyS: 49, KeyX: 50, KeyD: 51, KeyC: 52, KeyV: 53,
  KeyG: 54, KeyB: 55, KeyH: 56, KeyN: 57, KeyJ: 58, KeyM: 59,
  KeyQ: 60, Digit2: 61, KeyW: 62, Digit3: 63, KeyE: 64, KeyR: 65,
  Digit5: 66, KeyT: 67, Digit6: 68, KeyY: 69, Digit7: 70, KeyU: 71
};

export const NOTE_NAMES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

export class ResonxKeyboard {
  constructor(container, onNoteOn, onNoteOff) {
    this.container = container;
    this.onNoteOn = onNoteOn;
    this.onNoteOff = onNoteOff;
    this.activeNotes = new Set();
    this.keyElements = new Map();
    this.build();
    this.bindEvents();
  }

  build() {
    this.container.innerHTML = '';
    const startNote = 48; // C3
    const endNote = 72;   // C5
    for (let note = startNote; note <= endNote; note++) {
      const isBlack = [1, 3, 6, 8, 10].includes(note % 12);
      const key = document.createElement('div');
      key.className = `key ${isBlack ? 'black-key' : 'white-key'}`;
      key.dataset.note = note;
      const noteLabel = `${NOTE_NAMES[note % 12]}${Math.floor(note / 12) - 1}`;
      key.innerHTML = `<span class="key-label">${noteLabel}</span>`;
      this.container.appendChild(key);
      this.keyElements.set(note, key);
    }
    this.layoutBlackKeys();
  }

  layoutBlackKeys() {
    const whiteKeys = Array.from(this.container.querySelectorAll('.white-key'));
    const blackOffsets = { 1: 0, 3: 1, 6: 3, 8: 4, 10: 5 };
    const octaveBaseWhite = [0, 7]; // C3, C4
    octaveBaseWhite.forEach((baseIdx, oct) => {
      Object.entries(blackOffsets).forEach(([mod, wOffset]) => {
        const note = 48 + oct * 12 + parseInt(mod, 10);
        const el = this.keyElements.get(note);
        const refWhite = whiteKeys[baseIdx + wOffset];
        if (el && refWhite) {
          const leftPct = ((baseIdx + wOffset + 0.68) / whiteKeys.length) * 100;
          el.style.left = `${leftPct}%`;
        }
      });
    });
  }

  setKeyActive(note, active) {
    const el = this.keyElements.get(note);
    if (!el) return;
    if (active) el.classList.add('active');
    else el.classList.remove('active');
  }

  triggerNoteOn(note) {
    if (this.activeNotes.has(note)) return;
    this.activeNotes.add(note);
    this.setKeyActive(note, true);
    if (this.onNoteOn) this.onNoteOn(note, 0.8);
  }

  triggerNoteOff(note) {
    if (!this.activeNotes.has(note)) return;
    this.activeNotes.delete(note);
    this.setKeyActive(note, false);
    if (this.onNoteOff) this.onNoteOff(note);
  }

  releaseAll() {
    this.activeNotes.forEach(note => {
      this.setKeyActive(note, false);
      if (this.onNoteOff) this.onNoteOff(note);
    });
    this.activeNotes.clear();
  }

  bindEvents() {
    this.container.addEventListener('pointerdown', (e) => {
      const key = e.target.closest('.key');
      if (!key) return;
      const note = parseInt(key.dataset.note, 10);
      this.triggerNoteOn(note);
    });

    this.container.addEventListener('pointercancel', () => this.releaseAll());
    window.addEventListener('pointerup', () => this.releaseAll());
    window.addEventListener('blur', () => this.releaseAll());
    document.addEventListener('visibilitychange', () => {
      if (document.hidden) this.releaseAll();
    });

    window.addEventListener('keydown', (e) => {
      if (e.repeat || e.target.tagName === 'INPUT') return;
      const note = KEYBOARD_MAP[e.code];
      if (note !== undefined) this.triggerNoteOn(note);
    });

    window.addEventListener('keyup', (e) => {
      const note = KEYBOARD_MAP[e.code];
      if (note !== undefined) this.triggerNoteOff(note);
    });

    if (navigator.requestMIDIAccess) {
      navigator.requestMIDIAccess().then(access => {
        for (const input of access.inputs.values()) {
          input.onmidimessage = (m) => {
            const [status, note, vel] = m.data;
            const cmd = status >> 4;
            if (cmd === 9 && vel > 0) this.triggerNoteOn(note);
            else if (cmd === 8 || (cmd === 9 && vel === 0)) this.triggerNoteOff(note);
          };
        }
      }).catch(() => {});
    }
  }
}





