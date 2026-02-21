use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::state::{AppState, ModelStatus, RecordingState};
use crate::workers::bridge;

#[component]
pub fn TranslationPanel() -> impl IntoView {
    let state = expect_context::<AppState>();

    let translator_status = state.translator_status;
    let transcription_text = state.transcription_text;
    let translation_text = state.translation_text;
    let recording_state = state.recording_state;
    let target_language = state.target_language;

    let translate = move |_| {
        spawn_local(async move {
            let text = transcription_text.get_untracked();
            if text.is_empty() {
                return;
            }
            let target = target_language.get_untracked();
            translation_text.set(String::new());
            bridge::request_translation(&text, &target).await;
        });
    };

    let can_translate = move || {
        translator_status.get() == ModelStatus::Ready
            && !transcription_text.get().is_empty()
            && recording_state.get() != RecordingState::Recording
    };

    let copy_text = move |_| {
        let text = translation_text.get_untracked();
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
                <h2 class="text-lg font-semibold">"Translation"</h2>
                <div class="flex items-center gap-2">
                    <button
                        class="btn-primary text-sm"
                        on:click=translate
                        disabled=move || !can_translate()
                    >
                        "Translate"
                    </button>
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
                    let text = translation_text.get();
                    if text.is_empty() {
                        view! {
                            <span class="text-gray-400 dark:text-gray-600 italic">
                                "Translation will appear here\u{2026}"
                            </span>
                        }.into_any()
                    } else {
                        view! {
                            <span>{text}</span>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
