use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::state::{AppState, ModelStatus};
use crate::workers::model_cache;

#[component]
pub fn ModelLoader() -> impl IntoView {
    let state = expect_context::<AppState>();

    let whisper_status = state.whisper_status;
    let whisper_progress = state.whisper_progress;
    let translator_status = state.translator_status;
    let translator_progress = state.translator_progress;
    let error_message = state.error_message;

    let download_whisper = move |_| {
        spawn_local(async move {
            whisper_status.set(ModelStatus::Downloading);
            match model_cache::download_whisper_model(
                move |p| whisper_progress.set(p),
            ).await {
                Ok(()) => {
                    whisper_status.set(ModelStatus::Ready);
                }
                Err(e) => {
                    whisper_status.set(ModelStatus::Error);
                    error_message.set(Some(format!("Whisper download failed: {e}")));
                }
            }
        });
    };

    let download_translator = move |_| {
        spawn_local(async move {
            translator_status.set(ModelStatus::Downloading);
            match model_cache::download_translator_model(
                move |p| translator_progress.set(p),
            ).await {
                Ok(()) => {
                    translator_status.set(ModelStatus::Ready);
                }
                Err(e) => {
                    translator_status.set(ModelStatus::Error);
                    error_message.set(Some(format!("Translator download failed: {e}")));
                }
            }
        });
    };

    view! {
        <div class="card">
            <h2 class="text-lg font-semibold mb-4">"Model Management"</h2>
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-6">
                // Whisper model
                <div class="space-y-3">
                    <div class="flex items-center justify-between">
                        <div>
                            <h3 class="font-medium">"Whisper Small"</h3>
                            <p class="text-xs text-gray-500 dark:text-gray-400">"~460 MB \u{2022} Speech recognition"</p>
                        </div>
                        <span class={move || whisper_status.get().badge_class()}>
                            {move || whisper_status.get().label()}
                        </span>
                    </div>

                    {move || {
                        match whisper_status.get() {
                            ModelStatus::Downloading => {
                                let pct = whisper_progress.get();
                                view! {
                                    <div>
                                        <div class="progress-bar">
                                            <div
                                                class="progress-fill"
                                                style:width=format!("{}%", (pct * 100.0) as u32)
                                            ></div>
                                        </div>
                                        <p class="text-xs text-gray-500 mt-1">
                                            {format!("{:.0}%", pct * 100.0)}
                                        </p>
                                    </div>
                                }.into_any()
                            }
                            ModelStatus::NotDownloaded | ModelStatus::Error => {
                                view! {
                                    <button class="btn-primary w-full text-sm" on:click=download_whisper>
                                        "Download Whisper"
                                    </button>
                                }.into_any()
                            }
                            _ => view! { <div></div> }.into_any()
                        }
                    }}
                </div>

                // Translator model
                <div class="space-y-3">
                    <div class="flex items-center justify-between">
                        <div>
                            <h3 class="font-medium">"Phi-3.5 Mini"</h3>
                            <p class="text-xs text-gray-500 dark:text-gray-400">"~2 GB \u{2022} Translation"</p>
                        </div>
                        <span class={move || translator_status.get().badge_class()}>
                            {move || translator_status.get().label()}
                        </span>
                    </div>

                    {move || {
                        match translator_status.get() {
                            ModelStatus::Downloading => {
                                let pct = translator_progress.get();
                                view! {
                                    <div>
                                        <div class="progress-bar">
                                            <div
                                                class="progress-fill"
                                                style:width=format!("{}%", (pct * 100.0) as u32)
                                            ></div>
                                        </div>
                                        <p class="text-xs text-gray-500 mt-1">
                                            {format!("{:.0}%", pct * 100.0)}
                                        </p>
                                    </div>
                                }.into_any()
                            }
                            ModelStatus::NotDownloaded | ModelStatus::Error => {
                                view! {
                                    <button class="btn-primary w-full text-sm" on:click=download_translator>
                                        "Download Phi-3.5"
                                    </button>
                                }.into_any()
                            }
                            _ => view! { <div></div> }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
