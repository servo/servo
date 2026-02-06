/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::mpsc::{Receiver, channel};

use byte_slice_cast::*;
use gstreamer::Fraction;
use gstreamer::prelude::*;
use gstreamer_audio::AUDIO_FORMAT_F32;
use servo_media_audio::AudioStreamReader;
use servo_media_audio::block::{Block, FRAMES_PER_BLOCK_USIZE};
use servo_media_streams::registry::{MediaStreamId, get_stream};

use crate::media_stream::GStreamerMediaStream;

pub struct GStreamerAudioStreamReader {
    rx: Receiver<Block>,
    pipeline: gstreamer::Pipeline,
}

impl GStreamerAudioStreamReader {
    pub fn new(stream: MediaStreamId, sample_rate: f32) -> Result<Self, String> {
        let (tx, rx) = channel();
        let stream = get_stream(&stream).unwrap();
        let mut stream = stream.lock().unwrap();
        let g_stream = stream
            .as_mut_any()
            .downcast_mut::<GStreamerMediaStream>()
            .unwrap();
        let element = g_stream.src_element();
        let pipeline = g_stream.pipeline_or_new();
        drop(stream);
        let time_per_block = Fraction::new(FRAMES_PER_BLOCK_USIZE as i32, sample_rate as i32);

        // XXXManishearth this is only necessary because of an upstream
        // gstreamer bug. https://github.com/servo/media/pull/362#issuecomment-647947034
        let caps = gstreamer_audio::AudioCapsBuilder::new()
            .layout(gstreamer_audio::AudioLayout::Interleaved)
            .build();
        let capsfilter0 = gstreamer::ElementFactory::make("capsfilter")
            .property("caps", caps)
            .build()
            .map_err(|error| format!("capsfilter creation failed: {error:?}"))?;

        let split = gstreamer::ElementFactory::make("audiobuffersplit")
            .property("output-buffer-duration", time_per_block)
            .build()
            .map_err(|error| format!("audiobuffersplit creation failed: {error:?}"))?;
        let convert = gstreamer::ElementFactory::make("audioconvert")
            .build()
            .map_err(|error| format!("audioconvert creation failed: {error:?}"))?;
        let caps = gstreamer_audio::AudioCapsBuilder::new()
            .layout(gstreamer_audio::AudioLayout::NonInterleaved)
            .format(AUDIO_FORMAT_F32)
            .rate(sample_rate as i32)
            .build();
        let capsfilter = gstreamer::ElementFactory::make("capsfilter")
            .property("caps", caps)
            .build()
            .map_err(|error| format!("capsfilter creation failed: {error:?}"))?;
        let sink = gstreamer::ElementFactory::make("appsink")
            .property("sync", false)
            .build()
            .map_err(|error| format!("appsink creation failed: {error:?}"))?;

        let appsink = sink
            .clone()
            .dynamic_cast::<gstreamer_app::AppSink>()
            .unwrap();

        let elements = [&element, &capsfilter0, &split, &convert, &capsfilter, &sink];
        pipeline
            .add_many(&elements[1..])
            .map_err(|error| format!("pipeline adding failed: {error:?}"))?;
        gstreamer::Element::link_many(elements)
            .map_err(|error| format!("element linking failed: {error:?}"))?;
        for e in &elements {
            e.sync_state_with_parent().map_err(|e| e.to_string())?;
        }
        appsink.set_callbacks(
            gstreamer_app::AppSinkCallbacks::builder()
                .new_sample(move |appsink| {
                    let sample = appsink
                        .pull_sample()
                        .map_err(|_| gstreamer::FlowError::Eos)?;
                    let buffer = sample.buffer_owned().ok_or(gstreamer::FlowError::Error)?;

                    let buffer = buffer
                        .into_mapped_buffer_readable()
                        .map_err(|_| gstreamer::FlowError::Error)?;
                    let floatref = buffer
                        .as_slice()
                        .as_slice_of::<f32>()
                        .map_err(|_| gstreamer::FlowError::Error)?;

                    let block = Block::for_vec(floatref.into());
                    tx.send(block).map_err(|_| gstreamer::FlowError::Error)?;
                    Ok(gstreamer::FlowSuccess::Ok)
                })
                .build(),
        );
        Ok(Self { rx, pipeline })
    }
}

impl AudioStreamReader for GStreamerAudioStreamReader {
    fn pull(&self) -> Block {
        self.rx.recv().unwrap()
    }

    fn start(&self) {
        self.pipeline.set_state(gstreamer::State::Playing).unwrap();
    }

    fn stop(&self) {
        self.pipeline.set_state(gstreamer::State::Null).unwrap();
    }
}
