//! Zero-dependency 16-bit stereo PCM WAV file writer.

use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

/// Writes interleaved 16-bit stereo PCM audio to a RIFF/WAVE file.
pub fn write_wav_file<P: AsRef<Path>>(
    path: P,
    sample_rate: u32,
    left: &[f32],
    right: &[f32],
) -> io::Result<()> {
    let num_samples = left.len().min(right.len());
    let mut file = File::create(path)?;

    let num_channels: u16 = 2;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * (num_channels as u32) * ((bits_per_sample / 8) as u32);
    let block_align = num_channels * (bits_per_sample / 8);
    let data_chunk_size = (num_samples as u32) * (block_align as u32);
    let riff_chunk_size = 36 + data_chunk_size;

    // RIFF header
    file.write_all(b"RIFF")?;
    file.write_all(&riff_chunk_size.to_le_bytes())?;
    file.write_all(b"WAVE")?;

    // fmt chunk
    file.write_all(b"fmt ")?;
    file.write_all(&16u32.to_le_bytes())?; // Subchunk1Size (16 for PCM)
    file.write_all(&1u16.to_le_bytes())?; // AudioFormat (1 for PCM)
    file.write_all(&num_channels.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&block_align.to_le_bytes())?;
    file.write_all(&bits_per_sample.to_le_bytes())?;

    // data chunk
    file.write_all(b"data")?;
    file.write_all(&data_chunk_size.to_le_bytes())?;

    // Interleaved 16-bit PCM samples
    let mut buffer = Vec::with_capacity(num_samples * 4);
    for i in 0..num_samples {
        let sample_l = (left[i].clamp(-1.0, 1.0) * 32767.0) as i16;
        let sample_r = (right[i].clamp(-1.0, 1.0) * 32767.0) as i16;
        buffer.extend_from_slice(&sample_l.to_le_bytes());
        buffer.extend_from_slice(&sample_r.to_le_bytes());
    }
    file.write_all(&buffer)?;
    file.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wav_header_structure() {
        let l = [0.1, -0.2, 0.3];
        let r = [-0.1, 0.2, -0.3];
        let tmp_path = std::env::temp_dir().join("resonx_test_header.wav");
        write_wav_file(&tmp_path, 44100, &l, &r).unwrap();

        let bytes = std::fs::read(&tmp_path).unwrap();
        assert!(bytes.len() >= 44);
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(&bytes[12..16], b"fmt ");
        assert_eq!(&bytes[36..40], b"data");

        let _ = std::fs::remove_file(tmp_path);
    }
}
