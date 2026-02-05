pub mod capture;
pub mod device_monitor;
pub mod registry;

use std::any::Any;

pub use registry::*;

pub trait MediaStream: Any + Send {
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
    fn set_id(&mut self, id: registry::MediaStreamId);
    fn ty(&self) -> MediaStreamType;
}

/// A MediaSocket is a way for a backend to represent a
/// yet-to-be-connected source side of a MediaStream
pub trait MediaSocket: Any + Send {
    fn as_any(&self) -> &dyn Any;
}

/// This isn't part of the webrtc spec; it's a leaky abstaction while media streams
/// are under development and example consumers need to be able to inspect them.
pub trait MediaOutput: Send {
    fn add_stream(&mut self, stream: &registry::MediaStreamId);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MediaStreamType {
    Video,
    Audio,
}
