use std::io::Cursor;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

pub struct SoundEngine {
    _stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
    akm_sound: Vec<u8>,
    m4a1_sound: Vec<u8>,
    colt_sound: Vec<u8>,
    reload_mag_out: Vec<u8>,
    reload_mag_in: Vec<u8>,
    reload_bolt: Vec<u8>,
    dry_fire: Vec<u8>,
    hitmarker: Vec<u8>,
    explosion: Vec<u8>,
    footstep: Vec<u8>,
    hurt: Vec<u8>,
}

fn create_wav_header(sample_rate: u32, num_samples: u32) -> Vec<u8> {
    let mut header = Vec::with_capacity(44);
    let byte_rate = sample_rate * 2; // 16-bit mono = 2 bytes per sample
    let data_chunk_size = num_samples * 2;
    let total_chunk_size = 36 + data_chunk_size;

    header.extend_from_slice(b"RIFF");
    header.extend_from_slice(&(total_chunk_size as u32).to_le_bytes());
    header.extend_from_slice(b"WAVE");
    header.extend_from_slice(b"fmt ");
    header.extend_from_slice(&(16u32).to_le_bytes()); // Subchunk1Size (16 for PCM)
    header.extend_from_slice(&(1u16).to_le_bytes());  // AudioFormat (1 = PCM)
    header.extend_from_slice(&(1u16).to_le_bytes());  // NumChannels (1 = Mono)
    header.extend_from_slice(&sample_rate.to_le_bytes());
    header.extend_from_slice(&byte_rate.to_le_bytes());
    header.extend_from_slice(&(2u16).to_le_bytes());  // BlockAlign
    header.extend_from_slice(&(16u16).to_le_bytes()); // BitsPerSample
    header.extend_from_slice(b"data");
    header.extend_from_slice(&data_chunk_size.to_le_bytes());
    header
}

fn synthesize_gunshot(duration_sec: f32, bass_freq: f32, punch: f32) -> Vec<u8> {
    let sample_rate = 44100;
    let num_samples = (sample_rate as f32 * duration_sec) as usize;
    let mut wav = create_wav_header(sample_rate, num_samples as u32);

    let mut state: u32 = 123456789;
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        // Pseudo random white noise
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((state >> 16) as f32 / 32768.0) - 1.0;

        let env_attack = (t / 0.003).min(1.0);
        let env_decay = (-t * 18.0).exp();
        let body_env = (-t * 6.0).exp();

        let bass = (t * bass_freq * std::f32::consts::TAU * (1.0 - t * 1.5).max(0.2)).sin();
        let crack = noise * env_decay * punch;
        let boom = bass * body_env * 0.8;

        let sample_f = ((crack + boom) * env_attack).clamp(-1.0, 1.0);
        let sample_i16 = (sample_f * 32000.0) as i16;
        wav.extend_from_slice(&sample_i16.to_le_bytes());
    }
    wav
}

fn synthesize_tone(freq: f32, duration: f32, decay_rate: f32) -> Vec<u8> {
    let sample_rate = 44100;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut wav = create_wav_header(sample_rate, num_samples as u32);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let env = (-t * decay_rate).exp();
        let s = (t * freq * std::f32::consts::TAU).sin() * env * 0.6;
        let sample_i16 = (s.clamp(-1.0, 1.0) * 32000.0) as i16;
        wav.extend_from_slice(&sample_i16.to_le_bytes());
    }
    wav
}

fn synthesize_explosion() -> Vec<u8> {
    let sample_rate = 44100;
    let duration = 1.4;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut wav = create_wav_header(sample_rate, num_samples as u32);

    let mut state: u32 = 987654321;
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((state >> 16) as f32 / 32768.0) - 1.0;
        let low_rumble = (t * 55.0 * std::f32::consts::TAU).sin() * (-t * 2.5).exp();
        let noise_env = (-t * 4.0).exp();

        let s = (noise * noise_env * 0.8 + low_rumble * 0.9).clamp(-1.0, 1.0);
        let sample_i16 = (s * 32000.0) as i16;
        wav.extend_from_slice(&sample_i16.to_le_bytes());
    }
    wav
}

fn synthesize_click(freq: f32, duration: f32) -> Vec<u8> {
    synthesize_tone(freq, duration, 45.0)
}

impl SoundEngine {
    pub fn new() -> Self {
        let (stream, stream_handle) = match OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(_) => (None, None),
        };

        Self {
            _stream: stream,
            stream_handle,
            akm_sound: synthesize_gunshot(0.45, 95.0, 1.3),
            m4a1_sound: synthesize_gunshot(0.35, 130.0, 1.1),
            colt_sound: synthesize_gunshot(0.38, 110.0, 1.4),
            reload_mag_out: synthesize_click(340.0, 0.12),
            reload_mag_in: synthesize_click(520.0, 0.15),
            reload_bolt: synthesize_click(680.0, 0.20),
            dry_fire: synthesize_click(1200.0, 0.05),
            hitmarker: synthesize_tone(1800.0, 0.09, 30.0),
            explosion: synthesize_explosion(),
            footstep: synthesize_tone(90.0, 0.08, 35.0),
            hurt: synthesize_tone(140.0, 0.25, 12.0),
        }
    }

    fn play_raw(&self, data: &[u8], volume: f32) {
        if let Some(ref handle) = self.stream_handle {
            let cursor = Cursor::new(data.to_vec());
            if let Ok(source) = Decoder::new(cursor) {
                if let Ok(sink) = Sink::try_new(handle) {
                    sink.set_volume(volume);
                    sink.append(source);
                    sink.detach();
                }
            }
        }
    }

    pub fn play_akm(&self) {
        self.play_raw(&self.akm_sound, 0.9);
    }

    pub fn play_m4a1(&self) {
        self.play_raw(&self.m4a1_sound, 0.85);
    }

    pub fn play_colt(&self) {
        self.play_raw(&self.colt_sound, 1.0);
    }

    pub fn play_mag_out(&self) {
        self.play_raw(&self.reload_mag_out, 0.7);
    }

    pub fn play_mag_in(&self) {
        self.play_raw(&self.reload_mag_in, 0.75);
    }

    pub fn play_bolt_rack(&self) {
        self.play_raw(&self.reload_bolt, 0.8);
    }

    pub fn play_dry_fire(&self) {
        self.play_raw(&self.dry_fire, 0.5);
    }

    pub fn play_hitmarker(&self) {
        self.play_raw(&self.hitmarker, 0.6);
    }

    pub fn play_explosion(&self) {
        self.play_raw(&self.explosion, 1.0);
    }

    pub fn play_footstep(&self) {
        self.play_raw(&self.footstep, 0.35);
    }

    pub fn play_hurt(&self) {
        self.play_raw(&self.hurt, 0.8);
    }
}
