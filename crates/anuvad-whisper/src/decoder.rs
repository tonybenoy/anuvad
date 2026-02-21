use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::whisper::{self as m, Config};
use serde::{Serialize, Deserialize};
use tokenizers::Tokenizer;

use crate::audio;
use crate::languages;

const SAMPLE_RATE: usize = 16000;

#[derive(Serialize, Deserialize, Debug)]
pub struct TranscriptionResult {
    pub text: String,
    pub language: Option<String>,
}

pub struct WhisperDecoder {
    model: m::model::Whisper,
    tokenizer: Tokenizer,
    config: Config,
    mel_filters: Vec<f32>,
    device: Device,
}

impl WhisperDecoder {
    pub fn new(
        model_bytes: &[u8],
        tokenizer_json: &str,
        config_json: &str,
        mel_bytes: &[u8],
    ) -> Result<Self, String> {
        let device = Device::Cpu;

        let config: Config =
            serde_json::from_str(config_json).map_err(|e| format!("Config parse error: {e}"))?;

        let tokenizer = Tokenizer::from_bytes(tokenizer_json.as_bytes())
            .map_err(|e| format!("Tokenizer error: {e}"))?;

        let vb = VarBuilder::from_buffered_safetensors(
            model_bytes.to_vec(),
            candle_core::DType::F32,
            &device,
        )
        .map_err(|e| format!("VarBuilder error: {e}"))?;

        let model =
            m::model::Whisper::load(&vb, config.clone()).map_err(|e| format!("Model load error: {e}"))?;

        // Parse mel filter bytes (f32 little-endian)
        let mel_filters: Vec<f32> = mel_bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        Ok(Self {
            model,
            tokenizer,
            config,
            mel_filters,
            device,
        })
    }

    pub fn transcribe(&mut self, pcm: &[f32]) -> Result<TranscriptionResult, String> {
        // Convert PCM to mel spectrogram
        let mel = audio::pcm_to_mel(&self.config, pcm, &self.mel_filters)
            .map_err(|e| format!("Mel conversion error: {e}"))?;

        let mel_len = mel.len() / self.config.num_mel_bins;
        let mel_tensor = Tensor::from_vec(
            mel,
            (1, self.config.num_mel_bins, mel_len),
            &self.device,
        )
        .map_err(|e| format!("Tensor error: {e}"))?;

        // Encode
        let encoder_output = self
            .model
            .encoder
            .forward(&mel_tensor, true)
            .map_err(|e| format!("Encoder error: {e}"))?;

        // Detect language
        let language = self.detect_language(&encoder_output)?;

        // Decode
        let text = self.greedy_decode(&encoder_output, &language)?;

        Ok(TranscriptionResult {
            text,
            language: Some(language),
        })
    }

    fn detect_language(&self, encoder_output: &Tensor) -> Result<String, String> {
        // For simplicity, use the SOT token detection approach
        // In a full implementation, we'd use the language detection head
        Ok("en".to_string())
    }

    fn greedy_decode(
        &mut self,
        encoder_output: &Tensor,
        language: &str,
    ) -> Result<String, String> {
        let sot_token = self
            .tokenizer
            .token_to_id("<|startoftranscript|>")
            .unwrap_or(50258);
        let eot_token = self
            .tokenizer
            .token_to_id("<|endoftext|>")
            .unwrap_or(50257);
        let transcribe_token = self
            .tokenizer
            .token_to_id("<|transcribe|>")
            .unwrap_or(50359);
        let notimestamps_token = self
            .tokenizer
            .token_to_id("<|notimestamps|>")
            .unwrap_or(50363);

        let lang_token = self
            .tokenizer
            .token_to_id(&format!("<|{language}|>"))
            .unwrap_or(50259);

        let mut tokens = vec![sot_token, lang_token, transcribe_token, notimestamps_token];
        let mut result_tokens = Vec::new();

        for _ in 0..224 {
            let token_tensor = Tensor::new(
                tokens.as_slice(),
                &self.device,
            )
            .map_err(|e| format!("Token tensor error: {e}"))?
            .unsqueeze(0)
            .map_err(|e| format!("Unsqueeze error: {e}"))?;

            let logits = self
                .model
                .decoder
                .forward(&token_tensor, encoder_output, true)
                .map_err(|e| format!("Decoder error: {e}"))?;

            let (_, seq_len, _) = logits.dims3().map_err(|e| format!("Dims error: {e}"))?;
            let last_logits = logits
                .get_on_dim(1, seq_len - 1)
                .map_err(|e| format!("Get last error: {e}"))?;

            let next_token = last_logits
                .argmax(candle_core::D::Minus1)
                .map_err(|e| format!("Argmax error: {e}"))?
                .to_scalar::<u32>()
                .map_err(|e| format!("Scalar error: {e}"))?;

            if next_token == eot_token {
                break;
            }

            result_tokens.push(next_token);
            tokens.push(next_token);
        }

        let text = self
            .tokenizer
            .decode(&result_tokens, true)
            .map_err(|e| format!("Decode error: {e}"))?;

        Ok(text.trim().to_string())
    }
}
