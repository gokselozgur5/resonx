use std::process::Command;

#[test]
fn test_cli_presets_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_resonx-cli"))
        .arg("presets")
        .output()
        .expect("Failed to execute resonx-cli");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("analog-bass"));
    assert!(stdout.contains("blade-pad"));
    assert!(stdout.contains("acid-303"));
    assert!(stdout.contains("sub-pluck"));
}

#[test]
fn test_cli_render_analog_bass_wav() {
    let tmp_wav = std::env::temp_dir().join("resonx_cli_render_test.wav");
    let output = Command::new(env!("CARGO_BIN_EXE_resonx-cli"))
        .args([
            "render",
            "--preset",
            "analog-bass",
            "--out",
            tmp_wav.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute resonx-cli");

    assert!(output.status.success());
    assert!(tmp_wav.exists());
    let bytes = std::fs::read(&tmp_wav).unwrap();
    // 2 seconds of 44.1kHz 16-bit stereo PCM = 44100 * 2 * 4 = 352800 bytes data + 44 header
    assert_eq!(bytes.len(), 352844);
    assert_eq!(&bytes[0..4], b"RIFF");
    assert_eq!(&bytes[8..12], b"WAVE");

    let _ = std::fs::remove_file(tmp_wav);
}
