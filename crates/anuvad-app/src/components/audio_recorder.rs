use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::state::{AppState, AudioSource, ModelStatus, RecordingState};
use crate::workers::audio_capture;
#[cfg(feature = "extension")]
use crate::workers::tab_capture;
use crate::workers::mixed_capture;

#[component]
pub fn AudioRecorder() -> impl IntoView {
    let state = expect_context::<AppState>();

    let recording_state = state.recording_state;
    let whisper_status = state.whisper_status;
    let error_message = state.error_message;
    let audio_level = state.audio_level;
    let recording_duration = state.recording_duration;

    let audio_source = state.audio_source;

    let toggle_recording = move |_| {
        spawn_local(async move {
            match recording_state.get_untracked() {
                RecordingState::Idle => {
                    let result = match audio_source.get_untracked() {
                        AudioSource::Microphone => audio_capture::start_recording().await,
                        #[cfg(feature = "extension")]
                        AudioSource::TabAudio => tab_capture::start_tab_capture().await,
                        AudioSource::Both => mixed_capture::start_mixed_capture().await,
                    };
                    match result {
                        Ok(()) => {
                            recording_state.set(RecordingState::Recording);
                        }
                        Err(e) => {
                            error_message.set(Some(format!("Audio error: {e}")));
                        }
                    }
                }
                RecordingState::Recording => {
                    match audio_source.get_untracked() {
                        AudioSource::Microphone => audio_capture::stop_recording(),
                        #[cfg(feature = "extension")]
                        AudioSource::TabAudio => tab_capture::stop_tab_capture(),
                        AudioSource::Both => mixed_capture::stop_mixed_capture(),
                    }
                    recording_state.set(RecordingState::Idle);
                }
                RecordingState::Processing => {}
            }
        });
    };

    let can_record = move || {
        whisper_status.get() == ModelStatus::Ready
    };

    let button_text = move || {
        match recording_state.get() {
            RecordingState::Idle => "Record",
            RecordingState::Recording => "Stop",
            RecordingState::Processing => "Processing\u{2026}",
        }
    };

    let button_class = move || {
        let base = "px-8 py-4 font-semibold rounded-2xl transition-all duration-200 shadow-lg active:scale-95 text-lg";
        match recording_state.get() {
            RecordingState::Idle => format!("{base} bg-indigo-600 hover:bg-indigo-700 text-white hover:shadow-xl disabled:opacity-50 disabled:cursor-not-allowed"),
            RecordingState::Recording => format!("{base} bg-red-600 hover:bg-red-700 text-white animate-pulse"),
            RecordingState::Processing => format!("{base} bg-yellow-600 text-white cursor-wait"),
        }
    };

    view! {
        <div class="card flex flex-col items-center gap-4">
            <div class="flex items-center gap-4">
                <button
                    class=button_class
                    on:click=toggle_recording
                    disabled=move || !can_record() || recording_state.get() == RecordingState::Processing
                >
                    {button_text}
                </button>
            </div>

            {move || {
                if recording_state.get() == RecordingState::Recording {
                    let level = (audio_level.get() * 100.0).min(100.0);
                    Some(view! {
                        <div class="w-full max-w-md space-y-1">
                            <div class="progress-bar h-1.5">
                                <div
                                    class="h-full rounded-full bg-green-500 transition-all duration-100"
                                    style:width=format!("{level}%")
                                ></div>
                            </div>
                            <p class="text-xs text-center text-gray-500 dark:text-gray-400">
                                {move || format!("{:.1}s", recording_duration.get())}
                            </p>
                        </div>
                    })
                } else {
                    None
                }
            }}

            {move || {
                if !can_record() {
                    Some(view! {
                        <p class="text-sm text-gray-500 dark:text-gray-400">
                            "Download the Whisper model first to enable recording."
                        </p>
                    })
                } else {
                    None
                }
            }}
        </div>
    }
}
