/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::mpsc::Sender;

use servo_media_streams::MediaSocket;

use crate::block::Chunk;
use crate::render_thread::{AudioRenderThreadMsg, SinkEosCallback};

#[derive(Debug, PartialEq)]
pub enum AudioSinkError {
    /// Backend specific error.
    Backend(String),
    /// Could not push buffer into the audio sink.
    BufferPushFailed,
    /// Could not move to a different state.
    StateChangeFailed,
}

pub trait AudioSink: Send {
    fn init(
        &self,
        sample_rate: f32,
        render_thread_channel: Sender<AudioRenderThreadMsg>,
    ) -> Result<(), AudioSinkError>;
    fn init_stream(
        &self,
        channels: u8,
        sample_rate: f32,
        socket: Box<dyn MediaSocket>,
    ) -> Result<(), AudioSinkError>;
    fn play(&self) -> Result<(), AudioSinkError>;
    fn stop(&self) -> Result<(), AudioSinkError>;
    fn has_enough_data(&self) -> bool;
    fn push_data(&self, chunk: Chunk) -> Result<(), AudioSinkError>;
    fn set_eos_callback(&self, callback: SinkEosCallback);
}
