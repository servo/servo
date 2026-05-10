/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex, Weak};
use std::thread;

use log::{debug, warn};
use mime::Mime;
use servo_media::player::StreamType;
use servo_media::{
    Backend, BackendInit, BackendMsg, ClientContextId, MediaInstance, MediaInstanceError,
    SupportsMediaType,
};

use crate::player::OhosAvPlayer;
use crate::registry_scanner::OHOS_REGISTRY_SCANNER;
mod ohos_media;
mod player;
mod registry_scanner;

type MediaInstanceMap = HashMap<ClientContextId, Vec<(usize, Weak<Mutex<dyn MediaInstance>>)>>;

pub struct OhosBackend {
    instances: Arc<Mutex<MediaInstanceMap>>,
    next_instance_id: AtomicUsize,
    backend_chan: Arc<Mutex<Sender<BackendMsg>>>,
}

impl OhosBackend {
    fn media_instance_action(
        &self,
        id: &ClientContextId,
        cb: &dyn Fn(&dyn MediaInstance) -> Result<(), MediaInstanceError>,
    ) {
        let mut instances = self.instances.lock().unwrap();
        match instances.get_mut(id) {
            Some(vec) => vec.retain(|(_, weak)| {
                if let Some(instance) = weak.upgrade() {
                    if cb(&*(instance.lock().unwrap())).is_err() {
                        warn!("Error executing media instance action");
                    }
                    true
                } else {
                    false
                }
            }),
            None => {
                warn!("Trying to exec media action on an unknown client context");
            },
        }
    }
}

impl BackendInit for OhosBackend {
    fn init() -> Box<dyn Backend> {
        let instances: Arc<Mutex<MediaInstanceMap>> = Arc::new(Mutex::new(HashMap::new()));

        let instances_ = instances.clone();
        let (backend_chan, recvr) = mpsc::channel();
        thread::Builder::new()
            .name("OhosBackend ShutdownThread".to_owned())
            .spawn(move || {
                match recvr.recv().unwrap() {
                    BackendMsg::Shutdown {
                        context,
                        id,
                        tx_ack,
                    } => {
                        let mut map = instances_.lock().unwrap();
                        if let Some(vec) = map.get_mut(&context) {
                            vec.retain(|m| m.0 != id);
                            if vec.is_empty() {
                                map.remove(&context);
                            }
                        }
                        let _ = tx_ack.send(());
                    },
                };
            })
            .unwrap();
        Box::new(OhosBackend {
            next_instance_id: AtomicUsize::new(0),
            instances,
            backend_chan: Arc::new(Mutex::new(backend_chan)),
        })
    }
}

// https://developer.huawei.com/consumer/en/doc/harmonyos-guides/obtain-supported-codecs
// https://developer.huawei.com/consumer/en/doc/harmonyos-guides/media-kit-intro-V5#supported-formats-and-protocols

impl Backend for OhosBackend {
    fn create_player(
        &self,
        context_id: &servo_media::ClientContextId,
        stream_type: servo_media_player::StreamType,
        sender: servo_media_player::ipc_channel::ipc::IpcSender<servo_media_player::PlayerEvent>,
        video_renderer: Option<
            std::sync::Arc<std::sync::Mutex<dyn servo_media_player::video::VideoFrameRenderer>>,
        >,
        audio_renderer: Option<
            std::sync::Arc<std::sync::Mutex<dyn servo_media_player::audio::AudioRenderer>>,
        >,
        _gl_context: Box<dyn servo_media_player::context::PlayerGLContext>,
    ) -> std::sync::Arc<std::sync::Mutex<dyn servo_media_player::Player>> {
        // TODO: Choose different Player Impl depends on stream_type
        match stream_type {
            StreamType::Stream => {
                todo!("Stream Type currently not supported!")
            },
            StreamType::Seekable => (),
        }

        if let Some(_audio_renderer) = audio_renderer {
            warn!("Audio Rendering Currently Not Supported!");
        }

        let player_id = self.next_instance_id.fetch_add(1, Ordering::Relaxed);
        debug!("Creating Player in OhosBackend");
        let mut player = OhosAvPlayer::new(
            player_id,
            *context_id,
            sender,
            video_renderer,
            self.backend_chan.clone(),
        );

        player.setup_info_event();
        player.setup_data_source();
        Arc::new(Mutex::new(player))
    }

    fn create_audiostream(&self) -> servo_media_streams::MediaStreamId {
        todo!()
    }

    fn create_videostream(&self) -> servo_media_streams::MediaStreamId {
        todo!()
    }

    fn create_stream_output(&self) -> Box<dyn servo_media_streams::MediaOutput> {
        todo!()
    }

    fn create_stream_and_socket(
        &self,
        _ty: servo_media_streams::MediaStreamType,
    ) -> (
        Box<dyn servo_media_streams::MediaSocket>,
        servo_media_streams::MediaStreamId,
    ) {
        todo!()
    }

    fn create_audioinput_stream(
        &self,
        _set: servo_media_streams::capture::MediaTrackConstraintSet,
    ) -> Option<servo_media_streams::MediaStreamId> {
        todo!()
    }

    fn create_videoinput_stream(
        &self,
        _set: servo_media_streams::capture::MediaTrackConstraintSet,
    ) -> Option<servo_media_streams::MediaStreamId> {
        todo!()
    }

    fn create_audio_context(
        &self,
        _id: &servo_media::ClientContextId,
        _options: servo_media_audio::context::AudioContextOptions,
    ) -> Result<
        std::sync::Arc<std::sync::Mutex<servo_media_audio::context::AudioContext>>,
        servo_media_audio::sink::AudioSinkError,
    > {
        todo!()
    }

    fn create_webrtc(
        &self,
        _signaller: Box<dyn servo_media_webrtc::WebRtcSignaller>,
    ) -> servo_media_webrtc::WebRtcController {
        todo!()
    }

    fn can_play_type(&self, media_type: &str) -> servo_media::SupportsMediaType {
        if let Ok(mime) = media_type.parse::<Mime>() {
            let mime_type = mime.type_().as_str().to_owned() + "/" + mime.subtype().as_str();
            let codecs = match mime.get_param("codecs") {
                Some(codecs) => codecs
                    .as_str()
                    .split(',')
                    .map(|codec| codec.trim())
                    .collect(),
                None => vec![],
            };

            if OHOS_REGISTRY_SCANNER.are_mime_and_codecs_supported(&mime_type, &codecs) {
                if codecs.is_empty() {
                    return SupportsMediaType::Maybe;
                }
                return SupportsMediaType::Probably;
            }
        }
        SupportsMediaType::No
    }

    fn get_device_monitor(
        &self,
    ) -> Box<dyn servo_media_streams::device_monitor::MediaDeviceMonitor> {
        todo!()
    }

    fn mute(&self, id: &ClientContextId, val: bool) {
        self.media_instance_action(
            id,
            &(move |instance: &dyn MediaInstance| instance.mute(val)),
        );
    }

    fn resume(&self, id: &ClientContextId) {
        self.media_instance_action(id, &(move |instance: &dyn MediaInstance| instance.resume()));
    }

    fn suspend(&self, id: &ClientContextId) {
        self.media_instance_action(
            id,
            &(move |instance: &dyn MediaInstance| instance.suspend()),
        );
    }
}
