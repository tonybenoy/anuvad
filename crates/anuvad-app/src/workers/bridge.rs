use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Worker, WorkerOptions, WorkerType, MessageEvent};
use leptos::prelude::{Set, Update};
use serde::{Serialize, Deserialize};
use std::cell::RefCell;

use crate::state::AppState;

pub fn worker_script_url(filename: &str) -> String {
    #[cfg(feature = "extension")]
    {
        let global = js_sys::global();
        let chrome = js_sys::Reflect::get(&global, &"chrome".into()).unwrap_or(JsValue::UNDEFINED);
        if !chrome.is_undefined() {
            let runtime = js_sys::Reflect::get(&chrome, &"runtime".into()).unwrap_or(JsValue::UNDEFINED);
            if !runtime.is_undefined() {
                let get_url = js_sys::Reflect::get(&runtime, &"getURL".into()).unwrap_or(JsValue::UNDEFINED);
                if let Some(func) = get_url.dyn_ref::<js_sys::Function>() {
                    let path = format!("workers/{filename}");
                    if let Ok(url) = func.call1(&runtime, &path.into()) {
                        if let Some(s) = url.as_string() {
                            return s;
                        }
                    }
                }
            }
        }
    }
    format!("./{filename}")
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum WorkerMessage {
    // To whisper worker
    LoadModel { data: Vec<u8> },
    Transcribe { audio: Vec<f32> },

    // From whisper worker
    ModelLoaded,
    TranscriptionResult { text: String, language: Option<String> },
    TranscriptionPartial { text: String },

    // To translator worker
    LoadTranslatorModel { data: Vec<u8> },
    Translate { text: String, target_language: String },

    // From translator worker
    TranslatorModelLoaded,
    TranslationToken { token: String },
    TranslationDone { text: String },

    // Common
    Progress { percent: f64 },
    Error { message: String },
}

thread_local! {
    static WHISPER_WORKER: RefCell<Option<Worker>> = RefCell::new(None);
    static TRANSLATOR_WORKER: RefCell<Option<Worker>> = RefCell::new(None);
}

pub fn init_whisper_worker() -> Result<Worker, JsValue> {
    let opts = WorkerOptions::new();
    opts.set_type(WorkerType::Module);
    let url = worker_script_url("whisper_worker.js");
    let worker = Worker::new_with_options(&url, &opts)?;

    WHISPER_WORKER.with(|w| {
        *w.borrow_mut() = Some(worker.clone());
    });

    Ok(worker)
}

pub fn init_translator_worker() -> Result<Worker, JsValue> {
    let opts = WorkerOptions::new();
    opts.set_type(WorkerType::Module);
    let url = worker_script_url("translator_worker.js");
    let worker = Worker::new_with_options(&url, &opts)?;

    TRANSLATOR_WORKER.with(|w| {
        *w.borrow_mut() = Some(worker.clone());
    });

    Ok(worker)
}

pub fn send_to_whisper(msg: &WorkerMessage) {
    WHISPER_WORKER.with(|w| {
        if let Some(worker) = w.borrow().as_ref() {
            let val = serde_wasm_bindgen::to_value(msg).unwrap();
            let _ = worker.post_message(&val);
        }
    });
}

pub fn send_to_translator(msg: &WorkerMessage) {
    TRANSLATOR_WORKER.with(|w| {
        if let Some(worker) = w.borrow().as_ref() {
            let val = serde_wasm_bindgen::to_value(msg).unwrap();
            let _ = worker.post_message(&val);
        }
    });
}

pub async fn request_translation(text: &str, target_language: &str) {
    let msg = WorkerMessage::Translate {
        text: text.to_string(),
        target_language: target_language.to_string(),
    };
    send_to_translator(&msg);
}

pub fn setup_whisper_listener(state: AppState) {
    WHISPER_WORKER.with(|w| {
        if let Some(worker) = w.borrow().as_ref() {
            let state = state.clone();
            let onmessage = Closure::wrap(Box::new(move |event: MessageEvent| {
                let data = event.data();
                if let Ok(msg) = serde_wasm_bindgen::from_value::<WorkerMessage>(data) {
                    match msg {
                        WorkerMessage::TranscriptionResult { text, language } => {
                            state.transcription_text.set(text);
                            if let Some(lang) = language {
                                state.detected_language.set(Some(lang));
                            }
                        }
                        WorkerMessage::TranscriptionPartial { text } => {
                            state.transcription_text.set(text);
                        }
                        WorkerMessage::Error { message } => {
                            state.error_message.set(Some(message));
                        }
                        _ => {}
                    }
                }
            }) as Box<dyn FnMut(MessageEvent)>);
            worker.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
            onmessage.forget();
        }
    });
}

pub fn setup_translator_listener(state: AppState) {
    TRANSLATOR_WORKER.with(|w| {
        if let Some(worker) = w.borrow().as_ref() {
            let state = state.clone();
            let onmessage = Closure::wrap(Box::new(move |event: MessageEvent| {
                let data = event.data();
                if let Ok(msg) = serde_wasm_bindgen::from_value::<WorkerMessage>(data) {
                    match msg {
                        WorkerMessage::TranslationToken { token } => {
                            state.translation_text.update(|t| t.push_str(&token));
                        }
                        WorkerMessage::TranslationDone { text } => {
                            state.translation_text.set(text);
                        }
                        WorkerMessage::Error { message } => {
                            state.error_message.set(Some(message));
                        }
                        _ => {}
                    }
                }
            }) as Box<dyn FnMut(MessageEvent)>);
            worker.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
            onmessage.forget();
        }
    });
}
