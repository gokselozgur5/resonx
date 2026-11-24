// 60 FPS Canvas visualizer: Dual-mode CRT phosphor oscilloscope + FFT waterfall spectrum
export class ResonxVisualizer {
  constructor(canvas, engine) {
    this.canvas = canvas;
    this.ctx = canvas.getContext('2d');
    this.engine = engine;
    this.timeData = new Uint8Array(1024);
    this.freqData = new Uint8Array(256);
    this.running = false;
  }

  start() {
    if (this.running) return;
    this.running = true;
    const render = () => {
      if (!this.running) return;
      this.draw();
      requestAnimationFrame(render);
    };
    requestAnimationFrame(render);
  }

  stop() {
    this.running = false;
  }

  draw() {
    const w = this.canvas.width;
    const h = this.canvas.height;

    // Dark phosphor CRT background
    this.ctx.fillStyle = '#06090e';
    this.ctx.fillRect(0, 0, w, h);

    // Subtle retro grid lines
    this.ctx.strokeStyle = 'rgba(0, 229, 255, 0.05)';
    this.ctx.lineWidth = 1;
    this.ctx.beginPath();
    for (let x = 0; x < w; x += 40) {
      this.ctx.moveTo(x, 0);
      this.ctx.lineTo(x, h);
    }
    for (let y = 0; y < h; y += 28) {
      this.ctx.moveTo(0, y);
      this.ctx.lineTo(w, y);
    }
    this.ctx.stroke();

    // FFT Spectrum Analyzer bars (Stereo / Frequency bins)
    this.engine.getFrequencyData(this.freqData);
    const barCount = 64;
    const barWidth = (w / barCount) - 2;

    for (let i = 0; i < barCount; i++) {
      const idx = Math.floor(Math.pow(i / barCount, 1.4) * (this.freqData.length - 1));
      const val = this.freqData[idx] / 255.0;
      const barH = val * (h * 0.75);
      const x = i * (barWidth + 2);

      // Color gradient from emerald to amber to rose
      let fill = '#00e676';
      if (i > barCount * 0.6) fill = '#ff9100';
      if (i > barCount * 0.85) fill = '#ff1744';

      this.ctx.fillStyle = fill;
      this.ctx.globalAlpha = 0.25 + val * 0.6;
      this.ctx.fillRect(x, h - barH, barWidth, barH);
    }
    this.ctx.globalAlpha = 1.0;

    // Oscilloscope Time-Domain waveform (Phosphor Cyan)
    this.engine.getTimeDomainData(this.timeData);
    this.ctx.lineWidth = 2.2;
    this.ctx.strokeStyle = '#00e5ff';
    this.ctx.shadowColor = '#00e5ff';
    this.ctx.shadowBlur = 10;
    this.ctx.beginPath();

    const sliceWidth = w / this.timeData.length;
    let px = 0;
    for (let i = 0; i < this.timeData.length; i++) {
      const v = this.timeData[i] / 128.0;
      const py = (v * h) / 2.0;
      if (i === 0) {
        this.ctx.moveTo(px, py);
      } else {
        this.ctx.lineTo(px, py);
      }
      px += sliceWidth;
    }
    this.ctx.stroke();
    this.ctx.shadowBlur = 0;
  }
}
