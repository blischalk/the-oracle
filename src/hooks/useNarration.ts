import { useEffect, useState, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "../stores/settingsStore";

// OpenAI TTS caps at 4096 characters; stay safely under that.
const MAX_TTS_CHARS = 4000;

// Strip markdown markers so the TTS engine reads clean prose.
function stripForSpeech(text: string): string {
  return text
    .replace(/\*\*\*(.+?)\*\*\*/gs, "$1")
    .replace(/\*\*(.+?)\*\*/gs, "$1")
    .replace(/\*(.+?)\*/gs, "$1")
    .replace(/___(.+?)___/gs, "$1")
    .replace(/__(.+?)__/gs, "$1")
    .replace(/_(.+?)_/gs, "$1")
    .replace(/`(.+?)`/gs, "$1")
    .replace(/#{1,6}\s+/gm, "")
    .replace(/^>\s*/gm, "")
    .replace(/^[-*+]\s+/gm, "")
    .replace(/^\d+\.\s+/gm, "")
    .replace(/\[(.+?)\]\(.+?\)/g, "$1");
}

function truncate(text: string): string {
  return text.length > MAX_TTS_CHARS ? text.slice(0, MAX_TTS_CHARS) : text;
}

export function useNarration() {
  const { settings } = useSettingsStore();
  const [voices, setVoices] = useState<SpeechSynthesisVoice[]>([]);
  const currentAudioRef = useRef<HTMLAudioElement | null>(null);
  const currentBlobUrlRef = useRef<string | null>(null);

  // Load system voices for the "system" provider option.
  useEffect(() => {
    function loadVoices() {
      setVoices(window.speechSynthesis.getVoices());
    }
    loadVoices();
    window.speechSynthesis.addEventListener("voiceschanged", loadVoices);
    return () => {
      window.speechSynthesis.removeEventListener("voiceschanged", loadVoices);
    };
  }, []);

  const stop = useCallback(() => {
    window.speechSynthesis?.cancel();
    if (currentAudioRef.current) {
      currentAudioRef.current.pause();
      currentAudioRef.current = null;
    }
    if (currentBlobUrlRef.current) {
      URL.revokeObjectURL(currentBlobUrlRef.current);
      currentBlobUrlRef.current = null;
    }
  }, []);

  const speak = useCallback(
    async (text: string) => {
      if (!settings.narration_enabled) return;
      stop();

      const clean = truncate(stripForSpeech(text)).trim();
      if (!clean) return;

      if (settings.tts_provider === "openai") {
        try {
          const bytes = await invoke<number[]>("synthesize_speech", {
            text: clean,
            voice: settings.tts_openai_voice || "nova",
          });
          const blob = new Blob([new Uint8Array(bytes)], { type: "audio/mpeg" });
          const url = URL.createObjectURL(blob);
          const audio = new Audio(url);
          audio.playbackRate = settings.narration_rate;
          currentAudioRef.current = audio;
          currentBlobUrlRef.current = url;
          audio.play();
          audio.onended = () => {
            URL.revokeObjectURL(url);
            if (currentBlobUrlRef.current === url) currentBlobUrlRef.current = null;
            if (currentAudioRef.current === audio) currentAudioRef.current = null;
          };
        } catch (err) {
          console.error("OpenAI TTS failed:", err);
        }
        return;
      }

      // System TTS fallback.
      if (!window.speechSynthesis) return;
      const utterance = new SpeechSynthesisUtterance(clean);
      utterance.rate = settings.narration_rate;
      if (settings.narration_voice_uri) {
        const voice = window.speechSynthesis
          .getVoices()
          .find((v) => v.voiceURI === settings.narration_voice_uri);
        if (voice) utterance.voice = voice;
      }
      window.speechSynthesis.speak(utterance);
    },
    [settings, stop]
  );

  return { speak, stop, voices };
}
