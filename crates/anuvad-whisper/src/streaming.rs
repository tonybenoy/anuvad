const SAMPLE_RATE: usize = 16000;
const CHUNK_SECONDS: usize = 30;
const INFERENCE_INTERVAL_SECONDS: usize = 3;
const MAX_SAMPLES: usize = SAMPLE_RATE * CHUNK_SECONDS;
const INFERENCE_THRESHOLD: usize = SAMPLE_RATE * INFERENCE_INTERVAL_SECONDS;

pub struct StreamingBuffer {
    buffer: Vec<f32>,
    last_inference_pos: usize,
}

impl StreamingBuffer {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(MAX_SAMPLES),
            last_inference_pos: 0,
        }
    }

    pub fn push(&mut self, pcm: &[f32]) {
        self.buffer.extend_from_slice(pcm);

        // Keep rolling window of 30 seconds
        if self.buffer.len() > MAX_SAMPLES {
            let excess = self.buffer.len() - MAX_SAMPLES;
            self.buffer.drain(..excess);
            self.last_inference_pos = self.last_inference_pos.saturating_sub(excess);
        }
    }

    pub fn should_transcribe(&self) -> bool {
        self.buffer.len() - self.last_inference_pos >= INFERENCE_THRESHOLD
    }

    pub fn get_chunk(&mut self) -> Vec<f32> {
        if !self.should_transcribe() && self.buffer.len() < INFERENCE_THRESHOLD {
            return Vec::new();
        }
        self.last_inference_pos = self.buffer.len();
        self.buffer.clone()
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.last_inference_pos = 0;
    }

    pub fn duration_seconds(&self) -> f64 {
        self.buffer.len() as f64 / SAMPLE_RATE as f64
    }
}
