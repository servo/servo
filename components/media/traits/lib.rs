/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::num::NonZeroU32;
use std::sync::mpsc::Sender;
/// An ID for clients to track instances of Players and AudioContexts belonging to the same tab and mute them simultaneously.
/// Current tuple implementation matches one of Servo's BrowsingContextId.
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct ClientContextId(u32, NonZeroU32);

impl ClientContextId {
    pub fn build(a: u32, b: u32) -> ClientContextId {
        ClientContextId(a, NonZeroU32::new(b).unwrap())
    }
}

#[derive(Debug)]
pub struct MediaInstanceError;

impl std::fmt::Display for MediaInstanceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "MediaInstanceError")
    }
}

impl std::error::Error for MediaInstanceError {}

/// Common functionality for all high level media instances
/// These currently are WebAudio AudioContexts and Players.
pub trait MediaInstance: Send {
    fn get_id(&self) -> usize;
    fn mute(&self, val: bool) -> Result<(), MediaInstanceError>;
    fn suspend(&self) -> Result<(), MediaInstanceError>;
    fn resume(&self) -> Result<(), MediaInstanceError>;
}

pub enum BackendMsg {
    /// Message to notify about a media instance shutdown.
    /// The given `usize` is the media instance ID.
    Shutdown {
        context: ClientContextId,
        id: usize,
        tx_ack: Sender<()>,
    },
}
