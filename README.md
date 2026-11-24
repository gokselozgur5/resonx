# resonx

Real-time polyphonic DSP synthesizer engine and modular audio tools in Rust.

## Architecture

- `crates/resonx-dsp`: Zero-allocation, `no_std`-compatible DSP core with PolyBLEP anti-aliased oscillators, ZDF Moog ladder and SVF filters, ADSR envelopes, Schroeder reverb, ping-pong delay, and polyphonic voice management.
- `crates/resonx-cli`: Offline deterministic 16-bit stereo PCM WAV renderer and preset preview CLI.

## Quick Start

```bash
cargo build --workspace
cargo test --workspace
cargo run -p resonx-cli -- render --preset analog-bass --out demo.wav
```
