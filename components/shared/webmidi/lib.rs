use midir::{MidiInputConnection, MidiOutputConnection};
use serde::{Deserialize, Serialize};
use servo_base::generic_channel::GenericSender;
use servo_base::id::PipelineId;

pub struct CallbackData {
    pub port_id: String,
    pub sender: GenericSender<MIDIMsg>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum MidiPort {
    Input(MidiPortInfo),
    Output(MidiPortInfo),
}

impl MidiPort {
    pub fn id(&self) -> &str {
        match self {
            MidiPort::Input(port_info) => &port_info.id,
            MidiPort::Output(port_info) => &port_info.id,
        }
    }

    pub fn info(&self) -> &MidiPortInfo {
        match self {
            MidiPort::Input(midi_port_info) => midi_port_info,
            MidiPort::Output(midi_port_info) => midi_port_info,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct MidiPortInfo {
    pub id: String,
    pub name: String,
}

pub enum MidiConnection {
    Input(Option<MidiInputConnection<CallbackData>>),
    Output(Option<MidiOutputConnection>),
}

impl MidiConnection {
    pub fn is_input(&self) -> bool {
        match self {
            MidiConnection::Input(_) => true,
            MidiConnection::Output(_) => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MIDIMsg {
    Data(PipelineId, String, Vec<u8>),
    PortAvailable(PipelineId, MidiPort),
    PortUnavailable(PipelineId, MidiPort),
    PortOpened(PipelineId, MidiPort),
    PortClosed(PipelineId, MidiPort),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MIDIRequest {
    GetPorts(PipelineId, GenericSender<Vec<MidiPort>>),
    OpenPort(PipelineId, MidiPort),
    ClosePort(PipelineId, MidiPort),
    SendData(String, Vec<u8>),
}
