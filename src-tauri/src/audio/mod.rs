use cpal::traits::{DeviceTrait, HostTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

pub mod devices;
pub mod processing;
pub mod recording;
pub mod wav;

pub use devices::{is_probable_loopback_input, select_input_device, select_output_device};
pub use recording::run_recording_thread;
pub use recording::CaptureDeviceKind;
pub use wav::save_wav;

const TARGET_SAMPLE_RATE: u32 = 16_000;
const INITIAL_BUFFER_CAPACITY: usize = TARGET_SAMPLE_RATE as usize * 30;
const MAX_RECORDING_SECONDS: usize = 5 * 60;
const MAX_RECORDING_SAMPLES: usize = TARGET_SAMPLE_RATE as usize * MAX_RECORDING_SECONDS;

pub enum RecorderCommand {
    Stop,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AudioCaptureSource {
    Mic,
    System,
    Both,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AudioInputDevice {
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AudioOutputDevice {
    pub name: String,
    pub is_default: bool,
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

        {
            let mut samples = self.samples.lock().unwrap();
            samples.clear();
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

        let handle = std::thread::spawn(move || {
            if let Err(e) = recording::run_recording_thread(
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

        match init_rx.recv_timeout(std::time::Duration::from_secs(3)) {
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

        if let Some(sender) = self.command_sender.take() {
            let _ = sender.send(RecorderCommand::Stop);
        }

        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }

        let samples = {
            let mut guard = self.samples.lock().unwrap();
            let recorded = std::mem::take(&mut *guard);
            let capacity = recorded.capacity();
            *guard = Vec::with_capacity(capacity);
            recorded
        };

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
