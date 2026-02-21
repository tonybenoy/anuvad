use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Cache, Request, Response};

const CACHE_NAME: &str = "anuvad-models-v1";

const WHISPER_MODEL_URL: &str =
    "https://huggingface.co/openai/whisper-small/resolve/main/model.safetensors";
const WHISPER_TOKENIZER_URL: &str =
    "https://huggingface.co/openai/whisper-small/resolve/main/tokenizer.json";
const WHISPER_CONFIG_URL: &str =
    "https://huggingface.co/openai/whisper-small/resolve/main/config.json";
const WHISPER_MEL_URL: &str =
    "https://huggingface.co/openai/whisper-small/resolve/main/melfilters.bytes";

const PHI_MODEL_URL: &str =
    "https://huggingface.co/microsoft/Phi-3.5-mini-instruct-gguf/resolve/main/Phi-3.5-mini-instruct-Q4_K_M.gguf";
const PHI_TOKENIZER_URL: &str =
    "https://huggingface.co/microsoft/Phi-3.5-mini-instruct/resolve/main/tokenizer.json";

async fn open_cache() -> Result<Cache, String> {
    let window = web_sys::window().ok_or("No window")?;
    let caches = window.caches().map_err(|e| format!("{e:?}"))?;
    let cache_promise = caches.open(CACHE_NAME);
    let cache_js = JsFuture::from(cache_promise)
        .await
        .map_err(|e| format!("Cache open failed: {e:?}"))?;
    cache_js
        .dyn_into::<Cache>()
        .map_err(|_| "Not a Cache".to_string())
}

async fn is_cached(url: &str) -> Result<bool, String> {
    let cache = open_cache().await?;
    let request = Request::new_with_str(url).map_err(|e| format!("{e:?}"))?;
    let match_promise = cache.match_with_request(&request);
    let result = JsFuture::from(match_promise)
        .await
        .map_err(|e| format!("{e:?}"))?;
    Ok(!result.is_undefined())
}

async fn fetch_with_progress(
    url: &str,
    on_progress: impl Fn(f64) + 'static,
) -> Result<Vec<u8>, String> {
    // Check cache first
    let cache = open_cache().await?;
    let request = Request::new_with_str(url).map_err(|e| format!("{e:?}"))?;
    let match_result = JsFuture::from(cache.match_with_request(&request))
        .await
        .map_err(|e| format!("{e:?}"))?;

    if !match_result.is_undefined() {
        let response: Response = match_result.dyn_into().map_err(|_| "Not a Response")?;
        let ab = JsFuture::from(response.array_buffer().map_err(|e| format!("{e:?}"))?)
            .await
            .map_err(|e| format!("{e:?}"))?;
        let uint8 = js_sys::Uint8Array::new(&ab);
        on_progress(1.0);
        return Ok(uint8.to_vec());
    }

    // Fetch with progress tracking
    let window = web_sys::window().ok_or("No window")?;
    let resp_js = JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(|e| format!("Fetch failed: {e:?}"))?;
    let response: Response = resp_js.dyn_into().map_err(|_| "Not a Response")?;

    if !response.ok() {
        return Err(format!("HTTP {}", response.status()));
    }

    let content_length = response
        .headers()
        .get("content-length")
        .ok()
        .flatten()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    let body = response.body().ok_or("No response body")?;
    let reader = body
        .get_reader()
        .dyn_into::<web_sys::ReadableStreamDefaultReader>()
        .map_err(|_| "Not a reader")?;

    let mut received = Vec::new();
    let mut total_received: u64 = 0;

    loop {
        let chunk = JsFuture::from(reader.read())
            .await
            .map_err(|e| format!("Read failed: {e:?}"))?;

        let done = js_sys::Reflect::get(&chunk, &"done".into())
            .map_err(|e| format!("{e:?}"))?
            .as_bool()
            .unwrap_or(true);

        if done {
            break;
        }

        let value = js_sys::Reflect::get(&chunk, &"value".into())
            .map_err(|e| format!("{e:?}"))?;
        let array = js_sys::Uint8Array::new(&value);
        let mut buf = vec![0u8; array.length() as usize];
        array.copy_to(&mut buf);

        total_received += buf.len() as u64;
        received.extend(buf);

        if content_length > 0 {
            on_progress(total_received as f64 / content_length as f64);
        }
    }

    on_progress(1.0);

    // Cache the response for next time
    let resp_init = web_sys::ResponseInit::new();
    resp_init.set_status(200);
    let headers = web_sys::Headers::new().map_err(|e| format!("{e:?}"))?;
    headers
        .set("content-type", "application/octet-stream")
        .map_err(|e| format!("{e:?}"))?;
    resp_init.set_headers(&headers.into());

    let uint8 = js_sys::Uint8Array::from(received.as_slice());
    let blob_parts = js_sys::Array::new();
    blob_parts.push(&uint8.buffer());
    let blob = web_sys::Blob::new_with_u8_array_sequence(&blob_parts)
        .map_err(|e| format!("{e:?}"))?;
    let cache_resp = Response::new_with_opt_blob_and_init(Some(&blob), &resp_init)
        .map_err(|e| format!("{e:?}"))?;

    let cache_request = Request::new_with_str(url).map_err(|e| format!("{e:?}"))?;
    let put_promise = cache.put_with_request(&cache_request, &cache_resp);
    JsFuture::from(put_promise)
        .await
        .map_err(|e| format!("Cache put failed: {e:?}"))?;

    Ok(received)
}

pub async fn download_whisper_model(
    on_progress: impl Fn(f64) + Clone + 'static,
) -> Result<(), String> {
    // Request persistent storage
    request_persistent_storage().await;

    let urls = [
        WHISPER_MODEL_URL,
        WHISPER_TOKENIZER_URL,
        WHISPER_CONFIG_URL,
        WHISPER_MEL_URL,
    ];

    let total = urls.len() as f64;
    for (i, url) in urls.iter().enumerate() {
        let base = i as f64 / total;
        let on_progress = on_progress.clone();
        fetch_with_progress(url, move |p| {
            on_progress(base + p / total);
        })
        .await?;
    }

    Ok(())
}

pub async fn download_translator_model(
    on_progress: impl Fn(f64) + Clone + 'static,
) -> Result<(), String> {
    request_persistent_storage().await;

    let urls = [PHI_MODEL_URL, PHI_TOKENIZER_URL];
    let total = urls.len() as f64;
    for (i, url) in urls.iter().enumerate() {
        let base = i as f64 / total;
        let on_progress = on_progress.clone();
        fetch_with_progress(url, move |p| {
            on_progress(base + p / total);
        })
        .await?;
    }

    Ok(())
}

pub async fn get_cached_bytes(url: &str) -> Result<Vec<u8>, String> {
    fetch_with_progress(url, |_| {}).await
}

async fn request_persistent_storage() {
    let window = web_sys::window().unwrap();
    let navigator = window.navigator();
    let storage = navigator.storage();
    if let Ok(promise) = js_sys::Reflect::get(&storage, &"persist".into()) {
        if let Some(func) = promise.dyn_ref::<js_sys::Function>() {
            let _ = func.call0(&storage);
        }
    }
}

// Public URLs for use by worker crates
pub const fn whisper_model_url() -> &'static str {
    WHISPER_MODEL_URL
}

pub const fn whisper_tokenizer_url() -> &'static str {
    WHISPER_TOKENIZER_URL
}

pub const fn whisper_config_url() -> &'static str {
    WHISPER_CONFIG_URL
}

pub const fn whisper_mel_url() -> &'static str {
    WHISPER_MEL_URL
}

pub const fn phi_model_url() -> &'static str {
    PHI_MODEL_URL
}

pub const fn phi_tokenizer_url() -> &'static str {
    PHI_TOKENIZER_URL
}
