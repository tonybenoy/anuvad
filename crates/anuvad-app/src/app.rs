use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use crate::components::header::Header;
use crate::components::model_loader::ModelLoader;
use crate::components::audio_recorder::AudioRecorder;
use crate::components::audio_source_selector::AudioSourceSelector;
use crate::components::transcription::TranscriptionPanel;
use crate::components::translation::TranslationPanel;
use crate::components::language_selector::LanguageSelector;
use crate::state::{AppState, AudioSource, ModelStatus, RecordingState};
use crate::workers::audio_capture;
#[cfg(feature = "extension")]
use crate::workers::tab_capture;
use crate::workers::mixed_capture;

#[component]
pub fn App() -> impl IntoView {
    let state = AppState::new();
    let error_message = state.error_message;
    let whisper_status = state.whisper_status;
    let recording_state = state.recording_state;
    let audio_source = state.audio_source;
    provide_context(state);

    // Global keyboard shortcuts
    let window = web_sys::window().unwrap();
    let keydown_handler = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        // Only handle Space when not in an input/textarea
        if event.code() == "Space" {
            let target = event.target();
            let is_input = target
                .as_ref()
                .and_then(|t| t.dyn_ref::<web_sys::HtmlElement>())
                .map(|el| {
                    let tag = el.tag_name().to_lowercase();
                    tag == "input" || tag == "textarea" || tag == "select"
                })
                .unwrap_or(false);

            if !is_input && whisper_status.get_untracked() == ModelStatus::Ready {
                event.prevent_default();
                match recording_state.get_untracked() {
                    RecordingState::Idle => {
                        spawn_local(async move {
                            let result = match audio_source.get_untracked() {
                                AudioSource::Microphone => audio_capture::start_recording().await,
                                #[cfg(feature = "extension")]
                                AudioSource::TabAudio => tab_capture::start_tab_capture().await,
                                AudioSource::Both => mixed_capture::start_mixed_capture().await,
                            };
                            match result {
                                Ok(()) => recording_state.set(RecordingState::Recording),
                                Err(e) => error_message.set(Some(format!("Audio error: {e}"))),
                            }
                        });
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
                    _ => {}
                }
            }
        }
    }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

    window
        .add_event_listener_with_callback("keydown", keydown_handler.as_ref().unchecked_ref())
        .unwrap();
    keydown_handler.forget();

    view! {
        <div class="min-h-screen flex flex-col">
            <Header />

            <main class="flex-1 max-w-7xl mx-auto w-full px-4 sm:px-6 lg:px-8 py-8 space-y-8">
                // Error banner
                {move || {
                    error_message.get().map(|msg| {
                        view! {
                            <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl p-4 flex items-center justify-between">
                                <p class="text-red-800 dark:text-red-400 text-sm">{msg.clone()}</p>
                                <button
                                    class="text-red-600 dark:text-red-400 hover:text-red-800 dark:hover:text-red-300 font-bold"
                                    on:click=move |_| error_message.set(None)
                                >
                                    "\u{2715}"
                                </button>
                            </div>
                        }
                    })
                }}

                <ModelLoader />
                <LanguageSelector />
                <AudioSourceSelector />
                <AudioRecorder />

                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                    <TranscriptionPanel />
                    <TranslationPanel />
                </div>
            </main>

            <footer class="text-center py-4 text-xs text-gray-500 dark:text-gray-600">
                "All processing happens locally in your browser. No data leaves your device. "
                <kbd class="px-1.5 py-0.5 bg-gray-200 dark:bg-gray-800 rounded text-xs">"Space"</kbd>
                " to record."
            </footer>
        </div>
    }
}
