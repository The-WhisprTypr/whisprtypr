use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat, SupportedStreamConfig};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

// Pre-allocate buffer for ~30 seconds of 16kHz mono audio
// This reduces dynamic allocations during recording
const TARGET_SAMPLE_RATE: u32 = 16_000;
const INITIAL_BUFFER_CAPACITY: usize = TARGET_SAMPLE_RATE as usize * 30;
const MAX_RECORDING_SECONDS: usize = 5 * 60;
const MAX_RECORDING_SAMPLES: usize = TARGET_SAMPLE_RATE as usize * MAX_RECORDING_SECONDS;

pub enum RecorderCommand {
    Stop,
}

pub struct AudioRecorder {
    samples: Arc<Mutex<Vec<f32>>>,
    is_recording: Arc<AtomicBool>,
    input_device_name: Option<String>,
    output_device_name: Option<String>,
    capture_source: AudioCaptureSource,
    command_sender: Option<mpsc::Sender<RecorderCommand>>,
    thread_handle: Option<JoinHandle<()>>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AudioCaptureSource {
    Mic,
    System,
    Both,
}

#[derive(Debug, Clone, Serialize)]
pub struct AudioInputDevice {
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AudioOutputDevice {
    pub name: String,
    pub is_default: bool,
}

// Make AudioRecorder Send + Sync by not storing the Stream
unsafe impl Send for AudioRecorder {}
unsafe impl Sync for AudioRecorder {}

impl AudioRecorder {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            samples: Arc::new(Mutex::new(Vec::with_capacity(INITIAL_BUFFER_CAPACITY))),
            is_recording: Arc::new(AtomicBool::new(false)),
            input_device_name: None,
            output_device_name: None,
            capture_source: AudioCaptureSource::Mic,
            command_sender: None,
            thread_handle: None,
        })
    }

    pub fn list_input_devices() -> Result<Vec<AudioInputDevice>, String> {
        let host = cpal::default_host();
        let default_name = host
            .default_input_device()
            .and_then(|device| device.name().ok());

        let devices = host
            .input_devices()
            .map_err(|e| format!("Failed to list input devices: {}", e))?
            .filter_map(|device| {
                let name = device.name().ok()?;
                Some(AudioInputDevice {
                    is_default: default_name.as_deref() == Some(name.as_str()),
                    name,
                })
            })
            .collect();

        Ok(devices)
    }

    pub fn list_output_devices() -> Result<Vec<AudioOutputDevice>, String> {
        let host = cpal::default_host();
        let default_name = host
            .default_output_device()
            .and_then(|device| device.name().ok());

        let devices = host
            .output_devices()
            .map_err(|e| format!("Failed to list output devices: {}", e))?
            .filter_map(|device| {
                let name = device.name().ok()?;
                Some(AudioOutputDevice {
                    is_default: default_name.as_deref() == Some(name.as_str()),
                    name,
                })
            })
            .collect();

        Ok(devices)
    }

    pub fn set_input_device(&mut self, name: Option<String>) -> Result<(), String> {
        if self.is_recording.load(Ordering::SeqCst) {
            return Err("Cannot change input device while recording".to_string());
        }

        if let Some(ref device_name) = name {
            let exists = Self::list_input_devices()?
                .iter()
                .any(|device| device.name == *device_name);
            if !exists {
                return Err(format!("Input device not found: {}", device_name));
            }
        }

        self.input_device_name = name;
        Ok(())
    }

    pub fn set_capture_config(
        &mut self,
        capture_source: AudioCaptureSource,
        input_device_name: Option<String>,
        output_device_name: Option<String>,
    ) -> Result<(), String> {
        if self.is_recording.load(Ordering::SeqCst) {
            return Err("Cannot change audio capture source while recording".to_string());
        }

        if matches!(
            capture_source,
            AudioCaptureSource::Mic | AudioCaptureSource::Both
        ) {
            if let Some(ref device_name) = input_device_name {
                let exists = Self::list_input_devices()?
                    .iter()
                    .any(|device| device.name == *device_name);
                if !exists {
                    return Err(format!("Input device not found: {}", device_name));
                }
            }
        }

        if matches!(
            capture_source,
            AudioCaptureSource::System | AudioCaptureSource::Both
        ) {
            if let Some(ref device_name) = output_device_name {
                let exists = Self::list_output_devices()?
                    .iter()
                    .any(|device| device.name == *device_name);
                if !exists {
                    return Err(format!("Output device not found: {}", device_name));
                }
            }
        }

        self.capture_source = capture_source;
        self.input_device_name = input_device_name;
        self.output_device_name = output_device_name;
        Ok(())
    }

    pub fn start_recording(&mut self) -> Result<(), String> {
        if self.is_recording.load(Ordering::SeqCst) {
            return Err("Already recording".to_string());
        }

        // Clear previous samples but keep capacity
        {
            let mut samples = self.samples.lock().unwrap();
            samples.clear();
            // Ensure we have enough capacity pre-allocated
            let current_capacity = samples.capacity();
            if current_capacity < INITIAL_BUFFER_CAPACITY {
                samples.reserve(INITIAL_BUFFER_CAPACITY - current_capacity);
            }
        }

        let (cmd_tx, cmd_rx) = mpsc::channel::<RecorderCommand>();
        let (init_tx, init_rx) = mpsc::channel::<Result<(), String>>();
        let samples = self.samples.clone();
        let is_recording = self.is_recording.clone();
        let input_device_name = self.input_device_name.clone();
        let output_device_name = self.output_device_name.clone();
        let capture_source = self.capture_source;

        is_recording.store(true, Ordering::SeqCst);

        let handle = thread::spawn(move || {
            if let Err(e) = run_recording_thread(
                cmd_rx,
                init_tx,
                samples,
                is_recording.clone(),
                input_device_name,
                output_device_name,
                capture_source,
            ) {
                is_recording.store(false, Ordering::SeqCst);
                eprintln!("Recording thread error: {}", e);
            }
        });

        self.command_sender = Some(cmd_tx);
        self.thread_handle = Some(handle);

        match init_rx.recv_timeout(Duration::from_secs(3)) {
            Ok(Ok(())) => Ok(()),
            Ok(Err(error)) => {
                self.cleanup_failed_start();
                Err(error)
            }
            Err(_) => {
                self.cleanup_failed_start();
                Err("Timed out while starting audio input device".to_string())
            }
        }
    }

    fn cleanup_failed_start(&mut self) {
        self.is_recording.store(false, Ordering::SeqCst);

        if let Some(sender) = self.command_sender.take() {
            let _ = sender.send(RecorderCommand::Stop);
        }

        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }

    pub fn stop_recording(&mut self) -> Result<Vec<f32>, String> {
        self.is_recording.store(false, Ordering::SeqCst);

        // Signal thread to stop
        if let Some(sender) = self.command_sender.take() {
            let _ = sender.send(RecorderCommand::Stop);
        }

        // Wait for thread to finish
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }

        // No delay needed - samples are already collected via mutex
        // The stream is already stopped at this point

        let samples = self.samples.lock().unwrap().clone();

        if samples.is_empty() {
            return Err("No audio recorded".to_string());
        }

        Ok(samples)
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::SeqCst)
    }

    pub fn cancel_recording(&mut self) {
        self.is_recording.store(false, Ordering::SeqCst);

        if let Some(sender) = self.command_sender.take() {
            let _ = sender.send(RecorderCommand::Stop);
        }

        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }

        self.samples.lock().unwrap().clear();
    }
}

fn run_recording_thread(
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

    let mic_samples = Arc::new(Mutex::new(Vec::with_capacity(INITIAL_BUFFER_CAPACITY)));
    let system_samples = Arc::new(Mutex::new(Vec::with_capacity(INITIAL_BUFFER_CAPACITY)));
    let mut streams = Vec::new();

    if matches!(
        capture_source,
        AudioCaptureSource::Mic | AudioCaptureSource::Both
    ) {
        let device = select_input_device(&host, input_device_name.as_deref())?;
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
        let device = select_output_device(&host, output_device_name.as_deref())?;
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

    // Wait for stop command with minimal latency
    // Using 5ms polling for near-instant response when user stops recording
    loop {
        if let Ok(RecorderCommand::Stop) = cmd_rx.try_recv() {
            break;
        }
        if !is_recording.load(Ordering::SeqCst) {
            break;
        }
        thread::sleep(std::time::Duration::from_millis(5));
    }

    drop(streams);

    if capture_source == AudioCaptureSource::Both {
        let mic = mic_samples.lock().unwrap().clone();
        let system = system_samples.lock().unwrap().clone();
        let mixed = mix_audio_sources(&mic, &system);
        *samples.lock().unwrap() = mixed;
    }

    Ok(())
}

enum CaptureDeviceKind {
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
    let target_sample_rate = TARGET_SAMPLE_RATE;
    let err_fn = |err| eprintln!("[AUDIO ERROR] Audio stream error: {}", err);

    let stream = match config.sample_format() {
        SampleFormat::F32 => {
            let is_recording = is_recording.clone();
            let samples = samples.clone();
            device.build_input_stream(
                &config.into(),
                move |data: &[f32], _: &_| {
                    if is_recording.load(Ordering::SeqCst) {
                        process_audio_data(
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
                        process_audio_data(
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
                        process_audio_data(
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
    }
    .map_err(|e| format!("Failed to build audio stream: {}", e))?;

    Ok(stream)
}

fn select_input_device(host: &cpal::Host, name: Option<&str>) -> Result<cpal::Device, String> {
    if let Some(name) = name {
        let mut devices = host
            .input_devices()
            .map_err(|e| format!("Failed to list input devices: {}", e))?;

        if let Some(device) = devices.find(|device| {
            device
                .name()
                .map(|device_name| device_name == name)
                .unwrap_or(false)
        }) {
            return Ok(device);
        }

        return Err(format!("Input device not found: {}", name));
    }

    // If no explicit input device is provided, prefer a real microphone over
    // loopback-style virtual inputs (for example "Stereo Mix").
    let default_name = host
        .default_input_device()
        .and_then(|device| device.name().ok());

    if let Some(ref name) = default_name {
        if !is_probable_loopback_input(name) {
            return host
                .default_input_device()
                .ok_or_else(|| "No input device available".to_string());
        }
    }

    let mut devices = host
        .input_devices()
        .map_err(|e| format!("Failed to list input devices: {}", e))?;

    if let Some(device) = devices.find(|device| {
        device
            .name()
            .map(|device_name| !is_probable_loopback_input(&device_name))
            .unwrap_or(false)
    }) {
        return Ok(device);
    }

    host.default_input_device()
        .ok_or_else(|| "No input device available".to_string())
}

fn select_output_device(host: &cpal::Host, name: Option<&str>) -> Result<cpal::Device, String> {
    if let Some(name) = name {
        let mut devices = host
            .output_devices()
            .map_err(|e| format!("Failed to list output devices: {}", e))?;

        if let Some(device) = devices.find(|device| {
            device
                .name()
                .map(|device_name| device_name == name)
                .unwrap_or(false)
        }) {
            return Ok(device);
        }

        return Err(format!("Output device not found: {}", name));
    }

    host.default_output_device()
        .ok_or_else(|| "No output device available for system audio capture".to_string())
}

fn mix_audio_sources(primary: &[f32], secondary: &[f32]) -> Vec<f32> {
    let len = primary.len().max(secondary.len());
    let mut mixed = Vec::with_capacity(len);

    for index in 0..len {
        let a = primary.get(index).copied().unwrap_or(0.0);
        let b = secondary.get(index).copied().unwrap_or(0.0);
        mixed.push(((a + b) * 0.5).clamp(-1.0, 1.0));
    }

    mixed
}

fn process_audio_data(
    data: &[f32],
    channels: usize,
    source_rate: u32,
    target_rate: u32,
    samples: &Arc<Mutex<Vec<f32>>>,
    is_recording: &Arc<AtomicBool>,
) {
    // Convert to mono if stereo
    let mono: Vec<f32> = if channels > 1 {
        data.chunks(channels)
            .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
            .collect()
    } else {
        data.to_vec()
    };

    // Simple resampling (linear interpolation)
    let resampled = if source_rate != target_rate {
        resample(&mono, source_rate, target_rate)
    } else {
        mono
    };

    let mut samples = samples.lock().unwrap();
    let remaining = MAX_RECORDING_SAMPLES.saturating_sub(samples.len());
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

fn resample(samples: &[f32], source_rate: u32, target_rate: u32) -> Vec<f32> {
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

fn is_probable_loopback_input(device_name: &str) -> bool {
    let name = device_name.to_ascii_lowercase();
    [
        "stereo mix",
        "what u hear",
        "wave out",
        "loopback",
        "monitor of",
    ]
    .iter()
    .any(|pattern| name.contains(pattern))
}

// Save audio to WAV file for debugging
// Save audio to WAV file
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
