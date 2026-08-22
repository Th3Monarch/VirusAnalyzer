import { useCallback, useEffect, useRef, useState } from "react";
import { tauri } from "../lib/tauri";
import type { VoiceRecognitionState } from "../types/voice";

// Web Speech API type declarations
interface SpeechRecognitionInstance extends EventTarget {
  continuous: boolean;
  interimResults: boolean;
  lang: string;
  onstart: (() => void) | null;
  onresult: ((event: SpeechRecognitionEvent) => void) | null;
  onerror: ((event: SpeechRecognitionErrorEvent) => void) | null;
  onend: (() => void) | null;
  start(): void;
  stop(): void;
  abort(): void;
}

interface SpeechRecognitionConstructor {
  new (): SpeechRecognitionInstance;
}

interface SpeechRecognitionEvent extends Event {
  readonly results: SpeechRecognitionResultList;
  readonly resultIndex: number;
}

interface SpeechRecognitionErrorEvent extends Event {
  readonly error: string;
  readonly message: string;
}

interface SpeechRecognitionResultList {
  readonly length: number;
  item(index: number): SpeechRecognitionResult;
  [index: number]: SpeechRecognitionResult;
}

interface SpeechRecognitionResult {
  readonly length: number;
  item(index: number): SpeechRecognitionAlternative;
  [index: number]: SpeechRecognitionAlternative;
  readonly isFinal: boolean;
}

interface SpeechRecognitionAlternative {
  readonly transcript: string;
  readonly confidence: number;
}

// Global augmentation for the Web Speech API
declare global {
  interface Window {
    SpeechRecognition?: SpeechRecognitionConstructor;
    webkitSpeechRecognition?: SpeechRecognitionConstructor;
  }
}

interface UseVoiceReturn {
  state: VoiceRecognitionState;
  startListening: () => void;
  stopListening: () => void;
  available: boolean;
  transcript: string;
  error: string | null;
  speak: (text: string) => void;
  stopSpeaking: () => void;
  speaking: boolean;
  backendTtsAvailable: boolean;
}

function isSpeechRecognitionAvailable(): boolean {
  return (
    typeof window !== "undefined" &&
    ("SpeechRecognition" in window || "webkitSpeechRecognition" in window)
  );
}

function isSpeechSynthesisAvailable(): boolean {
  return typeof window !== "undefined" && "speechSynthesis" in window;
}

export function useVoice(language: string = "es", speechRate: number = 1.0, volume: number = 1.0): UseVoiceReturn {
  const [state, setState] = useState<VoiceRecognitionState>("idle");
  const [transcript, setTranscript] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [speaking, setSpeaking] = useState(false);
  const [backendTtsAvailable, setBackendTtsAvailable] = useState(false);
  const recognitionRef = useRef<SpeechRecognitionInstance | null>(null);
  const audioContextRef = useRef<AudioContext | null>(null);

  const available = isSpeechRecognitionAvailable();

  // Check backend TTS availability on mount
  useEffect(() => {
    void tauri.assistantVoiceHealth().then((health) => {
      setBackendTtsAvailable(health.ttsAvailable);
    }).catch(() => setBackendTtsAvailable(false));
  }, []);

  // Initialize recognition
  useEffect(() => {
    if (!available || typeof window === "undefined") return;

    const Ctor = window.SpeechRecognition ?? window.webkitSpeechRecognition;
    if (!Ctor) return;

    const recognition = new Ctor();
    recognition.continuous = false;
    recognition.interimResults = false;
    recognition.lang = language === "es" ? "es-ES" : "en-US";

    recognition.onstart = () => {
      setState("listening");
      setError(null);
    };

    recognition.onresult = (event: SpeechRecognitionEvent) => {
      const result = event.results[event.results.length - 1];
      if (result.isFinal) {
        setTranscript(result[0].transcript);
        setState("processing");
      }
    };

    recognition.onerror = (event: Event) => {
      const errEvent = event as unknown as SpeechRecognitionErrorEvent;
      if (errEvent.error === "no-speech") {
        setState("idle");
        return;
      }
      setState("error");
      setError(errEvent.error);
    };

    recognition.onend = () => {
      setState("idle");
    };

    recognitionRef.current = recognition;

    return () => {
      recognition.abort();
      recognitionRef.current = null;
    };
  }, [available, language]);

  const startListening = useCallback(() => {
    const recognition = recognitionRef.current;
    if (!recognition) return;

    setTranscript("");
    setError(null);

    try {
      recognition.start();
    } catch {
      // Already started, ignore
    }
  }, []);

  const stopListening = useCallback(() => {
    const recognition = recognitionRef.current;
    if (!recognition) return;

    try {
      recognition.stop();
    } catch {
      // Already stopped, ignore
    }
  }, []);

  const speak = useCallback(
    (text: string) => {
      // Try backend Kokoro TTS first
      if (backendTtsAvailable) {
        setSpeaking(true);
        void tauri
          .assistantSynthesize(text)
          .then((audioBytes) => {
            // Convert number[] to ArrayBuffer and play via Web Audio API
            const buffer = new Uint8Array(audioBytes).buffer;
            if (!audioContextRef.current) {
              audioContextRef.current = new AudioContext();
            }
            const ctx = audioContextRef.current;
            return ctx.decodeAudioData(buffer);
          })
          .then((audioBuffer) => {
            if (!audioContextRef.current) return;
            const ctx = audioContextRef.current;
            const gainNode = ctx.createGain();
            gainNode.gain.value = volume;
            const source = ctx.createBufferSource();
            source.buffer = audioBuffer;
            source.playbackRate.value = speechRate;
            source.connect(gainNode);
            gainNode.connect(ctx.destination);
            source.onended = () => setSpeaking(false);
            source.start(0);
          })
          .catch(() => {
            // Fallback to Web Speech API
            setSpeaking(false);
            speakWebSpeech(text);
          });
        return;
      }

      // Fallback: Web Speech API
      speakWebSpeech(text);
    },
    [backendTtsAvailable, language, speechRate, volume],
  );

  function speakWebSpeech(text: string) {
    if (!isSpeechSynthesisAvailable()) return;

    window.speechSynthesis.cancel();

    const utterance = new SpeechSynthesisUtterance(text);
    utterance.lang = language === "es" ? "es-ES" : "en-US";
    utterance.rate = speechRate;
    utterance.volume = volume;
    utterance.pitch = 1.0;

    utterance.onstart = () => setSpeaking(true);
    utterance.onend = () => setSpeaking(false);
    utterance.onerror = () => setSpeaking(false);

    window.speechSynthesis.speak(utterance);
  }

  const stopSpeaking = useCallback(() => {
    if (isSpeechSynthesisAvailable()) {
      window.speechSynthesis.cancel();
    }
    if (audioContextRef.current) {
      audioContextRef.current.close().catch(() => undefined);
      audioContextRef.current = null;
    }
    setSpeaking(false);
  }, []);

  useEffect(() => {
    return () => {
      if (isSpeechSynthesisAvailable()) {
        window.speechSynthesis.cancel();
      }
      if (audioContextRef.current) {
        audioContextRef.current.close().catch(() => undefined);
      }
    };
  }, []);

  return {
    state,
    startListening,
    stopListening,
    available,
    transcript,
    error,
    speak,
    stopSpeaking,
    speaking,
    backendTtsAvailable,
  };
}
