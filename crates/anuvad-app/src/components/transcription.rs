use leptos::prelude::*;

use crate::state::{AppState, RecordingState};

#[component]
pub fn TranscriptionPanel() -> impl IntoView {
    let state = expect_context::<AppState>();

    let transcription_text = state.transcription_text;
    let detected_language = state.detected_language;
    let recording_state = state.recording_state;

    let copy_text = move |_| {
        let text = transcription_text.get_untracked();
        if !text.is_empty() {
            let window = web_sys::window().unwrap();
            let nav = window.navigator();
            let clipboard = nav.clipboard();
            let _ = clipboard.write_text(&text);
        }
    };

    view! {
        <div class="card space-y-3">
            <div class="flex items-center justify-between">
                <h2 class="text-lg font-semibold">"Transcription"</h2>
                <div class="flex items-center gap-2">
                    {move || {
                        detected_language.get().map(|lang| {
                            view! {
                                <span class="badge-ready">
                                    {format!("Detected: {lang}")}
                                </span>
                            }
                        })
                    }}
                    <button
                        class="btn-secondary text-xs"
                        on:click=copy_text
                        title="Copy to clipboard"
                    >
                        "Copy"
                    </button>
                </div>
            </div>

            <div class="text-panel">
                {move || {
                    let text = transcription_text.get();
                    if text.is_empty() {
                        view! {
                            <span class="text-gray-400 dark:text-gray-600 italic">
                                "Transcription will appear here\u{2026}"
                            </span>
                        }.into_any()
                    } else {
                        let is_recording = recording_state.get() == RecordingState::Recording;
                        view! {
                            <span>{text}</span>
                            {if is_recording {
                                Some(view! { <span class="animate-pulse text-indigo-500">{"\u{2588}"}</span> })
                            } else {
                                None
                            }}
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
