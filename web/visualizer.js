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
}
