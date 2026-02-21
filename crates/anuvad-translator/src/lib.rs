use wasm_bindgen::prelude::*;

pub mod model;
pub mod generate;
pub mod prompt;

#[wasm_bindgen]
pub struct TranslatorWorker {
    generator: Option<generate::TextGenerator>,
}

#[wasm_bindgen]
impl TranslatorWorker {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Self { generator: None }
    }

    #[wasm_bindgen]
    pub fn load_model(
        &mut self,
        model_bytes: &[u8],
        tokenizer_json: &str,
    ) -> Result<(), JsValue> {
        let gen = generate::TextGenerator::new(model_bytes, tokenizer_json)
            .map_err(|e| JsValue::from_str(&e))?;
        self.generator = Some(gen);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn translate(
        &mut self,
        text: &str,
        target_language: &str,
        callback: &js_sys::Function,
    ) -> Result<String, JsValue> {
        let gen = self
            .generator
            .as_mut()
            .ok_or_else(|| JsValue::from_str("Model not loaded"))?;

        let prompt = prompt::build_translation_prompt(text, target_language);

        let result = gen
            .generate(&prompt, 512, |token| {
                let token_js = JsValue::from_str(token);
                let _ = callback.call1(&JsValue::NULL, &token_js);
            })
            .map_err(|e| JsValue::from_str(&e))?;

        Ok(result)
    }
}
