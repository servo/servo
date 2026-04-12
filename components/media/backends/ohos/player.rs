/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::ops::Range;
use std::sync::{Arc, Mutex, mpsc};
use std::time;

use crossbeam_channel::Sender;
use ipc_channel::ipc::{IpcReceiver, channel};
use log::{debug, error, warn};
use ohos_media_sys::avformat::{
    OH_AVFormat, OH_AVFormat_GetFloatValue, OH_AVFormat_GetIntValue, OH_AVFormat_GetLongValue,
};
use ohos_media_sys::avplayer_base::{
    AVPlayerOnInfoType, AVPlayerState, OH_PLAYER_BUFFERING_TYPE, OH_PLAYER_BUFFERING_VALUE,
    OH_PLAYER_CURRENT_POSITION, OH_PLAYER_DURATION, OH_PLAYER_IS_LIVE_STREAM,
    OH_PLAYER_SEEK_POSITION, OH_PLAYER_STATE, OH_PLAYER_STATE_CHANGE_REASON,
    OH_PLAYER_VIDEO_HEIGHT, OH_PLAYER_VIDEO_WIDTH, OH_PLAYER_VOLUME,
};
use servo_media::{BackendMsg, ClientContextId, MediaInstance, MediaInstanceError};
use servo_media_player::metadata::Metadata;
use servo_media_player::video::{self, Buffer, VideoFrame, VideoFrameData};
use servo_media_player::{PlaybackState, Player, PlayerEvent, SeekLock, SeekLockMsg};
use yuv::yuv_nv12_to_bgra;

use crate::ohos_media::avplayer::OhosPlayer as OhosPlayerInner;
#[cfg(not(sdk_api_21))]
use crate::ohos_media::dummy_source::MediaSourceWrapper;
#[cfg(sdk_api_21)]
use crate::ohos_media::source::MediaSourceWrapper;

// Height of decoded video frame from AVPlayer is padded to multiples of this value by the codec.
// https://developer.huawei.com/consumer/cn/doc/harmonyos-guides/video-decoding
const FRAME_HEIGHT_MULTIPLE: i32 = 32;

/// This is used to fill the gap between internal AVPlayer state and Player State exposed to Media Element.
pub struct StateManager {
    pub internal_state: InternalState,
    pub player_state: PlayerState,
}

pub struct InternalState {
    pub state: AVPlayerState,
}

pub struct PlayerState {
    pub paused: bool,
}

impl StateManager {
    pub fn new() -> Self {
        StateManager {
            internal_state: InternalState {
                state: AVPlayerState::AV_IDLE,
            },
            player_state: PlayerState { paused: true },
        }
    }
}

pub struct OhosAvPlayer {
    id: usize,
    context_id: ClientContextId,
    player_inner: Arc<Mutex<OhosPlayerInner>>,
    event_sender: Arc<Mutex<ipc_channel::ipc::IpcSender<servo_media::player::PlayerEvent>>>,
    video_sink: Option<Arc<Mutex<VideoSink>>>,
    backend_chan: Arc<Mutex<mpsc::Sender<BackendMsg>>>,
    last_metadata: Arc<Mutex<Cell<Metadata>>>,
    state_manager: Arc<Mutex<StateManager>>,
}

// Procedure for setting up AVPlayer, state change condition:
// 1. Create AVPlayer
// 2. Setup AVPlayer InfoCallback (this should be the first step, so that we can listen to state change)
// 3. Setup AVPlayer Media source.
// 4. wait for AVplayer into Initialized State, setup VideoSurface.
// 5. AVPlayer Prepare()
// 6. wait for ready for prepare, in the meantime, avplayer will try to read data from media source.
// 7. player ready to play.

impl OhosAvPlayer {
    pub fn new(
        id: usize,
        context_id: ClientContextId,
        sender: ipc_channel::ipc::IpcSender<servo_media::player::PlayerEvent>,
        video_renderer: Option<
            std::sync::Arc<std::sync::Mutex<dyn servo_media::player::video::VideoFrameRenderer>>,
        >,
        backend_chan: Arc<Mutex<mpsc::Sender<BackendMsg>>>,
    ) -> OhosAvPlayer {
        let player_inner = Arc::new(Mutex::new(OhosPlayerInner::new()));
        let event_sender = Arc::new(Mutex::new(sender));
        let video_sink = video_renderer.clone().map(|v| {
            Arc::new(Mutex::new(VideoSink::new(
                v,
                player_inner.clone(),
                event_sender.clone(),
            )))
        });
        OhosAvPlayer {
            id,
            context_id,
            player_inner,
            event_sender,
            video_sink,
            backend_chan,
            last_metadata: Arc::new(Mutex::new(Cell::new(Metadata {
                duration: None,
                width: 0,
                height: 0,
                format: String::new(),
                is_seekable: false,
                is_live: false,
                video_tracks: vec![],
                audio_tracks: vec![],
                title: None,
            }))),
            state_manager: Arc::new(Mutex::new(StateManager::new())),
        }
    }

    pub fn setup_info_event(&mut self) {
        let sender_clone = self.event_sender.clone();
        let player_inner_clone = self.player_inner.clone();
        let state_manager_clone = self.state_manager.clone();
        let video_sink_clone = self.video_sink.as_ref().map(|v| v.clone());
        let metadata_clone = self.last_metadata.clone();

        let event_info_closure =
            move |info_type: AVPlayerOnInfoType, info_body: *mut OH_AVFormat| {
                debug!(
                    "Info Type received!:{:?}, address: {:p}",
                    info_type, info_body
                );

                match info_type {
                    AVPlayerOnInfoType::AV_INFO_TYPE_STATE_CHANGE => {
                        let mut state_change_reason = -1;
                        let mut state = -1;
                        unsafe {
                            OH_AVFormat_GetIntValue(info_body, OH_PLAYER_STATE, &mut state);
                            OH_AVFormat_GetIntValue(
                                info_body,
                                OH_PLAYER_STATE_CHANGE_REASON,
                                &mut state_change_reason,
                            );
                        }
                        let av_player_state = AVPlayerState(state as u32);
                        debug!(
                            "AV Player State Change: {:?}, state change reason: {}",
                            av_player_state, state_change_reason
                        );
                        state_manager_clone.lock().unwrap().internal_state.state = av_player_state;
                        match av_player_state {
                            AVPlayerState::AV_INITIALIZED => {
                                debug!("Setup Video Sink");
                                if let Some(ref video_sink_clone) = video_sink_clone {
                                    video_sink_clone.lock().unwrap().setup(); // TODO: Hide internal state machine
                                }
                            },
                            AVPlayerState::AV_PREPARED => {
                                let _ = sender_clone
                                    .lock()
                                    .unwrap()
                                    .send(PlayerEvent::StateChanged(PlaybackState::Paused));
                            },
                            AVPlayerState::AV_PLAYING => {
                                let sender_clone_guard = sender_clone.lock().unwrap();
                                let _ = sender_clone_guard
                                    .send(PlayerEvent::StateChanged(PlaybackState::Playing));
                            },
                            AVPlayerState::AV_PAUSED => {
                                let _ = sender_clone
                                    .lock()
                                    .unwrap()
                                    .send(PlayerEvent::StateChanged(PlaybackState::Paused));
                            },
                            AVPlayerState::AV_STOPPED => {
                                let _ = sender_clone
                                    .lock()
                                    .unwrap()
                                    .send(PlayerEvent::StateChanged(PlaybackState::Stopped));
                            },
                            AVPlayerState::AV_COMPLETED => {
                                let _ = sender_clone.lock().unwrap().send(PlayerEvent::EndOfStream);
                            },
                            _ => {
                                warn!("Unhandled State: {:?}", av_player_state);
                            },
                        }
                        player_inner_clone
                            .lock()
                            .unwrap()
                            .set_state(av_player_state);
                    },
                    AVPlayerOnInfoType::AV_INFO_TYPE_RESOLUTION_CHANGE => {
                        let mut width = -1;
                        let mut height = -1;
                        unsafe {
                            OH_AVFormat_GetIntValue(info_body, OH_PLAYER_VIDEO_WIDTH, &mut width);
                            OH_AVFormat_GetIntValue(info_body, OH_PLAYER_VIDEO_HEIGHT, &mut height);
                        }
                        // Todo fix the metadata update logic, we should only report metadata once during intialization.
                        let mut last_metadata = metadata_clone.lock().unwrap();
                        last_metadata.get_mut().height = height as u32;
                        last_metadata.get_mut().width = width as u32;
                        let meta_data_clone_clone = last_metadata.get_mut().clone();
                        let _ = sender_clone
                            .lock()
                            .unwrap()
                            .send(PlayerEvent::MetadataUpdated(meta_data_clone_clone));
                        debug!("Resolution get: width: {}, height: {}", width, height);
                    },
                    AVPlayerOnInfoType::AV_INFO_TYPE_IS_LIVE_STREAM => {
                        let mut value = -1;
                        unsafe {
                            OH_AVFormat_GetIntValue(
                                info_body,
                                OH_PLAYER_IS_LIVE_STREAM,
                                &mut value,
                            );
                        }
                        let mut last_metadata = metadata_clone.lock().unwrap();
                        let last_metadata_mut = last_metadata.get_mut();
                        (last_metadata_mut.is_live, last_metadata_mut.is_seekable) = match value {
                            1 => (true, false),
                            _ => (false, true),
                        };
                        debug!("AVPlayer is live stream: {}. which is not supported", value);
                    },
                    AVPlayerOnInfoType::AV_INFO_TYPE_DURATION_UPDATE => {
                        let mut duration: i64 = -1;
                        unsafe {
                            OH_AVFormat_GetLongValue(info_body, OH_PLAYER_DURATION, &mut duration);
                        }
                        let duration = time::Duration::from_millis(duration as u64);
                        metadata_clone.lock().unwrap().get_mut().duration = Some(duration);
                        let mut last_metadata = metadata_clone.lock().unwrap();
                        last_metadata.get_mut().duration = Some(duration);
                        let metadata_clone_clone = last_metadata.get_mut().clone();
                        let _ = sender_clone
                            .lock()
                            .unwrap()
                            .send(PlayerEvent::MetadataUpdated(metadata_clone_clone));
                        debug!("DURATION UPDATE: {:?}", duration);
                    },
                    AVPlayerOnInfoType::AV_INFO_TYPE_BUFFERING_UPDATE => {
                        let mut buffer_type = -1;
                        let mut buffer_value = -1;
                        unsafe {
                            OH_AVFormat_GetIntValue(
                                info_body,
                                OH_PLAYER_BUFFERING_TYPE,
                                &mut buffer_type,
                            );
                            OH_AVFormat_GetIntValue(
                                info_body,
                                OH_PLAYER_BUFFERING_VALUE,
                                &mut buffer_value,
                            );
                        }
                        debug!("Buffering update: {}, value: {}", buffer_type, buffer_value);
                    },
                    AVPlayerOnInfoType::AV_INFO_TYPE_VOLUME_CHANGE => {
                        let mut volume = 0.0;
                        unsafe {
                            OH_AVFormat_GetFloatValue(info_body, OH_PLAYER_VOLUME, &mut volume);
                        }
                        debug!("Player Volume Change: {}", volume);
                    },
                    AVPlayerOnInfoType::AV_INFO_TYPE_POSITION_UPDATE => {
                        let mut position = -1;
                        unsafe {
                            OH_AVFormat_GetIntValue(
                                info_body,
                                OH_PLAYER_CURRENT_POSITION,
                                &mut position,
                            );
                        }
                        let _ = sender_clone
                            .lock()
                            .unwrap()
                            .send(PlayerEvent::PositionChanged(position as f64 / 1000.0));
                    },
                    AVPlayerOnInfoType::AV_INFO_TYPE_SEEKDONE => {
                        let mut position = -1;
                        unsafe {
                            OH_AVFormat_GetIntValue(
                                info_body,
                                OH_PLAYER_SEEK_POSITION,
                                &mut position,
                            );
                        }
                        let _ = sender_clone
                            .lock()
                            .unwrap()
                            .send(PlayerEvent::SeekDone(position as f64 / 1000.0));
                    },
                    _ => {
                        warn!("Unhandled info type: {:?}", info_type);
                    },
                }
            };

        self.player_inner
            .lock()
            .unwrap()
            .connect_info_event_callback(event_info_closure);
    }

    pub fn setup_data_source(&mut self) {
        let sender_clone = self.event_sender.clone();
        let sender_clone_clone = self.event_sender.clone();
        let seek_channel = Arc::new(Mutex::new(SeekChannel::new()));
        let seekdata_send_closure = move |pos: u64| {
            let _ = sender_clone.lock().unwrap().send(PlayerEvent::SeekData(
                pos,
                seek_channel.lock().unwrap().sender(),
            ));
            let (ret, ack_channel) = seek_channel.lock().unwrap().wait();
            let _ = ack_channel.send(());
            debug!("Seek Initiated! :{}", pos);
            let _ = sender_clone.lock().unwrap().send(PlayerEvent::NeedData);
            ret
        };

        let source = MediaSourceWrapper::builder()
            .set_enough_data(move || {
                let _ = sender_clone_clone
                    .lock()
                    .unwrap()
                    .send(PlayerEvent::EnoughData);
            })
            .set_seek_data(seekdata_send_closure)
            .build();

        self.player_inner.lock().unwrap().set_source(source);
        // To kickstart the first need data event.
        let _ = self
            .event_sender
            .lock()
            .unwrap()
            .send(PlayerEvent::NeedData);
    }
}

struct SeekChannel {
    sender: SeekLock,
    recv: IpcReceiver<SeekLockMsg>,
}

impl SeekChannel {
    fn new() -> Self {
        let (sender, recv) = channel::<SeekLockMsg>().expect("Couldn't create IPC channel");
        Self {
            sender: SeekLock {
                lock_channel: sender,
            },
            recv,
        }
    }
    fn sender(&self) -> SeekLock {
        self.sender.clone()
    }
    fn wait(&self) -> SeekLockMsg {
        self.recv.recv().unwrap()
    }
}

impl Drop for OhosAvPlayer {
    fn drop(&mut self) {
        debug!("Ohos Dropping");
        let (sender, _) = std::sync::mpsc::channel::<()>();
        let _ = self
            .backend_chan
            .lock()
            .unwrap()
            .send(BackendMsg::Shutdown {
                context: self.context_id,
                id: self.id,
                tx_ack: sender,
            });
    }
}

impl MediaInstance for OhosAvPlayer {
    fn get_id(&self) -> usize {
        self.id
    }

    fn mute(&self, val: bool) -> Result<(), MediaInstanceError> {
        self.set_mute(val).map_err(|_| MediaInstanceError)
    }

    fn suspend(&self) -> Result<(), MediaInstanceError> {
        self.pause().map_err(|_| MediaInstanceError)
    }

    fn resume(&self) -> Result<(), MediaInstanceError> {
        self.play().map_err(|_| MediaInstanceError)
    }
}

// TODO: Connect Error.
impl Player for OhosAvPlayer {
    fn play(&self) -> Result<(), servo_media::player::PlayerError> {
        debug!("Start playing ohos player");
        self.state_manager.lock().unwrap().player_state.paused = false;
        self.player_inner.lock().unwrap().play();
        Ok(())
    }

    fn pause(&self) -> Result<(), servo_media::player::PlayerError> {
        self.state_manager.lock().unwrap().player_state.paused = true;
        self.player_inner.lock().unwrap().pause();
        Ok(())
    }

    fn stop(&self) -> Result<(), servo_media::player::PlayerError> {
        self.player_inner.lock().unwrap().stop();
        Ok(())
    }

    fn seek(&self, time: f64) -> Result<(), servo_media::player::PlayerError> {
        log::error!("Seeking to {} seconds", time);
        self.player_inner
            .lock()
            .unwrap()
            .seek((time * 1000.0) as i32);
        let state_manger_lock = self.state_manager.lock().unwrap();
        if !state_manger_lock.player_state.paused &&
            state_manger_lock.internal_state.state == AVPlayerState::AV_COMPLETED
        {
            self.player_inner.lock().unwrap().play();
        }
        Ok(())
    }

    fn seekable(&self) -> Vec<std::ops::Range<f64>> {
        if let Some(duration) = self.last_metadata.lock().unwrap().get_mut().duration {
            return vec![Range {
                start: 0.0,
                end: duration.as_secs_f64(),
            }];
        }
        self.buffered()
    }

    fn set_mute(&self, val: bool) -> Result<(), servo_media::player::PlayerError> {
        self.player_inner.lock().unwrap().set_mute(val);
        Ok(())
    }

    fn set_volume(&self, value: f64) -> Result<(), servo_media::player::PlayerError> {
        self.player_inner.lock().unwrap().set_volume(value);
        Ok(())
    }

    fn set_input_size(&self, size: u64) -> Result<(), servo_media::player::PlayerError> {
        debug!("SetInputSize: {}", size);

        self.player_inner.lock().unwrap().set_input_size(size);
        Ok(())
    }

    fn set_playback_rate(
        &self,
        playback_rate: f64,
    ) -> Result<(), servo_media::player::PlayerError> {
        self.player_inner.lock().unwrap().set_rate(playback_rate);
        Ok(())
    }

    fn push_data(&self, data: Vec<u8>) -> Result<(), servo_media::player::PlayerError> {
        self.player_inner.lock().unwrap().push_data(data);
        Ok(())
    }

    fn end_of_stream(&self) -> Result<(), servo_media::player::PlayerError> {
        debug!("Player: Current Request End of Stream reached!");
        self.player_inner.lock().unwrap().end_of_stream();
        Ok(())
    }

    fn buffered(&self) -> Vec<std::ops::Range<f64>> {
        vec![]
    }

    fn set_stream(
        &self,
        _stream: &servo_media::streams::MediaStreamId,
        _only_stream: bool,
    ) -> Result<(), servo_media::player::PlayerError> {
        Ok(())
    }

    fn render_use_gl(&self) -> bool {
        warn!("Render use gl not supported!");
        false
    }

    fn set_audio_track(
        &self,
        _stream_index: i32,
        _enabled: bool,
    ) -> Result<(), servo_media::player::PlayerError> {
        Ok(())
    }

    fn set_video_track(
        &self,
        _stream_index: i32,
        _enabled: bool,
    ) -> Result<(), servo_media::player::PlayerError> {
        Ok(())
    }

    fn can_resume(&self) -> bool {
        todo!()
    }

    fn paused(&self) -> bool {
        self.state_manager.lock().unwrap().player_state.paused
    }

    fn muted(&self) -> bool {
        self.player_inner.lock().unwrap().muted()
    }

    fn volume(&self) -> f64 {
        self.player_inner.lock().unwrap().volume()
    }

    fn playback_rate(&self) -> f64 {
        self.player_inner.lock().unwrap().playback_rate()
    }
}

/// Used when acquiring the decoded Video Frame,
/// and upload it to Media Frame Renderer in media element.
struct VideoSink {
    video_render:
        std::sync::Arc<std::sync::Mutex<dyn servo_media::player::video::VideoFrameRenderer>>,
    player_inner: Arc<Mutex<OhosPlayerInner>>,
    event_sender: Arc<Mutex<ipc_channel::ipc::IpcSender<servo_media::player::PlayerEvent>>>,
    thread_send_chan: Cell<Option<Sender<RenderMsg>>>,
}

pub enum RenderMsg {
    Terminate,
    FrameAvailable,
}

impl VideoSink {
    pub fn new(
        video_render: std::sync::Arc<
            std::sync::Mutex<dyn servo_media::player::video::VideoFrameRenderer>,
        >,
        player_inner: Arc<Mutex<OhosPlayerInner>>,
        event_sender: Arc<Mutex<ipc_channel::ipc::IpcSender<servo_media::player::PlayerEvent>>>,
    ) -> Self {
        VideoSink {
            video_render,
            player_inner,
            event_sender,
            thread_send_chan: Cell::new(None),
        }
    }
    // For VideoSink, Need to think of better way to retrieve data.
    pub fn setup(&self) {
        let (sender, receiver) = crossbeam_channel::unbounded::<RenderMsg>();
        let sender_clone = sender.clone();
        self.thread_send_chan.set(Some(sender));
        let event_sender_clone = self.event_sender.clone();
        let player_inner_clone = self.player_inner.clone();
        let renderer_clone = self.video_render.clone();

        let frame_available_closure = move || {
            let res = sender_clone.send(RenderMsg::FrameAvailable);
            if res.is_err() {
                debug!("Failed to send frame available: {:?}", res.err());
            }
        };
        self.player_inner
            .lock()
            .unwrap()
            .setup_window_buffer_listener(frame_available_closure);

        std::thread::Builder::new()
            .name("Media Worker Thread".to_owned())
            .spawn(move || {
            loop {
                let Ok(msg) = receiver.recv() else {
                    debug!("error receiving message");
                    break;
                };
                match msg {
                    RenderMsg::Terminate => {
                        break;
                    },
                    RenderMsg::FrameAvailable => {
                        let frame_info = match player_inner_clone.lock().unwrap().acquire_buffer() {
                            Some(frame_info) => frame_info,
                            None => continue,
                        };
                        debug!(
                            "fd: {}, width: {}, height: {}, stride: {}, size: {}, format: {}, virt addr: {:p}",
                            frame_info.fd,
                            frame_info.width,
                            frame_info.height,
                            frame_info.stride,
                            frame_info.size,
                            frame_info.format,
                            frame_info.vir_addr
                        );

                        let coded_height = ((frame_info.height + FRAME_HEIGHT_MULTIPLE - 1) / FRAME_HEIGHT_MULTIPLE) * FRAME_HEIGHT_MULTIPLE;
                        let y_plane_size = (frame_info.stride * coded_height) as usize;
                        let uv_plane_size = (frame_info.stride * coded_height / 2) as usize;
                        let total_needed = y_plane_size + uv_plane_size;
                        if total_needed > frame_info.size as usize || frame_info.vir_addr.is_null() {
                            error!(
                                "Buffer too small or null: needed {} bytes (y={}, uv={}), have {} bytes, vir_addr null={}",
                                total_needed, y_plane_size, uv_plane_size, frame_info.size, frame_info.vir_addr.is_null()
                            );
                            player_inner_clone
                                .lock()
                                .unwrap()
                                .release_buffer(frame_info);
                            continue;
                        }
                        let bi_planar_image = yuv::YuvBiPlanarImage {
                            y_plane: unsafe {
                                std::slice::from_raw_parts(
                                    frame_info.vir_addr as *const u8,
                                    y_plane_size,
                                )
                            },
                            uv_plane: unsafe {
                                std::slice::from_raw_parts(
                                    (frame_info.vir_addr as usize + y_plane_size) as *const u8,
                                    uv_plane_size,
                                )
                            },
                            width: frame_info.width as u32,
                            height: frame_info.height as u32,
                            y_stride: frame_info.stride as u32,
                            uv_stride: frame_info.stride as u32,
                        };
                        let mut bgra = vec![0u8; (frame_info.width * frame_info.height * 4) as usize];

                        // Conversion from yuv to bgra8
                        let Ok(_) = yuv_nv12_to_bgra(
                            &bi_planar_image,
                            &mut bgra,
                            frame_info.width as u32 *4,
                            yuv::YuvRange::Full,
                            yuv::YuvStandardMatrix::Bt709,
                            yuv::YuvConversionMode::Balanced
                        )else{
                            error!("Failed to convert YUV to BGRA");
                            player_inner_clone
                                .lock()
                                .unwrap()
                                .release_buffer(frame_info);
                            continue;
                        };

                        let Some(frame) = VideoFrame::new(
                            frame_info.width,
                            frame_info.height,
                            Arc::new(OhosBuffer::new(bgra)),
                        ) else {
                            error!("Failed to create VideoFrame");
                            player_inner_clone
                                .lock()
                                .unwrap()
                                .release_buffer(frame_info);
                            continue;
                        };
                        renderer_clone.lock().expect(
                            "Failed to acquire video renderer lock"
                        ).render(frame);
                        player_inner_clone
                            .lock()
                            .unwrap()
                            .release_buffer(frame_info);
                        match event_sender_clone
                            .lock()
                            .unwrap()
                            .send(PlayerEvent::VideoFrameUpdated)
                        {
                            Ok(()) => {},
                            Err(e) => {
                                warn!("Send PlayerEvent::VideoFrameUpdated Error: {}", e);
                            },
                        };
                    },
                }
            }
        })
        .unwrap();
    }
}

impl Drop for VideoSink {
    fn drop(&mut self) {
        if let Some(sender) = self.thread_send_chan.get_mut() {
            let _ = sender.send(RenderMsg::Terminate);
        }
    }
}

struct OhosBuffer {
    data: Vec<u8>,
}

impl OhosBuffer {
    pub fn new(data: Vec<u8>) -> OhosBuffer {
        OhosBuffer { data }
    }
}

impl Buffer for OhosBuffer {
    fn to_vec(&self) -> Option<video::VideoFrameData> {
        Some(VideoFrameData::Raw(Arc::new(self.data.to_owned())))
    }
}
