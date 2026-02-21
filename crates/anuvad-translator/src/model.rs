use candle_core::{Device, Tensor};
use candle_transformers::models::quantized_llama as llama;

pub struct QuantizedModel {
    pub model: llama::ModelWeights,
    pub device: Device,
}

impl QuantizedModel {
    pub fn from_gguf(data: &[u8]) -> Result<Self, String> {
        let device = Device::Cpu;

        let mut cursor = std::io::Cursor::new(data);
        let gguf = candle_core::quantized::gguf_file::Content::read(&mut cursor)
            .map_err(|e| format!("GGUF parse error: {e}"))?;

        let model = llama::ModelWeights::from_gguf(gguf, &mut cursor, &device)
            .map_err(|e| format!("Model load error: {e}"))?;

        Ok(Self { model, device })
    }

    pub fn forward(&mut self, tokens: &[u32], pos: usize) -> Result<Tensor, String> {
        let input = Tensor::new(tokens, &self.device)
            .map_err(|e| format!("Tensor error: {e}"))?;

        self.model
            .forward(&input, pos)
            .map_err(|e| format!("Forward error: {e}"))
    }
}
