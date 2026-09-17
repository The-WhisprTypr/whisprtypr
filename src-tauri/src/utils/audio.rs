use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::utils::AUDIO_TARGET_SAMPLE_RATE;
use crate::utils::MAX_FILE_AUDIO_SAMPLES;
use crate::utils::MAX_FILE_TRANSCRIPTION_SECONDS;

pub fn read_audio_file(file_path: &Path) -> Result<Vec<f32>, String> {
    use std::fs::File;
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let file = File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;

    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("Failed to probe audio format: {}", e))?;

    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or_else(|| "No audio track found".to_string())?;

    let track_id = track.id;
    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    let channels = track
        .codec_params
        .channels
        .map(|c| c.count())
        .unwrap_or(2)
        .max(1);

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| format!("Failed to create decoder: {}", e))?;

    let mut samples =
        Vec::with_capacity((AUDIO_TARGET_SAMPLE_RATE as usize * 60).min(MAX_FILE_AUDIO_SAMPLES));

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break
            }
            Err(symphonia::core::errors::Error::ResetRequired) => {
                decoder.reset();
                continue;
            }
            Err(e) => return Err(format!("Failed to read packet: {}", e)),
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(e) => return Err(format!("Failed to decode: {}", e)),
        };

        let spec = *decoded.spec();
        let duration = decoded.capacity() as u64;
        let mut sample_buf = SampleBuffer::<f32>::new(duration, spec);
        sample_buf.copy_interleaved_ref(decoded);

        let mono = interleaved_to_mono(sample_buf.samples(), channels);

        let normalized = if sample_rate != AUDIO_TARGET_SAMPLE_RATE {
            resample_audio(&mono, sample_rate, AUDIO_TARGET_SAMPLE_RATE)
        } else {
            mono
        };

        append_audio_samples_with_limit(&mut samples, &normalized, MAX_FILE_AUDIO_SAMPLES)?;
    }

    Ok(samples)
}

pub fn append_audio_samples_with_limit(
    target: &mut Vec<f32>,
    source: &[f32],
    max_samples: usize,
) -> Result<(), String> {
    let remaining = max_samples.saturating_sub(target.len());
    if source.len() > remaining {
        return Err(format!(
            "Audio is too long. Maximum supported duration is {} minutes.",
            MAX_FILE_TRANSCRIPTION_SECONDS / 60
        ));
    }

    target.extend_from_slice(source);
    Ok(())
}

pub fn interleaved_to_mono(samples: &[f32], channels: usize) -> Vec<f32> {
    if channels > 1 {
        samples
            .chunks(channels)
            .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
            .collect()
    } else {
        samples.to_vec()
    }
}

pub fn resample_audio(samples: &[f32], source_rate: u32, target_rate: u32) -> Vec<f32> {
    let ratio = source_rate as f64 / target_rate as f64;
    let output_len = (samples.len() as f64 / ratio) as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_idx = i as f64 * ratio;
        let idx = src_idx as usize;
        let frac = src_idx - idx as f64;

        let sample = if idx + 1 < samples.len() {
            samples[idx] * (1.0 - frac as f32) + samples[idx + 1] * frac as f32
        } else if idx < samples.len() {
            samples[idx]
        } else {
            0.0
        };

        output.push(sample);
    }

    output
}
