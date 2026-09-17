use hound;

pub fn save_wav(samples: &[f32], path: &str) -> Result<(), String> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(path, spec)
        .map_err(|e| format!("Failed to create WAV file: {}", e))?;

    for &sample in samples {
        let amplitude = (sample * 32767.0) as i16;
        writer
            .write_sample(amplitude)
            .map_err(|e| format!("Failed to write sample: {}", e))?;
    }

    writer
        .finalize()
        .map_err(|e| format!("Failed to finalize WAV: {}", e))?;

    Ok(())
}
