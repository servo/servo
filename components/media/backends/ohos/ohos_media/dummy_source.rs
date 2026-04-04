use crate::ohos_media::source_builder::MediaSourceBuilder;

pub struct MediaSourceWrapper {}

impl MediaSourceWrapper {
    pub fn new() -> Self {
        Self {}
    }
}

impl MediaSourceWrapper {
    pub fn builder() -> MediaSourceBuilder {
        MediaSourceBuilder {
            enough_data: None,
            seek_data: None,
        }
    }
    pub fn set_input_size(&self, _size: usize) {
        // No-op for dummy source
    }

    pub fn end_of_stream(&self) {
        // No-op for dummy source
    }

    pub fn push_data(&self, _data: Vec<u8>) {
        // No-op for dummy source
    }

    pub fn set_data_src(&mut self, _av_player: *mut ohos_media_sys::avplayer_base::OH_AVPlayer) {
        // No-op for dummy source.
    }
}
