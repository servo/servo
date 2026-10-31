/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Data and main loop of WebMIDI thread.

use std::collections::HashSet;
use std::thread::JoinHandle;
use std::time::Duration;

use log::warn;
use midir::{MidiInput, MidiOutput};
use rustc_hash::FxHashMap;
use servo_base::generic_channel::{self, GenericReceiver, GenericSender, TryReceiveError};
use servo_base::id::PipelineId;
use webmidi_traits::{CallbackData, MIDIMsg, MIDIRequest, MidiConnection, MidiPort, MidiPortInfo};

pub struct MIDIPoller {
    handle: Option<JoinHandle<()>>,
    ports_receiver: GenericReceiver<Vec<MidiPort>>,
    shutdown_sender: GenericSender<()>,
}

impl MIDIPoller {
    fn new() -> Self {
        let (ports_sender, ports_receiver) =
            generic_channel::channel().expect("Failed to create generic channel!");
        let (shutdown_sender, shutdown_receiver) =
            generic_channel::channel().expect("Failed to create generic channel!");
        let handle = std::thread::spawn(move || {
            loop {
                match shutdown_receiver.try_recv_timeout(Duration::from_secs(5)) {
                    Ok(()) => break,
                    Err(TryReceiveError::Empty) => {
                        let ports = get_port_info();
                        let _ = ports_sender.send(ports);
                    },
                    Err(TryReceiveError::ReceiveError(e)) => {
                        warn!("{}", e);
                        break;
                    },
                };
            }
        });
        Self {
            handle: Some(handle),
            ports_receiver,
            shutdown_sender,
        }
    }
}

#[expect(clippy::upper_case_acronyms)] // Name of the library
pub(crate) struct MIDI {
    pub(crate) script_sender: GenericSender<MIDIMsg>,
    script_receiver: GenericReceiver<MIDIRequest>,
    pipelines: Vec<PipelineId>,
    ports: FxHashMap<MidiPort, MidiConnection>,
    poller: MIDIPoller,
}

impl MIDI {
    pub(crate) fn new(
        script_receiver: GenericReceiver<MIDIRequest>,
        script_sender: GenericSender<MIDIMsg>,
    ) -> Self {
        MIDI {
            script_receiver,
            script_sender,
            pipelines: Vec::new(),
            ports: FxHashMap::default(),
            poller: MIDIPoller::new(),
        }
    }

    pub(crate) fn run(&mut self) {
        loop {
            // Handle messages from script thread
            if let Ok(msg) = self
                .script_receiver
                .try_recv_timeout(Duration::from_millis(100))
            {
                match msg {
                    MIDIRequest::GetPorts(pipeline_id, sender) => {
                        self.pipelines.push(pipeline_id);
                        let ports = get_port_info();
                        for port in &ports {
                            let conn = match port {
                                MidiPort::Input(_) => MidiConnection::Input(None),
                                MidiPort::Output(_) => MidiConnection::Output(None),
                            };
                            self.ports.insert(port.clone(), conn);
                        }
                        let _ = sender.send(ports);
                    },
                    MIDIRequest::OpenPort(pipeline_id, midi_port) => {
                        let Some((_, conn)) =
                            self.ports.iter_mut().find(|(port, _)| **port == midi_port)
                        else {
                            warn!("Failed to locate MIDI port entry for opening!");
                            continue;
                        };

                        match conn {
                            MidiConnection::Input(c) => {
                                if c.is_none() {
                                    let input = MidiInput::new("Servo")
                                        .expect("Failed to establish connection to MIDI inputs.");
                                    let port = input
                                        .find_port_by_id(midi_port.id())
                                        .expect("Failed to find MIDI input port to open!");
                                    let port_connection = input
                                        .connect(
                                            &port,
                                            "Servo WebMIDI Port",
                                            move |_timestamp, message, data| {
                                                let msg = MIDIMsg::Data(
                                                    pipeline_id,
                                                    data.port_id.clone(),
                                                    message.to_vec(),
                                                );
                                                let _ = data.sender.send(msg);
                                            },
                                            CallbackData {
                                                port_id: midi_port.id().to_string(),
                                                sender: self.script_sender.clone(),
                                            },
                                        )
                                        .expect("Failed to connect to MidiInputPort!");
                                    *conn = MidiConnection::Input(Some(port_connection));
                                }
                            },
                            MidiConnection::Output(c) => {
                                if c.is_none() {
                                    let output = MidiOutput::new("Servo")
                                        .expect("Failed to establish connection to MIDI outputs.");
                                    let port = output
                                        .find_port_by_id(midi_port.id())
                                        .expect("Failed to find MIDI output port to open!");
                                    let port_connection = output
                                        .connect(&port, "Servo WebMIDI Port")
                                        .expect("Failed to connect to MidiOutputPort!");

                                    *conn = MidiConnection::Output(Some(port_connection));
                                }
                            },
                        }

                        let _ = self
                            .script_sender
                            .send(MIDIMsg::PortOpened(pipeline_id, midi_port));
                    },
                    MIDIRequest::ClosePort(pipeline_id, midi_port) => {
                        let Some((_, conn)) =
                            self.ports.iter_mut().find(|(port, _)| **port == midi_port)
                        else {
                            warn!("Failed to locate MIDI port entry for closing!");
                            continue;
                        };

                        match conn {
                            MidiConnection::Input(midi_input_connection) => {
                                midi_input_connection.take().map(|c| c.close());
                            },
                            MidiConnection::Output(midi_output_connection) => {
                                midi_output_connection.take().map(|c| c.close());
                            },
                        }
                        self.ports
                            .remove(&midi_port)
                            .expect("Failed to remove MIDI port after close!");
                        let _ = self
                            .script_sender
                            .send(MIDIMsg::PortClosed(pipeline_id, midi_port));
                    },
                    MIDIRequest::SendData(id, data) => {
                        if let Some((_, conn)) = self
                            .ports
                            .iter_mut()
                            .find(|(info, conn)| info.id() == id && !conn.is_input())
                        {
                            let MidiConnection::Output(Some(output)) = conn else {
                                warn!("Failed to find MIDI output port to send data from!");
                                continue;
                            };
                            let _ = output.send(&data);
                        }
                    },
                }
            }
            // Handle messages from MIDI polling thread
            if let Ok(ports) = self.poller.ports_receiver.try_recv() {
                let current_port_ids: HashSet<&str> = ports.iter().map(|p| p.id()).collect();
                let new_ports: Vec<MidiPort> = ports
                    .iter()
                    .filter(|p| !self.ports.contains_key(p))
                    .cloned()
                    .collect();
                let old_ports: Vec<MidiPort> = self
                    .ports
                    .keys()
                    .filter(|info| !current_port_ids.contains(info.id()))
                    .cloned()
                    .collect();
                new_ports.into_iter().for_each(|port| {
                    let conn = match port {
                        MidiPort::Input(_) => MidiConnection::Input(None),
                        MidiPort::Output(_) => MidiConnection::Output(None),
                    };
                    self.ports.insert(port.clone(), conn);
                    for pipeline_id in self.pipelines.iter() {
                        let _ = self
                            .script_sender
                            .send(MIDIMsg::PortAvailable(*pipeline_id, port.clone()));
                    }
                });
                old_ports.into_iter().for_each(|info| {
                    if let Some(conn) = self.ports.get_mut(&info) {
                        match conn {
                            MidiConnection::Input(_) => {
                                self.ports.get(&info).replace(&MidiConnection::Input(None));
                            },
                            MidiConnection::Output(_) => {
                                self.ports.get(&info).replace(&MidiConnection::Output(None));
                            },
                        }
                    }
                    for pipeline_id in self.pipelines.iter() {
                        let _ = self
                            .script_sender
                            .send(MIDIMsg::PortUnavailable(*pipeline_id, info.clone()));
                    }
                });
            }
        }
    }
}

fn get_port_info() -> Vec<MidiPort> {
    let mut ports = Vec::new();
    let midi_input =
        MidiInput::new("Servo WebMIDI").expect("Failed to establish connection to MIDI inputs.");
    let midi_output =
        MidiOutput::new("Servo WebMIDI").expect("Failed to establish connection to MIDI outputs.");
    midi_input.ports().iter().for_each(|port| {
        let info = MidiPort::Input(MidiPortInfo {
            id: port.id(),
            name: midi_input.port_name(port).unwrap_or_else(|e| {
                warn!(
                    "Failed to get MIDI input port name (falling back to ID): {}",
                    e
                );
                port.id()
            }),
        });
        ports.push(info);
    });
    midi_output.ports().iter().for_each(|port| {
        let info = MidiPort::Output(MidiPortInfo {
            id: port.id(),
            name: midi_output.port_name(port).unwrap_or_else(|e| {
                warn!(
                    "Failed to get MIDI output port name (falling back to ID): {}",
                    e
                );
                port.id()
            }),
        });
        ports.push(info);
    });
    ports
}

impl Drop for MIDI {
    fn drop(&mut self) {
        for (_, midi_connection) in self.ports.drain() {
            match midi_connection {
                MidiConnection::Input(midi_input_connection) => {
                    if let Some(connection) = midi_input_connection {
                        connection.close();
                    }
                },
                MidiConnection::Output(midi_output_connection) => {
                    if let Some(connection) = midi_output_connection {
                        connection.close();
                    }
                },
            }
        }
        self.poller
            .shutdown_sender
            .send(())
            .expect("Failed to shut down MIDI polling thread!");
        if let Some(handle) = self.poller.handle.take() {
            let _ = handle.join();
        }
    }
}
