import { useEffect, useCallback, useRef } from "react";
import {
  registerHotkey,
  unregisterHotkeys,
  onHotkeyPressed,
  onHotkeyReleased,
  startRecording,
  stopRecording,
  loadModel,
  transcribeAudio,
  injectText,
  addTranscription,
} from "@/lib/voice-api";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { useAppStore } from "@/store";

export function useHotkey() {
  const { setupComplete, settings, selectedModel, setRecordingStatus, setLastTranscription, setErrorMessage } = useAppStore();
  const isRecordingRef = useRef(false);
  const unlistenPressedRef = useRef<UnlistenFn | null>(null);
  const unlistenReleasedRef = useRef<UnlistenFn | null>(null);

  const currentHotkey = settings.hotkeyMode === "push-to-talk" ? settings.pushToTalkKey : settings.toggleKey;

  // Only register hotkey when setup is complete
  const isEnabled = setupComplete;

  // Handle recording start
  const handleRecordingStart = useCallback(async () => {
    console.log("handleRecordingStart called, isRecording:", isRecordingRef.current);
    if (isRecordingRef.current) return;

    try {
      // Load model if specified
      if (selectedModel?.id) {
        await loadModel(selectedModel.id, settings.language);
      }

      await startRecording();
      isRecordingRef.current = true;
      setRecordingStatus("recording");
      console.log("Recording started");
    } catch (error) {
      console.error("Failed to start recording:", error);
      setErrorMessage(error instanceof Error ? error.message : "Failed to start recording");
    }
  }, [selectedModel?.id, settings.language, setRecordingStatus, setErrorMessage]);

  // Handle recording stop
  const handleRecordingStop = useCallback(async () => {
    console.log("handleRecordingStop called, isRecording:", isRecordingRef.current);
    if (!isRecordingRef.current) return;

    try {
      setRecordingStatus("processing");

      // Stop recording and get audio data
      const audioData = await stopRecording();
      console.log("Recording stopped, audio data length:", audioData.length);

      // Transcribe the audio
      const text = await transcribeAudio(audioData);
      console.log("Transcription result:", text);

      if (text) {
        // Inject text at cursor
        await injectText(text);

        // Save to history
        if (selectedModel?.id) {
          await addTranscription(text, selectedModel.id, settings.language, audioData.length);
        }

        setLastTranscription(text);
      }

      isRecordingRef.current = false;
      setRecordingStatus("idle");
    } catch (error) {
      console.error("Failed to stop recording:", error);
      isRecordingRef.current = false;
      setErrorMessage(error instanceof Error ? error.message : "Failed to transcribe");
      setRecordingStatus("error");
      setTimeout(() => setRecordingStatus("idle"), 2000);
    }
  }, [selectedModel?.id, settings.language, setRecordingStatus, setLastTranscription, setErrorMessage]);

  // Register hotkey and set up listeners
  useEffect(() => {
    if (!isEnabled || !currentHotkey) {
      console.log("Hotkey not enabled or no hotkey configured");
      return;
    }

    console.log("Setting up hotkey:", currentHotkey, "mode:", settings.hotkeyMode);

    // Register hotkey
    registerHotkey(currentHotkey).catch(err => {
      console.error("Failed to register hotkey:", err);
    });

    // Set up event listeners
    onHotkeyPressed(() => {
      console.log("Hotkey PRESSED");
      if (settings.hotkeyMode === "push-to-talk") {
        handleRecordingStart();
      } else {
        // Toggle mode
        if (isRecordingRef.current) {
          handleRecordingStop();
        } else {
          handleRecordingStart();
        }
      }
    }).then(fn => {
      unlistenPressedRef.current = fn;
      console.log("onHotkeyPressed listener set up");
    }).catch(err => {
      console.error("Failed to set up onHotkeyPressed:", err);
    });

    onHotkeyReleased(() => {
      console.log("Hotkey RELEASED");
      if (settings.hotkeyMode === "push-to-talk") {
        handleRecordingStop();
      }
    }).then(fn => {
      unlistenReleasedRef.current = fn;
      console.log("onHotkeyReleased listener set up");
    }).catch(err => {
      console.error("Failed to set up onHotkeyReleased:", err);
    });

    return () => {
      console.log("Cleaning up hotkey");
      unlistenPressedRef.current?.();
      unlistenReleasedRef.current?.();
      unregisterHotkeys().catch(console.error);
    };
  }, [isEnabled, currentHotkey, settings.hotkeyMode, handleRecordingStart, handleRecordingStop]);

  return {
    isRecording: isRecordingRef.current,
  };
}
