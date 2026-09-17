use cpal::traits::{DeviceTrait, HostTrait};

pub fn select_input_device(host: &cpal::Host, name: Option<&str>) -> Result<cpal::Device, String> {
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

pub fn select_output_device(host: &cpal::Host, name: Option<&str>) -> Result<cpal::Device, String> {
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

pub fn is_probable_loopback_input(device_name: &str) -> bool {
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
