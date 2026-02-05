use std::sync::Mutex;

#[derive(Debug, PartialEq)]
pub enum AudioDecoderError {
    /// Backend specific error.
    Backend(String),
    /// Could not read the audio buffer content.
    BufferReadFailed,
    /// The media trying to be decoded has an invalid format.
    InvalidMediaFormat,
    /// An invalid sample was found while decoding the audio.
    InvalidSample,
    /// Could not move to a different state.
    StateChangeFailed,
}

type AudioDecoderEosCallback = Box<dyn FnOnce() + Send + 'static>;
type AudioDecoderErrorCallback = Box<dyn FnOnce(AudioDecoderError) + Send + 'static>;
type AudioDecoderProgressCallback = Box<dyn Fn(Box<dyn AsRef<[f32]>>, u32) + Send + Sync + 'static>;
type AudioDecoderReadyCallback = Box<dyn FnOnce(u32) + Send + 'static>;

pub struct AudioDecoderCallbacks {
    pub eos: Mutex<Option<AudioDecoderEosCallback>>,
    pub error: Mutex<Option<AudioDecoderErrorCallback>>,
    pub progress: Option<AudioDecoderProgressCallback>,
    pub ready: Mutex<Option<AudioDecoderReadyCallback>>,
}

impl AudioDecoderCallbacks {
    pub fn eos(&self) {
        if let Some(callback) = self.eos.lock().unwrap().take() {
            callback();
        }
    }

    pub fn error(&self, error: AudioDecoderError) {
        if let Some(callback) = self.error.lock().unwrap().take() {
            callback(error);
        }
    }

    pub fn progress(&self, buffer: Box<dyn AsRef<[f32]>>, channel: u32) {
        if let Some(callback) = self.progress.as_ref() {
            callback(buffer, channel);
        }
    }

    pub fn ready(&self, channels: u32) {
        if let Some(callback) = self.ready.lock().unwrap().take() {
            callback(channels);
        }
    }
}

#[derive(Default)]
pub struct AudioDecoderCallbacksBuilder {
    eos: Option<AudioDecoderEosCallback>,
    error: Option<AudioDecoderErrorCallback>,
    progress: Option<AudioDecoderProgressCallback>,
    ready: Option<AudioDecoderReadyCallback>,
}

impl AudioDecoderCallbacksBuilder {
    pub fn eos<F: FnOnce() + Send + 'static>(self, eos: F) -> Self {
        Self {
            eos: Some(Box::new(eos)),
            ..self
        }
    }

    pub fn error<F: FnOnce(AudioDecoderError) + Send + 'static>(self, error: F) -> Self {
        Self {
            error: Some(Box::new(error)),
            ..self
        }
    }

    pub fn progress<F: Fn(Box<dyn AsRef<[f32]>>, u32) + Send + Sync + 'static>(
        self,
        progress: F,
    ) -> Self {
        Self {
            progress: Some(Box::new(progress)),
            ..self
        }
    }

    pub fn ready<F: FnOnce(u32) + Send + 'static>(self, ready: F) -> Self {
        Self {
            ready: Some(Box::new(ready)),
            ..self
        }
    }

    pub fn build(self) -> AudioDecoderCallbacks {
        AudioDecoderCallbacks {
            eos: Mutex::new(self.eos),
            error: Mutex::new(self.error),
            progress: self.progress,
            ready: Mutex::new(self.ready),
        }
    }
}

pub struct AudioDecoderOptions {
    pub sample_rate: f32,
}

impl Default for AudioDecoderOptions {
    fn default() -> Self {
        AudioDecoderOptions {
            sample_rate: 44100.,
        }
    }
}

pub trait AudioDecoder {
    fn decode(
        &self,
        data: Vec<u8>,
        callbacks: AudioDecoderCallbacks,
        options: Option<AudioDecoderOptions>,
    );
}
