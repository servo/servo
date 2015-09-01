/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;

pub struct MIMEClassifier {
    image_classifier: GroupedClassifier,
    audio_video_classifer: GroupedClassifier,
    scriptable_classifier: GroupedClassifier,
    plaintext_classifier: GroupedClassifier,
    archive_classifer: GroupedClassifier,
    binary_or_plaintext: BinaryOrPlaintextClassifier,
    feeds_classifier: FeedsClassifier
}

impl MIMEClassifier {
    //Performs MIME Type Sniffing Algorithm (section 7)
    pub fn classify(&self,
                    no_sniff: bool,
                    check_for_apache_bug: bool,
                    supplied_type: &Option<(String, String)>,
                    data: &[u8]) -> Option<(String, String)> {

        match *supplied_type {
            None => self.sniff_unknown_type(!no_sniff, data),
            Some((ref media_type, ref media_subtype)) => {
                match (&**media_type, &**media_subtype) {
                    ("unknown", "unknown") |
                    ("application", "unknown") |
                    ("*", "*") => self.sniff_unknown_type(!no_sniff, data),
                    _ => {
                        if no_sniff {
                            supplied_type.clone()
                        } else if check_for_apache_bug {
                            self.sniff_text_or_data(data)
                        } else if MIMEClassifier::is_xml(media_type, media_subtype) {
                            supplied_type.clone()
                        } else if MIMEClassifier::is_html(media_type, media_subtype) {
                            //Implied in section 7.3, but flow is not clear
                            self.feeds_classifier.classify(data).or(supplied_type.clone())
                        } else {
                            match (&**media_type, &**media_subtype) {
                                ("image", _) => self.image_classifier.classify(data),
                                ("audio", _) | ("video", _) | ("application", "ogg") =>
                                    self.audio_video_classifer.classify(data),
                                _ => None
                            }.or(supplied_type.clone())
                        }
                    }
                }
            }
        }
    }

    pub fn new() -> MIMEClassifier {
         MIMEClassifier {
             image_classifier: GroupedClassifier::image_classifer(),
             audio_video_classifer: GroupedClassifier::audio_video_classifer(),
             scriptable_classifier: GroupedClassifier::scriptable_classifier(),
             plaintext_classifier: GroupedClassifier::plaintext_classifier(),
             archive_classifer: GroupedClassifier::archive_classifier(),
             binary_or_plaintext: BinaryOrPlaintextClassifier,
             feeds_classifier: FeedsClassifier
         }
    }
    //some sort of iterator over the classifiers might be better?
    fn sniff_unknown_type(&self, sniff_scriptable: bool, data: &[u8]) ->
      Option<(String, String)> {
        if sniff_scriptable {
            self.scriptable_classifier.classify(data)
        } else {
            None
        }.or_else(|| self.plaintext_classifier.classify(data))
         .or_else(|| self.image_classifier.classify(data))
         .or_else(|| self.audio_video_classifer.classify(data))
         .or_else(|| self.archive_classifer.classify(data))
         .or_else(|| self.binary_or_plaintext.classify(data))
    }

    fn sniff_text_or_data(&self, data: &[u8]) -> Option<(String, String)> {
        self.binary_or_plaintext.classify(data)
    }

    fn is_xml(tp: &str, sub_tp: &str) -> bool {
        sub_tp.ends_with("+xml") ||
        match (tp, sub_tp) {
            ("application", "xml") | ("text", "xml") => true,
            _ => false
        }
    }

    fn is_html(tp: &str, sub_tp: &str) -> bool {
        tp == "text" && sub_tp == "html"
    }
}

pub fn as_string_option(tup: Option<(&'static str, &'static str)>) -> Option<(String, String)> {
    tup.map(|(a, b)| (a.to_owned(), b.to_owned()))
}

//Interface used for composite types
trait MIMEChecker {
    fn classify(&self, data: &[u8]) -> Option<(String, String)>;
}

trait Matches {
    fn matches(&mut self, matches: &[u8]) -> bool;
}

impl <'a, T: Iterator<Item=&'a u8> + Clone> Matches for T {

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
            return false
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
    content_type: (&'static str, &'static str)
}

impl ByteMatcher {
    fn matches(&self, data: &[u8]) -> Option<usize> {
        if data.len() < self.pattern.len() {
            None
        } else if data == self.pattern {
            Some(self.pattern.len())
        } else {
            data[..data.len() - self.pattern.len()].iter()
                .position(|x| !self.leading_ignore.contains(x))
                .and_then(|start|
                    if data[start..].iter()
                        .zip(self.pattern.iter()).zip(self.mask.iter())
                        .all(|((&data, &pattern), &mask)| (data & mask) == (pattern & mask)) {
                        Some(start + self.pattern.len())
                    } else {
                        None
                    })
        }
    }
}

impl MIMEChecker for ByteMatcher {
    fn classify(&self, data: &[u8]) -> Option<(String, String)> {
        self.matches(data).map(|_| {
            (self.content_type.0.to_owned(), self.content_type.1.to_owned())
        })
    }
}

struct TagTerminatedByteMatcher {
  matcher: ByteMatcher
}

impl MIMEChecker for TagTerminatedByteMatcher {
    fn classify(&self, data: &[u8]) -> Option<(String, String)> {
        self.matcher.matches(data).and_then(|j|
            if j < data.len() && (data[j] == b' ' || data[j] == b'>') {
                Some((self.matcher.content_type.0.to_owned(),
                      self.matcher.content_type.1.to_owned()))
            } else {
                None
            })
    }
}
pub struct Mp4Matcher;

impl Mp4Matcher {
    pub fn matches(&self, data: &[u8]) -> bool {
        if data.len() < 12 {
            return false;
        }

        let box_size = ((data[0] as u32) << 3 | (data[1] as u32) << 2 |
                        (data[2] as u32) << 1 | (data[3] as u32)) as usize;
        if (data.len() < box_size) || (box_size % 4 != 0) {
            return false;
        }

        let ftyp = [0x66, 0x74, 0x79, 0x70];
        if !data[4..].starts_with(&ftyp) {
            return false;
        }

        let mp4 = [0x6D, 0x70, 0x34];
        data[8..].starts_with(&mp4) ||
        data[16..box_size].chunks(4).any(|chunk| chunk.starts_with(&mp4))
    }

}
impl MIMEChecker for Mp4Matcher {
    fn classify(&self, data: &[u8]) -> Option<(String, String)> {
        if self.matches(data) {
            Some(("video".to_owned(), "mp4".to_owned()))
        } else {
            None
        }
    }
}

struct BinaryOrPlaintextClassifier;

impl BinaryOrPlaintextClassifier {
    fn classify_impl(&self, data: &[u8]) -> (&'static str, &'static str) {
        if data == &[0xFFu8, 0xFEu8] ||
           data == &[0xFEu8, 0xFFu8] ||
           data.starts_with(&[0xEFu8, 0xBBu8, 0xBFu8])
        {
            ("text", "plain")
        } else if data.iter().any(|&x| x <= 0x08u8 ||
                                       x == 0x0Bu8 ||
                                      (x >= 0x0Eu8 && x <= 0x1Au8) ||
                                      (x >= 0x1Cu8 && x <= 0x1Fu8)) {
            ("application", "octet-stream")
        } else {
            ("text", "plain")
        }
    }
}
impl MIMEChecker for BinaryOrPlaintextClassifier {
    fn classify(&self, data: &[u8]) -> Option<(String, String)> {
        as_string_option(Some(self.classify_impl(data)))
    }
}
struct GroupedClassifier {
    byte_matchers: Vec<Box<MIMEChecker + Send + Sync>>,
}
impl GroupedClassifier {
    fn image_classifer() -> GroupedClassifier {
        GroupedClassifier {
            byte_matchers: vec![
                box ByteMatcher::image_x_icon(),
                box ByteMatcher::image_x_icon_cursor(),
                box ByteMatcher::image_bmp(),
                box ByteMatcher::image_gif89a(),
                box ByteMatcher::image_gif87a(),
                box ByteMatcher::image_webp(),
                box ByteMatcher::image_png(),
                box ByteMatcher::image_jpeg(),
            ]
        }
    }
    fn audio_video_classifer() -> GroupedClassifier {
        GroupedClassifier {
            byte_matchers: vec![
                box ByteMatcher::video_webm(),
                box ByteMatcher::audio_basic(),
                box ByteMatcher::audio_aiff(),
                box ByteMatcher::audio_mpeg(),
                box ByteMatcher::application_ogg(),
                box ByteMatcher::audio_midi(),
                box ByteMatcher::video_avi(),
                box ByteMatcher::audio_wave(),
                box Mp4Matcher
            ]
        }
    }
    fn scriptable_classifier() -> GroupedClassifier {
        GroupedClassifier {
            byte_matchers: vec![
                box ByteMatcher::text_html_doctype(),
                box ByteMatcher::text_html_page(),
                box ByteMatcher::text_html_head(),
                box ByteMatcher::text_html_script(),
                box ByteMatcher::text_html_iframe(),
                box ByteMatcher::text_html_h1(),
                box ByteMatcher::text_html_div(),
                box ByteMatcher::text_html_font(),
                box ByteMatcher::text_html_table(),
                box ByteMatcher::text_html_a(),
                box ByteMatcher::text_html_style(),
                box ByteMatcher::text_html_title(),
                box ByteMatcher::text_html_b(),
                box ByteMatcher::text_html_body(),
                box ByteMatcher::text_html_br(),
                box ByteMatcher::text_html_p(),
                box ByteMatcher::text_html_comment(),
                box ByteMatcher::text_xml(),
                box ByteMatcher::application_pdf()
            ]
        }
    }
    fn plaintext_classifier() -> GroupedClassifier {
        GroupedClassifier {
            byte_matchers: vec![
                box ByteMatcher::text_plain_utf_8_bom(),
                box ByteMatcher::text_plain_utf_16le_bom(),
                box ByteMatcher::text_plain_utf_16be_bom(),
                box ByteMatcher::application_postscript()
            ]
        }
    }
    fn archive_classifier() -> GroupedClassifier {
        GroupedClassifier {
            byte_matchers: vec![
                box ByteMatcher::application_x_gzip(),
                box ByteMatcher::application_zip(),
                box ByteMatcher::application_x_rar_compressed()
            ]
        }
    }

    // TODO: Use this in font context classifier
    #[allow(dead_code)]
    fn font_classifier() -> GroupedClassifier {
        GroupedClassifier {
            byte_matchers: vec![
                box ByteMatcher::application_font_woff(),
                box ByteMatcher::true_type_collection(),
                box ByteMatcher::open_type(),
                box ByteMatcher::true_type(),
                box ByteMatcher::application_vnd_ms_font_object(),
            ]
        }
    }
}
impl MIMEChecker for GroupedClassifier {
    fn classify(&self, data: &[u8]) -> Option<(String, String)> {
        self.byte_matchers
            .iter()
            .filter_map(|matcher| matcher.classify(data))
            .next()
    }
}

enum Match {
    Start,
    DidNotMatch,
    StartAndEnd
}

impl Match {
    fn chain<F: FnOnce() -> Match>(self, f: F) -> Match {
        if let Match::DidNotMatch = self {
            return f();
        }
        self
    }
}

fn eats_until<'a, T>(matcher: &mut T, start: &[u8], end: &[u8]) -> Match
where T: Iterator<Item=&'a u8> + Clone {
    if !matcher.matches(start) {
        Match::DidNotMatch
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
    fn classify_impl(&self, data: &[u8]) -> Option<(&'static str, &'static str)> {

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

            if matcher.find(|&x| *x == b'<').is_none() {
                return None;
            }

            // Steps 5.2.1 to 5.2.4
            match eats_until(&mut matcher, b"?", b"?>")
               .chain(|| eats_until(&mut matcher, b"!--", b"-->"))
               .chain(|| eats_until(&mut matcher, b"!", b">")) {
                Match::StartAndEnd => continue,
                Match::DidNotMatch => {},
                Match::Start       => return None
            }

            // Step 5.2.5
            if matcher.matches(b"rss") {
                return Some(("application", "rss+xml"));
            }
            // Step 5.2.6
            if matcher.matches(b"feed") {
                return Some(("application", "atom+xml"));
            }
            // Step 5.2.7
            if matcher.matches(b"rdf:RDF") {
                while matcher.next().is_some() {
                    match eats_until(&mut matcher,
                                     b"http://purl.org/rss/1.0/",
                                     b"http://www.w3.org/1999/02/22-rdf-syntax-ns#")
                       .chain(|| eats_until(&mut matcher,
                                            b"http://www.w3.org/1999/02/22-rdf-syntax-ns#",
                                            b"http://purl.org/rss/1.0/")) {
                        Match::StartAndEnd => return Some(("application", "rss+xml")),
                        Match::DidNotMatch => {},
                        Match::Start       => return None
                    }
                }
                return None;
            }
        }
    }
}

impl MIMEChecker for FeedsClassifier {
    fn classify(&self, data: &[u8]) -> Option<(String, String)> {
       as_string_option(self.classify_impl(data))
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
            content_type: ("image", "x-icon"),
            leading_ignore: &[]
        }
    }
    //A Windows Cursor signature.
    fn image_x_icon_cursor() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\x00\x00\x02\x00",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: ("image", "x-icon"),
            leading_ignore: &[]
        }
    }
    //The string "BM", a BMP signature.
    fn image_bmp() -> ByteMatcher {
        ByteMatcher {
            pattern: b"BM",
            mask: b"\xFF\xFF",
            content_type: ("image", "bmp"),
            leading_ignore: &[]
        }
    }
    //The string "GIF89a", a GIF signature.
    fn image_gif89a() -> ByteMatcher {
        ByteMatcher {
            pattern: b"GIF89a",
            mask: b"\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: ("image", "gif"),
            leading_ignore: &[]
        }
    }
    //The string "GIF87a", a GIF signature.
    fn image_gif87a() -> ByteMatcher {
        ByteMatcher {
            pattern: b"GIF87a",
            mask: b"\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: ("image", "gif"),
            leading_ignore: &[]
        }
    }
    //The string "RIFF" followed by four bytes followed by the string "WEBPVP".
    fn image_webp() -> ByteMatcher {
        ByteMatcher {
            pattern: b"RIFF\x00\x00\x00\x00WEBPVP",
            mask: b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00,\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: ("image", "webp"),
            leading_ignore: &[]
        }
    }
    //An error-checking byte followed by the string "PNG" followed by CR LF SUB LF, the PNG
    //signature.
    fn image_png() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\x89PNG\r\n\x1A\n",
            mask: b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: ("image", "png"),
            leading_ignore: &[]
        }
    }
    // The JPEG Start of Image marker followed by the indicator byte of another marker.
    fn image_jpeg() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\xFF\xD8\xFF",
            mask: b"\xFF\xFF\xFF",
            content_type: ("image", "jpeg"),
            leading_ignore: &[]
        }
    }
    //The WebM signature. [TODO: Use more bytes?]
    fn video_webm() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\x1A\x45\xDF\xA3",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: ("video", "webm"),
            leading_ignore: &[]
        }
    }
    //The string ".snd", the basic audio signature.
    fn audio_basic() -> ByteMatcher {
        ByteMatcher {
            pattern: b".snd",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: ("audio", "basic"),
            leading_ignore: &[]
        }
    }
    //The string "FORM" followed by four bytes followed by the string "AIFF", the AIFF signature.
    fn audio_aiff() -> ByteMatcher {
        ByteMatcher {
            pattern:  b"FORM\x00\x00\x00\x00AIFF",
            mask:  b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF",
            content_type: ("audio", "aiff"),
            leading_ignore: &[]
        }
    }
    //The string "ID3", the ID3v2-tagged MP3 signature.
    fn audio_mpeg() -> ByteMatcher {
        ByteMatcher {
            pattern: b"ID3",
            mask: b"\xFF\xFF\xFF",
            content_type: ("audio", "mpeg"),
            leading_ignore: &[]
        }
    }
    //The string "OggS" followed by NUL, the Ogg container signature.
    fn application_ogg() -> ByteMatcher {
        ByteMatcher {
            pattern: b"OggS",
            mask: b"\xFF\xFF\xFF\xFF\xFF",
            content_type: ("application", "ogg"),
            leading_ignore: &[]
        }
    }
    //The string "MThd" followed by four bytes representing the number 6 in 32 bits (big-endian),
    //the MIDI signature.
    fn audio_midi() -> ByteMatcher {
        ByteMatcher {
            pattern: b"MThd\x00\x00\x00\x06",
            mask: b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: ("audio", "midi"),
            leading_ignore: &[]
        }
    }
    //The string "RIFF" followed by four bytes followed by the string "AVI ", the AVI signature.
    fn video_avi() -> ByteMatcher {
        ByteMatcher {
            pattern: b"RIFF\x00\x00\x00\x00AVI ",
            mask: b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF",
            content_type: ("video", "avi"),
            leading_ignore: &[]
        }
    }
    // The string "RIFF" followed by four bytes followed by the string "WAVE", the WAVE signature.
    fn audio_wave() -> ByteMatcher {
        ByteMatcher {
            pattern: b"RIFF\x00\x00\x00\x00WAVE",
            mask: b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF",
            content_type: ("audio", "wave"),
            leading_ignore: &[]
        }
    }
    // doctype terminated with Tag terminating (TT) Byte
    fn text_html_doctype() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<!DOCTYPE HTML",
                mask: b"\xFF\xFF\xDF\xDF\xDF\xDF\xDF\xDF\xDF\xFF\xDF\xDF\xDF\xDF",
                content_type: ("text", "html"),
                leading_ignore: b"\t\n\x0C\r "
           }
        }
    }

    // HTML terminated with Tag terminating (TT) Byte: 0x20 (SP)
    fn text_html_page() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<HTML",
                mask: b"\xFF\xDF\xDF\xDF\xDF\xFF",
                content_type: ("text", "html"),
                leading_ignore: b"\t\n\x0C\r "
            }
        }
    }

    // head terminated with Tag Terminating (TT) Byte
    fn text_html_head() -> TagTerminatedByteMatcher {
         TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<HEAD",
                mask: b"\xFF\xDF\xDF\xDF\xDF",
                content_type: ("text", "html"),
                leading_ignore: b"\t\n\x0C\r "
            }
        }
    }

    // script terminated with Tag Terminating (TT) Byte
    fn text_html_script() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<SCRIPT",
                mask: b"\xFF\xDF\xDF\xDF\xDF\xDF\xDF",
                content_type: ("text", "html"),
                leading_ignore: b"\t\n\x0C\r "
            }
        }
    }

    // iframe terminated with Tag Terminating (TT) Byte
    fn text_html_iframe() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<IFRAME",
                mask: b"\xFF\xDF\xDF\xDF\xDF\xDF\xDF",
                content_type: ("text", "html"),
                leading_ignore: b"\t\n\x0C\r "
            }
        }
    }

    // h1 terminated with Tag Terminating (TT) Byte
    fn text_html_h1() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<H1",
                mask: b"\xFF\xDF\xFF",
                content_type: ("text", "html"),
                leading_ignore: b"\t\n\x0C\r "
            }
        }
    }

    // div terminated with Tag Terminating (TT) Byte
    fn text_html_div() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<DIV",
                mask: b"\xFF\xDF\xDF\xDF",
                content_type: ("text", "html"),
                leading_ignore: b"\t\n\x0C\r "
            }
        }
    }

    // font terminated with Tag Terminating (TT) Byte
    fn text_html_font() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<FONT",
                mask: b"\xFF\xDF\xDF\xDF\xDF",
                content_type: ("text", "html"),
                leading_ignore: b"\t\n\x0C\r "
            }
        }
    }

    // table terminated with Tag Terminating (TT) Byte
    fn text_html_table() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
            pattern: b"<TABLE",
            mask: b"\xFF\xDF\xDF\xDF\xDF\xDF",
            content_type: ("text", "html"),
            leading_ignore: b"\t\n\x0C\r "
            }
        }
    }

    // a terminated with Tag Terminating (TT) Byte
    fn text_html_a() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<A",
                mask: b"\xFF\xDF",
                content_type: ("text", "html"),
                leading_ignore: b"\t\n\x0C\r "
            }
        }
    }

    // style terminated with Tag Terminating (TT) Byte
    fn text_html_style() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<STYLE",
                mask: b"\xFF\xDF\xDF\xDF\xDF\xDF",
                content_type: ("text", "html"),
                leading_ignore: b"\t\n\x0C\r "
            }
        }
    }

    // title terminated with Tag Terminating (TT) Byte
    fn text_html_title() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<TITLE",
                mask: b"\xFF\xDF\xDF\xDF\xDF\xDF",
                content_type: ("text", "html"),
                leading_ignore: b"\t\n\x0C\r "
            }
        }
    }

    // b terminated with Tag Terminating (TT) Byte
    fn text_html_b() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<B",
                mask: b"\xFF\xDF",
                content_type: ("text", "html"),
                leading_ignore: b"\t\n\x0C\r "
            }
        }
    }

    // body terminated with Tag Terminating (TT) Byte
    fn text_html_body() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<BODY",
                mask: b"\xFF\xDF\xDF\xDF\xDF",
                content_type: ("text", "html"),
                leading_ignore: b"\t\n\x0C\r "
            }
        }
    }

    // br terminated with Tag Terminating (TT) Byte
    fn text_html_br() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<BR",
                mask: b"\xFF\xDF\xDF",
                content_type: ("text", "html"),
                leading_ignore: b"\t\n\x0C\r "
            }
        }
    }

    // p terminated with Tag Terminating (TT) Byte
    fn text_html_p() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<P",
                mask: b"\xFF\xDF",
                content_type: ("text", "html"),
                leading_ignore: b"\t\n\x0C\r "
            }
        }
    }

    // comment terminated with Tag Terminating (TT) Byte
    fn text_html_comment() -> TagTerminatedByteMatcher {
        TagTerminatedByteMatcher {
            matcher: ByteMatcher {
                pattern: b"<!--",
                mask: b"\xFF\xFF\xFF\xFF",
                content_type: ("text", "html"),
                leading_ignore: b"\t\n\x0C\r "
            }
        }
    }

    //The string "<?xml".
    fn text_xml() -> ByteMatcher {
        ByteMatcher {
            pattern: b"<?xml",
            mask: b"\xFF\xFF\xFF\xFF\xFF",
            content_type: ("text", "xml"),
            leading_ignore: b"\t\n\x0C\r "
        }
    }
    //The string "%PDF-", the PDF signature.
    fn application_pdf() -> ByteMatcher {
        ByteMatcher {
            pattern: b"%PDF",
            mask: b"\xFF\xFF\xFF\xFF\xFF",
            content_type: ("application", "pdf"),
            leading_ignore: &[]
        }
    }
    //34 bytes followed by the string "LP", the Embedded OpenType signature.
    // TODO: Use this in font context classifier
    #[allow(dead_code)]
    fn application_vnd_ms_font_object() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                       \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                       \x00\x00LP",
            mask: b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                    \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                    \x00\x00\xFF\xFF",
            content_type: ("application", "vnd.ms-fontobject"),
            leading_ignore: &[]
        }
    }
    //4 bytes representing the version number 1.0, a TrueType signature.
    // TODO: Use this in font context classifier
    #[allow(dead_code)]
    fn true_type() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\x00\x01\x00\x00",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: ("(TrueType)", ""),
            leading_ignore: &[]
        }
    }
    //The string "OTTO", the OpenType signature.
    // TODO: Use this in font context classifier
    #[allow(dead_code)]
    fn open_type() -> ByteMatcher {
        ByteMatcher {
            pattern: b"OTTO",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: ("(OpenType)", ""),
            leading_ignore: &[]
        }
    }
    // The string "ttcf", the TrueType Collection signature.
    // TODO: Use this in font context classifier
    #[allow(dead_code)]
    fn true_type_collection() -> ByteMatcher {
        ByteMatcher {
            pattern: b"ttcf",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: ("(TrueType Collection)", ""),
            leading_ignore: &[]
        }
    }
    // The string "wOFF", the Web Open Font Format signature.
    // TODO: Use this in font context classifier
    #[allow(dead_code)]
    fn application_font_woff() -> ByteMatcher {
        ByteMatcher {
            pattern: b"wOFF",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: ("application", "font-woff"),
            leading_ignore: &[]
        }
    }
    //The GZIP archive signature.
    fn application_x_gzip() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\x1F\x8B\x08",
            mask: b"\xFF\xFF\xFF",
            content_type: ("application", "x-gzip"),
            leading_ignore: &[]
        }
    }
    //The string "PK" followed by ETX EOT, the ZIP archive signature.
    fn application_zip() -> ByteMatcher {
        ByteMatcher {
            pattern: b"PK\x03\x04",
            mask: b"\xFF\xFF\xFF\xFF",
            content_type: ("application", "zip"),
            leading_ignore: &[]
        }
    }
    //The string "Rar " followed by SUB BEL NUL, the RAR archive signature.
    fn application_x_rar_compressed() -> ByteMatcher {
        ByteMatcher {
            pattern: b"Rar \x1A\x07\x00",
            mask: b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: ("application", "x-rar-compressed"),
            leading_ignore: &[]
        }
    }
    // The string "%!PS-Adobe-", the PostScript signature.
    fn application_postscript() -> ByteMatcher {
        ByteMatcher {
            pattern: b"%!PS-Adobe-",
            mask:  b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF",
            content_type: ("application", "postscript"),
            leading_ignore: &[]
        }
    }
    // UTF-16BE BOM
    fn text_plain_utf_16be_bom() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\xFE\xFF\x00\x00",
            mask: b"\xFF\xFF\x00\x00",
            content_type: ("text", "plain"),
            leading_ignore: &[]
        }
    }
    //UTF-16LE BOM
    fn text_plain_utf_16le_bom() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\xFF\xFE\x00\x00",
            mask: b"\xFF\xFF\x00\x00",
            content_type: ("text", "plain"),
            leading_ignore: &[]
        }
    }
    //UTF-8 BOM
    fn text_plain_utf_8_bom() -> ByteMatcher {
        ByteMatcher {
            pattern: b"\xEF\xBB\xBF\x00",
            mask: b"\xFF\xFF\xFF\x00",
            content_type: ("text", "plain"),
            leading_ignore: &[]
        }
    }
}
