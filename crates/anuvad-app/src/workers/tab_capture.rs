use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{AudioContext, AudioContextOptions, MediaStream, ScriptProcessorNode};
use std::cell::RefCell;

use crate::workers::bridge::{self, WorkerMessage};

thread_local! {
    static TAB_AUDIO_CTX: RefCell<Option<AudioContext>> = RefCell::new(None);
    static TAB_SCRIPT_PROCESSOR: RefCell<Option<ScriptProcessorNode>> = RefCell::new(None);
    static TAB_MEDIA_STREAM: RefCell<Option<MediaStream>> = RefCell::new(None);
}

pub async fn start_tab_capture() -> Result<(), String> {
    let global = js_sys::global();
    let chrome = js_sys::Reflect::get(&global, &"chrome".into())
        .map_err(|_| "chrome API not available")?;
    if chrome.is_undefined() {
        return Err("chrome API not available — not running as extension".into());
    }

    let tab_capture = js_sys::Reflect::get(&chrome, &"tabCapture".into())
        .map_err(|_| "chrome.tabCapture not available")?;
    if tab_capture.is_undefined() {
        return Err("chrome.tabCapture not available — missing tabCapture permission".into());
    }

    let capture_fn = js_sys::Reflect::get(&tab_capture, &"capture".into())
        .map_err(|e| format!("chrome.tabCapture.capture not found: {e:?}"))?;
    let capture_fn: js_sys::Function = capture_fn
        .dyn_into()
        .map_err(|_| "chrome.tabCapture.capture is not a function")?;

    // Build constraints: { audio: true, video: false }
    let constraints = js_sys::Object::new();
    js_sys::Reflect::set(&constraints, &"audio".into(), &JsValue::TRUE)
        .map_err(|e| format!("{e:?}"))?;
    js_sys::Reflect::set(&constraints, &"video".into(), &JsValue::FALSE)
        .map_err(|e| format!("{e:?}"))?;

    // chrome.tabCapture.capture uses a callback pattern — wrap in a Promise
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let callback = Closure::once(move |stream: JsValue| {
            if stream.is_null() || stream.is_undefined() {
                let _ = reject.call1(&JsValue::NULL, &"tabCapture returned null — user denied or no audible tab".into());
            } else {
                let _ = resolve.call1(&JsValue::NULL, &stream);
            }
        });
        let _ = capture_fn.call2(&tab_capture, &constraints, callback.as_ref());
        callback.forget();
    });

    let stream_js = wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|e| format!("tabCapture failed: {e:?}"))?;

    let stream: MediaStream = stream_js
        .dyn_into()
        .map_err(|_| "tabCapture did not return a MediaStream")?;

    // Create AudioContext at 16kHz (same pipeline as mic capture)
    let opts = AudioContextOptions::new();
    opts.set_sample_rate(16000.0);
    let ctx = AudioContext::new_with_context_options(&opts)
        .map_err(|e| format!("AudioContext failed: {e:?}"))?;

    let source = ctx
        .create_media_stream_source(&stream)
        .map_err(|e| format!("createMediaStreamSource failed: {e:?}"))?;

    let processor = ctx
        .create_script_processor_with_buffer_size_and_number_of_input_channels_and_number_of_output_channels(
            4096, 1, 1,
        )
        .map_err(|e| format!("createScriptProcessor failed: {e:?}"))?;

    let onaudioprocess = Closure::wrap(Box::new(move |event: web_sys::AudioProcessingEvent| {
        let input_buffer = event.input_buffer().unwrap();
        let channel_data = input_buffer.get_channel_data(0).unwrap();

        let rms: f32 = (channel_data.iter().map(|s| s * s).sum::<f32>()
            / channel_data.len() as f32)
            .sqrt();

        let msg = WorkerMessage::Transcribe {
            audio: channel_data.to_vec(),
        };
        bridge::send_to_whisper(&msg);

        update_audio_level(rms as f64);
    }) as Box<dyn FnMut(web_sys::AudioProcessingEvent)>);

    processor.set_onaudioprocess(Some(onaudioprocess.as_ref().unchecked_ref()));
    onaudioprocess.forget();

    source
        .connect_with_audio_node(&processor)
        .map_err(|e| format!("connect failed: {e:?}"))?;
    processor
        .connect_with_audio_node(&ctx.destination())
        .map_err(|e| format!("connect to destination failed: {e:?}"))?;

    TAB_AUDIO_CTX.with(|c| *c.borrow_mut() = Some(ctx));
    TAB_SCRIPT_PROCESSOR.with(|p| *p.borrow_mut() = Some(processor));
    TAB_MEDIA_STREAM.with(|m| *m.borrow_mut() = Some(stream));

    Ok(())
}

pub fn stop_tab_capture() {
    TAB_SCRIPT_PROCESSOR.with(|p| {
        if let Some(proc) = p.borrow_mut().take() {
            proc.disconnect().ok();
        }
    });

    TAB_AUDIO_CTX.with(|c| {
        if let Some(ctx) = c.borrow_mut().take() {
            let _ = ctx.close();
        }
    });

    TAB_MEDIA_STREAM.with(|m| {
        if let Some(stream) = m.borrow_mut().take() {
            let tracks = stream.get_tracks();
            for i in 0..tracks.length() {
                let track = tracks.get(i);
                if !track.is_undefined() && !track.is_null() {
                    let track: web_sys::MediaStreamTrack = track.unchecked_into();
                    track.stop();
                }
            }
        }
    });
}

fn update_audio_level(level: f64) {
    if let Some(state) = leptos::prelude::use_context::<crate::state::AppState>() {
        leptos::prelude::Set::set(&state.audio_level, level);
    }
}
