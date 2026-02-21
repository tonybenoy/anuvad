use candle_transformers::models::whisper::Config;

const HOP_LENGTH: usize = 160;
const N_FFT: usize = 400;
const CHUNK_LENGTH: usize = 30; // seconds

pub fn pcm_to_mel(
    config: &Config,
    pcm: &[f32],
    mel_filters: &[f32],
) -> Result<Vec<f32>, String> {
    let sample_rate = 16000;
    let n_mels = config.num_mel_bins;
    let expected_frames = CHUNK_LENGTH * sample_rate / HOP_LENGTH;

    // Pad or truncate to 30 seconds
    let n_samples = CHUNK_LENGTH * sample_rate;
    let mut padded = vec![0.0f32; n_samples];
    let copy_len = pcm.len().min(n_samples);
    padded[..copy_len].copy_from_slice(&pcm[..copy_len]);

    // Compute STFT magnitude
    let n_frames = (n_samples - N_FFT) / HOP_LENGTH + 1;
    let fft_size = N_FFT / 2 + 1;

    let mut magnitudes = vec![0.0f32; fft_size * n_frames];

    // Hann window
    let window: Vec<f32> = (0..N_FFT)
        .map(|i| {
            0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / N_FFT as f32).cos())
        })
        .collect();

    for frame_idx in 0..n_frames {
        let start = frame_idx * HOP_LENGTH;

        // Apply window and compute DFT
        for k in 0..fft_size {
            let mut real = 0.0f32;
            let mut imag = 0.0f32;
            for n in 0..N_FFT {
                let sample = if start + n < padded.len() {
                    padded[start + n] * window[n]
                } else {
                    0.0
                };
                let angle = -2.0 * std::f32::consts::PI * k as f32 * n as f32 / N_FFT as f32;
                real += sample * angle.cos();
                imag += sample * angle.sin();
            }
            magnitudes[k * n_frames + frame_idx] = real * real + imag * imag;
        }
    }

    // Apply mel filterbank
    // mel_filters shape: (n_mels, fft_size)
    let mut mel = vec![0.0f32; n_mels * n_frames];

    for m_idx in 0..n_mels {
        for frame_idx in 0..n_frames {
            let mut sum = 0.0f32;
            for k in 0..fft_size {
                sum += mel_filters[m_idx * fft_size + k] * magnitudes[k * n_frames + frame_idx];
            }
            // Log mel spectrogram
            mel[m_idx * n_frames + frame_idx] = (sum.max(1e-10)).ln();
        }
    }

    // Normalize
    let max_val = mel.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let min_val = (max_val - 8.0).max(mel.iter().cloned().fold(f32::INFINITY, f32::min));
    for v in mel.iter_mut() {
        *v = ((*v).max(min_val) - min_val) / (max_val - min_val) * 2.0 - 1.0;
    }

    Ok(mel)
}
