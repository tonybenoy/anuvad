use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioSource {
    Microphone,
    #[cfg(feature = "extension")]
    TabAudio,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelStatus {
    NotDownloaded,
    Downloading,
    Loading,
    Ready,
    Error,
}

impl ModelStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::NotDownloaded => "Not Downloaded",
            Self::Downloading => "Downloading…",
            Self::Loading => "Loading…",
            Self::Ready => "Ready",
            Self::Error => "Error",
        }
    }

    pub fn badge_class(&self) -> &'static str {
        match self {
            Self::NotDownloaded => "badge bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400",
            Self::Downloading | Self::Loading => "badge-loading",
            Self::Ready => "badge-ready",
            Self::Error => "badge-error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordingState {
    Idle,
    Recording,
    Processing,
}

#[derive(Clone)]
pub struct AppState {
    pub whisper_status: RwSignal<ModelStatus>,
    pub translator_status: RwSignal<ModelStatus>,
    pub whisper_progress: RwSignal<f64>,
    pub translator_progress: RwSignal<f64>,
    pub recording_state: RwSignal<RecordingState>,
    pub transcription_text: RwSignal<String>,
    pub translation_text: RwSignal<String>,
    pub source_language: RwSignal<String>,
    pub target_language: RwSignal<String>,
    pub detected_language: RwSignal<Option<String>>,
    pub audio_level: RwSignal<f64>,
    pub error_message: RwSignal<Option<String>>,
    pub recording_duration: RwSignal<f64>,
    pub audio_source: RwSignal<AudioSource>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            whisper_status: RwSignal::new(ModelStatus::NotDownloaded),
            translator_status: RwSignal::new(ModelStatus::NotDownloaded),
            whisper_progress: RwSignal::new(0.0),
            translator_progress: RwSignal::new(0.0),
            recording_state: RwSignal::new(RecordingState::Idle),
            transcription_text: RwSignal::new(String::new()),
            translation_text: RwSignal::new(String::new()),
            source_language: RwSignal::new("auto".to_string()),
            target_language: RwSignal::new("en".to_string()),
            detected_language: RwSignal::new(None),
            audio_level: RwSignal::new(0.0),
            error_message: RwSignal::new(None),
            recording_duration: RwSignal::new(0.0),
            audio_source: RwSignal::new(AudioSource::Microphone),
        }
    }
}
