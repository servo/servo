use uuid::Uuid;

use crate::WebRtcError;

pub type DataChannelId = usize;

#[derive(Debug)]
pub enum DataChannelMessage {
    Text(String),
    Binary(Vec<u8>),
}

#[derive(Debug)]
pub enum DataChannelState {
    Connecting,
    Open,
    Closing,
    Closed,
    __Unknown(i32),
}

pub enum DataChannelEvent {
    NewChannel,
    Open,
    Close,
    Error(WebRtcError),
    OnMessage(DataChannelMessage),
    StateChange(DataChannelState),
}

// https://www.w3.org/TR/webrtc/#dom-rtcdatachannelinit
// plus `label`
pub struct DataChannelInit {
    pub label: String,
    pub ordered: bool,
    pub max_packet_life_time: Option<u16>,
    pub max_retransmits: Option<u16>,
    pub protocol: String,
    pub negotiated: bool,
    pub id: Option<u16>,
}

impl Default for DataChannelInit {
    fn default() -> DataChannelInit {
        DataChannelInit {
            label: Uuid::new_v4().to_string(),
            ordered: true,
            max_packet_life_time: None,
            max_retransmits: None,
            protocol: String::new(),
            negotiated: false,
            id: None,
        }
    }
}
