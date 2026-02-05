use std::sync::Arc;

#[derive(Clone)]
pub enum VideoFrameData {
    Raw(Arc<Vec<u8>>),
    Texture(u32),
    OESTexture(u32),
}

pub trait Buffer: Send + Sync {
    fn to_vec(&self) -> Option<VideoFrameData>;
}

#[derive(Clone)]
pub struct VideoFrame {
    width: i32,
    height: i32,
    data: VideoFrameData,
    _buffer: Arc<dyn Buffer>,
}

impl VideoFrame {
    pub fn new(width: i32, height: i32, buffer: Arc<dyn Buffer>) -> Option<Self> {
        let data = buffer.to_vec()?;
        Some(VideoFrame {
            width,
            height,
            data,
            _buffer: buffer,
        })
    }

    pub fn get_width(&self) -> i32 {
        self.width
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }

    pub fn get_data(&self) -> Arc<Vec<u8>> {
        match self.data {
            VideoFrameData::Raw(ref data) => data.clone(),
            _ => unreachable!("invalid raw data request for texture frame"),
        }
    }

    pub fn get_texture_id(&self) -> u32 {
        match self.data {
            VideoFrameData::Texture(data) | VideoFrameData::OESTexture(data) => data,
            _ => unreachable!("invalid texture id request for raw data frame"),
        }
    }

    pub fn is_gl_texture(&self) -> bool {
        matches!(
            self.data,
            VideoFrameData::Texture(_) | VideoFrameData::OESTexture(_)
        )
    }

    pub fn is_external_oes(&self) -> bool {
        matches!(self.data, VideoFrameData::OESTexture(_))
    }
}

pub trait VideoFrameRenderer: Send + 'static {
    fn render(&mut self, frame: VideoFrame);
}
