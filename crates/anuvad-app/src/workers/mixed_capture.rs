use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    AudioContext, AudioContextOptions, MediaStream, MediaStreamConstraints, ScriptProcessorNode,
};
use std::cell::RefCell;

use crate::workers::bridge::{self, WorkerMessage};

thread_local! {
    static MIXED_AUDIO_CTX: RefCell<Option<AudioContext>> = RefCell::new(None);
    static MIXED_SCRIPT_PROCESSOR: RefCell<Option<ScriptProcessorNode>> = RefCell::new(None);
    static MIXED_MIC_STREAM: RefCell<Option<MediaStream>> = RefCell::new(None);
    static MIXED_TAB_STREAM: RefCell<Option<MediaStream>> = RefCell::new(None);
}

pub async fn start_mixed_capture() -> Result<(), String> {
    // 1. Get mic stream via getUserMedia
    let window = web_sys::window().ok_or("No window")?;
    let navigator = window.navigator();
    let media_devices = navigator.media_devices().map_err(|e| format!("{e:?}"))?;

    let mic_constraints = MediaStreamConstraints::new();
    mic_constraints.set_audio(&JsValue::TRUE);
    mic_constraints.set_video(&JsValue::FALSE);

    let mic_promise = media_devices
        .get_user_media_with_constraints(&mic_constraints)
        .map_err(|e| format!("{e:?}"))?;

    let mic_js = wasm_bindgen_futures::JsFuture::from(mic_promise)
        .await
        .map_err(|e| format!("getUserMedia failed: {e:?}"))?;

    let mic_stream: MediaStream = mic_js.dyn_into().map_err(|_| "Not a MediaStream")?;

    // 2. Get system audio stream
    let tab_stream = get_system_audio_stream().await?;

    // 3. Create one AudioContext at 16kHz
    let opts = AudioContextOptions::new();
    opts.set_sample_rate(16000.0);
    let ctx = AudioContext::new_with_context_options(&opts)
        .map_err(|e| format!("AudioContext failed: {e:?}"))?;

    // 4. Create two MediaStreamAudioSourceNodes
    let mic_source = ctx
        .create_media_stream_source(&mic_stream)
        .map_err(|e| format!("createMediaStreamSource (mic) failed: {e:?}"))?;

    let tab_source = ctx
        .create_media_stream_source(&tab_stream)
        .map_err(|e| format!("createMediaStreamSource (tab) failed: {e:?}"))?;

    // 5. Create a single ScriptProcessorNode — WebAudio mixes both inputs automatically
    let processor = ctx
        .create_script_processor_with_buffer_size_and_number_of_input_channels_and_number_of_output_channels(
            4096, 1, 1,
        )
        .map_err(|e| format!("createScriptProcessor failed: {e:?}"))?;

    // 6. onaudioprocess — sends mixed PCM to whisper + updates audio level
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

    // Connect both sources to the single processor, then processor to destination
    mic_source
        .connect_with_audio_node(&processor)
        .map_err(|e| format!("connect mic failed: {e:?}"))?;
    tab_source
        .connect_with_audio_node(&processor)
        .map_err(|e| format!("connect tab failed: {e:?}"))?;
    processor
        .connect_with_audio_node(&ctx.destination())
        .map_err(|e| format!("connect to destination failed: {e:?}"))?;

    // 7. Store for cleanup
    MIXED_AUDIO_CTX.with(|c| *c.borrow_mut() = Some(ctx));
    MIXED_SCRIPT_PROCESSOR.with(|p| *p.borrow_mut() = Some(processor));
    MIXED_MIC_STREAM.with(|m| *m.borrow_mut() = Some(mic_stream));
    MIXED_TAB_STREAM.with(|m| *m.borrow_mut() = Some(tab_stream));

    Ok(())
}

pub fn stop_mixed_capture() {
    MIXED_SCRIPT_PROCESSOR.with(|p| {
        if let Some(proc) = p.borrow_mut().take() {
            proc.disconnect().ok();
        }
    });

    MIXED_AUDIO_CTX.with(|c| {
        if let Some(ctx) = c.borrow_mut().take() {
            let _ = ctx.close();
        }
    });

    // Stop tracks on both streams
    for cell in [&MIXED_MIC_STREAM, &MIXED_TAB_STREAM] {
        cell.with(|m| {
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
}

#[cfg(feature = "extension")]
async fn get_system_audio_stream() -> Result<MediaStream, String> {
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

    let constraints = js_sys::Object::new();
    js_sys::Reflect::set(&constraints, &"audio".into(), &JsValue::TRUE)
        .map_err(|e| format!("{e:?}"))?;
    js_sys::Reflect::set(&constraints, &"video".into(), &JsValue::FALSE)
        .map_err(|e| format!("{e:?}"))?;

    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let callback = Closure::once(move |stream: JsValue| {
            if stream.is_null() || stream.is_undefined() {
                let _ = reject.call1(
                    &JsValue::NULL,
                    &"tabCapture returned null — user denied or no audible tab".into(),
                );
            } else {
                let _ = resolve.call1(&JsValue::NULL, &stream);
            }
        });
        let _ = capture_fn.call2(&tab_capture, &constraints, callback.as_ref());
        callback.forget();
    });

    let tab_js = wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|e| format!("tabCapture failed: {e:?}"))?;

    tab_js
        .dyn_into()
        .map_err(|_| "tabCapture did not return a MediaStream".into())
}

#[cfg(not(feature = "extension"))]
async fn get_system_audio_stream() -> Result<MediaStream, String> {
    // Use getDisplayMedia to capture system/tab audio.
    // We call it via JS interop with {audio: true, video: true} since some browsers
    // require video for getDisplayMedia. We immediately discard the video track.
    let window = web_sys::window().ok_or("No window")?;
    let navigator = window.navigator();
    let media_devices = navigator.media_devices().map_err(|e| format!("{e:?}"))?;

    // Build constraints object manually via JS reflection
    let constraints = js_sys::Object::new();
    js_sys::Reflect::set(&constraints, &"audio".into(), &JsValue::TRUE)
        .map_err(|e| format!("{e:?}"))?;
    js_sys::Reflect::set(&constraints, &"video".into(), &JsValue::TRUE)
        .map_err(|e| format!("{e:?}"))?;

    // Call navigator.mediaDevices.getDisplayMedia(constraints)
    let gdm_fn = js_sys::Reflect::get(&media_devices, &"getDisplayMedia".into())
        .map_err(|_| "getDisplayMedia not available")?;
    let gdm_fn: js_sys::Function = gdm_fn
        .dyn_into()
        .map_err(|_| "getDisplayMedia is not a function")?;

    let promise: js_sys::Promise = gdm_fn
        .call1(&media_devices, &constraints)
        .map_err(|e| format!("getDisplayMedia call failed: {e:?}"))?
        .dyn_into()
        .map_err(|_| "getDisplayMedia did not return a Promise")?;

    let stream_js = wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|e| format!("getDisplayMedia failed: {e:?}"))?;

    let stream: MediaStream = stream_js
        .dyn_into()
        .map_err(|_| "getDisplayMedia did not return a MediaStream")?;

    // Stop video tracks immediately — we only need audio
    let video_tracks = stream.get_video_tracks();
    for i in 0..video_tracks.length() {
        let track = video_tracks.get(i);
        if !track.is_undefined() && !track.is_null() {
            let track: web_sys::MediaStreamTrack = track.unchecked_into();
            track.stop();
        }
    }

    Ok(stream)
}

fn update_audio_level(level: f64) {
    if let Some(state) = leptos::prelude::use_context::<crate::state::AppState>() {
        leptos::prelude::Set::set(&state.audio_level, level);
    }
}
