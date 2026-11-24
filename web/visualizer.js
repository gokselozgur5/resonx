// 60 FPS Canvas dual visualizer: oscilloscope waveform + FFT spectrum
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
    const w = this.canvas.width; const h = this.canvas.height;
    this.ctx.fillStyle = '#0b0f19';
    this.ctx.fillRect(0, 0, w, h);

    this.engine.getFrequencyData(this.freqData);
    const barCount = 48; const barWidth = (w / barCount) - 2;
    for (let i = 0; i < barCount; i++) {
      const idx = Math.floor(Math.pow(i / barCount, 1.5) * (this.freqData.length - 1));
      const val = this.freqData[idx] / 255.0;
      const barH = val * (h * 0.7);
      const x = i * (barWidth + 2);
      this.ctx.fillStyle = i < barCount * 0.6 ? '#10b981' : '#f59e0b';
      this.ctx.globalAlpha = 0.4 + val * 0.5;
      this.ctx.fillRect(x, h - barH, barWidth, barH);
    }
    this.ctx.globalAlpha = 1.0;

    this.engine.getTimeDomainData(this.timeData);
    this.ctx.lineWidth = 2.5;
    this.ctx.strokeStyle = '#06b6d4'; // neon cyan
    this.ctx.shadowColor = '#06b6d4';
    this.ctx.shadowBlur = 8;
    this.ctx.beginPath();
    const sliceWidth = w / this.timeData.length;
    let px = 0;
    for (let i = 0; i < this.timeData.length; i++) {
      const v = this.timeData[i] / 128.0;
      const py = (v * h) / 2.0;
      if (i === 0) this.ctx.moveTo(px, py);
      else this.ctx.lineTo(px, py);
      px += sliceWidth;
    }
    this.ctx.stroke();
    this.ctx.shadowBlur = 0;
  }
}



