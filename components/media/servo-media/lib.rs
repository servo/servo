/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub extern crate servo_media_audio as audio;
pub extern crate servo_media_player as player;
pub extern crate servo_media_streams as streams;
pub extern crate servo_media_traits as traits;
pub extern crate servo_media_webrtc as webrtc;

extern crate once_cell;

use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread;

use audio::context::{AudioContext, AudioContextOptions};
use audio::sink::AudioSinkError;
use once_cell::sync::OnceCell;
use player::audio::AudioRenderer;
use player::context::PlayerGLContext;
use player::ipc_channel::ipc::IpcSender;
use player::video::VideoFrameRenderer;
use player::{Player, PlayerEvent, StreamType};
use streams::capture::MediaTrackConstraintSet;
use streams::device_monitor::MediaDeviceMonitor;
use streams::registry::MediaStreamId;
use streams::{MediaOutput, MediaSocket, MediaStreamType};
pub use traits::*;
use webrtc::{WebRtcController, WebRtcSignaller};

pub struct ServoMedia(Box<dyn Backend>);

static INSTANCE: OnceCell<Arc<ServoMedia>> = OnceCell::new();

pub trait BackendInit {
    fn init() -> Box<dyn Backend>;
}

pub trait BackendDeInit {
    fn deinit(&self) {}
}

pub trait Backend: Send + Sync {
    fn create_player(
        &self,
        id: &ClientContextId,
        stream_type: StreamType,
        sender: IpcSender<PlayerEvent>,
        video_renderer: Option<Arc<Mutex<dyn VideoFrameRenderer>>>,
        audio_renderer: Option<Arc<Mutex<dyn AudioRenderer>>>,
        gl_context: Box<dyn PlayerGLContext>,
    ) -> Arc<Mutex<dyn Player>>;
    fn create_audiostream(&self) -> MediaStreamId;
    fn create_videostream(&self) -> MediaStreamId;
    fn create_stream_output(&self) -> Box<dyn MediaOutput>;
    fn create_stream_and_socket(
        &self,
        ty: MediaStreamType,
    ) -> (Box<dyn MediaSocket>, MediaStreamId);
    fn create_audioinput_stream(&self, set: MediaTrackConstraintSet) -> Option<MediaStreamId>;
    fn create_videoinput_stream(&self, set: MediaTrackConstraintSet) -> Option<MediaStreamId>;
    fn create_audio_context(
        &self,
        id: &ClientContextId,
        options: AudioContextOptions,
    ) -> Result<Arc<Mutex<AudioContext>>, AudioSinkError>;
    fn create_webrtc(&self, signaller: Box<dyn WebRtcSignaller>) -> WebRtcController;
    fn can_play_type(&self, media_type: &str) -> SupportsMediaType;
    fn set_capture_mocking(&self, _mock: bool) {}
    /// Allow muting/unmuting the media instances associated with the given client context identifier.
    /// Backend implementations are responsible for keeping a match between client contexts
    /// and the media instances created for these contexts.
    /// The client context identifier is currently an abstraction of Servo's PipelineId.
    fn mute(&self, _id: &ClientContextId, _val: bool) {}
    /// Allow suspending the activity of all media instances associated with the given client
    /// context identifier.
    /// Note that suspending does not involve releasing any resources, so media playback can
    /// be restarted.
    /// Backend implementations are responsible for keeping a match between client contexts
    /// and the media instances created for these contexts.
    /// The client context identifier is currently an abstraction of Servo's PipelineId.
    fn suspend(&self, _id: &ClientContextId) {}
    /// Allow resuming the activity of all the media instances associated with the given client
    /// context identifier.
    /// Backend implementations are responsible for keeping a match between client contexts
    /// and the media instances created for these contexts.
    /// The client context identifier is currently an abstraction of Servo's PipelineId.
    fn resume(&self, _id: &ClientContextId) {}

    fn get_device_monitor(&self) -> Box<dyn MediaDeviceMonitor>;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SupportsMediaType {
    Maybe,
    No,
    Probably,
}

impl ServoMedia {
    pub fn init<B: BackendInit>() {
        thread::spawn(|| INSTANCE.get_or_init(|| Arc::new(ServoMedia(B::init()))));
    }

    pub fn init_with_backend<F>(backend_factory: F)
    where
        F: Fn() -> Box<dyn Backend> + Send + 'static,
    {
        thread::spawn(move || INSTANCE.get_or_init(|| Arc::new(ServoMedia(backend_factory()))));
    }

    pub fn get() -> Arc<ServoMedia> {
        INSTANCE.wait().clone()
    }
}

impl Deref for ServoMedia {
    type Target = dyn Backend + 'static;
    fn deref(&self) -> &(dyn Backend + 'static) {
        &*self.0
    }
}
