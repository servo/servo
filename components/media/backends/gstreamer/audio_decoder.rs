/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::io::{Cursor, Read};
use std::sync::{Arc, Mutex, mpsc};

use byte_slice_cast::*;
use gstreamer::prelude::*;
use servo_media_audio::decoder::{
    AudioDecoder, AudioDecoderCallbacks, AudioDecoderError, AudioDecoderOptions,
};
use {gstreamer, gstreamer_app, gstreamer_audio};

pub struct GStreamerAudioDecoderProgress(
    gstreamer::buffer::MappedBuffer<gstreamer::buffer::Readable>,
);

impl AsRef<[f32]> for GStreamerAudioDecoderProgress {
    fn as_ref(&self) -> &[f32] {
        self.0.as_ref().as_slice_of::<f32>().unwrap()
    }
}

#[derive(Default)]
pub struct GStreamerAudioDecoder {}

impl GStreamerAudioDecoder {
    pub fn new() -> Self {
        Default::default()
    }
}

impl AudioDecoder for GStreamerAudioDecoder {
    fn decode(
        &self,
        data: Vec<u8>,
        callbacks: AudioDecoderCallbacks,
        options: Option<AudioDecoderOptions>,
    ) {
        let pipeline = gstreamer::Pipeline::new();
        let callbacks = Arc::new(callbacks);

        let appsrc = match gstreamer::ElementFactory::make("appsrc").build() {
            Ok(appsrc) => appsrc,
            _ => {
                return callbacks.error(AudioDecoderError::Backend(
                    "appsrc creation failed".to_owned(),
                ));
            },
        };

        let decodebin = match gstreamer::ElementFactory::make("decodebin").build() {
            Ok(decodebin) => decodebin,
            _ => {
                return callbacks.error(AudioDecoderError::Backend(
                    "decodebin creation failed".to_owned(),
                ));
            },
        };

        // decodebin uses something called a "sometimes-pad", which is basically
        // a pad that will show up when a certain condition is met,
        // in decodebins case that is media being decoded
        if let Err(e) = pipeline.add_many([&appsrc, &decodebin]) {
            return callbacks.error(AudioDecoderError::Backend(e.to_string()));
        }

        if let Err(e) = gstreamer::Element::link_many([&appsrc, &decodebin]) {
            return callbacks.error(AudioDecoderError::Backend(e.to_string()));
        }

        let appsrc = appsrc.downcast::<gstreamer_app::AppSrc>().unwrap();

        let options = options.unwrap_or_default();

        let (sender, receiver) = mpsc::channel();
        let sender = Arc::new(Mutex::new(sender));

        let pipeline_ = pipeline.downgrade();
        let callbacks_ = callbacks.clone();
        let sender_ = sender.clone();
        // Initial pipeline looks like
        //
        // appsrc ! decodebin2! ...
        //
        // We plug in the second part of the pipeline, including the deinterleave element,
        // once the media starts being decoded.
        decodebin.connect_pad_added(move |_, src_pad| {
            // A decodebin pad was added, if this is an audio file,
            // plug in a deinterleave element to separate each planar channel.
            //
            // Sub pipeline looks like
            //
            // ... decodebin2 ! audioconvert ! audioresample ! capsfilter ! deinterleave ...
            //
            // deinterleave also uses a sometime-pad, so we need to wait until
            // a pad for a planar channel is added to plug in the last part of
            // the pipeline, with the appsink that will be pulling the data from
            // each channel.

            let callbacks = &callbacks_;
            let sender = &sender_;
            let pipeline = match pipeline_.upgrade() {
                Some(pipeline) => pipeline,
                None => {
                    callbacks.error(AudioDecoderError::Backend(
                        "Pipeline failed upgrade".to_owned(),
                    ));
                    let _ = sender.lock().unwrap().send(());
                    return;
                },
            };

            let (is_audio, caps) = {
                let media_type = src_pad.current_caps().and_then(|caps| {
                    caps.structure(0).map(|s| {
                        let name = s.name();
                        (name.starts_with("audio/"), caps.clone())
                    })
                });

                match media_type {
                    None => {
                        callbacks.error(AudioDecoderError::Backend(
                            "Failed to get media type from pad".to_owned(),
                        ));
                        let _ = sender.lock().unwrap().send(());
                        return;
                    },
                    Some(media_type) => media_type,
                }
            };

            if !is_audio {
                callbacks.error(AudioDecoderError::InvalidMediaFormat);
                let _ = sender.lock().unwrap().send(());
                return;
            }

            let sample_audio_info = match gstreamer_audio::AudioInfo::from_caps(&caps) {
                Ok(sample_audio_info) => sample_audio_info,
                _ => {
                    callbacks.error(AudioDecoderError::Backend("AudioInfo failed".to_owned()));
                    let _ = sender.lock().unwrap().send(());
                    return;
                },
            };
            let channels = sample_audio_info.channels();
            callbacks.ready(channels);

            let insert_deinterleave = || -> Result<(), AudioDecoderError> {
                let convert = gstreamer::ElementFactory::make("audioconvert")
                    .build()
                    .map_err(|error| {
                        AudioDecoderError::Backend(format!(
                            "audioconvert creation failed: {error:?}"
                        ))
                    })?;
                let resample = gstreamer::ElementFactory::make("audioresample")
                    .build()
                    .map_err(|error| {
                        AudioDecoderError::Backend(format!(
                            "audioresample creation failed: {error:?}"
                        ))
                    })?;
                let filter = gstreamer::ElementFactory::make("capsfilter")
                    .build()
                    .map_err(|error| {
                        AudioDecoderError::Backend(format!("capsfilter creation failed: {error:?}"))
                    })?;
                let deinterleave = gstreamer::ElementFactory::make("deinterleave")
                    .name("deinterleave")
                    .property("keep-positions", true)
                    .build()
                    .map_err(|error| {
                        AudioDecoderError::Backend(format!(
                            "deinterleave creation failed: {error:?}"
                        ))
                    })?;

                let pipeline_ = pipeline.downgrade();
                let callbacks_ = callbacks.clone();
                deinterleave.connect_pad_added(move |_, src_pad| {
                    // A new pad for a planar channel was added in deinterleave.
                    // Plug in an appsink so we can pull the data from each channel.
                    //
                    // The end of the pipeline looks like:
                    //
                    // ... deinterleave ! queue ! appsink.
                    let callbacks = &callbacks_;
                    let pipeline = match pipeline_.upgrade() {
                        Some(pipeline) => pipeline,
                        None => {
                            return callbacks.error(AudioDecoderError::Backend(
                                "Pipeline failedupgrade".to_owned(),
                            ));
                        },
                    };
                    let insert_sink = || -> Result<(), AudioDecoderError> {
                        let queue =
                            gstreamer::ElementFactory::make("queue")
                                .build()
                                .map_err(|error| {
                                    AudioDecoderError::Backend(format!(
                                        "queue creation failed: {error:?}"
                                    ))
                                })?;
                        let sink = gstreamer::ElementFactory::make("appsink").build().map_err(
                            |error| {
                                AudioDecoderError::Backend(format!(
                                    "appsink creation failed: {error:?}"
                                ))
                            },
                        )?;
                        let appsink = sink
                            .clone()
                            .dynamic_cast::<gstreamer_app::AppSink>()
                            .unwrap();
                        sink.set_property("sync", false);

                        let callbacks_ = callbacks.clone();
                        appsink.set_callbacks(
                            gstreamer_app::AppSinkCallbacks::builder()
                                .new_sample(move |appsink| {
                                    let sample = appsink
                                        .pull_sample()
                                        .map_err(|_| gstreamer::FlowError::Eos)?;
                                    let buffer = sample.buffer_owned().ok_or_else(|| {
                                        callbacks_.error(AudioDecoderError::InvalidSample);
                                        gstreamer::FlowError::Error
                                    })?;

                                    let audio_info = sample
                                        .caps()
                                        .and_then(|caps| {
                                            gstreamer_audio::AudioInfo::from_caps(caps).ok()
                                        })
                                        .ok_or_else(|| {
                                            callbacks_.error(AudioDecoderError::Backend(
                                                "Could not get caps from sample".to_owned(),
                                            ));
                                            gstreamer::FlowError::Error
                                        })?;
                                    let positions = audio_info.positions().ok_or_else(|| {
                                        callbacks_.error(AudioDecoderError::Backend(
                                            "AudioInfo failed".to_owned(),
                                        ));
                                        gstreamer::FlowError::Error
                                    })?;

                                    for position in positions.iter() {
                                        let buffer = buffer.clone();
                                        let map = match buffer.into_mapped_buffer_readable() {
                                            Ok(map) => map,
                                            _ => {
                                                callbacks_
                                                    .error(AudioDecoderError::BufferReadFailed);
                                                return Err(gstreamer::FlowError::Error);
                                            },
                                        };
                                        let progress = Box::new(GStreamerAudioDecoderProgress(map));
                                        let channel = position.to_mask() as u32;
                                        callbacks_.progress(progress, channel);
                                    }

                                    Ok(gstreamer::FlowSuccess::Ok)
                                })
                                .build(),
                        );

                        let elements = &[&queue, &sink];
                        pipeline
                            .add_many(elements)
                            .map_err(|e| AudioDecoderError::Backend(e.to_string()))?;
                        gstreamer::Element::link_many(elements)
                            .map_err(|e| AudioDecoderError::Backend(e.to_string()))?;

                        for e in elements {
                            e.sync_state_with_parent()
                                .map_err(|e| AudioDecoderError::Backend(e.to_string()))?;
                        }

                        let sink_pad = queue.static_pad("sink").ok_or(
                            AudioDecoderError::Backend("Could not get static pad sink".to_owned()),
                        )?;
                        src_pad.link(&sink_pad).map(|_| ()).map_err(|e| {
                            AudioDecoderError::Backend(format!("Sink pad link failed: {}", e))
                        })
                    };

                    if let Err(e) = insert_sink() {
                        callbacks.error(e);
                    }
                });

                let mut audio_info_builder = gstreamer_audio::AudioInfo::builder(
                    gstreamer_audio::AUDIO_FORMAT_F32,
                    options.sample_rate as u32,
                    channels,
                );
                if let Some(positions) = sample_audio_info.positions() {
                    audio_info_builder = audio_info_builder.positions(positions);
                }
                let audio_info = audio_info_builder.build().map_err(|error| {
                    AudioDecoderError::Backend(format!("AudioInfo failed: {error:?}"))
                })?;
                let caps = audio_info.to_caps().map_err(|error| {
                    AudioDecoderError::Backend(format!("AudioInfo failed: {error:?}"))
                })?;
                filter.set_property("caps", caps);

                let elements = &[&convert, &resample, &filter, &deinterleave];
                pipeline
                    .add_many(elements)
                    .map_err(|e| AudioDecoderError::Backend(e.to_string()))?;
                gstreamer::Element::link_many(elements)
                    .map_err(|e| AudioDecoderError::Backend(e.to_string()))?;

                for e in elements {
                    e.sync_state_with_parent()
                        .map_err(|e| AudioDecoderError::Backend(e.to_string()))?;
                }

                let sink_pad = convert
                    .static_pad("sink")
                    .ok_or(AudioDecoderError::Backend(
                        "Get static pad sink failed".to_owned(),
                    ))?;
                src_pad
                    .link(&sink_pad)
                    .map(|_| ())
                    .map_err(|e| AudioDecoderError::Backend(format!("Sink pad link failed: {}", e)))
            };

            if let Err(e) = insert_deinterleave() {
                callbacks.error(e);
                let _ = sender.lock().unwrap().send(());
            }
        });

        appsrc.set_format(gstreamer::Format::Bytes);
        appsrc.set_block(true);

        let bus = match pipeline.bus() {
            Some(bus) => bus,
            None => {
                callbacks.error(AudioDecoderError::Backend(
                    "Pipeline without bus. Shouldn't happen!".to_owned(),
                ));
                let _ = sender.lock().unwrap().send(());
                return;
            },
        };

        let callbacks_ = callbacks.clone();
        bus.set_sync_handler(move |_, msg| {
            use gstreamer::MessageView;

            match msg.view() {
                MessageView::Error(e) => {
                    callbacks_.error(AudioDecoderError::Backend(
                        e.debug()
                            .map(|d| d.to_string())
                            .unwrap_or_else(|| "Unknown".to_owned()),
                    ));
                    let _ = sender.lock().unwrap().send(());
                },
                MessageView::Eos(_) => {
                    callbacks_.eos();
                    let _ = sender.lock().unwrap().send(());
                },
                _ => (),
            }
            gstreamer::BusSyncReply::Drop
        });

        if pipeline.set_state(gstreamer::State::Playing).is_err() {
            callbacks.error(AudioDecoderError::StateChangeFailed);
            return;
        }

        let max_bytes = appsrc.max_bytes() as usize;
        let data_len = data.len();
        let mut reader = Cursor::new(data);
        while (reader.position() as usize) < data_len {
            let data_left = data_len - reader.position() as usize;
            let buffer_size = if data_left < max_bytes {
                data_left
            } else {
                max_bytes
            };
            let mut buffer = gstreamer::Buffer::with_size(buffer_size).unwrap();
            {
                let buffer = buffer.get_mut().unwrap();
                let mut map = buffer.map_writable().unwrap();
                let buffer = map.as_mut_slice();
                let _ = reader.read(buffer);
            }
            let _ = appsrc.push_buffer(buffer);
        }
        let _ = appsrc.end_of_stream();

        // Wait until we get an error or EOS.
        receiver.recv().unwrap();
        let _ = pipeline.set_state(gstreamer::State::Null);
    }
}
