/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

extern crate ipc_channel;
extern crate servo_media;
extern crate servo_media_audio;
extern crate servo_media_player;
extern crate servo_media_streams;
extern crate servo_media_traits;
extern crate servo_media_webrtc;

use std::any::Any;
use std::ops::Range;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};

use ipc_channel::ipc::IpcSender;
use servo_media::{Backend, BackendInit, MediaInstanceError, SupportsMediaType};
use servo_media_audio::block::{Block, Chunk};
use servo_media_audio::context::{AudioContext, AudioContextOptions};
use servo_media_audio::decoder::{AudioDecoder, AudioDecoderCallbacks, AudioDecoderOptions};
use servo_media_audio::render_thread::AudioRenderThreadMsg;
use servo_media_audio::sink::{AudioSink, AudioSinkError};
use servo_media_audio::{AudioBackend, AudioStreamReader};
use servo_media_player::context::PlayerGLContext;
use servo_media_player::{Player, PlayerError, PlayerEvent, StreamType, audio, video};
use servo_media_streams::capture::MediaTrackConstraintSet;
use servo_media_streams::device_monitor::{MediaDeviceInfo, MediaDeviceMonitor};
use servo_media_streams::registry::{MediaStreamId, register_stream, unregister_stream};
use servo_media_streams::{MediaOutput, MediaSocket, MediaStream, MediaStreamType};
use servo_media_traits::{ClientContextId, MediaInstance};
use servo_media_webrtc::{
    BundlePolicy, DataChannelId, DataChannelInit, DataChannelMessage, IceCandidate,
    SessionDescription, WebRtcBackend, WebRtcController, WebRtcControllerBackend,
    WebRtcDataChannelResult, WebRtcResult, WebRtcSignaller, thread,
};

pub struct DummyBackend;

impl BackendInit for DummyBackend {
    fn init() -> Box<dyn Backend> {
        Box::new(DummyBackend)
    }
}

impl Backend for DummyBackend {
    fn create_audiostream(&self) -> MediaStreamId {
        register_stream(Arc::new(Mutex::new(DummyMediaStream {
            id: MediaStreamId::new(),
        })))
    }

    fn create_videostream(&self) -> MediaStreamId {
        register_stream(Arc::new(Mutex::new(DummyMediaStream {
            id: MediaStreamId::new(),
        })))
    }

    fn create_stream_output(&self) -> Box<dyn MediaOutput> {
        Box::new(DummyMediaOutput)
    }

    fn create_audioinput_stream(&self, _: MediaTrackConstraintSet) -> Option<MediaStreamId> {
        Some(register_stream(Arc::new(Mutex::new(DummyMediaStream {
            id: MediaStreamId::new(),
        }))))
    }

    fn create_stream_and_socket(
        &self,
        _: MediaStreamType,
    ) -> (Box<dyn MediaSocket>, MediaStreamId) {
        let id = register_stream(Arc::new(Mutex::new(DummyMediaStream {
            id: MediaStreamId::new(),
        })));
        (Box::new(DummySocket), id)
    }

    fn create_videoinput_stream(&self, _: MediaTrackConstraintSet) -> Option<MediaStreamId> {
        Some(register_stream(Arc::new(Mutex::new(DummyMediaStream {
            id: MediaStreamId::new(),
        }))))
    }

    fn create_player(
        &self,
        _id: &ClientContextId,
        _: StreamType,
        _: IpcSender<PlayerEvent>,
        _: Option<Arc<Mutex<dyn video::VideoFrameRenderer>>>,
        _: Option<Arc<Mutex<dyn audio::AudioRenderer>>>,
        _: Box<dyn PlayerGLContext>,
    ) -> Arc<Mutex<dyn Player>> {
        Arc::new(Mutex::new(DummyPlayer))
    }

    fn create_audio_context(
        &self,
        _id: &ClientContextId,
        options: AudioContextOptions,
    ) -> Result<Arc<Mutex<AudioContext>>, AudioSinkError> {
        let (sender, _) = mpsc::channel();
        let sender = Arc::new(Mutex::new(sender));
        Ok(Arc::new(Mutex::new(AudioContext::new::<Self>(
            0,
            &ClientContextId::build(1, 1),
            sender,
            options,
        )?)))
    }

    fn create_webrtc(&self, signaller: Box<dyn WebRtcSignaller>) -> WebRtcController {
        WebRtcController::new::<Self>(signaller)
    }

    fn can_play_type(&self, _media_type: &str) -> SupportsMediaType {
        SupportsMediaType::No
    }

    fn get_device_monitor(&self) -> Box<dyn MediaDeviceMonitor> {
        Box::new(DummyMediaDeviceMonitor {})
    }
}

impl AudioBackend for DummyBackend {
    type Sink = DummyAudioSink;
    fn make_decoder() -> Box<dyn AudioDecoder> {
        Box::new(DummyAudioDecoder)
    }

    fn make_sink() -> Result<Self::Sink, AudioSinkError> {
        Ok(DummyAudioSink)
    }
    fn make_streamreader(
        _id: MediaStreamId,
        _sample_rate: f32,
    ) -> Result<Box<dyn AudioStreamReader + Send>, AudioSinkError> {
        Ok(Box::new(DummyStreamReader))
    }
}

pub struct DummyPlayer;

pub struct DummyStreamReader;

impl AudioStreamReader for DummyStreamReader {
    fn pull(&self) -> Block {
        Default::default()
    }
    fn start(&self) {}
    fn stop(&self) {}
}

impl Player for DummyPlayer {
    fn play(&self) -> Result<(), PlayerError> {
        Ok(())
    }
    fn pause(&self) -> Result<(), PlayerError> {
        Ok(())
    }

    fn paused(&self) -> bool {
        true
    }
    fn can_resume(&self) -> bool {
        true
    }

    fn stop(&self) -> Result<(), PlayerError> {
        Ok(())
    }
    fn seek(&self, _: f64) -> Result<(), PlayerError> {
        Ok(())
    }

    fn set_mute(&self, _: bool) -> Result<(), PlayerError> {
        Ok(())
    }

    fn muted(&self) -> bool {
        false
    }

    fn set_volume(&self, _: f64) -> Result<(), PlayerError> {
        Ok(())
    }

    fn volume(&self) -> f64 {
        1.0
    }

    fn set_input_size(&self, _: u64) -> Result<(), PlayerError> {
        Ok(())
    }

    fn set_playback_rate(&self, _: f64) -> Result<(), PlayerError> {
        Ok(())
    }

    fn playback_rate(&self) -> f64 {
        1.0
    }

    fn push_data(&self, _: Vec<u8>) -> Result<(), PlayerError> {
        Ok(())
    }
    fn end_of_stream(&self) -> Result<(), PlayerError> {
        Ok(())
    }

    fn buffered(&self) -> Vec<Range<f64>> {
        vec![]
    }

    fn seekable(&self) -> Vec<Range<f64>> {
        vec![]
    }

    fn set_stream(&self, _: &MediaStreamId, _: bool) -> Result<(), PlayerError> {
        Ok(())
    }

    fn render_use_gl(&self) -> bool {
        false
    }
    fn set_audio_track(&self, _: i32, _: bool) -> Result<(), PlayerError> {
        Ok(())
    }
    fn set_video_track(&self, _: i32, _: bool) -> Result<(), PlayerError> {
        Ok(())
    }
}

impl WebRtcBackend for DummyBackend {
    type Controller = DummyWebRtcController;
    fn construct_webrtc_controller(
        _: Box<dyn WebRtcSignaller>,
        _: WebRtcController,
    ) -> Self::Controller {
        DummyWebRtcController
    }
}

pub struct DummyAudioDecoder;

impl AudioDecoder for DummyAudioDecoder {
    fn decode(&self, _: Vec<u8>, _: AudioDecoderCallbacks, _: Option<AudioDecoderOptions>) {}
}

pub struct DummySocket;

impl MediaSocket for DummySocket {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct DummyMediaStream {
    id: MediaStreamId,
}

impl MediaStream for DummyMediaStream {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
    fn set_id(&mut self, _: MediaStreamId) {}

    fn ty(&self) -> MediaStreamType {
        MediaStreamType::Audio
    }
}

impl Drop for DummyMediaStream {
    fn drop(&mut self) {
        unregister_stream(&self.id);
    }
}

pub struct DummyAudioSink;

impl AudioSink for DummyAudioSink {
    fn init(&self, _: f32, _: Sender<AudioRenderThreadMsg>) -> Result<(), AudioSinkError> {
        Ok(())
    }
    fn init_stream(&self, _: u8, _: f32, _: Box<dyn MediaSocket>) -> Result<(), AudioSinkError> {
        Ok(())
    }
    fn play(&self) -> Result<(), AudioSinkError> {
        Ok(())
    }
    fn stop(&self) -> Result<(), AudioSinkError> {
        Ok(())
    }
    fn has_enough_data(&self) -> bool {
        true
    }
    fn push_data(&self, _: Chunk) -> Result<(), AudioSinkError> {
        Ok(())
    }
    fn set_eos_callback(&self, _: Box<dyn Fn(Box<dyn AsRef<[f32]>>) + Send + Sync + 'static>) {}
}

pub struct DummyMediaOutput;
impl MediaOutput for DummyMediaOutput {
    fn add_stream(&mut self, _stream: &MediaStreamId) {}
}

pub struct DummyWebRtcController;

impl WebRtcControllerBackend for DummyWebRtcController {
    fn configure(&mut self, _: &str, _: BundlePolicy) -> WebRtcResult {
        Ok(())
    }
    fn set_remote_description(
        &mut self,
        _: SessionDescription,
        _: Box<dyn FnOnce() + Send + 'static>,
    ) -> WebRtcResult {
        Ok(())
    }
    fn set_local_description(
        &mut self,
        _: SessionDescription,
        _: Box<dyn FnOnce() + Send + 'static>,
    ) -> WebRtcResult {
        Ok(())
    }
    fn add_ice_candidate(&mut self, _: IceCandidate) -> WebRtcResult {
        Ok(())
    }
    fn create_offer(
        &mut self,
        _: Box<dyn FnOnce(SessionDescription) + Send + 'static>,
    ) -> WebRtcResult {
        Ok(())
    }
    fn create_answer(
        &mut self,
        _: Box<dyn FnOnce(SessionDescription) + Send + 'static>,
    ) -> WebRtcResult {
        Ok(())
    }
    fn add_stream(&mut self, _: &MediaStreamId) -> WebRtcResult {
        Ok(())
    }
    fn create_data_channel(&mut self, _: &DataChannelInit) -> WebRtcDataChannelResult {
        Ok(0)
    }
    fn close_data_channel(&mut self, _: &DataChannelId) -> WebRtcResult {
        Ok(())
    }
    fn send_data_channel_message(
        &mut self,
        _: &DataChannelId,
        _: &DataChannelMessage,
    ) -> WebRtcResult {
        Ok(())
    }
    fn internal_event(&mut self, _: thread::InternalEvent) -> WebRtcResult {
        Ok(())
    }
    fn quit(&mut self) {}
}

impl MediaInstance for DummyPlayer {
    fn get_id(&self) -> usize {
        0
    }

    fn mute(&self, _val: bool) -> Result<(), MediaInstanceError> {
        Ok(())
    }

    fn suspend(&self) -> Result<(), MediaInstanceError> {
        Ok(())
    }

    fn resume(&self) -> Result<(), MediaInstanceError> {
        Ok(())
    }
}

struct DummyMediaDeviceMonitor;

impl MediaDeviceMonitor for DummyMediaDeviceMonitor {
    fn enumerate_devices(&self) -> Option<Vec<MediaDeviceInfo>> {
        Some(vec![])
    }
}
