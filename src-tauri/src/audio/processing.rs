use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;

pub fn mix_audio_sources(primary: &[f32], secondary: &[f32]) -> Vec<f32> {
    let len = primary.len().max(secondary.len());
    let mut mixed = Vec::with_capacity(len);

    for index in 0..len {
        let a = primary.get(index).copied().unwrap_or(0.0);
        let b = secondary.get(index).copied().unwrap_or(0.0);
        mixed.push(((a + b) * 0.5).clamp(-1.0, 1.0));
    }

    mixed
}

pub fn process_audio_data(
    data: &[f32],
    channels: usize,
    source_rate: u32,
    target_rate: u32,
    samples: &Arc<Mutex<Vec<f32>>>,
    is_recording: &AtomicBool,
) {
    let mono: Vec<f32> = if channels > 1 {
        data.chunks(channels)
            .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
            .collect()
    } else {
        data.to_vec()
    };

    let resampled = if source_rate != target_rate {
        resample(&mono, source_rate, target_rate)
    } else {
        mono
    };

    let mut samples = samples.lock().unwrap();
    let remaining = super::MAX_RECORDING_SAMPLES.saturating_sub(samples.len());
    if remaining == 0 {
        is_recording.store(false, Ordering::SeqCst);
        return;
    }

    if resampled.len() >= remaining {
        samples.extend_from_slice(&resampled[..remaining]);
        is_recording.store(false, Ordering::SeqCst);
    } else {
        samples.extend(resampled);
    }
}

pub fn resample(samples: &[f32], source_rate: u32, target_rate: u32) -> Vec<f32> {
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
