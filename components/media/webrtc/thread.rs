use std::sync::mpsc::{Sender, channel};
use std::thread;

use log::error;
use servo_media_streams::MediaStreamType;

use crate::datachannel::{DataChannelEvent, DataChannelId, DataChannelInit, DataChannelMessage};
use crate::{
    BundlePolicy, DescriptionType, IceCandidate, MediaStreamId, SdpType, SessionDescription,
    WebRtcBackend, WebRtcControllerBackend, WebRtcSignaller,
};

#[derive(Clone)]
/// Entry point for all client webrtc interactions.
pub struct WebRtcController {
    sender: Sender<RtcThreadEvent>,
}

impl WebRtcController {
    pub fn new<T: WebRtcBackend>(signaller: Box<dyn WebRtcSignaller>) -> Self {
        let (sender, receiver) = channel();

        let t = WebRtcController { sender };

        let mut controller = T::construct_webrtc_controller(signaller, t.clone());

        thread::spawn(move || {
            while let Ok(event) = receiver.recv() {
                if !handle_rtc_event(&mut controller, event) {
                    // shut down event loop
                    break;
                }
            }
        });

        t
    }
    pub fn configure(&self, stun_server: String, policy: BundlePolicy) {
        let _ = self
            .sender
            .send(RtcThreadEvent::ConfigureStun(stun_server, policy));
    }
    pub fn set_remote_description(
        &self,
        desc: SessionDescription,
        cb: Box<dyn FnOnce() + Send + 'static>,
    ) {
        let _ = self
            .sender
            .send(RtcThreadEvent::SetRemoteDescription(desc, cb));
    }
    pub fn set_local_description(
        &self,
        desc: SessionDescription,
        cb: Box<dyn FnOnce() + Send + 'static>,
    ) {
        let _ = self
            .sender
            .send(RtcThreadEvent::SetLocalDescription(desc, cb));
    }
    pub fn add_ice_candidate(&self, candidate: IceCandidate) {
        let _ = self.sender.send(RtcThreadEvent::AddIceCandidate(candidate));
    }
    pub fn create_offer(&self, cb: Box<dyn FnOnce(SessionDescription) + Send + 'static>) {
        let _ = self.sender.send(RtcThreadEvent::CreateOffer(cb));
    }
    pub fn create_answer(&self, cb: Box<dyn FnOnce(SessionDescription) + Send + 'static>) {
        let _ = self.sender.send(RtcThreadEvent::CreateAnswer(cb));
    }
    pub fn add_stream(&self, stream: &MediaStreamId) {
        let _ = self.sender.send(RtcThreadEvent::AddStream(stream.clone()));
    }
    pub fn create_data_channel(&self, init: DataChannelInit) -> Option<DataChannelId> {
        let (sender, receiver) = channel();
        let _ = self
            .sender
            .send(RtcThreadEvent::CreateDataChannel(init, sender));
        receiver.recv().unwrap()
    }
    pub fn send_data_channel_message(&self, id: &DataChannelId, message: DataChannelMessage) {
        let _ = self
            .sender
            .send(RtcThreadEvent::SendDataChannelMessage(*id, message));
    }
    pub fn close_data_channel(&self, id: &DataChannelId) {
        let _ = self.sender.send(RtcThreadEvent::CloseDataChannel(*id));
    }

    /// This should not be invoked by clients
    pub fn internal_event(&self, event: InternalEvent) {
        let _ = self.sender.send(RtcThreadEvent::InternalEvent(event));
    }

    pub fn quit(&self) {
        let _ = self.sender.send(RtcThreadEvent::Quit);
    }
}

pub enum RtcThreadEvent {
    ConfigureStun(String, BundlePolicy),
    SetRemoteDescription(SessionDescription, Box<dyn FnOnce() + Send + 'static>),
    SetLocalDescription(SessionDescription, Box<dyn FnOnce() + Send + 'static>),
    AddIceCandidate(IceCandidate),
    CreateOffer(Box<dyn FnOnce(SessionDescription) + Send + 'static>),
    CreateAnswer(Box<dyn FnOnce(SessionDescription) + Send + 'static>),
    AddStream(MediaStreamId),
    CreateDataChannel(DataChannelInit, Sender<Option<DataChannelId>>),
    CloseDataChannel(DataChannelId),
    SendDataChannelMessage(DataChannelId, DataChannelMessage),
    InternalEvent(InternalEvent),
    Quit,
}

/// To allow everything to occur on the event loop,
/// the backend may need to send signals to itself
///
/// This is a somewhat leaky abstraction, but we don't
/// plan on having too many backends anyway
pub enum InternalEvent {
    OnNegotiationNeeded,
    OnIceCandidate(IceCandidate),
    OnAddStream(MediaStreamId, MediaStreamType),
    OnDataChannelEvent(DataChannelId, DataChannelEvent),
    DescriptionAdded(
        Box<dyn FnOnce() + Send + 'static>,
        DescriptionType,
        SdpType,
        /* remote offer generation */ u32,
    ),
    UpdateSignalingState,
    UpdateGatheringState,
    UpdateIceConnectionState,
}

pub fn handle_rtc_event(
    controller: &mut dyn WebRtcControllerBackend,
    event: RtcThreadEvent,
) -> bool {
    let result = match event {
        RtcThreadEvent::ConfigureStun(server, policy) => controller.configure(&server, policy),
        RtcThreadEvent::SetRemoteDescription(desc, cb) => {
            controller.set_remote_description(desc, cb)
        },
        RtcThreadEvent::SetLocalDescription(desc, cb) => controller.set_local_description(desc, cb),
        RtcThreadEvent::AddIceCandidate(candidate) => controller.add_ice_candidate(candidate),
        RtcThreadEvent::CreateOffer(cb) => controller.create_offer(cb),
        RtcThreadEvent::CreateAnswer(cb) => controller.create_answer(cb),
        RtcThreadEvent::AddStream(media) => controller.add_stream(&media),
        RtcThreadEvent::CreateDataChannel(init, sender) => controller
            .create_data_channel(&init)
            .map(|id| {
                let _ = sender.send(Some(id));
                ()
            })
            .map_err(|e| {
                let _ = sender.send(None);
                e
            }),
        RtcThreadEvent::CloseDataChannel(id) => controller.close_data_channel(&id),
        RtcThreadEvent::SendDataChannelMessage(id, message) => {
            controller.send_data_channel_message(&id, &message)
        },
        RtcThreadEvent::InternalEvent(e) => controller.internal_event(e),
        RtcThreadEvent::Quit => {
            controller.quit();
            return false;
        },
    };
    if let Err(e) = result {
        error!("WebRTC backend encountered error: {:?}", e);
    }
    true
}
