use candle_core::{Device, Tensor};
use tokenizers::Tokenizer;

use crate::model::QuantizedModel;

pub struct TextGenerator {
    model: QuantizedModel,
    tokenizer: Tokenizer,
}

impl TextGenerator {
    pub fn new(model_bytes: &[u8], tokenizer_json: &str) -> Result<Self, String> {
        let model = QuantizedModel::from_gguf(model_bytes)?;
        let tokenizer = Tokenizer::from_bytes(tokenizer_json.as_bytes())
            .map_err(|e| format!("Tokenizer error: {e}"))?;
        Ok(Self { model, tokenizer })
    }

    pub fn generate(
        &mut self,
        prompt: &str,
        max_tokens: usize,
        mut on_token: impl FnMut(&str),
    ) -> Result<String, String> {
        let encoding = self
            .tokenizer
            .encode(prompt, true)
            .map_err(|e| format!("Encode error: {e}"))?;
        let prompt_tokens: Vec<u32> = encoding.get_ids().to_vec();

        let mut all_tokens = prompt_tokens.clone();
        let mut generated_text = String::new();
        let mut pos = 0;

        // Process prompt tokens
        let prompt_len = prompt_tokens.len();
        if prompt_len > 0 {
            let _ = self.model.forward(&prompt_tokens, 0)?;
            pos = prompt_len;
        }

        let eos_token = self
            .tokenizer
            .token_to_id("<|endoftext|>")
            .or_else(|| self.tokenizer.token_to_id("</s>"))
            .or_else(|| self.tokenizer.token_to_id("<|end|>"))
            .unwrap_or(2);

        // Generate tokens
        for _ in 0..max_tokens {
            let last_token = *all_tokens.last().unwrap();
            let logits = self.model.forward(&[last_token], pos)?;

            // Sample: greedy (argmax)
            let next_token = logits
                .argmax(candle_core::D::Minus1)
                .map_err(|e| format!("Argmax error: {e}"))?
                .to_scalar::<u32>()
                .map_err(|e| format!("Scalar error: {e}"))?;

            if next_token == eos_token {
                break;
            }

            all_tokens.push(next_token);
            pos += 1;

            // Decode the new token
            let token_text = self
                .tokenizer
                .decode(&[next_token], false)
                .map_err(|e| format!("Decode error: {e}"))?;

            generated_text.push_str(&token_text);
            on_token(&token_text);
        }

        Ok(generated_text)
    }
}
