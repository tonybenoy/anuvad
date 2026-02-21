use leptos::prelude::*;

use crate::state::AppState;

#[component]
pub fn Header() -> impl IntoView {
    let state = expect_context::<AppState>();

    let toggle_dark = move |_| {
        let window = web_sys::window().unwrap();
        let doc = window.document().unwrap();
        let html = doc.document_element().unwrap();
        let class_list = html.class_list();
        let _ = class_list.toggle("dark");
    };

    view! {
        <header class="border-b border-gray-200 dark:border-gray-800 bg-white/80 dark:bg-gray-900/80 backdrop-blur-sm sticky top-0 z-50">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4 flex items-center justify-between">
                <div class="flex items-center gap-3">
                    <h1 class="text-2xl font-bold bg-gradient-to-r from-indigo-600 to-purple-600 bg-clip-text text-transparent">
                        "Anuvad"
                    </h1>
                    <span class="text-xs text-gray-500 dark:text-gray-400 hidden sm:inline">
                        "Local Transcription & Translation"
                    </span>
                </div>

                <div class="flex items-center gap-3">
                    <div class="flex items-center gap-2">
                        <span class="text-xs text-gray-500 dark:text-gray-400">"Whisper:"</span>
                        <span class={move || state.whisper_status.get().badge_class()}>
                            {move || state.whisper_status.get().label()}
                        </span>
                    </div>
                    <div class="flex items-center gap-2">
                        <span class="text-xs text-gray-500 dark:text-gray-400">"Phi-3.5:"</span>
                        <span class={move || state.translator_status.get().badge_class()}>
                            {move || state.translator_status.get().label()}
                        </span>
                    </div>

                    <button
                        class="p-2 rounded-lg bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
                        on:click=toggle_dark
                        title="Toggle dark mode"
                    >
                        <span class="text-sm">{"\u{263E}"}</span>
                    </button>
                </div>
            </div>
        </header>
    }
}
