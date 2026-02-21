use leptos::prelude::*;
use leptos::ev;

use crate::state::AppState;

const LANGUAGES: &[(&str, &str)] = &[
    ("auto", "Auto-detect"),
    ("en", "English"),
    ("es", "Spanish"),
    ("fr", "French"),
    ("de", "German"),
    ("it", "Italian"),
    ("pt", "Portuguese"),
    ("nl", "Dutch"),
    ("pl", "Polish"),
    ("ru", "Russian"),
    ("uk", "Ukrainian"),
    ("ar", "Arabic"),
    ("hi", "Hindi"),
    ("bn", "Bengali"),
    ("ta", "Tamil"),
    ("te", "Telugu"),
    ("mr", "Marathi"),
    ("gu", "Gujarati"),
    ("kn", "Kannada"),
    ("ml", "Malayalam"),
    ("pa", "Punjabi"),
    ("ur", "Urdu"),
    ("zh", "Chinese"),
    ("ja", "Japanese"),
    ("ko", "Korean"),
    ("vi", "Vietnamese"),
    ("th", "Thai"),
    ("id", "Indonesian"),
    ("ms", "Malay"),
    ("tr", "Turkish"),
    ("sv", "Swedish"),
    ("da", "Danish"),
    ("no", "Norwegian"),
    ("fi", "Finnish"),
    ("el", "Greek"),
    ("cs", "Czech"),
    ("ro", "Romanian"),
    ("hu", "Hungarian"),
    ("he", "Hebrew"),
    ("fa", "Persian"),
    ("sw", "Swahili"),
];

// TARGET_LANGUAGES: skip "auto" - done inline below

#[component]
pub fn LanguageSelector() -> impl IntoView {
    let state = expect_context::<AppState>();

    let on_source_change = move |ev: ev::Event| {
        let target = event_target_value(&ev);
        state.source_language.set(target);
    };

    let on_target_change = move |ev: ev::Event| {
        let target = event_target_value(&ev);
        state.target_language.set(target);
    };

    view! {
        <div class="card">
            <div class="flex flex-col sm:flex-row items-center gap-4">
                <div class="flex-1 w-full">
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        "Source Language"
                    </label>
                    <select
                        class="w-full px-3 py-2 bg-gray-100 dark:bg-gray-800 border border-gray-300 dark:border-gray-700 rounded-lg text-sm focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
                        on:change=on_source_change
                    >
                        {LANGUAGES.iter().map(|(code, name)| {
                            let code = *code;
                            let name = *name;
                            view! {
                                <option value=code selected=move || state.source_language.get() == code>
                                    {name}
                                </option>
                            }
                        }).collect::<Vec<_>>()}
                    </select>
                </div>

                <div class="hidden sm:flex items-center pt-6">
                    <span class="text-gray-400 text-xl">"\u{2192}"</span>
                </div>

                <div class="flex-1 w-full">
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        "Target Language"
                    </label>
                    <select
                        class="w-full px-3 py-2 bg-gray-100 dark:bg-gray-800 border border-gray-300 dark:border-gray-700 rounded-lg text-sm focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
                        on:change=on_target_change
                    >
                        {LANGUAGES[1..].iter().map(|(code, name)| {
                            let code = *code;
                            let name = *name;
                            view! {
                                <option value=code selected=move || state.target_language.get() == code>
                                    {name}
                                </option>
                            }
                        }).collect::<Vec<_>>()}
                    </select>
                </div>
            </div>
        </div>
    }
}
