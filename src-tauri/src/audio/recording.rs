use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Sample, SampleFormat, SupportedStreamConfig};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::audio::devices;
use crate::audio::processing;
use crate::audio::AudioCaptureSource;
use crate::audio::RecorderCommand;

pub fn run_recording_thread(
    cmd_rx: mpsc::Receiver<RecorderCommand>,
    init_tx: mpsc::Sender<Result<(), String>>,
    samples: Arc<Mutex<Vec<f32>>>,
    is_recording: Arc<AtomicBool>,
    input_device_name: Option<String>,
    output_device_name: Option<String>,
    capture_source: AudioCaptureSource,
) -> Result<(), String> {
    println!("[AUDIO] Recording thread started");

    let host = cpal::default_host();
    println!("[AUDIO] Host: {:?}", host.id());

    let mic_samples = Arc::new(Mutex::new(Vec::with_capacity(
        super::INITIAL_BUFFER_CAPACITY,
    )));
    let system_samples = Arc::new(Mutex::new(Vec::with_capacity(
        super::INITIAL_BUFFER_CAPACITY,
    )));
    let mut streams = Vec::new();

    if matches!(
        capture_source,
        AudioCaptureSource::Mic | AudioCaptureSource::Both
    ) {
        let device = devices::select_input_device(&host, input_device_name.as_deref())?;
        let target = if capture_source == AudioCaptureSource::Mic {
            samples.clone()
        } else {
            mic_samples.clone()
        };
        streams.push(build_capture_stream(
            device,
            CaptureDeviceKind::Input,
            target,
            is_recording.clone(),
        )?);
    }

    if matches!(
        capture_source,
        AudioCaptureSource::System | AudioCaptureSource::Both
    ) {
        let device = devices::select_output_device(&host, output_device_name.as_deref())?;
        let target = if capture_source == AudioCaptureSource::System {
            samples.clone()
        } else {
            system_samples.clone()
        };
        streams.push(build_capture_stream(
            device,
            CaptureDeviceKind::OutputLoopback,
            target,
            is_recording.clone(),
        )?);
    }

    if streams.is_empty() {
        let error = "No audio capture source selected".to_string();
        let _ = init_tx.send(Err(error.clone()));
        return Err(error);
    }

    for stream in &streams {
        if let Err(error) = stream
            .play()
            .map_err(|e| format!("Failed to start audio stream: {}", e))
        {
            let _ = init_tx.send(Err(error.clone()));
            return Err(error);
        }
    }

    let _ = init_tx.send(Ok(()));

    loop {
        if let Ok(RecorderCommand::Stop) = cmd_rx.try_recv() {
            break;
        }
        if !is_recording.load(Ordering::SeqCst) {
            break;
        }
        thread::sleep(Duration::from_millis(5));
    }

    drop(streams);

    if capture_source == AudioCaptureSource::Both {
        let mic = mic_samples.lock().unwrap().clone();
        let system = system_samples.lock().unwrap().clone();
        let mixed = processing::mix_audio_sources(&mic, &system);
        *samples.lock().unwrap() = mixed;
    }

    Ok(())
}

pub enum CaptureDeviceKind {
    Input,
    OutputLoopback,
}

fn build_capture_stream(
    device: cpal::Device,
    kind: CaptureDeviceKind,
    samples: Arc<Mutex<Vec<f32>>>,
    is_recording: Arc<AtomicBool>,
) -> Result<cpal::Stream, String> {
    let device_name = device
        .name()
        .unwrap_or_else(|_| "Unknown device".to_string());
    let config = match kind {
        CaptureDeviceKind::Input => device.default_input_config().map_err(|e| {
            format!(
                "Failed to get default input config for {}: {}",
                device_name, e
            )
        })?,
        CaptureDeviceKind::OutputLoopback => device.default_output_config().map_err(|e| {
            format!(
                "Failed to get default output config for system audio device {}: {}",
                device_name, e
            )
        })?,
    };

    println!(
        "[AUDIO] Device: {} | Sample rate: {}, Channels: {}, Format: {:?}",
        device_name,
        config.sample_rate().0,
        config.channels(),
        config.sample_format()
    );

    build_stream_for_config(device, config, samples, is_recording)
}

fn build_stream_for_config(
    device: cpal::Device,
    config: SupportedStreamConfig,
    samples: Arc<Mutex<Vec<f32>>>,
    is_recording: Arc<AtomicBool>,
) -> Result<cpal::Stream, String> {
    let sample_rate = config.sample_rate().0;
    let channels = config.channels() as usize;
    let target_sample_rate = crate::audio::TARGET_SAMPLE_RATE;
    let err_fn = |err| eprintln!("[AUDIO ERROR] Audio stream error: {}", err);

    let stream = match config.sample_format() {
        SampleFormat::F32 => {
            let is_recording = is_recording.clone();
            let samples = samples.clone();
            device.build_input_stream(
                &config.into(),
                move |data: &[f32], _: &_| {
                    if is_recording.load(Ordering::SeqCst) {
                        processing::process_audio_data(
                            data,
                            channels,
                            sample_rate,
                            target_sample_rate,
                            &samples,
                            &is_recording,
                        );
                    }
                },
                err_fn,
                None,
            )
        }
        SampleFormat::I16 => {
            let is_recording = is_recording.clone();
            let samples = samples.clone();
            device.build_input_stream(
                &config.into(),
                move |data: &[i16], _: &_| {
                    if is_recording.load(Ordering::SeqCst) {
                        let float_data: Vec<f32> =
                            data.iter().map(|&s| s.to_float_sample()).collect();
                        processing::process_audio_data(
                            &float_data,
                            channels,
                            sample_rate,
                            target_sample_rate,
                            &samples,
                            &is_recording,
                        );
                    }
                },
                err_fn,
                None,
            )
        }
        SampleFormat::U16 => {
            let is_recording = is_recording.clone();
            let samples = samples.clone();
            device.build_input_stream(
                &config.into(),
                move |data: &[u16], _: &_| {
                    if is_recording.load(Ordering::SeqCst) {
                        let float_data: Vec<f32> =
                            data.iter().map(|&s| s.to_float_sample()).collect();
                        processing::process_audio_data(
                            &float_data,
                            channels,
                            sample_rate,
                            target_sample_rate,
                            &samples,
                            &is_recording,
                        );
                    }
                },
                err_fn,
                None,
            )
        }
        _ => return Err("Unsupported sample format".to_string()),
    };

    stream.map_err(|e| format!("Failed to build audio stream: {}", e))
}
