/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use mime::{self, Mime};
use net_traits::LoadContext;

pub struct MimeClassifier {
    image_classifier: GroupedClassifier,
    audio_video_classifier: GroupedClassifier,
    scriptable_classifier: GroupedClassifier,
    plaintext_classifier: GroupedClassifier,
    archive_classifier: GroupedClassifier,
    binary_or_plaintext: BinaryOrPlaintextClassifier,
    feeds_classifier: FeedsClassifier,
    font_classifier: GroupedClassifier,
}

pub enum MediaType {
    Xml,
    Html,
    AudioVideo,
    Image,
}

pub enum ApacheBugFlag {
    On,
    Off,
}

impl ApacheBugFlag {
    /// <https://mimesniff.spec.whatwg.org/#supplied-mime-type-detection-algorithm>
    pub fn from_content_type(last_raw_content_type: &[u8]) -> ApacheBugFlag {
        if last_raw_content_type == b"text/plain" ||
            last_raw_content_type == b"text/plain; charset=ISO-8859-1" ||
            last_raw_content_type == b"text/plain; charset=iso-8859-1" ||
            last_raw_content_type == b"text/plain; charset=UTF-8"
        {
            ApacheBugFlag::On
        } else {
            ApacheBugFlag::Off
        }
    }
}

#[derive(PartialEq)]
pub enum NoSniffFlag {
    On,
    Off,
}

impl Default for MimeClassifier {
    fn default() -> Self {
        Self {
            image_classifier: GroupedClassifier::image_classifer(),
            audio_video_classifier: GroupedClassifier::audio_video_classifier(),
            scriptable_classifier: GroupedClassifier::scriptable_classifier(),
            plaintext_classifier: GroupedClassifier::plaintext_classifier(),
            archive_classifier: GroupedClassifier::archive_classifier(),
            binary_or_plaintext: BinaryOrPlaintextClassifier,
            feeds_classifier: FeedsClassifier,
            font_classifier: GroupedClassifier::font_classifier(),
        }
    }
}

impl MimeClassifier {
    //Performs MIME Type Sniffing Algorithm (sections 7 and 8)
    pub fn classify<'a>(
        &'a self,
        context: LoadContext,
        no_sniff_flag: NoSniffFlag,
        apache_bug_flag: ApacheBugFlag,
        supplied_type: &Option<Mime>,
        data: &'a [u8],
    ) -> Mime {
        let supplied_type_or_octet_stream = supplied_type
            .clone()
            .unwrap_or(mime::APPLICATION_OCTET_STREAM);
        match context {
            LoadContext::Browsing => match *supplied_type {
                None => self.sniff_unknown_type(no_sniff_flag, data),
                Some(ref supplied_type) => {
                    if MimeClassifier::is_explicit_unknown(supplied_type) {
                        self.sniff_unknown_type(no_sniff_flag, data)
                    } else {
                        match no_sniff_flag {
                            NoSniffFlag::On => supplied_type.clone(),
                            NoSniffFlag::Off => match apache_bug_flag {
                                ApacheBugFlag::On => self.sniff_text_or_data(data),
                                ApacheBugFlag::Off => {
                                    match MimeClassifier::get_media_type(supplied_type) {
                                        Some(MediaType::Html) => {
                                            self.feeds_classifier.classify(data)
                                        },
                                        Some(MediaType::Image) => {
                                            self.image_classifier.classify(data)
                                        },
                                        Some(MediaType::AudioVideo) => {
                                            self.audio_video_classifier.classify(data)
                                        },
                                        Some(MediaType::Xml) | None => None,
                                    }
                                    .unwrap_or(supplied_type.clone())
                                },
                            },
                        }
                    }
                },
            },
            LoadContext::Image => {
                // Section 8.2 Sniffing an image context
                match MimeClassifier::maybe_get_media_type(supplied_type) {
                    Some(MediaType::Xml) => None,
                    _ => self.image_classifier.classify(data),
                }
                .unwrap_or(supplied_type_or_octet_stream)
            },
            LoadContext::AudioVideo => {
                // Section 8.3 Sniffing an image context
                match MimeClassifier::maybe_get_media_type(supplied_type) {
                    Some(MediaType::Xml) => None,
                    _ => self.audio_video_classifier.classify(data),
                }
                .unwrap_or(supplied_type_or_octet_stream)
            },
            LoadContext::Plugin => {
                // 8.4 Sniffing in a plugin context
                //
                // This section was *not* finalized in the specs at the time
                // of this implementation.
                match *supplied_type {
                    None => mime::APPLICATION_OCTET_STREAM,
                    _ => supplied_type_or_octet_stream,
                }
            },
            LoadContext::Style => {
                // 8.5 Sniffing in a style context
                //
                // This section was *not* finalized in the specs at the time
                // of this implementation.
                match *supplied_type {
                    None => mime::TEXT_CSS,
                    _ => supplied_type_or_octet_stream,
                }
            },
            LoadContext::Script => {
                // 8.6 Sniffing in a script context
                //
                // This section was *not* finalized in the specs at the time
                // of this implementation.
                match *supplied_type {
                    None => mime::TEXT_JAVASCRIPT,
                    _ => supplied_type_or_octet_stream,
                }
            },
            LoadContext::Font => {
                // 8.7 Sniffing in a font context
                match MimeClassifier::maybe_get_media_type(supplied_type) {
                    Some(MediaType::Xml) => None,
                    _ => self.font_classifier.classify(data),
                }
                .unwrap_or(supplied_type_or_octet_stream)
            },
            LoadContext::TextTrack => {
                // 8.8 Sniffing in a text track context
                //
                // This section was *not* finalized in the specs at the time
                // of this implementation.
                "text/vtt".parse().unwrap()
            },
            LoadContext::CacheManifest => {
                // 8.9 Sniffing in a cache manifest context
                //
                // This section was *not* finalized in the specs at the time
                // of this implementation.
                "text/cache-manifest".parse().unwrap()
            },
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        self.image_classifier.validate()?;
        self.audio_video_classifier.validate()?;
        self.scriptable_classifier.validate()?;
        self.plaintext_classifier.validate()?;
        self.archive_classifier.validate()?;
        self.binary_or_plaintext.validate()?;
        self.feeds_classifier.validate()?;
        self.font_classifier.validate()?;
        Ok(())
    }

    //some sort of iterator over the classifiers might be better?
    fn sniff_unknown_type(&self, no_sniff_flag: NoSniffFlag, data: &[u8]) -> Mime {
        let should_sniff_scriptable = no_sniff_flag == NoSniffFlag::Off;
        let sniffed = if should_sniff_scriptable {
            self.scriptable_classifier.classify(data)
        } else {
            None
        };

        sniffed
            .or_else(|| self.plaintext_classifier.classify(data))
            .or_else(|| self.image_classifier.classify(data))
            .or_else(|| self.audio_video_classifier.classify(data))
            .or_else(|| self.archive_classifier.classify(data))
            .or_else(|| self.binary_or_plaintext.classify(data))
            .expect("BinaryOrPlaintextClassifier always succeeds")
    }

    fn sniff_text_or_data<'a>(&'a self, data: &'a [u8]) -> Mime {
        self.binary_or_plaintext
            .classify(data)
            .expect("BinaryOrPlaintextClassifier always succeeds")
    }

    fn is_xml(mt: &Mime) -> bool {
        mt.suffix() == Some(mime::XML) ||
            (mt.type_() == mime::APPLICATION && mt.subtype() == mime::XML) ||
            (mt.type_() == mime::TEXT && mt.subtype() == mime::XML)
    }

    fn is_html(mt: &Mime) -> bool {
        mt.type_() == mime::TEXT && mt.subtype() == mime::HTML
    }

    fn is_image(mt: &Mime) -> bool {
        mt.type_() == mime::IMAGE
    }

    fn is_audio_video(mt: &Mime) -> bool {
        mt.type_() == mime::AUDIO ||
            mt.type_() == mime::VIDEO ||
            mt.type_() == mime::APPLICATION && mt.subtype() == mime::OGG
    }

    fn is_explicit_unknown(mt: &Mime) -> bool {
        mt.type_().as_str() == "unknown" && mt.subtype().as_str() == "unknown" ||
            mt.type_() == mime::APPLICATION && mt.subtype().as_str() == "unknown" ||
            mt.type_() == mime::STAR && mt.subtype() == mime::STAR
    }

    fn get_media_type(mime: &Mime) -> Option<MediaType> {
        if MimeClassifier::is_xml(mime) {
            Some(MediaType::Xml)
        } else if MimeClassifier::is_html(mime) {
            Some(MediaType::Html)
        } else if MimeClassifier::is_image(mime) {
            Some(MediaType::Image)
        } else if MimeClassifier::is_audio_video(mime) {
            Some(MediaType::AudioVideo)
        } else {
            None
        }
    }

    fn maybe_get_media_type(supplied_type: &Option<Mime>) -> Option<MediaType> {
        supplied_type
            .as_ref()
            .and_then(MimeClassifier::get_media_type)
    }
}

//Interface used for composite types
trait MIMEChecker {
    fn classify(&self, data: &[u8]) -> Option<Mime>;
    /// Validate the MIME checker configuration
    fn validate(&self) -> Result<(), String>;
}

trait Matches {
    fn matches(&mut self, matches: &[u8]) -> bool;
}

impl<'a, T: Iterator<Item = &'a u8> + Clone> Matches for T {
    // Matching function that works on an iterator.
    // see if the next matches.len() bytes in data_iterator equal matches
    // move iterator and return true or just return false
    //
    // Params
    // self: an iterator
    // matches: a vector of bytes to match
    //
    // Return
    // true if the next n elements of self match n elements of matches
    // false otherwise
    //
    // Side effects
    // moves the iterator when match is found
    fn matches(&mut self, matches: &[u8]) -> bool {
        if self.clone().nth(matches.len()).is_none() {
            // there are less than matches.len() elements in self
            return false;
        }
        let result = self.clone().zip(matches).all(|(s, m)| *s == *m);
        if result {
            self.nth(matches.len());
        }
        result
    }
}

struct ByteMatcher {
    pattern: &'static [u8],
    mask: &'static [u8],
    leading_ignore: &'static [u8],
    content_type: Mime,
}

impl ByteMatcher {
    fn matches(&self, data: &[u8]) -> Option<usize> {
        if data.len() < self.pattern.len() {
            None
        } else if data == self.pattern {
            Some(self.pattern.len())
        } else {
            data[..data.len() - self.pattern.len() + 1]
                .iter()
                .position(|x| !self.leading_ignore.contains(x))
                .and_then(|start| {
                    if data[start..]
                        .iter()
                        .zip(self.pattern.iter())
                        .zip(self.mask.iter())
                        .all(|((&data, &pattern), &mask)| (data & mask) == pattern)
                    {
                        Some(start + self.pattern.len())
                    } else {
                        None
                    }
                })
        }
    }
}

impl MIMEChecker for ByteMatcher {
    fn classify(&self, data: &[u8]) -> Option<Mime> {
        self.matches(data).map(|_| self.content_type.clone())
    }

    fn validate(&self) -> Result<(), String> {
        if self.pattern.is_empty() {
            return Err(format!("Zero length pattern for {:?}", self.content_type));
        }
        if self.pattern.len() != self.mask.len() {
            return Err(format!(
                "Unequal pattern and mask length for {:?}",
                self.content_type
            ));
        }
        if self
            .pattern
            .iter()
            .zip(self.mask.iter())
            .any(|(&pattern, &mask)| pattern & mask != pattern)
        {
            return Err(format!(
                "Pattern not pre-masked for {:?}",
                self.content_type
            ));
        }
        Ok(())
    }
}

struct TagTerminatedByteMatcher {
    matcher: ByteMatcher,
}

impl MIMEChecker for TagTerminatedByteMatcher {
    fn classify(&self, data: &[u8]) -> Option<Mime> {
        self.matcher.matches(data).and_then(|j| {
            if j < data.len() && (data[j] == b' ' || data[j] == b'>') {
                Some(self.matcher.content_type.clone())
            } else {
                None
            }
        })
    }

    fn validate(&self) -> Result<(), String> {
        self.matcher.validate()
    }
}

pub struct Mp4Matcher;

impl Mp4Matcher {
    pub fn matches(&self, data: &[u8]) -> bool {
        if data.len() < 12 {
            return false;
        }

        let box_size = ((data[0] as u32) << 24 |
            (data[1] as u32) << 16 |
            (data[2] as u32) << 8 |
            (data[3] as u32)) as usize;
        if (data.len() < box_size) || (box_size % 4 != 0) {
            return false;
        }

        let ftyp = [0x66, 0x74, 0x79, 0x70];
        if !data[4..].starts_with(&ftyp) {
            return false;
        }

        let mp4 = [0x6D, 0x70, 0x34];
        data[8..].starts_with(&mp4) ||
            data[16..box_size]
                .chunks(4)
                .any(|chunk| chunk.starts_with(&mp4))
    }
}
impl MIMEChecker for Mp4Matcher {
    fn classify(&self, data: &[u8]) -> Option<Mime> {
        if self.matches(data) {
            Some("video/mp4".parse().unwrap())
        } else {
            None
        }
    }

    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

struct BinaryOrPlaintextClassifier;

impl BinaryOrPlaintextClassifier {
    fn classify_impl(&self, data: &[u8]) -> Mime {
        if data.starts_with(&[0xFFu8, 0xFEu8]) ||
            data.starts_with(&[0xFEu8, 0xFFu8]) ||
            data.starts_with(&[0xEFu8, 0xBBu8, 0xBFu8])
        {
            mime::TEXT_PLAIN
        } else if data.iter().any(|&x| {
            x <= 0x08u8 ||
                x == 0x0Bu8 ||
                (0x0Eu8..=0x1Au8).contains(&x) ||
                (0x1Cu8..=0x1Fu8).contains(&x)
        }) {
            mime::APPLICATION_OCTET_STREAM
        } else {
            mime::TEXT_PLAIN
        }
    }
}
impl MIMEChecker for BinaryOrPlaintextClassifier {
    fn classify(&self, data: &[u8]) -> Option<Mime> {
        Some(self.classify_impl(data))
    }

    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}
struct GroupedClassifier {
    byte_matchers: Vec<Box<dyn MIMEChecker + Send + Sync>>,
}
impl GroupedClassifier {
    fn image_classifer() -> GroupedClassifier {
        GroupedClassifier {
            byte_matchers: vec![
                // Keep this in sync with 'is_supported_mime_type' from
                // components/style/servo/media_queries.rs
                Box::new(ByteMatcher::image_x_icon()),
                Box::new(ByteMatcher::image_x_icon_cursor()),
                Box::new(ByteMatcher::image_bmp()),
                Box::new(ByteMatcher::image_gif89a()),
                Box::new(ByteMatcher::image_gif87a()),
                Box::new(ByteMatcher::image_webp()),
                Box::new(ByteMatcher::image_png()),
                Box::new(ByteMatcher::image_jpeg()),
            ],
        }
    }
    fn audio_video_classifier() -> GroupedClassifier {
        GroupedClassifier {
            byte_matchers: vec![
                Box::new(ByteMatcher::video_webm()),
                Box::new(ByteMatcher::audio_basic()),
                Box::new(ByteMatcher::audio_aiff()),
                Box::new(ByteMatcher::audio_mpeg()),
                Box::new(ByteMatcher::application_ogg()),
                Box::new(ByteMatcher::audio_midi()),
                Box::new(ByteMatcher::video_avi()),
                Box::new(ByteMatcher::audio_wave()),
                Box::new(Mp4Matcher),
            ],
        }
    }
    fn scriptable_classifier() -> GroupedClassifier {
        GroupedClassifier {
            byte_matchers: vec![
                Box::new(ByteMatcher::text_html_doctype()),
                Box::new(ByteMatcher::text_html_page()),
                Box::new(ByteMatcher::text_html_head()),
                Box::new(ByteMatcher::text_html_script()),
                Box::new(ByteMatcher::text_html_iframe()),
                Box::new(ByteMatcher::text_html_h1()),
                Box::new(ByteMatcher::text_html_div()),
                Box::new(ByteMatcher::text_html_font()),
                Box::new(ByteMatcher::text_html_table()),
                Box::new(ByteMatcher::text_html_a()),
                Box::new(ByteMatcher::text_html_style()),
                Box::new(ByteMatcher::text_html_title()),
                Box::new(ByteMatcher::text_html_b()),
                Box::new(ByteMatcher::text_html_body()),
                Box::new(ByteMatcher::text_html_br()),
                Box::new(ByteMatcher::text_html_p()),
                Box::new(ByteMatcher::text_html_comment()),
                Box::new(ByteMatcher::text_xml()),
                Box::new(ByteMatcher::application_pdf()),
            ],
        }
    }
    fn plaintext_classifier() -> GroupedClassifier {
        GroupedClassifier {
            byte_matchers: vec![
                Box::new(ByteMatcher::text_plain_utf_8_bom()),
                Box::new(ByteMatcher::text_plain_utf_16le_bom()),
                Box::new(ByteMatcher::text_plain_utf_16be_bom()),
                Box::new(ByteMatcher::application_postscript()),
            ],
        }
    }
    fn archive_classifier() -> GroupedClassifier {
        GroupedClassifier {
            byte_matchers: vec![
                Box::new(ByteMatcher::application_x_gzip()),
                Box::new(ByteMatcher::application_zip()),
                Box::new(ByteMatcher::application_x_rar_compressed()),
            ],
        }
    }

    fn font_classifier() -> GroupedClassifier {
        GroupedClassifier {
            byte_matchers: vec![
                Box::new(ByteMatcher::application_font_woff()),
                Box::new(ByteMatcher::true_type_collection()),
                Box::new(ByteMatcher::open_type()),
                Box::new(ByteMatcher::true_type()),
                Box::new(ByteMatcher::application_vnd_ms_font_object()),
            ],
        }
    }
}
impl MIMEChecker for GroupedClassifier {
    fn classify(&self, data: &[u8]) -> Option<Mime> {
        self.byte_matchers
            .iter()
            .filter_map(|matcher| matcher.classify(data))
            .next()
    }

    fn validate(&self) -> Result<(), String> {
        for byte_matcher in &self.byte_matchers {
            byte_matcher.validate()?
        }
        Ok(())
    }
}

enum Match {
    None,
    Start,
    StartAndEnd,
}

impl Match {
    fn chain<F: FnOnce() -> Match>(self, f: F) -> Match {
        if let Match::None = self {
            return f();
        }
        self
    }
}

fn eats_until<'a, T>(matcher: &mut T, start: &[u8], end: &[u8]) -> Match
where
    T: Iterator<Item = &'a u8> + Clone,
{
    if !matcher.matches(start) {
        Match::None
    } else if end.len() == 1 {
        if matcher.any(|&x| x == end[0]) {
            Match::StartAndEnd
        } else {
            Match::Start
        }
    } else {
        while !matcher.matches(end) {
            if matcher.next().is_none() {
                return Match::Start;
            }
        }
        Match::StartAndEnd
    }
}

struct FeedsClassifier;
impl FeedsClassifier {
    // Implements sniffing for mislabeled feeds (https://mimesniff.spec.whatwg.org/#sniffing-a-mislabeled-feed)
    fn classify_impl(&self, data: &[u8]) -> Option<Mime> {
        // Step 4: can not be feed unless length is > 3
        if data.len() < 3 {
            return None;
        }

        let mut matcher = data.iter();

        // eat the first three acceptable byte sequences if they are equal to UTF-8 BOM
        let utf8_bom = &[0xEFu8, 0xBBu8, 0xBFu8];
        matcher.matches(utf8_bom);

        // continuously search for next "<" until end of matcher
        // TODO: need max_bytes to prevent inadvertently examining html document
        //       eg. an html page with a feed example
        loop {
            if !matcher.any(|x| *x == b'<') {
                return None;
            }

            // Steps 5.2.1 to 5.2.4
            match eats_until(&mut matcher, b"?", b"?>")
                .chain(|| eats_until(&mut matcher, b"!--", b"-->"))
                .chain(|| eats_until(&mut matcher, b"!", b">"))
            {
                Match::StartAndEnd => continue,
                Match::None => {},
                Match::Start => return None,
            }

            // Step 5.2.5
            if matcher.matches(b"rss") {
                return Some("application/rss+xml".parse().unwrap());
            }
            // Step 5.2.6
            if matcher.matches(b"feed") {
                return Some("application/atom+xml".parse().unwrap());
            }
            // Step 5.2.7
            if matcher.matches(b"rdf:RDF") {
                while matcher.next().is_some() {
                    match eats_until(
                        &mut matcher,
                        b"http://purl.org/rss/1.0/",
                        b"http://www.w3.org/1999/02/22-rdf-syntax-ns#",
                    )
                    .chain(|| {
                        eats_until(
                            &mut matcher,
                            b"http://www.w3.org/1999/02/22-rdf-syntax-ns#",
                            b"http://purl.org/rss/1.0/",
                        )
                    }) {
                        Match::StartAndEnd => return Some("application/rss+xml".parse().unwrap()),
                        Match::None => {},
                        Match::Start => return None,
                    }
                }
                return None;
            }
        }
    }
}

impl MIMEChecker for FeedsClassifier {
    fn classify(&self, data: &[u8]) -> Option<Mime> {
        self.classify_impl(data)
    }

    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

//Contains hard coded byte matchers
//TODO: These should be configured and not hard coded
impl ByteMatcher {
    //A Windows Icon signature
    fn image_x_icon() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\x00\x00\x01\x00",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "image/x-icon".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    //A Windows Cursor signature.
    fn image_x_icon_cursor() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\x00\x00\x02\x00",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "image/x-icon".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    //The string "BM", a BMP signature.
    fn image_bmp() -> ByteMatcher {
        ByteMatcher {
            pattern: b"BM",
            mask: b"\xFF\xFF",
            content_type: mime::IMAGE_BMP,
            leading_ignore: &[],
        }
    }
    //The string "GIF89a", a GIF signature.
    fn image_gif89a() -> ByteMatcher {
        ByteMatcher {
            pattern: b"GIF89a",
            mask: b"\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: mime::IMAGE_GIF,
            leading_ignore: &[],
        }
    }
    //The string "GIF87a", a GIF signature.
    fn image_gif87a() -> ByteMatcher {
        ByteMatcher {
            pattern: b"GIF87a",
            mask: b"\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: mime::IMAGE_GIF,
            leading_ignore: &[],
        }
    }
    //The string "RIFF" followed by four bytes followed by the string "WEBPVP".
    fn image_webp() -> ByteMatcher {
        ByteMatcher {
            pattern: b"RIFF\x00\x00\x00\x00WEBPVP",
            mask: b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: "image/webp".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    //An error-checking byte followed by the string "PNG" followed by CR LF SUB LF, the PNG
    //signature.
    fn image_png() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\x89PNG\r\n\x1A\n",
            mask: b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: mime::IMAGE_PNG,
            leading_ignore: &[],
        }
    }
    // The JPEG Start of Image marker followed by the indicator byte of another marker.
    fn image_jpeg() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\xFF\xD8\xFF",
            mask: b"\xFF\xFF\xFF",
            content_type: mime::IMAGE_JPEG,
            leading_ignore: &[],
        }
    }
    //The WebM signature. [TODO: Use more bytes?]
    fn video_webm() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\x1A\x45\xDF\xA3",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "video/webm".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    //The string ".snd", the basic audio signature.
    fn audio_basic() -> ByteMatcher {
        ByteMatcher {
            pattern: b".snd",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "audio/basic".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    //The string "FORM" followed by four bytes followed by the string "AIFF", the AIFF signature.
    fn audio_aiff() -> ByteMatcher {
        ByteMatcher {
            pattern: b"FORM\x00\x00\x00\x00AIFF",
            mask: b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF",
            content_type: "audio/aiff".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    //The string "ID3", the ID3v2-tagged MP3 signature.
    fn audio_mpeg() -> ByteMatcher {
        ByteMatcher {
            pattern: b"ID3",
            mask: b"\xFF\xFF\xFF",
            content_type: "audio/mpeg".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    //The string "OggS" followed by NUL, the Ogg container signature.
    fn application_ogg() -> ByteMatcher {
        ByteMatcher {
            pattern: b"OggS\x00",
            mask: b"\xFF\xFF\xFF\xFF\xFF",
            content_type: "application/ogg".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    //The string "MThd" followed by four bytes representing the number 6 in 32 bits (big-endian),
    //the MIDI signature.
    fn audio_midi() -> ByteMatcher {
        ByteMatcher {
            pattern: b"MThd\x00\x00\x00\x06",
            mask: b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: "audio/midi".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    //The string "RIFF" followed by four bytes followed by the string "AVI ", the AVI signature.
    fn video_avi() -> ByteMatcher {
        ByteMatcher {
            pattern: b"RIFF\x00\x00\x00\x00AVI ",
            mask: b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF",
            content_type: "video/avi".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "RIFF" followed by four bytes followed by the string "WAVE", the WAVE signature.
    fn audio_wave() -> ByteMatcher {
        ByteMatcher {
            pattern: b"RIFF\x00\x00\x00\x00WAVE",
            mask: b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF",
            content_type: "audio/wave".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // doctype terminated with Tag terminating (TT) Byte
    fn text_html_doctype() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<!DOCTYPE HTML",
                mask: b"\xFF\xFF\xDF\xDF\xDF\xDF\xDF\xDF\xDF\xFF\xDF\xDF\xDF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // HTML terminated with Tag terminating (TT) Byte: 0x20 (SP)
    fn text_html_page() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<HTML",
                mask: b"\xFF\xDF\xDF\xDF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // head terminated with Tag Terminating (TT) Byte
    fn text_html_head() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<HEAD",
                mask: b"\xFF\xDF\xDF\xDF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // script terminated with Tag Terminating (TT) Byte
    fn text_html_script() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<SCRIPT",
                mask: b"\xFF\xDF\xDF\xDF\xDF\xDF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // iframe terminated with Tag Terminating (TT) Byte
    fn text_html_iframe() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<IFRAME",
                mask: b"\xFF\xDF\xDF\xDF\xDF\xDF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // h1 terminated with Tag Terminating (TT) Byte
    fn text_html_h1() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<H1",
                mask: b"\xFF\xDF\xFF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // div terminated with Tag Terminating (TT) Byte
    fn text_html_div() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<DIV",
                mask: b"\xFF\xDF\xDF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // font terminated with Tag Terminating (TT) Byte
    fn text_html_font() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<FONT",
                mask: b"\xFF\xDF\xDF\xDF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // table terminated with Tag Terminating (TT) Byte
    fn text_html_table() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<TABLE",
                mask: b"\xFF\xDF\xDF\xDF\xDF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // a terminated with Tag Terminating (TT) Byte
    fn text_html_a() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<A",
                mask: b"\xFF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // style terminated with Tag Terminating (TT) Byte
    fn text_html_style() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<STYLE",
                mask: b"\xFF\xDF\xDF\xDF\xDF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // title terminated with Tag Terminating (TT) Byte
    fn text_html_title() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<TITLE",
                mask: b"\xFF\xDF\xDF\xDF\xDF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // b terminated with Tag Terminating (TT) Byte
    fn text_html_b() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<B",
                mask: b"\xFF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // body terminated with Tag Terminating (TT) Byte
    fn text_html_body() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<BODY",
                mask: b"\xFF\xDF\xDF\xDF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // br terminated with Tag Terminating (TT) Byte
    fn text_html_br() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<BR",
                mask: b"\xFF\xDF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // p terminated with Tag Terminating (TT) Byte
    fn text_html_p() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<P",
                mask: b"\xFF\xDF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    // comment terminated with Tag Terminating (TT) Byte
    fn text_html_comment() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<!--",
                mask: b"\xFF\xFF\xFF\xFF",
                content_type: mime::TEXT_HTML,
                leading_ignore: b"\t\n\x0C\r ",
            },
        }
    }

    //The string "<?xml".
    fn text_xml() -> ByteMatcher {
        ByteMatcher {
            pattern: b"<?xml",
            mask: b"\xFF\xFF\xFF\xFF\xFF",
            content_type: mime::TEXT_XML,
            leading_ignore: b"\t\n\x0C\r ",
        }
    }
    //The string "%PDF-", the PDF signature.
    fn application_pdf() -> ByteMatcher {
        ByteMatcher {
            pattern: b"%PDF-",
            mask: b"\xFF\xFF\xFF\xFF\xFF",
            content_type: mime::APPLICATION_PDF,
            leading_ignore: &[],
        }
    }
    //34 bytes followed by the string "LP", the Embedded OpenType signature.
    fn application_vnd_ms_font_object() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                       \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                       \x00\x00LP",
            mask: b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                    \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                    \x00\x00\xFF\xFF",
            content_type: "application/vnd.ms-fontobject".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    //4 bytes representing the version number 1.0, a TrueType signature.
    fn true_type() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\x00\x01\x00\x00",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "application/font-sfnt".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    //The string "OTTO", the OpenType signature.
    fn open_type() -> ByteMatcher {
        ByteMatcher {
            pattern: b"OTTO",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "application/font-sfnt".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "ttcf", the TrueType Collection signature.
    fn true_type_collection() -> ByteMatcher {
        ByteMatcher {
            pattern: b"ttcf",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "application/font-sfnt".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "wOFF", the Web Open Font Format signature.
    fn application_font_woff() -> ByteMatcher {
        ByteMatcher {
            pattern: b"wOFF",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "application/font-woff".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    //The GZIP archive signature.
    fn application_x_gzip() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\x1F\x8B\x08",
            mask: b"\xFF\xFF\xFF",
            content_type: "application/x-gzip".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    //The string "PK" followed by ETX EOT, the ZIP archive signature.
    fn application_zip() -> ByteMatcher {
        ByteMatcher {
            pattern: b"PK\x03\x04",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: "application/zip".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    //The string "Rar " followed by SUB BEL NUL, the RAR archive signature.
    fn application_x_rar_compressed() -> ByteMatcher {
        ByteMatcher {
            pattern: b"Rar \x1A\x07\x00",
            mask: b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: "application/x-rar-compressed".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // The string "%!PS-Adobe-", the PostScript signature.
    fn application_postscript() -> ByteMatcher {
        ByteMatcher {
            pattern: b"%!PS-Adobe-",
            mask: b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: "application/postscript".parse().unwrap(),
            leading_ignore: &[],
        }
    }
    // UTF-16BE BOM
    fn text_plain_utf_16be_bom() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\xFE\xFF\x00\x00",
            mask: b"\xFF\xFF\x00\x00",
            content_type: mime::TEXT_PLAIN,
            leading_ignore: &[],
        }
    }
    //UTF-16LE BOM
    fn text_plain_utf_16le_bom() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\xFF\xFE\x00\x00",
            mask: b"\xFF\xFF\x00\x00",
            content_type: mime::TEXT_PLAIN,
            leading_ignore: &[],
        }
    }
    //UTF-8 BOM
    fn text_plain_utf_8_bom() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\xEF\xBB\xBF\x00",
            mask: b"\xFF\xFF\xFF\x00",
            content_type: mime::TEXT_PLAIN,
            leading_ignore: &[],
        }
    }
}
