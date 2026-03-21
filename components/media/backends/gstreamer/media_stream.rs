/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::any::Any;
use std::sync::{Arc, Mutex};

use glib::BoolError;
use gstreamer;
use gstreamer::prelude::*;
use once_cell::sync::Lazy;
use servo_media_streams::registry::{
    MediaStreamId, get_stream, register_stream, unregister_stream,
};
use servo_media_streams::{MediaOutput, MediaSocket, MediaStream, MediaStreamType};

use super::BACKEND_BASE_TIME;

pub static RTP_CAPS_OPUS: Lazy<gstreamer::Caps> = Lazy::new(|| {
    gstreamer::Caps::builder("application/x-rtp")
        .field("media", "audio")
        .field("encoding-name", "OPUS")
        .build()
});

pub static RTP_CAPS_VP8: Lazy<gstreamer::Caps> = Lazy::new(|| {
    gstreamer::Caps::builder("application/x-rtp")
        .field("media", "video")
        .field("encoding-name", "VP8")
        .build()
});

pub struct GStreamerMediaStream {
    id: Option<MediaStreamId>,
    type_: MediaStreamType,
    elements: Vec<gstreamer::Element>,
    pipeline: Option<gstreamer::Pipeline>,
}

impl MediaStream for GStreamerMediaStream {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }

    fn set_id(&mut self, id: MediaStreamId) {
        self.id = Some(id);
    }

    fn ty(&self) -> MediaStreamType {
        self.type_
    }
}

impl GStreamerMediaStream {
    pub fn new(type_: MediaStreamType, elements: Vec<gstreamer::Element>) -> Self {
        Self {
            id: None,
            type_,
            elements,
            pipeline: None,
        }
    }

    pub fn caps(&self) -> &gstreamer::Caps {
        match self.type_ {
            MediaStreamType::Audio => &RTP_CAPS_OPUS,
            MediaStreamType::Video => &RTP_CAPS_VP8,
        }
    }

    pub fn caps_with_payload(&self, payload: i32) -> gstreamer::Caps {
        match self.type_ {
            MediaStreamType::Audio => gstreamer::Caps::builder("application/x-rtp")
                .field("media", "audio")
                .field("encoding-name", "OPUS")
                .field("payload", payload)
                .build(),
            MediaStreamType::Video => gstreamer::Caps::builder("application/x-rtp")
                .field("media", "video")
                .field("encoding-name", "VP8")
                .field("payload", payload)
                .build(),
        }
    }

    pub fn src_element(&self) -> gstreamer::Element {
        self.elements.last().unwrap().clone()
    }

    pub fn attach_to_pipeline(&mut self, pipeline: &gstreamer::Pipeline) {
        assert!(self.pipeline.is_none());
        let elements: Vec<_> = self.elements.iter().collect();
        pipeline.add_many(&elements[..]).unwrap();
        gstreamer::Element::link_many(&elements[..]).unwrap();
        for element in elements {
            element.sync_state_with_parent().unwrap();
        }
        self.pipeline = Some(pipeline.clone());
    }

    pub fn pipeline_or_new(&mut self) -> gstreamer::Pipeline {
        match self.pipeline {
            Some(ref pipeline) => pipeline.clone(),
            _ => {
                let pipeline =
                    gstreamer::Pipeline::with_name("gstreamermediastream fresh pipeline");
                let clock = gstreamer::SystemClock::obtain();
                pipeline.set_start_time(gstreamer::ClockTime::NONE);
                pipeline.set_base_time(*BACKEND_BASE_TIME);
                pipeline.use_clock(Some(&clock));
                self.attach_to_pipeline(&pipeline);
                pipeline
            },
        }
    }

    pub fn create_video() -> MediaStreamId {
        let videotestsrc = gstreamer::ElementFactory::make("videotestsrc")
            .property_from_str("pattern", "ball")
            .property("is-live", true)
            .build()
            .unwrap();
        Self::create_video_from(videotestsrc)
    }

    /// Attaches encoding adapters to the stream, returning the source element when successful.
    pub fn encoded(&mut self) -> Result<gstreamer::Element, BoolError> {
        let pipeline = self
            .pipeline
            .as_ref()
            .expect("GStreamerMediaStream::encoded() should not be called without a pipeline");
        let src = self.src_element();

        let capsfilter = gstreamer::ElementFactory::make("capsfilter")
            .property("caps", self.caps())
            .build()?;
        match self.type_ {
            MediaStreamType::Video => {
                let vp8enc = gstreamer::ElementFactory::make("vp8enc")
                    .property("deadline", 1i64)
                    .property("error-resilient", "default")
                    .property("cpu-used", -16i32)
                    .property("lag-in-frames", 0i32)
                    .build()?;

                let rtpvp8pay = gstreamer::ElementFactory::make("rtpvp8pay")
                    .property("picture-id-mode", "15-bit")
                    .property("mtu", 1200u32)
                    .build()?;
                let queue2 = gstreamer::ElementFactory::make("queue").build()?;

                pipeline.add_many([&vp8enc, &rtpvp8pay, &queue2, &capsfilter])?;
                gstreamer::Element::link_many([&src, &vp8enc, &rtpvp8pay, &queue2, &capsfilter])?;
                vp8enc.sync_state_with_parent()?;
                rtpvp8pay.sync_state_with_parent()?;
                queue2.sync_state_with_parent()?;
                capsfilter.sync_state_with_parent()?;
                Ok(capsfilter)
            },
            MediaStreamType::Audio => {
                let opusenc = gstreamer::ElementFactory::make("opusenc").build()?;
                let rtpopuspay = gstreamer::ElementFactory::make("rtpopuspay")
                    .property("mtu", 1200u32)
                    .build()?;
                let queue3 = gstreamer::ElementFactory::make("queue").build()?;
                pipeline.add_many([&opusenc, &rtpopuspay, &queue3, &capsfilter])?;
                gstreamer::Element::link_many([&src, &opusenc, &rtpopuspay, &queue3, &capsfilter])?;
                opusenc.sync_state_with_parent()?;
                rtpopuspay.sync_state_with_parent()?;
                queue3.sync_state_with_parent()?;
                Ok(capsfilter)
            },
        }
    }

    pub fn create_video_from(source: gstreamer::Element) -> MediaStreamId {
        let videoconvert = gstreamer::ElementFactory::make("videoconvert")
            .build()
            .unwrap();
        let queue = gstreamer::ElementFactory::make("queue").build().unwrap();

        register_stream(Arc::new(Mutex::new(GStreamerMediaStream::new(
            MediaStreamType::Video,
            vec![source, videoconvert, queue],
        ))))
    }

    pub fn create_audio() -> MediaStreamId {
        let audiotestsrc = gstreamer::ElementFactory::make("audiotestsrc")
            .property_from_str("wave", "sine")
            .property("is-live", true)
            .build()
            .unwrap();

        Self::create_audio_from(audiotestsrc)
    }

    pub fn create_audio_from(source: gstreamer::Element) -> MediaStreamId {
        let queue = gstreamer::ElementFactory::make("queue").build().unwrap();
        let audioconvert = gstreamer::ElementFactory::make("audioconvert")
            .build()
            .unwrap();
        let audioresample = gstreamer::ElementFactory::make("audioresample")
            .build()
            .unwrap();
        let queue2 = gstreamer::ElementFactory::make("queue").build().unwrap();

        register_stream(Arc::new(Mutex::new(GStreamerMediaStream::new(
            MediaStreamType::Audio,
            vec![source, queue, audioconvert, audioresample, queue2],
        ))))
    }

    pub fn create_proxy(ty: MediaStreamType) -> (MediaStreamId, GstreamerMediaSocket) {
        let proxy_sink = gstreamer::ElementFactory::make("proxysink")
            .build()
            .unwrap();
        let proxy_src = gstreamer::ElementFactory::make("proxysrc")
            .property("proxysink", &proxy_sink)
            .build()
            .unwrap();
        let stream = match ty {
            MediaStreamType::Audio => Self::create_audio_from(proxy_src),
            MediaStreamType::Video => Self::create_video_from(proxy_src),
        };

        (stream, GstreamerMediaSocket { proxy_sink })
    }
}

impl Drop for GStreamerMediaStream {
    fn drop(&mut self) {
        if let Some(ref id) = self.id {
            unregister_stream(id);
        }
    }
}

#[derive(Default)]
pub struct MediaSink {
    streams: Vec<Arc<Mutex<dyn MediaStream>>>,
}

impl MediaOutput for MediaSink {
    fn add_stream(&mut self, stream: &MediaStreamId) {
        let stream = get_stream(stream).expect("Media streams registry does not contain such ID");
        {
            let mut stream = stream.lock().unwrap();
            let stream = stream
                .as_mut_any()
                .downcast_mut::<GStreamerMediaStream>()
                .unwrap();
            let pipeline = stream.pipeline_or_new();
            let last_element = stream.elements.last();
            let last_element = last_element.as_ref().unwrap();
            let sink = match stream.type_ {
                MediaStreamType::Audio => "autoaudiosink",
                MediaStreamType::Video => "autovideosink",
            };
            let sink = gstreamer::ElementFactory::make(sink).build().unwrap();
            pipeline.add(&sink).unwrap();
            gstreamer::Element::link_many(&[last_element, &sink][..]).unwrap();

            pipeline.set_state(gstreamer::State::Playing).unwrap();
            sink.sync_state_with_parent().unwrap();
        }
        self.streams.push(stream.clone());
    }
}

pub struct GstreamerMediaSocket {
    proxy_sink: gstreamer::Element,
}

impl GstreamerMediaSocket {
    pub fn proxy_sink(&self) -> &gstreamer::Element {
        &self.proxy_sink
    }
}

impl MediaSocket for GstreamerMediaSocket {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
