use std::sync::Mutex;

use glib::prelude::*;
use gstreamer_webrtc::{WebRTCDataChannel, WebRTCDataChannelState};
use servo_media_webrtc::thread::InternalEvent;
use servo_media_webrtc::{
    DataChannelEvent, DataChannelId, DataChannelInit, DataChannelMessage, DataChannelState,
    WebRtcController as WebRtcThread, WebRtcError,
};

pub struct GStreamerWebRtcDataChannel {
    channel: WebRTCDataChannel,
    id: DataChannelId,
    thread: WebRtcThread,
}

impl GStreamerWebRtcDataChannel {
    pub fn new(
        servo_channel_id: &DataChannelId,
        webrtc: &gstreamer::Element,
        thread: &WebRtcThread,
        init: &DataChannelInit,
    ) -> Result<Self, String> {
        let label = &init.label;
        let mut init_struct = gstreamer::Structure::builder("options")
            .field("ordered", init.ordered)
            .field("protocol", &init.protocol)
            .field("negotiated", init.negotiated)
            .build();

        if let Some(max_packet_life_time) = init.max_packet_life_time {
            init_struct.set_value(
                "max-packet-lifetime",
                (max_packet_life_time as u32).to_send_value(),
            );
        }

        if let Some(max_retransmits) = init.max_retransmits {
            init_struct.set_value("max-retransmits", (max_retransmits as u32).to_send_value());
        }

        if let Some(id) = init.id {
            init_struct.set_value("id", (id as u32).to_send_value());
        }

        let channel = webrtc
            .emit_by_name::<WebRTCDataChannel>("create-data-channel", &[&label, &init_struct]);

        GStreamerWebRtcDataChannel::from(servo_channel_id, channel, thread)
    }

    pub fn from(
        id: &DataChannelId,
        channel: WebRTCDataChannel,
        thread: &WebRtcThread,
    ) -> Result<Self, String> {
        let id_ = *id;
        let thread_ = Mutex::new(thread.clone());
        channel.connect_on_open(move |_| {
            thread_
                .lock()
                .unwrap()
                .internal_event(InternalEvent::OnDataChannelEvent(
                    id_,
                    DataChannelEvent::Open,
                ));
        });

        let id_ = *id;
        let thread_ = Mutex::new(thread.clone());
        channel.connect_on_close(move |_| {
            thread_
                .lock()
                .unwrap()
                .internal_event(InternalEvent::OnDataChannelEvent(
                    id_,
                    DataChannelEvent::Close,
                ));
        });

        let id_ = *id;
        let thread_ = Mutex::new(thread.clone());
        channel.connect_on_error(move |_, error| {
            thread_
                .lock()
                .unwrap()
                .internal_event(InternalEvent::OnDataChannelEvent(
                    id_,
                    DataChannelEvent::Error(WebRtcError::Backend(error.to_string())),
                ));
        });

        let id_ = *id;
        let thread_ = Mutex::new(thread.clone());
        channel.connect_on_message_string(move |_, message| {
            let Some(message) = message.map(|s| s.to_owned()) else {
                return;
            };
            thread_
                .lock()
                .unwrap()
                .internal_event(InternalEvent::OnDataChannelEvent(
                    id_,
                    DataChannelEvent::OnMessage(DataChannelMessage::Text(message)),
                ));
        });

        let id_ = *id;
        let thread_ = Mutex::new(thread.clone());
        channel.connect_on_message_data(move |_, message| {
            let Some(message) = message.map(|b| b.to_owned()) else {
                return;
            };
            thread_
                .lock()
                .unwrap()
                .internal_event(InternalEvent::OnDataChannelEvent(
                    id_,
                    DataChannelEvent::OnMessage(DataChannelMessage::Binary(message.to_vec())),
                ));
        });

        let id_ = *id;
        let thread_ = Mutex::new(thread.clone());
        channel.connect_ready_state_notify(move |channel| {
            let ready_state = channel.ready_state();
            let ready_state = match ready_state {
                WebRTCDataChannelState::Connecting => DataChannelState::Connecting,
                WebRTCDataChannelState::Open => DataChannelState::Open,
                WebRTCDataChannelState::Closing => DataChannelState::Closing,
                WebRTCDataChannelState::Closed => DataChannelState::Closed,
                WebRTCDataChannelState::__Unknown(state) => DataChannelState::__Unknown(state),
                _ => return,
            };
            thread_
                .lock()
                .unwrap()
                .internal_event(InternalEvent::OnDataChannelEvent(
                    id_,
                    DataChannelEvent::StateChange(ready_state),
                ));
        });

        Ok(Self {
            id: *id,
            thread: thread.to_owned(),
            channel,
        })
    }

    pub fn send(&self, message: &DataChannelMessage) {
        match message {
            DataChannelMessage::Text(text) => self.channel.send_string(Some(text)),
            DataChannelMessage::Binary(data) => self
                .channel
                .send_data(Some(&glib::Bytes::from(data.as_slice()))),
        }
    }

    pub fn close(&self) {
        self.channel.close()
    }
}

impl Drop for GStreamerWebRtcDataChannel {
    fn drop(&mut self) {
        self.thread
            .internal_event(InternalEvent::OnDataChannelEvent(
                self.id,
                DataChannelEvent::Close,
            ));
    }
}
