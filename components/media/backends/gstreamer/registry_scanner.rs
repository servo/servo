/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;
use std::str::FromStr;

use once_cell::sync::Lazy;

// The GStreamer registry holds the metadata of the set of plugins available in the host.
// This scanner is used to lazily analyze the registry and to provide information about
// the set of supported mime types and codecs that the backend is able to deal with.
pub static GSTREAMER_REGISTRY_SCANNER: Lazy<GStreamerRegistryScanner> =
    Lazy::new(GStreamerRegistryScanner::new);

pub struct GStreamerRegistryScanner {
    supported_mime_types: HashSet<&'static str>,
    supported_codecs: HashSet<&'static str>,
}

impl GStreamerRegistryScanner {
    fn new() -> GStreamerRegistryScanner {
        let mut registry_scanner = GStreamerRegistryScanner {
            supported_mime_types: HashSet::new(),
            supported_codecs: HashSet::new(),
        };
        registry_scanner.initialize();
        registry_scanner
    }

    pub fn is_container_type_supported(&self, container_type: &str) -> bool {
        self.supported_mime_types.contains(container_type)
    }

    fn is_codec_supported(&self, codec: &str) -> bool {
        self.supported_codecs.contains(codec)
    }

    pub fn are_all_codecs_supported(&self, codecs: &Vec<&str>) -> bool {
        codecs.iter().all(|&codec| self.is_codec_supported(codec))
    }

    fn initialize(&mut self) {
        let audio_decoder_factories = gstreamer::ElementFactory::factories_with_type(
            gstreamer::ElementFactoryType::DECODER | gstreamer::ElementFactoryType::MEDIA_AUDIO,
            gstreamer::Rank::MARGINAL,
        );
        let audio_parser_factories = gstreamer::ElementFactory::factories_with_type(
            gstreamer::ElementFactoryType::PARSER | gstreamer::ElementFactoryType::MEDIA_AUDIO,
            gstreamer::Rank::NONE,
        );
        let video_decoder_factories = gstreamer::ElementFactory::factories_with_type(
            gstreamer::ElementFactoryType::DECODER | gstreamer::ElementFactoryType::MEDIA_VIDEO,
            gstreamer::Rank::MARGINAL,
        );
        let video_parser_factories = gstreamer::ElementFactory::factories_with_type(
            gstreamer::ElementFactoryType::PARSER | gstreamer::ElementFactoryType::MEDIA_VIDEO,
            gstreamer::Rank::MARGINAL,
        );
        let demux_factories = gstreamer::ElementFactory::factories_with_type(
            gstreamer::ElementFactoryType::DEMUXER,
            gstreamer::Rank::MARGINAL,
        );

        if has_element_for_media_type(&audio_decoder_factories, "audio/mpeg, mpegversion=(int)4") {
            self.supported_mime_types.insert("audio/aac");
            self.supported_mime_types.insert("audio/mp4");
            self.supported_mime_types.insert("audio/x-m4a");
            self.supported_codecs.insert("mpeg");
            self.supported_codecs.insert("mp4a*");
        }

        let is_opus_supported =
            has_element_for_media_type(&audio_decoder_factories, "audio/x-opus");
        if is_opus_supported && has_element_for_media_type(&audio_parser_factories, "audio/x-opus")
        {
            self.supported_mime_types.insert("audio/opus");
            self.supported_codecs.insert("opus");
            self.supported_codecs.insert("x-opus");
        }

        let is_vorbis_supported =
            has_element_for_media_type(&audio_decoder_factories, "audio/x-vorbis");
        if is_vorbis_supported &&
            has_element_for_media_type(&audio_parser_factories, "audio/x-vorbis")
        {
            self.supported_codecs.insert("vorbis");
            self.supported_codecs.insert("x-vorbis");
        }

        if has_element_for_media_type(&demux_factories, "video/x-matroska") {
            let is_vp8_decoder_available =
                has_element_for_media_type(&video_decoder_factories, "video/x-vp8");
            let is_vp9_decoder_available =
                has_element_for_media_type(&video_decoder_factories, "video/x-vp9");

            if is_vp8_decoder_available || is_vp9_decoder_available {
                self.supported_mime_types.insert("video/webm");
            }

            if is_vp8_decoder_available {
                self.supported_codecs.insert("vp8");
                self.supported_codecs.insert("x-vp8");
                self.supported_codecs.insert("vp8.0");
            }

            if is_vp9_decoder_available {
                self.supported_codecs.insert("vp9");
                self.supported_codecs.insert("x-vp9");
                self.supported_codecs.insert("vp9.0");
            }

            if is_opus_supported {
                self.supported_mime_types.insert("audio/webm");
            }
        }

        let is_h264_decoder_available = has_element_for_media_type(
            &video_decoder_factories,
            "video/x-h264, profile=(string){ constrained-baseline, baseline, high }",
        );
        if is_h264_decoder_available &&
            has_element_for_media_type(&video_parser_factories, "video/x-h264")
        {
            self.supported_mime_types.insert("video/mp4");
            self.supported_mime_types.insert("video/x-m4v");
            self.supported_codecs.insert("x-h264");
            self.supported_codecs.insert("avc*");
            self.supported_codecs.insert("mp4v*");
        }

        if has_element_for_media_type(&audio_decoder_factories, "audio/midi") {
            self.supported_mime_types.insert("audio/midi");
            self.supported_mime_types.insert("audio/riff-midi");
        }

        if has_element_for_media_type(&audio_decoder_factories, "audio/x-ac3") {
            self.supported_mime_types.insert("audio/x-ac3");
        }

        if has_element_for_media_type(&audio_decoder_factories, "audio/x-flac") {
            self.supported_mime_types.insert("audio/flac");
            self.supported_mime_types.insert("audio/x-flac");
        }

        if has_element_for_media_type(&audio_decoder_factories, "audio/x-speex") {
            self.supported_mime_types.insert("audio/speex");
            self.supported_mime_types.insert("audio/x-speex");
        }

        if has_element_for_media_type(&audio_decoder_factories, "audio/x-wavpack") {
            self.supported_mime_types.insert("audio/x-wavpack");
        }

        if has_element_for_media_type(
            &video_decoder_factories,
            "video/mpeg, mpegversion=(int){1,2}, systemstream=(boolean)false",
        ) {
            self.supported_mime_types.insert("video/mpeg");
            self.supported_codecs.insert("mpeg");
        }

        if has_element_for_media_type(&video_decoder_factories, "video/x-flash-video") {
            self.supported_mime_types.insert("video/flv");
            self.supported_mime_types.insert("video/x-flv");
        }

        if has_element_for_media_type(&video_decoder_factories, "video/x-msvideocodec") {
            self.supported_mime_types.insert("video/x-msvideo");
        }

        if has_element_for_media_type(&demux_factories, "application/x-hls") {
            self.supported_mime_types
                .insert("application/vnd.apple.mpegurl");
            self.supported_mime_types.insert("application/x-mpegurl");
        }

        if has_element_for_media_type(&demux_factories, "application/x-wav") ||
            has_element_for_media_type(&demux_factories, "audio/x-wav")
        {
            self.supported_mime_types.insert("audio/wav");
            self.supported_mime_types.insert("audio/vnd.wav");
            self.supported_mime_types.insert("audio/x-wav");
            self.supported_codecs.insert("1");
        }

        if has_element_for_media_type(&demux_factories, "video/quicktime, variant=(string)3gpp") {
            self.supported_mime_types.insert("video/3gpp");
        }

        if has_element_for_media_type(&demux_factories, "application/ogg") {
            self.supported_mime_types.insert("application/ogg");

            if is_vorbis_supported {
                self.supported_mime_types.insert("audio/ogg");
                self.supported_mime_types.insert("audio/x-vorbis+ogg");
            }

            if has_element_for_media_type(&audio_decoder_factories, "audio/x-speex") {
                self.supported_mime_types.insert("audio/ogg");
                self.supported_codecs.insert("speex");
            }

            if has_element_for_media_type(&video_decoder_factories, "video/x-theora") {
                self.supported_mime_types.insert("video/ogg");
                self.supported_codecs.insert("theora");
            }
        }

        let mut is_audio_mpeg_supported = false;
        if has_element_for_media_type(
            &audio_decoder_factories,
            "audio/mpeg, mpegversion=(int)1, layer=(int)[1, 3]",
        ) {
            is_audio_mpeg_supported = true;
            self.supported_mime_types.insert("audio/mp1");
            self.supported_mime_types.insert("audio/mp3");
            self.supported_mime_types.insert("audio/x-mp3");
            self.supported_codecs.insert("audio/mp3");
        }

        if has_element_for_media_type(&audio_decoder_factories, "audio/mpeg, mpegversion=(int)2") {
            is_audio_mpeg_supported = true;
            self.supported_mime_types.insert("audio/mp2");
        }

        is_audio_mpeg_supported |= self.is_container_type_supported("video/mp4");
        if is_audio_mpeg_supported {
            self.supported_mime_types.insert("audio/mpeg");
            self.supported_mime_types.insert("audio/x-mpeg");
        }

        let is_matroska_supported =
            has_element_for_media_type(&demux_factories, "video/x-matroska");
        if is_matroska_supported {
            self.supported_mime_types.insert("video/x-matroska");

            if has_element_for_media_type(&video_decoder_factories, "video/x-vp10") {
                self.supported_mime_types.insert("video/webm");
            }
        }

        if (is_matroska_supported || self.is_container_type_supported("video/mp4")) &&
            has_element_for_media_type(&video_decoder_factories, "video/x-av1")
        {
            self.supported_codecs.insert("av01*");
        }
    }
}

fn has_element_for_media_type(
    factories: &glib::List<gstreamer::ElementFactory>,
    media_type: &str,
) -> bool {
    match gstreamer::caps::Caps::from_str(media_type) {
        Ok(caps) => {
            for factory in factories {
                if factory.can_sink_all_caps(&caps) {
                    return true;
                }
            }
            false
        },
        _ => false,
    }
}
