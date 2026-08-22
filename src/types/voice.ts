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
  voiceId: string;
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

/** Voz disponible de Kokoro. */
export interface VoiceInfo {
  id: string;
  name: string;
}

/** Información del acento nativo para un idioma. */
export interface AccentInfo {
  label: string;
  code: string;
  nativeAvailable: boolean;
  limitation: string | null;
}

/** Web Speech API recognition state */
export type VoiceRecognitionState = "idle" | "listening" | "processing" | "error";

export {};
