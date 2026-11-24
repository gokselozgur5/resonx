// Professional tactile vector knob controller with drag, wheel, double-click reset, and fine control
export class ResonxKnob {
  constructor(container, options = {}) {
    this.container = container;
    this.id = options.id || '';
    this.label = options.label || 'PARAM';
    this.unit = options.unit || '';
    this.min = options.min !== undefined ? options.min : 0;
    this.max = options.max !== undefined ? options.max : 100;
    this.step = options.step || 1;
    this.value = options.value !== undefined ? options.value : this.min;
    this.defaultValue = this.value;
    this.color = options.color || '#00e5ff'; // neon cyan
    this.size = options.size || 58;
    this.onChange = options.onChange || null;

    this.startAngle = -135; // degrees
    this.endAngle = 135;   // 270 degree rotation sweep
    this.isDragging = false;
    this.startY = 0;
    this.startVal = 0;

    this.build();
    this.bindEvents();
    this.update();
  }

  build() {
    this.wrapper = document.createElement('div');
    this.wrapper.className = 'resonx-knob-wrapper';

    const r = 24;
    const cx = 29;
    const cy = 29;
    this.r = r;
    this.cx = cx;
    this.cy = cy;

    this.wrapper.innerHTML = `
      <div class="knob-label">${this.label}</div>
      <div class="knob-dial-container" style="width: ${this.size}px; height: ${this.size}px;">
        <svg class="knob-svg" viewBox="0 0 58 58">
          <circle class="knob-track" cx="${cx}" cy="${cy}" r="${r}" fill="none" stroke="#182030" stroke-width="4" stroke-linecap="round" />
          <path class="knob-arc" cx="${cx}" cy="${cy}" r="${r}" fill="none" stroke="${this.color}" stroke-width="4" stroke-linecap="round" />
          <circle class="knob-cap" cx="${cx}" cy="${cy}" r="${r - 5}" fill="url(#knob-grad)" stroke="#26334d" stroke-width="1.5" />
          <line class="knob-pointer" x1="${cx}" y1="${cy - 7}" x2="${cx}" y2="${cy - r + 8}" stroke="${this.color}" stroke-width="2.5" stroke-linecap="round" />
          <defs>
            <radialGradient id="knob-grad" cx="40%" cy="35%" r="70%">
              <stop offset="0%" stop-color="#2a364f" />
              <stop offset="60%" stop-color="#141a26" />
              <stop offset="100%" stop-color="#0a0e17" />
            </radialGradient>
          </defs>
        </svg>
      </div>
      <div class="knob-readout">
        <span class="knob-val">0</span><span class="knob-unit">${this.unit}</span>
      </div>
    `;

    this.container.appendChild(this.wrapper);
    this.svg = this.wrapper.querySelector('.knob-svg');
    this.arcPath = this.wrapper.querySelector('.knob-arc');
    this.pointer = this.wrapper.querySelector('.knob-pointer');
    this.track = this.wrapper.querySelector('.knob-track');
    this.readoutVal = this.wrapper.querySelector('.knob-val');
    this.dialContainer = this.wrapper.querySelector('.knob-dial-container');

    // Build static track arc
    this.setupTrackArc();
  }

  polarToCartesian(cx, cy, r, angleInDegrees) {
    const angleInRadians = (angleInDegrees - 90) * Math.PI / 180.0;
    return {
      x: cx + (r * Math.cos(angleInRadians)),
      y: cy + (r * Math.sin(angleInRadians))
    };
  }

  describeArc(x, y, radius, startAngle, endAngle) {
    const start = this.polarToCartesian(x, y, radius, endAngle);
    const end = this.polarToCartesian(x, y, radius, startAngle);
    const largeArcFlag = endAngle - startAngle <= 180 ? '0' : '1';
    return [
      'M', start.x, start.y,
      'A', radius, radius, 0, largeArcFlag, 0, end.x, end.y
    ].join(' ');
  }

  setupTrackArc() {
    const d = this.describeArc(this.cx, this.cy, this.r, this.startAngle, this.endAngle);
    this.track.setAttribute('d', d);
  }

  setValue(val, triggerChange = true) {
    const clamped = Math.max(this.min, Math.min(this.max, val));
    const stepped = Math.round((clamped - this.min) / this.step) * this.step + this.min;
    const precision = this.step < 0.01 ? 3 : (this.step < 1 ? 2 : (this.step % 1 === 0 ? 0 : 1));
    this.value = parseFloat(stepped.toFixed(precision));
    this.update();
    if (triggerChange && this.onChange) {
      this.onChange(this.value);
    }
  }

  update() {
    const norm = (this.value - this.min) / (this.max - this.min || 1);
    const currentAngle = this.startAngle + norm * (this.endAngle - this.startAngle);

    // Update active arc
    if (norm > 0.005) {
      const d = this.describeArc(this.cx, this.cy, this.r, this.startAngle, currentAngle);
      this.arcPath.setAttribute('d', d);
      this.arcPath.style.display = 'block';
    } else {
      this.arcPath.style.display = 'none';
    }

    // Rotate pointer around center
    this.pointer.setAttribute('transform', `rotate(${currentAngle}, ${this.cx}, ${this.cy})`);

    // Format readout text
    if (this.readoutVal) {
      const precision = this.step < 0.01 ? 3 : (this.step < 0.1 ? 2 : (this.step < 1 ? 1 : 0));
      this.readoutVal.textContent = this.value >= 1000 ? (this.value / 1000).toFixed(1) + 'k' : this.value.toFixed(precision);
    }
  }

  bindEvents() {
    const onPointerDown = (e) => {
      this.isDragging = true;
      this.startY = e.clientY;
      this.startVal = this.value;
      this.dialContainer.classList.add('active');
      window.addEventListener('pointermove', onPointerMove);
      window.addEventListener('pointerup', onPointerUp);
      e.preventDefault();
    };

    const onPointerMove = (e) => {
      if (!this.isDragging) return;
      const dy = this.startY - e.clientY;
      const range = this.max - this.min;
      const pixelsForFullRange = e.shiftKey ? 800 : 180; // Shift for fine-tuning!
      const deltaVal = (dy / pixelsForFullRange) * range;
      this.setValue(this.startVal + deltaVal, true);
    };

    const onPointerUp = () => {
      if (!this.isDragging) return;
      this.isDragging = false;
      this.dialContainer.classList.remove('active');
      window.removeEventListener('pointermove', onPointerMove);
      window.removeEventListener('pointerup', onPointerUp);
    };

    this.dialContainer.addEventListener('pointerdown', onPointerDown);

    // Mouse wheel adjustments
    this.dialContainer.addEventListener('wheel', (e) => {
      e.preventDefault();
      const direction = e.deltaY < 0 ? 1 : -1;
      const stepDelta = (e.shiftKey ? this.step : this.step * 2) * direction;
      this.setValue(this.value + stepDelta, true);
    }, { passive: false });

    // Double click to reset to default
    this.dialContainer.addEventListener('dblclick', (e) => {
      e.preventDefault();
      this.setValue(this.defaultValue, true);
    });
  }
}
