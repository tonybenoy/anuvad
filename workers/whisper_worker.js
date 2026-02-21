import init, { WhisperWorker } from './pkg_whisper/anuvad_whisper.js';

let worker = null;

async function initWorker() {
    await init();
    worker = new WhisperWorker();
    console.log('[WhisperWorker] Initialized');
}

self.onmessage = async function(event) {
    const msg = event.data;

    try {
        switch (msg.type) {
            case 'LoadModel': {
                if (!worker) await initWorker();
                worker.load_model(
                    msg.model_bytes,
                    msg.tokenizer_json,
                    msg.config_json,
                    msg.mel_bytes
                );
                self.postMessage({ type: 'ModelLoaded' });
                break;
            }

            case 'Transcribe': {
                if (!worker) {
                    self.postMessage({ type: 'Error', message: 'Worker not initialized' });
                    return;
                }
                worker.push_audio(new Float32Array(msg.audio));
                const result = worker.transcribe();
                if (result) {
                    self.postMessage({
                        type: 'TranscriptionResult',
                        text: result.text,
                        language: result.language || null
                    });
                }
                break;
            }

            default:
                console.warn('[WhisperWorker] Unknown message type:', msg.type);
        }
    } catch (e) {
        self.postMessage({ type: 'Error', message: String(e) });
    }
};

// Auto-initialize
initWorker().catch(e => {
    self.postMessage({ type: 'Error', message: 'Init failed: ' + String(e) });
});
