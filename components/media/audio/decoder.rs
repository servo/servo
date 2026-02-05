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

pub struct AudioDecoderCallbacks {
    pub eos: Mutex<Option<Box<dyn FnOnce() + Send + 'static>>>,
    pub error: Mutex<Option<Box<dyn FnOnce(AudioDecoderError) + Send + 'static>>>,
    pub progress: Option<Box<dyn Fn(Box<dyn AsRef<[f32]>>, u32) + Send + Sync + 'static>>,
    pub ready: Mutex<Option<Box<dyn FnOnce(u32) + Send + 'static>>>,
}

impl AudioDecoderCallbacks {
    pub fn new() -> AudioDecoderCallbacksBuilder {
        AudioDecoderCallbacksBuilder {
            eos: None,
            error: None,
            progress: None,
            ready: None,
        }
    }

    pub fn eos(&self) {
        let eos = self.eos.lock().unwrap().take();
        match eos {
            None => return,
            Some(callback) => callback(),
        };
    }

    pub fn error(&self, error: AudioDecoderError) {
        let callback = self.error.lock().unwrap().take();
        match callback {
            None => return,
            Some(callback) => callback(error),
        };
    }

    pub fn progress(&self, buffer: Box<dyn AsRef<[f32]>>, channel: u32) {
        match self.progress {
            None => return,
            Some(ref callback) => callback(buffer, channel),
        };
    }

    pub fn ready(&self, channels: u32) {
        let ready = self.ready.lock().unwrap().take();
        match ready {
            None => return,
            Some(callback) => callback(channels),
        };
    }
}

pub struct AudioDecoderCallbacksBuilder {
    eos: Option<Box<dyn FnOnce() + Send + 'static>>,
    error: Option<Box<dyn FnOnce(AudioDecoderError) + Send + 'static>>,
    progress: Option<Box<dyn Fn(Box<dyn AsRef<[f32]>>, u32) + Send + Sync + 'static>>,
    ready: Option<Box<dyn FnOnce(u32) + Send + 'static>>,
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
