use rodio::{OutputStream, OutputStreamHandle, Sink, buffer::SamplesBuffer};
use std::path::Path;
use std::{cell::RefCell, thread_local};

const SAMPLE_RATE: u32 = 22050;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuSfxKind {
    TopMenuMove,
    TopMenuConfirm,
    TopMenuCancel,
    ToolMenuClick,
}

pub struct MenuSfxEngine {
    pub enabled: bool,
    pub master_gain: f32,
}

impl Default for MenuSfxEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuSfxEngine {
    pub fn new() -> Self {
        Self {
            enabled: true,
            master_gain: 0.45,
        }
    }

    pub fn play(&self, kind: MenuSfxKind) {
        if !self.enabled {
            return;
        }
        let samples = synth_8bit(kind);
        if samples.is_empty() {
            return;
        }
        AUDIO_DEVICE.with(|device| {
            let mut device = device.borrow_mut();
            if device.is_none() {
                *device = OutputStream::try_default().ok();
            }

            if let Some((_, handle)) = device.as_ref()
                && let Ok(sink) = Sink::try_new(handle)
            {
                let src = SamplesBuffer::new(1, SAMPLE_RATE, samples);
                sink.set_volume(self.master_gain.clamp(0.0, 1.0));
                sink.append(src);
                sink.detach();
            }
        });
    }

    pub fn export_default_bank<P: AsRef<Path>>(&self, base_dir: P) -> Result<Vec<String>, String> {
        let base_dir = base_dir.as_ref();
        std::fs::create_dir_all(base_dir).map_err(|e| e.to_string())?;

        let presets = [
            ("menu_move_8bit.wav", MenuSfxKind::TopMenuMove),
            ("menu_confirm_8bit.wav", MenuSfxKind::TopMenuConfirm),
            ("menu_cancel_8bit.wav", MenuSfxKind::TopMenuCancel),
            ("tool_click_8bit.wav", MenuSfxKind::ToolMenuClick),
        ];

        let mut exported = Vec::with_capacity(presets.len());
        for (file_name, kind) in presets {
            let path = base_dir.join(file_name);
            write_wav_mono_i16(&path, SAMPLE_RATE, &synth_8bit(kind)).map_err(|e| e.to_string())?;
            exported.push(path.to_string_lossy().to_string());
        }
        Ok(exported)
    }
}

thread_local! {
    static AUDIO_DEVICE: RefCell<Option<(OutputStream, OutputStreamHandle)>> = const { RefCell::new(None) };
}

fn synth_8bit(kind: MenuSfxKind) -> Vec<f32> {
    match kind {
        MenuSfxKind::TopMenuMove => synth_square_blip(840.0, 0.055, 0.85, 16),
        MenuSfxKind::TopMenuConfirm => {
            let mut a = synth_square_blip(900.0, 0.045, 0.9, 16);
            let mut b = synth_square_blip(1280.0, 0.05, 0.85, 16);
            a.append(&mut b);
            a
        }
        MenuSfxKind::TopMenuCancel => synth_square_blip(440.0, 0.065, 0.8, 16),
        MenuSfxKind::ToolMenuClick => {
            let tone = synth_square_blip(700.0, 0.04, 0.8, 12);
            let mut noise = synth_noise_burst(0.045, 0.35, 12);
            for (idx, sample) in noise.iter_mut().enumerate() {
                if let Some(t) = tone.get(idx) {
                    *sample = (*sample + *t * 0.6).clamp(-1.0, 1.0);
                }
            }
            noise
        }
    }
}

fn synth_square_blip(freq_hz: f32, duration_s: f32, peak: f32, levels: u8) -> Vec<f32> {
    let len = (duration_s * SAMPLE_RATE as f32) as usize;
    if len == 0 {
        return vec![];
    }
    let mut out = Vec::with_capacity(len);
    let step = freq_hz / SAMPLE_RATE as f32;
    let quant = levels.max(2) as f32;
    for i in 0..len {
        let t = i as f32 / len as f32;
        let env = (1.0 - t).powf(2.0);
        let phase = (i as f32 * step) % 1.0;
        let raw = if phase < 0.5 { 1.0 } else { -1.0 };
        let value = raw * peak * env;
        out.push(bitcrush(value, quant));
    }
    out
}

fn synth_noise_burst(duration_s: f32, peak: f32, levels: u8) -> Vec<f32> {
    let len = (duration_s * SAMPLE_RATE as f32) as usize;
    if len == 0 {
        return vec![];
    }
    let mut out = Vec::with_capacity(len);
    let quant = levels.max(2) as f32;
    let mut seed: u32 = 0xA53C_9E17;
    for i in 0..len {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let n = (((seed >> 8) & 0xFFFF) as f32 / 32767.5) - 1.0;
        let t = i as f32 / len as f32;
        let env = (1.0 - t).powf(2.4);
        out.push(bitcrush(n * peak * env, quant));
    }
    out
}

fn bitcrush(sample: f32, levels: f32) -> f32 {
    let s = sample.clamp(-1.0, 1.0);
    ((s * levels).round() / levels).clamp(-1.0, 1.0)
}

fn write_wav_mono_i16(path: &Path, sample_rate: u32, samples: &[f32]) -> std::io::Result<()> {
    let mut pcm: Vec<i16> = Vec::with_capacity(samples.len());
    for s in samples {
        let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        pcm.push(v);
    }

    let data_chunk_size = (pcm.len() * std::mem::size_of::<i16>()) as u32;
    let riff_chunk_size = 36 + data_chunk_size;
    let byte_rate = sample_rate * 2; // mono i16
    let block_align: u16 = 2; // mono i16

    let mut bytes: Vec<u8> = Vec::with_capacity(44 + data_chunk_size as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&riff_chunk_size.to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes()); // PCM chunk size
    bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM format
    bytes.extend_from_slice(&1u16.to_le_bytes()); // channels
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_chunk_size.to_le_bytes());
    for s in pcm {
        bytes.extend_from_slice(&s.to_le_bytes());
    }
    std::fs::write(path, bytes)
}
