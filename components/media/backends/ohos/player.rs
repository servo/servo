/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use servo_media::MediaInstance;
use servo_media_player::Player;

pub struct OhosAVPlayer {}

impl OhosAVPlayer {
    pub fn new() -> OhosAVPlayer {
        OhosAVPlayer {}
    }
}

impl MediaInstance for OhosAVPlayer {
    fn get_id(&self) -> usize {
        todo!()
    }

    fn mute(&self, val: bool) -> Result<(), ()> {
        todo!()
    }

    fn suspend(&self) -> Result<(), ()> {
        todo!()
    }

    fn resume(&self) -> Result<(), ()> {
        todo!()
    }
}

impl Player for OhosAVPlayer {
    fn play(&self) -> Result<(), servo_media_player::PlayerError> {
        todo!()
    }

    fn pause(&self) -> Result<(), servo_media_player::PlayerError> {
        todo!()
    }

    fn paused(&self) -> bool {
        todo!()
    }

    fn can_resume(&self) -> bool {
        todo!()
    }

    fn stop(&self) -> Result<(), servo_media_player::PlayerError> {
        todo!()
    }

    fn seek(&self, time: f64) -> Result<(), servo_media_player::PlayerError> {
        todo!()
    }

    fn seekable(&self) -> Vec<std::ops::Range<f64>> {
        todo!()
    }

    fn set_mute(&self, muted: bool) -> Result<(), servo_media_player::PlayerError> {
        todo!()
    }

    fn muted(&self) -> bool {
        todo!()
    }

    fn set_volume(&self, volume: f64) -> Result<(), servo_media_player::PlayerError> {
        todo!()
    }

    fn volume(&self) -> f64 {
        todo!()
    }

    fn set_input_size(&self, size: u64) -> Result<(), servo_media_player::PlayerError> {
        todo!()
    }

    fn set_playback_rate(&self, playback_rate: f64) -> Result<(), servo_media_player::PlayerError> {
        todo!()
    }

    fn playback_rate(&self) -> f64 {
        todo!()
    }

    fn push_data(&self, data: Vec<u8>) -> Result<(), servo_media_player::PlayerError> {
        todo!()
    }

    fn end_of_stream(&self) -> Result<(), servo_media_player::PlayerError> {
        todo!()
    }

    fn buffered(&self) -> Vec<std::ops::Range<f64>> {
        todo!()
    }

    fn set_stream(
        &self,
        stream: &servo_media_streams::MediaStreamId,
        only_stream: bool,
    ) -> Result<(), servo_media_player::PlayerError> {
        todo!()
    }

    fn render_use_gl(&self) -> bool {
        todo!()
    }

    fn set_audio_track(
        &self,
        stream_index: i32,
        enabled: bool,
    ) -> Result<(), servo_media_player::PlayerError> {
        todo!()
    }

    fn set_video_track(
        &self,
        stream_index: i32,
        enabled: bool,
    ) -> Result<(), servo_media_player::PlayerError> {
        todo!()
    }
}
