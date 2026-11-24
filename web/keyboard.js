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
  }
}


