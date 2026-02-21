use leptos::prelude::*;

use crate::state::{AppState, AudioSource};

#[component]
pub fn AudioSourceSelector() -> impl IntoView {
    let state = expect_context::<AppState>();
    let audio_source = state.audio_source;

    let mic_class = move || {
        let base = "px-4 py-2 text-sm font-medium rounded-lg transition-all duration-150";
        if audio_source.get() == AudioSource::Microphone {
            format!("{base} bg-indigo-600 text-white shadow-md")
        } else {
            format!("{base} bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700")
        }
    };

    #[cfg(feature = "extension")]
    let tab_class = move || {
        let base = "px-4 py-2 text-sm font-medium rounded-lg transition-all duration-150";
        if audio_source.get() == AudioSource::TabAudio {
            format!("{base} bg-indigo-600 text-white shadow-md")
        } else {
            format!("{base} bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700")
        }
    };

    let both_class = move || {
        let base = "px-4 py-2 text-sm font-medium rounded-lg transition-all duration-150";
        if audio_source.get() == AudioSource::Both {
            format!("{base} bg-indigo-600 text-white shadow-md")
        } else {
            format!("{base} bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700")
        }
    };

    view! {
        <div class="flex items-center justify-center gap-2">
            <span class="text-sm text-gray-500 dark:text-gray-400 mr-1">"Source:"</span>
            <button
                class=mic_class
                on:click=move |_| audio_source.set(AudioSource::Microphone)
            >
                "Microphone"
            </button>
            {
                #[cfg(feature = "extension")]
                view! {
                    <button
                        class=tab_class
                        on:click=move |_| audio_source.set(AudioSource::TabAudio)
                    >
                        "Tab Audio"
                    </button>
                }
            }
            <button
                class=both_class
                on:click=move |_| audio_source.set(AudioSource::Both)
            >
                "Both"
            </button>
        </div>
    }
}
