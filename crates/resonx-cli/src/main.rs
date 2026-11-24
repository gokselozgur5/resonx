//! Command-line interface and offline renderer for resonx.

use std::env;
use std::process::ExitCode;

use resonx_dsp::synth::{PatchPreset, PolySynth};

mod wav;

fn print_usage() {
    println!("resonx-cli: offline polyphonic DSP synthesizer renderer");
    println!();
    println!("Usage:");
    println!("  resonx render --preset <name> --out <filename.wav>");
    println!("  resonx presets");
    println!();
    println!("Presets: analog-bass, blade-pad, acid-303, sub-pluck");
}

fn parse_preset(name: &str) -> Option<PatchPreset> {
    match name {
        "analog-bass" => Some(PatchPreset::AnalogBass),
        "blade-pad" => Some(PatchPreset::BladePad),
        "acid-303" => Some(PatchPreset::Acid303),
        "sub-pluck" => Some(PatchPreset::SubPluck),
        _ => None,
    }
}

fn render_preset(preset: PatchPreset, out_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let sample_rate = 44100;
    let duration_sec = 2.0;
    let total_samples = (sample_rate as f32 * duration_sec) as usize;

    let mut synth = PolySynth::new(sample_rate as f32);
    synth.set_preset(preset);

    // Play a root triad chord (C3, E3, G3)
    synth.note_on(48, 0.9);
    synth.note_on(52, 0.8);
    synth.note_on(55, 0.85);

    let mut left = vec![0.0_f32; total_samples];
    let mut right = vec![0.0_f32; total_samples];
    synth.render_stereo(&mut left, &mut right);

    wav::write_wav_file(out_path, sample_rate, &left, &right)?;
    println!("Rendered {total_samples} stereo samples to {out_path}");
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return ExitCode::SUCCESS;
    }

    match args[1].as_str() {
        "presets" => {
            println!("Available presets:");
            println!("  analog-bass");
            println!("  blade-pad");
            println!("  acid-303");
            println!("  sub-pluck");
            ExitCode::SUCCESS
        }
        "render" => {
            let mut preset_name = "analog-bass";
            let mut out_file = "output.wav";

            let mut i = 2;
            while i < args.len() {
                if args[i] == "--preset" && i + 1 < args.len() {
                    preset_name = &args[i + 1];
                    i += 2;
                } else if args[i] == "--out" && i + 1 < args.len() {
                    out_file = &args[i + 1];
                    i += 2;
                } else {
                    i += 1;
                }
            }

            let preset = match parse_preset(preset_name) {
                Some(p) => p,
                None => {
                    eprintln!("Unknown preset: {preset_name}");
                    return ExitCode::FAILURE;
                }
            };

            if let Err(e) = render_preset(preset, out_file) {
                eprintln!("Rendering failed: {e}");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        _ => {
            print_usage();
            ExitCode::FAILURE
        }
    }
}
