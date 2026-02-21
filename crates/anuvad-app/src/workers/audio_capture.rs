use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use leptos::prelude::Set;
use web_sys::{
    AudioContext, AudioContextOptions, MediaStream, MediaStreamConstraints,
    ScriptProcessorNode,
};
use std::cell::RefCell;

use crate::state::AppState;
use crate::workers::bridge::{self, WorkerMessage};

thread_local! {
    static AUDIO_CTX: RefCell<Option<AudioContext>> = RefCell::new(None);
    static SCRIPT_PROCESSOR: RefCell<Option<ScriptProcessorNode>> = RefCell::new(None);
    static MEDIA_STREAM: RefCell<Option<MediaStream>> = RefCell::new(None);
}

pub async fn start_recording() -> Result<(), String> {
    let window = web_sys::window().ok_or("No window")?;
    let navigator = window.navigator();
    let media_devices = navigator.media_devices().map_err(|e| format!("{e:?}"))?;

    let constraints = MediaStreamConstraints::new();
    constraints.set_audio(&JsValue::TRUE);
    constraints.set_video(&JsValue::FALSE);

    let stream_promise = media_devices
        .get_user_media_with_constraints(&constraints)
        .map_err(|e| format!("{e:?}"))?;

    let stream_js = wasm_bindgen_futures::JsFuture::from(stream_promise)
        .await
        .map_err(|e| format!("getUserMedia failed: {e:?}"))?;

    let stream: MediaStream = stream_js.dyn_into().map_err(|_| "Not a MediaStream")?;

    // Create AudioContext at 16kHz
    let opts = AudioContextOptions::new();
    opts.set_sample_rate(16000.0);
    let ctx = AudioContext::new_with_context_options(&opts)
        .map_err(|e| format!("AudioContext failed: {e:?}"))?;

    let source = ctx
        .create_media_stream_source(&stream)
        .map_err(|e| format!("createMediaStreamSource failed: {e:?}"))?;

    // ScriptProcessorNode with 4096 buffer, 1 input, 1 output
    let processor = ctx
        .create_script_processor_with_buffer_size_and_number_of_input_channels_and_number_of_output_channels(
            4096, 1, 1,
        )
        .map_err(|e| format!("createScriptProcessor failed: {e:?}"))?;

    let onaudioprocess = Closure::wrap(Box::new(move |event: web_sys::AudioProcessingEvent| {
        let input_buffer = event.input_buffer().unwrap();
        let channel_data = input_buffer.get_channel_data(0).unwrap();

        // Calculate audio level (RMS)
        let rms: f32 = (channel_data.iter().map(|s| s * s).sum::<f32>()
            / channel_data.len() as f32)
            .sqrt();

        // Send PCM data to whisper worker
        let msg = WorkerMessage::Transcribe {
            audio: channel_data.to_vec(),
        };
        bridge::send_to_whisper(&msg);

        // Update audio level in state (if available)
        // Level is updated via a global since we can't easily pass state here
        update_audio_level(rms as f64);
    }) as Box<dyn FnMut(web_sys::AudioProcessingEvent)>);

    processor.set_onaudioprocess(Some(onaudioprocess.as_ref().unchecked_ref()));
    onaudioprocess.forget();

    // Connect: source → processor → destination
    source
        .connect_with_audio_node(&processor)
        .map_err(|e| format!("connect failed: {e:?}"))?;
    processor
        .connect_with_audio_node(&ctx.destination())
        .map_err(|e| format!("connect to destination failed: {e:?}"))?;

    // Store references for cleanup
    AUDIO_CTX.with(|c| *c.borrow_mut() = Some(ctx));
    SCRIPT_PROCESSOR.with(|p| *p.borrow_mut() = Some(processor));
    MEDIA_STREAM.with(|m| *m.borrow_mut() = Some(stream));

    Ok(())
}

pub fn stop_recording() {
    // Disconnect and close audio context
    SCRIPT_PROCESSOR.with(|p| {
        if let Some(proc) = p.borrow_mut().take() {
            proc.disconnect().ok();
        }
    });

    AUDIO_CTX.with(|c| {
        if let Some(ctx) = c.borrow_mut().take() {
            let _ = ctx.close();
        }
    });

    // Stop all media tracks
    MEDIA_STREAM.with(|m| {
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
    // Best-effort update via leptos context
    // In the browser main thread, we can access the reactive system
    if let Some(state) = leptos::prelude::use_context::<AppState>() {
        state.audio_level.set(level);
    }
}
