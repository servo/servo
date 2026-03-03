/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use glib::subclass::prelude::*;
use gstreamer::prelude::*;
use gstreamer::subclass::prelude::*;
use gstreamer_base::UniqueFlowCombiner;
use once_cell::sync::Lazy;
use servo_media_player::PlayerError;
use servo_media_streams::{MediaStream, MediaStreamType};
use url::Url;

use crate::media_stream::{GStreamerMediaStream, RTP_CAPS_OPUS, RTP_CAPS_VP8};

// Implementation sub-module of the GObject
mod imp {
    use super::*;

    static AUDIO_SRC_PAD_TEMPLATE: Lazy<gstreamer::PadTemplate> = Lazy::new(|| {
        gstreamer::PadTemplate::new(
            "audio_src",
            gstreamer::PadDirection::Src,
            gstreamer::PadPresence::Sometimes,
            &RTP_CAPS_OPUS,
        )
        .expect("Could not create audio src pad template")
    });

    static VIDEO_SRC_PAD_TEMPLATE: Lazy<gstreamer::PadTemplate> = Lazy::new(|| {
        gstreamer::PadTemplate::new(
            "video_src",
            gstreamer::PadDirection::Src,
            gstreamer::PadPresence::Sometimes,
            &RTP_CAPS_VP8,
        )
        .expect("Could not create video src pad template")
    });

    pub struct ServoMediaStreamSrc {
        cat: gstreamer::DebugCategory,
        audio_proxysrc: gstreamer::Element,
        audio_srcpad: gstreamer::GhostPad,
        video_proxysrc: gstreamer::Element,
        video_srcpad: gstreamer::GhostPad,
        flow_combiner: Arc<Mutex<UniqueFlowCombiner>>,
        has_audio_stream: Arc<AtomicBool>,
        has_video_stream: Arc<AtomicBool>,
    }

    impl ServoMediaStreamSrc {
        pub fn set_stream(
            &self,
            stream: &mut GStreamerMediaStream,
            src: &gstreamer::Element,
            only_stream: bool,
        ) -> Result<(), PlayerError> {
            // XXXferjm the current design limits the number of streams to one
            // per type. This fulfills the basic use case for WebRTC, but we should
            // implement support for multiple streams per type at some point, which
            // likely involves encoding and muxing all streams of the same type
            // in a single stream.

            gstreamer::log!(self.cat, "Setting stream");

            // Append a proxysink to the media stream pipeline.
            let pipeline = stream.pipeline_or_new();
            let last_element = stream.encoded().map_err(|_| PlayerError::SetStreamFailed)?;
            let sink = gstreamer::ElementFactory::make("proxysink")
                .build()
                .map_err(|_| PlayerError::SetStreamFailed)?;
            pipeline
                .add(&sink)
                .map_err(|_| PlayerError::SetStreamFailed)?;
            gstreamer::Element::link_many(&[&last_element, &sink][..])
                .map_err(|_| PlayerError::SetStreamFailed)?;

            // Create the appropriate proxysrc depending on the stream type
            // and connect the media stream proxysink to it.
            self.setup_proxy_src(stream.ty(), &sink, src, only_stream);

            sink.sync_state_with_parent()
                .map_err(|_| PlayerError::SetStreamFailed)?;
            pipeline
                .set_state(gstreamer::State::Playing)
                .map_err(|_| PlayerError::SetStreamFailed)?;

            Ok(())
        }

        fn setup_proxy_src(
            &self,
            stream_type: MediaStreamType,
            sink: &gstreamer::Element,
            src: &gstreamer::Element,
            only_stream: bool,
        ) {
            let (proxysrc, src_pad, no_more_pads) = match stream_type {
                MediaStreamType::Audio => {
                    self.has_audio_stream.store(true, Ordering::Relaxed);
                    (
                        &self.audio_proxysrc,
                        &self.audio_srcpad,
                        self.has_video_stream.load(Ordering::Relaxed),
                    )
                },
                MediaStreamType::Video => {
                    self.has_video_stream.store(true, Ordering::Relaxed);
                    (
                        &self.video_proxysrc,
                        &self.video_srcpad,
                        self.has_audio_stream.load(Ordering::Relaxed),
                    )
                },
            };
            proxysrc.set_property("proxysink", sink);

            // Add proxysrc to bin
            let bin = src.downcast_ref::<gstreamer::Bin>().unwrap();
            bin.add(proxysrc)
                .expect("Could not add proxysrc element to bin");

            let target_pad = proxysrc
                .static_pad("src")
                .expect("Could not get proxysrc's static src pad");
            src_pad
                .set_target(Some(&target_pad))
                .expect("Could not set target pad");

            src.add_pad(src_pad)
                .expect("Could not add source pad to media stream src");
            src.set_element_flags(gstreamer::ElementFlags::SOURCE);

            let proxy_pad = src_pad.internal().unwrap();
            src_pad.set_active(true).expect("Could not active pad");
            self.flow_combiner.lock().unwrap().add_pad(&proxy_pad);

            src.sync_state_with_parent().unwrap();

            if no_more_pads || only_stream {
                src.no_more_pads();
            }
        }
    }

    // Basic declaration of our type for the GObject type system.
    #[glib::object_subclass]
    impl ObjectSubclass for ServoMediaStreamSrc {
        const NAME: &'static str = "ServoMediaStreamSrc";
        type Type = super::ServoMediaStreamSrc;
        type ParentType = gstreamer::Bin;
        type Interfaces = (gstreamer::URIHandler,);

        // Called once at the very beginning of instantiation of each instance and
        // creates the data structure that contains all our state
        fn with_class(_klass: &Self::Class) -> Self {
            let flow_combiner = Arc::new(Mutex::new(UniqueFlowCombiner::new()));

            fn create_ghost_pad_with_template(
                name: &str,
                pad_template: &gstreamer::PadTemplate,
                flow_combiner: Arc<Mutex<UniqueFlowCombiner>>,
            ) -> gstreamer::GhostPad {
                gstreamer::GhostPad::builder_from_template(pad_template)
                    .name(name)
                    .chain_function({
                        move |pad, parent, buffer| {
                            let chain_result =
                                gstreamer::ProxyPad::chain_default(pad, parent, buffer);
                            let result = flow_combiner
                                .lock()
                                .unwrap()
                                .update_pad_flow(pad, chain_result);
                            if result == Err(gstreamer::FlowError::Flushing) {
                                return chain_result;
                            }
                            result
                        }
                    })
                    .build()
            }

            let audio_proxysrc = gstreamer::ElementFactory::make("proxysrc")
                .build()
                .expect("Could not create proxysrc element");
            let audio_srcpad = create_ghost_pad_with_template(
                "audio_src",
                &AUDIO_SRC_PAD_TEMPLATE,
                flow_combiner.clone(),
            );

            let video_proxysrc = gstreamer::ElementFactory::make("proxysrc")
                .build()
                .expect("Could not create proxysrc element");
            let video_srcpad = create_ghost_pad_with_template(
                "video_src",
                &VIDEO_SRC_PAD_TEMPLATE,
                flow_combiner.clone(),
            );

            Self {
                cat: gstreamer::DebugCategory::new(
                    "servomediastreamsrc",
                    gstreamer::DebugColorFlags::empty(),
                    Some("Servo media stream source"),
                ),
                audio_proxysrc,
                audio_srcpad,
                video_proxysrc,
                video_srcpad,
                flow_combiner,
                has_video_stream: Arc::new(AtomicBool::new(false)),
                has_audio_stream: Arc::new(AtomicBool::new(false)),
            }
        }
    }

    // The ObjectImpl trait provides the setters/getters for GObject properties.
    // Here we need to provide the values that are internally stored back to the
    // caller, or store whatever new value the caller is providing.
    //
    // This maps between the GObject properties and our internal storage of the
    // corresponding values of the properties.
    impl ObjectImpl for ServoMediaStreamSrc {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    // Let playbin3 know we are a live source.
                    glib::ParamSpecBoolean::builder("is-live")
                        .nick("Is Live")
                        .blurb("Let playbin3 know we are a live source")
                        .default_value(true)
                        .readwrite()
                        .build(),
                ]
            });

            &PROPERTIES
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "is-live" => true.to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl GstObjectImpl for ServoMediaStreamSrc {}

    // Implementation of gstreamer::Element virtual methods
    impl ElementImpl for ServoMediaStreamSrc {
        fn metadata() -> Option<&'static gstreamer::subclass::ElementMetadata> {
            static ELEMENT_METADATA: Lazy<gstreamer::subclass::ElementMetadata> = Lazy::new(|| {
                gstreamer::subclass::ElementMetadata::new(
                    "Servo Media Stream Source",
                    "Source/Audio/Video",
                    "Feed player with media stream data",
                    "Servo developers",
                )
            });

            Some(&*ELEMENT_METADATA)
        }

        fn pad_templates() -> &'static [gstreamer::PadTemplate] {
            static PAD_TEMPLATES: Lazy<Vec<gstreamer::PadTemplate>> = Lazy::new(|| {
                // Add pad templates for our audio and video source pads.
                // These are later used for actually creating the pads and beforehand
                // already provide information to GStreamer about all possible
                // pads that could exist for this type.
                vec![
                    AUDIO_SRC_PAD_TEMPLATE.clone(),
                    VIDEO_SRC_PAD_TEMPLATE.clone(),
                ]
            });

            PAD_TEMPLATES.as_ref()
        }
    }

    // Implementation of gstreamer::Bin virtual methods
    impl BinImpl for ServoMediaStreamSrc {}

    impl URIHandlerImpl for ServoMediaStreamSrc {
        const URI_TYPE: gstreamer::URIType = gstreamer::URIType::Src;

        fn protocols() -> &'static [&'static str] {
            &["mediastream"]
        }

        fn uri(&self) -> Option<String> {
            Some("mediastream://".to_string())
        }

        fn set_uri(&self, uri: &str) -> Result<(), glib::Error> {
            if let Ok(uri) = Url::parse(uri) {
                if uri.scheme() == "mediastream" {
                    return Ok(());
                }
            }
            Err(glib::Error::new(
                gstreamer::URIError::BadUri,
                format!("Invalid URI '{:?}'", uri,).as_str(),
            ))
        }
    }
}

// Public part of the ServoMediaStreamSrc type. This behaves like a normal
// GObject binding
glib::wrapper! {
    pub struct ServoMediaStreamSrc(ObjectSubclass<imp::ServoMediaStreamSrc>)
        @extends gstreamer::Bin, gstreamer::Element, gstreamer::Object, @implements gstreamer::URIHandler;
}

unsafe impl Send for ServoMediaStreamSrc {}
unsafe impl Sync for ServoMediaStreamSrc {}

impl ServoMediaStreamSrc {
    pub fn set_stream(
        &self,
        stream: &mut GStreamerMediaStream,
        only_stream: bool,
    ) -> Result<(), PlayerError> {
        self.imp()
            .set_stream(stream, self.upcast_ref::<gstreamer::Element>(), only_stream)
    }
}

// Registers the type for our element, and then registers in GStreamer
// under the name "servomediastreamsrc" for being able to instantiate it via e.g.
// gstreamer::ElementFactory::make().
pub fn register_servo_media_stream_src() -> Result<(), glib::BoolError> {
    gstreamer::Element::register(
        None,
        "servomediastreamsrc",
        gstreamer::Rank::NONE,
        ServoMediaStreamSrc::static_type(),
    )
}
