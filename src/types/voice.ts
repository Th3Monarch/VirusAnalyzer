// Types for voice functionality. These support both the Web Speech API
// (current implementation) and native STT/TTS providers (Kokoro, Whisper).

/** Backend VoiceConfig (matches Rust serde camelCase) */
export interface VoiceConfig {
  enabled: boolean;
  autoSpeak: boolean;
  language: string;
  sttProvider: string;
  ttsProvider: string;
  ttsUrl: string;
  sttUrl: string;
  speechRate: number;
  volume: number;
}

export interface VoiceRecordingState {
  recording: boolean;
  available: boolean;
  provider: string;
}

/** Estado de salud de los providers de voz. */
export interface VoiceHealth {
  ttsAvailable: boolean;
  sttAvailable: boolean;
  ttsUrl: string;
  sttUrl: string;
}

/** Web Speech API recognition state */
export type VoiceRecognitionState = "idle" | "listening" | "processing" | "error";

export {};
