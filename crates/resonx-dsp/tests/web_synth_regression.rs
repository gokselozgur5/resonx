//! Web synthesizer asset and regression verification tests.
use std::fs;
use std::path::Path;

#[test]
fn test_web_synth_assets_exist_and_zero_cdn() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
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
