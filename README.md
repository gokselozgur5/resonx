# resonx

Real-time polyphonic DSP synthesizer engine and modular audio tools in Rust.

## Architecture

- `web/`: Browser synthesizer running off-thread via AudioWorklet with oscilloscope, FFT, and WAV export.
- `crates/resonx-dsp`: Zero-allocation, `no_std`-compatible DSP core with PolyBLEP anti-aliased oscillators, ZDF Moog ladder and SVF filters, ADSR envelopes, Schroeder reverb, ping-pong delay, and polyphonic voice management.
- `crates/resonx-cli`: Offline deterministic 16-bit stereo PCM WAV renderer and preset preview CLI.

## Live Demo

Play the live browser synthesizer directly on GitHub Pages:
[https://gokselozgur5.github.io/resonx/](https://gokselozgur5.github.io/resonx/)


## Quick Start

```bash
cargo build --workspace
cargo test --workspace
cargo run -p resonx-cli -- render --preset analog-bass --out demo.wav
```
