/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Range;
use std::time::Duration;

pub extern crate ipc_channel;
#[macro_use]
extern crate serde_derive;
extern crate servo_media_streams as streams;
extern crate servo_media_traits;

pub mod audio;
pub mod context;
pub mod metadata;
pub mod video;

use ipc_channel::ipc::{self, IpcSender};
use servo_media_traits::MediaInstance;
use streams::registry::MediaStreamId;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PlaybackState {
    Stopped,
    Buffering,
    Paused,
    Playing,
}

#[derive(Debug, PartialEq)]
pub enum PlayerError {
    /// Backend specific error.
    Backend(String),
    /// Could not push buffer contents to the player.
    BufferPushFailed,
    /// The player cannot consume more data.
    EnoughData,
    /// Setting End Of Stream failed.
    EOSFailed,
    /// The media stream is not seekable.
    NonSeekableStream,
    /// Tried to seek out of range.
    SeekOutOfRange,
    /// Setting an audio or video stream failed.
    /// Possibly because the type of source is not PlayerSource::Stream.
    SetStreamFailed,
    // Setting an audio or video track failed.
    SetTrackFailed,
}

pub type SeekLockMsg = (bool, IpcSender<()>);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SeekLock {
    pub lock_channel: IpcSender<SeekLockMsg>,
}

impl SeekLock {
    pub fn unlock(&self, result: bool) {
        let (ack_sender, ack_recv) = ipc::channel::<()>().expect("Could not create IPC channel");
        self.lock_channel.send((result, ack_sender)).unwrap();
        ack_recv.recv().unwrap()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PlayerEvent {
    EndOfStream,
    /// The player has enough data. The client should stop pushing data into.
    EnoughData,
    Error(String),
    VideoFrameUpdated,
    MetadataUpdated(metadata::Metadata),
    // The `None` value means the duration is unknown, in which case this is likely a live stream.
    DurationChanged(Option<Duration>),
    /// The internal player queue is running out of data. The client should start
    /// pushing more data.
    NeedData,
    PositionChanged(f64),
    /// The player needs the data to perform a seek to the given offset in bytes.
    /// The next push_data should get the buffers from the new offset.
    /// The player will be blocked until the user unlocks it through
    /// the given SeekLock instance.
    /// This event is only received for seekable stream types.
    SeekData(u64, SeekLock),
    /// The player has performed a seek to the given time offset in seconds.
    SeekDone(f64),
    StateChanged(PlaybackState),
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum StreamType {
    /// No seeking is supported in the stream, such as a live stream.
    Stream,
    /// The stream is seekable.
    Seekable,
}

pub trait Player: Send + MediaInstance {
    fn play(&self) -> Result<(), PlayerError>;
    fn pause(&self) -> Result<(), PlayerError>;
    fn paused(&self) -> bool;
    fn can_resume(&self) -> bool;
    fn stop(&self) -> Result<(), PlayerError>;
    fn seek(&self, time: f64) -> Result<(), PlayerError>;
    fn seekable(&self) -> Vec<Range<f64>>;
    fn set_mute(&self, muted: bool) -> Result<(), PlayerError>;
    fn muted(&self) -> bool;
    fn set_volume(&self, volume: f64) -> Result<(), PlayerError>;
    fn volume(&self) -> f64;
    fn set_input_size(&self, size: u64) -> Result<(), PlayerError>;
    fn set_playback_rate(&self, playback_rate: f64) -> Result<(), PlayerError>;
    fn playback_rate(&self) -> f64;
    fn push_data(&self, data: Vec<u8>) -> Result<(), PlayerError>;
    fn end_of_stream(&self) -> Result<(), PlayerError>;
    /// Get the list of time ranges in seconds that have been buffered.
    fn buffered(&self) -> Vec<Range<f64>>;
    /// Set the stream to be played by the player.
    /// Only a single stream of the same type (audio or video) can be set.
    /// Subsequent calls with a stream of the same type will override the previously
    /// set stream.
    /// This method requires the player to be constructed with StreamType::Stream.
    /// It is important to give the correct value of `only_stream` indicating
    /// that the audio or video stream being set is the only one expected.
    /// Subsequent calls to `set_stream` after the `only_stream` flag has been
    /// set to true will fail.
    fn set_stream(&self, stream: &MediaStreamId, only_stream: bool) -> Result<(), PlayerError>;
    /// If player's rendering draws using GL textures
    fn render_use_gl(&self) -> bool;
    fn set_audio_track(&self, stream_index: i32, enabled: bool) -> Result<(), PlayerError>;
    fn set_video_track(&self, stream_index: i32, enabled: bool) -> Result<(), PlayerError>;
}
