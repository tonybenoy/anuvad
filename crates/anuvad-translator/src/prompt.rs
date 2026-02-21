pub fn build_translation_prompt(text: &str, target_language: &str) -> String {
    let lang_name = language_display_name(target_language);

    format!(
        "<|system|>\nYou are a professional translator. Translate the given text accurately to {lang_name}. Output ONLY the translation, nothing else.<|end|>\n<|user|>\nTranslate the following text to {lang_name}:\n\n{text}<|end|>\n<|assistant|>\n"
    )
}

fn language_display_name(code: &str) -> &str {
    match code {
        "en" => "English",
        "es" => "Spanish",
        "fr" => "French",
        "de" => "German",
        "it" => "Italian",
        "pt" => "Portuguese",
        "nl" => "Dutch",
        "pl" => "Polish",
        "ru" => "Russian",
        "uk" => "Ukrainian",
        "ar" => "Arabic",
        "hi" => "Hindi",
        "bn" => "Bengali",
        "ta" => "Tamil",
        "te" => "Telugu",
        "mr" => "Marathi",
        "gu" => "Gujarati",
        "kn" => "Kannada",
        "ml" => "Malayalam",
        "pa" => "Punjabi",
        "ur" => "Urdu",
        "zh" => "Chinese",
        "ja" => "Japanese",
        "ko" => "Korean",
        "vi" => "Vietnamese",
        "th" => "Thai",
        "id" => "Indonesian",
        "ms" => "Malay",
        "tr" => "Turkish",
        "sv" => "Swedish",
        "da" => "Danish",
        "no" => "Norwegian",
        "fi" => "Finnish",
        "el" => "Greek",
        "cs" => "Czech",
        "ro" => "Romanian",
        "hu" => "Hungarian",
        "he" => "Hebrew",
        "fa" => "Persian",
        "sw" => "Swahili",
        _ => code,
    }
}
