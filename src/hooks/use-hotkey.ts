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
  showRecordingOverlay,
  hideRecordingOverlay,
  recordAndTranslate,
  postProcessText,
  formatTextWithAi,
  extractVoiceCommands,
  processVoiceCommands,
  stripVoiceCommandTokens,
  type HotkeyRegistration,
  type HotkeyEventPayload,
} from "@/lib/voice-api";
import { playFeedbackSound } from "@/lib/preferences-api";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { useAppStore } from "@/store";

export function useHotkey() {
  const {
    setupComplete,
    settings,
    selectedModel,
    setRecordingStatus,
    setLastTranscription,
    setErrorMessage,
  } = useAppStore();
  const isRecordingRef = useRef(false);
  const isTranslationActiveRef = useRef(false);
  const unlistenPressedRef = useRef<UnlistenFn | null>(null);
  const unlistenReleasedRef = useRef<UnlistenFn | null>(null);

  const currentHotkey =
    settings.hotkeyMode === "push-to-talk"
      ? settings.pushToTalkKey
      : settings.toggleKey;

  // Refs so the listeners always see current values without re-binding
  const settingsRef = useRef(settings);
  const selectedModelRef = useRef(selectedModel);
  useEffect(() => {
    settingsRef.current = settings;
    selectedModelRef.current = selectedModel;
  }, [settings, selectedModel]);

  const isEnabled = setupComplete;

  // Show / hide the OS-level recording overlay. We fire-and-forget because
  // show/hide are best-effort UI hints — failing to show the overlay must
  // not block the actual recording from starting.
  const showOverlay = useCallback(() => {
    if (settingsRef.current.showRecordingOverlay) {
      showRecordingOverlay(settingsRef.current.recordingOverlayPosition).catch((err) => {
        console.warn("Failed to show recording overlay:", err);
      });
    }
  }, []);

  const hideOverlay = useCallback(() => {
    hideRecordingOverlay().catch((err) => {
      console.warn("Failed to hide recording overlay:", err);
    });
  }, []);

  // Handle recording start
  const handleRecordingStart = useCallback(async (sourceLanguage?: string) => {
    if (isRecordingRef.current) return;

    try {
      const model = selectedModelRef.current;
      const lang = sourceLanguage || settingsRef.current.language;

      const recordingPromise = startRecording();

      if (model?.id) {
        await loadModel(model.id, lang);
      }

      await recordingPromise;
      isRecordingRef.current = true;
      setRecordingStatus("recording");
      showOverlay();

      // Play start feedback sound if enabled
      if (settingsRef.current.playAudioFeedback) {
        playFeedbackSound("start");
      }
    } catch (error) {
      console.error("Failed to start recording:", error);
      setErrorMessage(
        error instanceof Error ? error.message : "Failed to start recording",
      );
    }
  }, [setRecordingStatus, setErrorMessage, showOverlay]);

  /**
   * Apply post-processing and voice commands to transcribed text based
   * on the user's settings.
   *
   * The backend `post_process_text` command handles BOTH voice commands
   * (producing control markers like [[UNDO]]) AND smart text formatting
   * (camelCase, file paths, etc.) in a single pass. When voice commands
   * are enabled but smart text processing is disabled, we use the
   * dedicated `extract_voice_commands` command which skips formatting.
   *
   * AI formatting is applied AFTER post-processing (and AFTER voice commands
   * are stripped) so that the LLM receives the cleaned-up transcription
   * rather than raw voice-command markers.
   */
  const processTranscription = useCallback(
    async (
      text: string,
      appSettings: {
        postProcessingEnabled: boolean;
        voiceCommandsEnabled: boolean;
        aiFormattingEnabled: boolean;
      }
    ): Promise<string> => {
      let result = text;

      if (appSettings.voiceCommandsEnabled && appSettings.postProcessingEnabled) {
        result = await postProcessText(result);
        result = await processVoiceCommands(result);
      } else if (appSettings.voiceCommandsEnabled) {
        result = await extractVoiceCommands(result);
        result = await processVoiceCommands(result);
      } else if (appSettings.postProcessingEnabled) {
        result = await postProcessText(result);
      } else {
        result = stripVoiceCommandTokens(result);
      }

      if (appSettings.aiFormattingEnabled && result.trim()) {
        result = await formatTextWithAi(result);
      }

      return result;
    },
    [],
  );

  // Handle recording stop (regular dictation)
  const handleRecordingStop = useCallback(async () => {
    if (!isRecordingRef.current) return;

    try {
      setRecordingStatus("processing");
      hideOverlay();

      const audioData = await stopRecording();

      const text = await transcribeAudio(audioData);

      if (text) {
        const processed = await processTranscription(text, {
          postProcessingEnabled: settingsRef.current.postProcessingEnabled,
          voiceCommandsEnabled: settingsRef.current.voiceCommandsEnabled,
          aiFormattingEnabled: settingsRef.current.aiFormattingEnabled,
        });

        if (processed) {
          await injectText(processed);
        }

        const model = selectedModelRef.current;
        const lang = settingsRef.current.language;
        if (model?.id) {
          // Convert audio samples to duration in milliseconds (samples at 16kHz)
          const durationMs = Math.round((audioData.length / 16000) * 1000);
          await addTranscription(processed || text, model.id, lang, durationMs);
        }

        setLastTranscription(processed || text);
      }

      // Play stop feedback sound if enabled
      if (settingsRef.current.playAudioFeedback) {
        playFeedbackSound("stop");
      }

      isRecordingRef.current = false;
      setRecordingStatus("idle");
    } catch (error) {
      console.error("Failed to stop recording:", error);
      isRecordingRef.current = false;
      hideOverlay();
      // Play stop feedback sound even on error if enabled
      if (settingsRef.current.playAudioFeedback) {
        playFeedbackSound("stop");
      }
      setErrorMessage(
        error instanceof Error ? error.message : "Failed to transcribe",
      );
      setRecordingStatus("error");
      setTimeout(() => setRecordingStatus("idle"), 2000);
    }
  }, [
    setRecordingStatus,
    setLastTranscription,
    setErrorMessage,
    hideOverlay,
  ]);

  // Handle recording stop with translation
  const handleRecordingStopWithTranslation = useCallback(async (sourceLanguage: string, targetLanguage: string) => {
    if (!isRecordingRef.current) return;

    try {
      setRecordingStatus("processing");
      hideOverlay();

      const text = await recordAndTranslate(sourceLanguage, targetLanguage, settingsRef.current.postProcessingEnabled, settingsRef.current.translationApiKey);

      if (text && text.trim()) {
        await injectText(text);

        const model = selectedModelRef.current;
        if (model?.id) {
          const durationMs = 0;
          await addTranscription(text, `translation:${sourceLanguage}->${targetLanguage}`, sourceLanguage, durationMs);
        }

        setLastTranscription(text);
      }

      // Play stop feedback sound if enabled
      if (settingsRef.current.playAudioFeedback) {
        playFeedbackSound("stop");
      }

      isRecordingRef.current = false;
      setRecordingStatus("idle");
    } catch (error) {
      console.error("Failed to stop recording with translation:", error);
      isRecordingRef.current = false;
      hideOverlay();
      // Play stop feedback sound even on error if enabled
      if (settingsRef.current.playAudioFeedback) {
        playFeedbackSound("stop");
      }
      setErrorMessage(
        error instanceof Error ? error.message : "Failed to translate",
      );
      setRecordingStatus("error");
      setTimeout(() => setRecordingStatus("idle"), 2000);
    }
  }, [setRecordingStatus, setLastTranscription, setErrorMessage, hideOverlay]);

  // Register hotkey + set up listeners. Re-runs only when the bound hotkey
  // string or the enabled state actually changes.
  useEffect(() => {
    if (!isEnabled) {
      return;
    }

    const registrations: HotkeyRegistration[] = [];

    if (currentHotkey) {
      registrations.push({ hotkey: currentHotkey, label: "dictation" });
    }

    if (settings.translationEnabled && settings.translationHotkey) {
      registrations.push({ hotkey: settings.translationHotkey, label: "translate" });
    }

    if (registrations.length === 0) {
      return;
    }

    let cancelled = false;

    // Register the hotkey. Await it so the cleanup that unregisters runs
    // only after a real registration is in place.
    (async () => {
      try {
        await registerHotkey(registrations);
        if (cancelled) {
          await unregisterHotkeys().catch(console.error);
        }
      } catch (err) {
        console.error("Failed to register hotkey:", err);
      }
    })();

    const trigger = (label: string) => {
      if (label === "translate") {
        if (settingsRef.current.translationEnabled && settingsRef.current.translationHotkey) {
          if (settingsRef.current.hotkeyMode === "push-to-talk") {
            if (isTranslationActiveRef.current) {
              isTranslationActiveRef.current = false;
              handleRecordingStopWithTranslation(
                settingsRef.current.translationSourceLanguage,
                settingsRef.current.translationTargetLanguage,
              );
            } else {
              isTranslationActiveRef.current = true;
              handleRecordingStart(settingsRef.current.translationSourceLanguage);
            }
          } else {
            if (isRecordingRef.current) {
              handleRecordingStop();
            } else {
              handleRecordingStart(settingsRef.current.translationSourceLanguage);
            }
          }
        }
      } else {
        const mode = settingsRef.current.hotkeyMode;
        if (mode === "push-to-talk") {
          if (isRecordingRef.current) {
            handleRecordingStop();
          } else {
            handleRecordingStart();
          }
        } else {
          if (isRecordingRef.current) {
            handleRecordingStop();
          } else {
            handleRecordingStart();
          }
        }
      }
    };

    const onEvent = (payload: HotkeyEventPayload) => {
      trigger(payload.label);
    };

    onHotkeyPressed(onEvent)
      .then((fn) => {
        if (cancelled) {
          fn();
        } else {
          unlistenPressedRef.current = fn;
        }
      })
      .catch((err) => {
        console.error("Failed to set up onHotkeyPressed:", err);
      });

    onHotkeyReleased(onEvent)
      .then((fn) => {
        if (cancelled) {
          fn();
        } else {
          unlistenReleasedRef.current = fn;
        }
      })
      .catch((err) => {
        console.error("Failed to set up onHotkeyReleased:", err);
      });

    return () => {
      cancelled = true;
      unlistenPressedRef.current?.();
      unlistenPressedRef.current = null;
      unlistenReleasedRef.current?.();
      unlistenReleasedRef.current = null;
      unregisterHotkeys().catch(console.error);
    };
  }, [
    isEnabled,
    currentHotkey,
    settings.translationEnabled,
    settings.translationHotkey,
    settings.translationSourceLanguage,
    settings.translationTargetLanguage,
    settings.hotkeyMode,
    handleRecordingStart,
    handleRecordingStop,
    handleRecordingStopWithTranslation,
  ]);

  // Preload the selected model in the background as soon as setup is
  // complete. This makes the first hotkey press feel instant — the heavy
  // 3-4 second ONNX load happens here, off the hotkey critical path.
  const selectedModelId = selectedModel?.id;
  const selectedModelDownloaded = selectedModel?.downloaded === true;
  const currentLanguage = settings.language;
  const setModelReady = useAppStore((s) => s.setModelReady);
  useEffect(() => {
    if (!isEnabled) return;
    if (!selectedModelId) return;
    if (!selectedModelDownloaded) return;

    setModelReady(false);

    let cancelled = false;
    loadModel(selectedModelId, currentLanguage)
      .then(() => {
        if (cancelled) return;
        console.log(`Model ${selectedModelId} preloaded and ready`);
        setModelReady(true);
      })
      .catch((err) => {
        if (cancelled) return;
        console.warn("Background model preload failed:", err);
        setModelReady(false);
      });

    return () => {
      cancelled = true;
    };
  }, [
    isEnabled,
    selectedModelId,
    selectedModelDownloaded,
    currentLanguage,
    setModelReady,
  ]);

  // On unmount, make sure the overlay window isn't left visible
  useEffect(() => {
    return () => {
      hideRecordingOverlay().catch(console.error);
    };
  }, []);

  return {
    isRecording: isRecordingRef.current,
  };
}
