use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

pub mod decoder;
pub mod audio;
pub mod streaming;
pub mod languages;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum WhisperMessage {
    LoadModel {
        model_bytes: Vec<u8>,
        tokenizer_json: String,
        config_json: String,
        mel_bytes: Vec<u8>,
    },
    Transcribe {
        audio: Vec<f32>,
    },
    ModelLoaded,
    TranscriptionResult {
        text: String,
        language: Option<String>,
    },
    TranscriptionPartial {
        text: String,
    },
    Progress {
        percent: f64,
    },
    Error {
        message: String,
    },
}

#[wasm_bindgen]
pub struct WhisperWorker {
    decoder: Option<decoder::WhisperDecoder>,
    streaming: streaming::StreamingBuffer,
}

#[wasm_bindgen]
impl WhisperWorker {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Self {
            decoder: None,
            streaming: streaming::StreamingBuffer::new(),
        }
    }

    #[wasm_bindgen]
    pub fn load_model(
        &mut self,
        model_bytes: &[u8],
        tokenizer_json: &str,
        config_json: &str,
        mel_bytes: &[u8],
    ) -> Result<(), JsValue> {
        let dec = decoder::WhisperDecoder::new(model_bytes, tokenizer_json, config_json, mel_bytes)
            .map_err(|e| JsValue::from_str(&e))?;
        self.decoder = Some(dec);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn push_audio(&mut self, pcm: &[f32]) {
        self.streaming.push(pcm);
    }

    #[wasm_bindgen]
    pub fn transcribe(&mut self) -> Result<JsValue, JsValue> {
        let decoder = self
            .decoder
            .as_mut()
            .ok_or_else(|| JsValue::from_str("Model not loaded"))?;

        let audio = self.streaming.get_chunk();
        if audio.is_empty() {
            return Ok(JsValue::NULL);
        }

        let result = decoder
            .transcribe(&audio)
            .map_err(|e| JsValue::from_str(&e))?;

        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&format!("{e}")))
    }
}
