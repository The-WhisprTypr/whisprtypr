use serde_json;
use vox_ai_lib::CommandError;

#[test]
fn test_license_error_serialization_sanitizes_messages() {
    // 1. Raw network errors
    let err = CommandError::License("Network error: Could not connect to API".to_string());
    let serialized = serde_json::to_string(&err).unwrap();
    assert_eq!(
        serialized,
        "\"License error: Could not reach the license server. Please check your internet connection and try again.\""
    );

    // 2. Activation limit
    let err = CommandError::License("Activation limit reached".to_string());
    let serialized = serde_json::to_string(&err).unwrap();
    assert_eq!(
        serialized,
        "\"License error: This license has reached its device limit. Please deactivate it on another device first.\""
    );

    // 3. Fallback / generic error (HTTP 400)
    let err = CommandError::License("Server returned HTTP 400 Bad Request".to_string());
    let serialized = serde_json::to_string(&err).unwrap();
    assert_eq!(
        serialized,
        "\"License error: License verification failed. Please try again.\""
    );

    // 4. Invalid license
    let err = CommandError::License("Invalid license key provided".to_string());
    let serialized = serde_json::to_string(&err).unwrap();
    assert_eq!(
        serialized,
        "\"License error: That license key could not be verified. Please check the key and try again.\""
    );
}

#[test]
fn test_other_errors_serialize_normally() {
    let err = CommandError::Recording("Device missing".to_string());
    let serialized = serde_json::to_string(&err).unwrap();
    assert_eq!(serialized, "\"Recording error: Device missing\"");

    let err = CommandError::Transcription("No model loaded".to_string());
    let serialized = serde_json::to_string(&err).unwrap();
    assert_eq!(serialized, "\"Transcription error: No model loaded\"");
}
