#[derive(Clone, Copy, Debug)]
pub enum MediaDeviceKind {
    AudioInput,
    AudioOutput,
    VideoInput,
}

#[derive(Clone, Debug)]
pub struct MediaDeviceInfo {
    pub device_id: String,
    pub kind: MediaDeviceKind,
    pub label: String,
}

pub trait MediaDeviceMonitor {
    fn enumerate_devices(&self) -> Result<Vec<MediaDeviceInfo>, ()>;
}
