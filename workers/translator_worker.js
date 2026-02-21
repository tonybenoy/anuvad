import init, { TranslatorWorker } from './pkg_translator/anuvad_translator.js';

let worker = null;

async function initWorker() {
    await init();
    worker = new TranslatorWorker();
    console.log('[TranslatorWorker] Initialized');
}

self.onmessage = async function(event) {
    const msg = event.data;

    try {
        switch (msg.type) {
            case 'LoadTranslatorModel': {
                if (!worker) await initWorker();
                worker.load_model(msg.model_bytes, msg.tokenizer_json);
                self.postMessage({ type: 'TranslatorModelLoaded' });
                break;
            }

            case 'Translate': {
                if (!worker) {
                    self.postMessage({ type: 'Error', message: 'Worker not initialized' });
                    return;
                }

                const tokenCallback = (token) => {
                    self.postMessage({ type: 'TranslationToken', token: token });
                };

                const result = worker.translate(msg.text, msg.target_language, tokenCallback);
                self.postMessage({ type: 'TranslationDone', text: result });
                break;
            }

            default:
                console.warn('[TranslatorWorker] Unknown message type:', msg.type);
        }
    } catch (e) {
        self.postMessage({ type: 'Error', message: String(e) });
    }
};

// Auto-initialize
initWorker().catch(e => {
    self.postMessage({ type: 'Error', message: 'Init failed: ' + String(e) });
});
