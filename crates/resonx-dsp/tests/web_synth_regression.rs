//! Web synthesizer asset and regression verification tests.
use std::fs;
use std::path::Path;

#[test]
fn test_web_synth_assets_exist_and_zero_cdn() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let index_path = root.join("index.html");
    assert!(index_path.exists(), "index.html must exist at repo root");

    let index_content = fs::read_to_string(&index_path).expect("Failed to read index.html");
    assert!(
        !index_content.contains("http://") && !index_content.contains("https://"),
        "index.html must not contain external CDN or remote URLs"
    );
    assert!(index_content.contains("RESONX // POLYPHONIC SYNTH"));
    assert!(index_content.contains("visualizer-canvas"));
    assert!(index_content.contains("keyboard-container"));
}

#[test]
fn test_web_js_modules_and_styles_exist() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let web_dir = root.join("web");
    let required_files = [
        "presets.js",
        "worklet_dsp.js",
        "audio_engine.js",
        "wav_export.js",
        "visualizer.js",
        "keyboard.js",
        "styles.css",
        "app.js",
    ];

    for file in &required_files {
        let p = web_dir.join(file);
        assert!(p.exists(), "Required web asset {file} must exist");
        let content = fs::read_to_string(&p).expect("Failed to read asset");
        assert!(
            !content.contains("http://") && !content.contains("https://"),
            "Web asset {file} must not contain external URLs"
        );
    }
}

#[test]
fn test_worklet_dsp_has_polyblep_and_filter() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let worklet_js =
        fs::read_to_string(root.join("web/worklet_dsp.js")).expect("Failed to read worklet_dsp.js");
    assert!(worklet_js.contains("RESONX_WORKLET_CODE"));
    assert!(worklet_js.contains("polyBlep"));
    assert!(worklet_js.contains("processLadderFilter"));
    assert!(worklet_js.contains("ResonxWorkletProcessor"));
    assert!(worklet_js.contains("registerProcessor"));
    assert!(
        worklet_js.contains("const rate = 1.0 / (Math.max(0.001, r) * this.sampleRate);"),
        "Envelope release rate must use unit scale to prevent stuck voices on zero-sustain patches"
    );

    let presets_js =
        fs::read_to_string(root.join("web/presets.js")).expect("Failed to read presets.js");
    assert!(presets_js.contains("analog-bass"));
    assert!(presets_js.contains("blade-pad"));
    assert!(presets_js.contains("DEMO_SEQUENCE"));

    let app_js = fs::read_to_string(root.join("web/app.js")).expect("Failed to read app.js");
    assert!(
        app_js.contains("if (!keyboard.activeNotes.has(note)) return;"),
        "app.js must check activeNotes after ensureInit to prevent stuck notes on quick tap"
    );

    let keyboard_js =
        fs::read_to_string(root.join("web/keyboard.js")).expect("Failed to read keyboard.js");
    assert!(
        keyboard_js.contains("releaseAll()"),
        "keyboard.js must provide releaseAll method"
    );
    assert!(
        keyboard_js.contains("visibilitychange"),
        "keyboard.js must bind releaseAll to visibilitychange"
    );

    let audio_engine_js =
        fs::read_to_string(root.join("web/audio_engine.js")).expect("Failed to read audio_engine.js");
    assert!(
        audio_engine_js.contains("finally") && audio_engine_js.contains("URL.revokeObjectURL"),
        "audio_engine.js must revoke blob URL inside finally block"
    );
    assert!(
        audio_engine_js.contains("createOscillator")
            && audio_engine_js.contains("createBiquadFilter")
            && audio_engine_js.contains("createGain"),
        "audio_engine.js must implement fallback polyphonic WebAudio synthesis"
    );

    let wav_js =
        fs::read_to_string(root.join("web/wav_export.js")).expect("Failed to read wav_export.js");
    assert!(wav_js.contains("encodeWav16"));
}
