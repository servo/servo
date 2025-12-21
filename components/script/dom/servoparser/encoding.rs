/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::mem;
use std::time::{Duration, Instant};

use encoding_rs::{Encoding, UTF_8, UTF_16BE, UTF_16LE, WINDOWS_1252, X_USER_DEFINED};
use tendril::fmt::UTF8;
use tendril::stream::LossyDecoder;
use tendril::{ByteTendril, StrTendril, TendrilSink};

use crate::dom::document::Document;

#[derive(JSTraceable, MallocSizeOf)]
pub(super) struct DetectingState {
    /// The `charset` that was specified in the `Content-Type` header, if any.
    #[no_trace]
    encoding_hint_from_content_type: Option<&'static Encoding>,
    /// The encoding of a same-origin container document, if this document is in an
    /// `<iframe>`.
    #[no_trace]
    encoding_of_container_document: Option<&'static Encoding>,
    start_timestamp: Instant,
    attempted_bom_sniffing: bool,
    buffered_bytes: Vec<u8>,
}

#[derive(JSTraceable, MallocSizeOf)]
pub(super) struct DecodingState {
    /// The actual decoder.
    ///
    /// This field is `None` after we've finished parsing, because `LossyDecoder::finish`
    /// takes ownership of the decoder.
    #[ignore_malloc_size_of = "Defined in tendril"]
    #[no_trace]
    decoder: Option<LossyDecoder<NetworkSink>>,
    #[no_trace]
    pub(super) encoding: &'static Encoding,
}

#[derive(JSTraceable, MallocSizeOf)]
pub(super) enum NetworkDecoderState {
    /// In this stage the decoder is buffering bytes until it has enough to determine the encoding.
    Detecting(DetectingState),
    Decoding(DecodingState),
}

impl DetectingState {
    /// The maximum amount of bytes to buffer before attempting to determine the encoding
    const BUFFER_THRESHOLD: usize = 1024;

    /// The time threshold after which we will attempt to determine the encoding and start decoding,
    /// even if there are less than [BUFFER_THRESHOLD] bytes in the buffer.
    const MAX_TIME_TO_BUFFER: Duration = Duration::from_secs(1);

    /// Appends some data to the internal buffer and attempts to [determine the character encoding].
    ///
    /// If an encoding was detected then it is returned. A return value of `None` indicates that
    /// more bytes are required.
    ///
    /// [determine the character encoding]: https://html.spec.whatwg.org/multipage/#determining-the-character-encoding
    fn buffer(
        &mut self,
        data: &[u8],
        document: &Document,
        is_at_end_of_file: AtEndOfFile,
    ) -> Option<&'static Encoding> {
        self.buffered_bytes.extend_from_slice(data);
        let can_wait_longer = self.start_timestamp.elapsed() < Self::MAX_TIME_TO_BUFFER;
        self.determine_the_character_encoding(document, can_wait_longer, is_at_end_of_file)
    }

    /// <https://html.spec.whatwg.org/multipage/#determining-the-character-encoding>
    fn determine_the_character_encoding(
        &mut self,
        document: &Document,
        potentially_wait_for_more_data: bool,
        is_at_end_of_file: AtEndOfFile,
    ) -> Option<&'static Encoding> {
        // Step 1. If the result of BOM sniffing is an encoding, return that encoding with confidence certain.
        if !self.attempted_bom_sniffing && self.buffered_bytes.len() > 2 {
            self.attempted_bom_sniffing = true;

            // https://encoding.spec.whatwg.org/#bom-sniff
            match self.buffered_bytes.as_slice() {
                [0xEF, 0xBB, 0xBF, ..] => {
                    log::debug!("Determined that the document is UTF-8 via BOM-sniffing");
                    return Some(UTF_8);
                },
                [0xFE, 0xFF, ..] => {
                    log::debug!("Determined that the document is UTF-16BE via BOM-sniffing");
                    return Some(UTF_16BE);
                },
                [0xFF, 0xFE, ..] => {
                    log::debug!("Determined that the document is UTF-16LE via BOM-sniffing");
                    return Some(UTF_16LE);
                },
                _ => {},
            }
        }

        // Step 2. If the user has explicitly instructed the user agent to override the document's character
        // encoding with a specific encoding, optionally return that encoding with the confidence certain.
        // NOTE: Our users have no way to do that.

        // Step 3. The user agent may wait for more bytes of the resource to be available, either in this
        // step or at any later step in this algorithm.
        if potentially_wait_for_more_data && self.buffered_bytes.len() < Self::BUFFER_THRESHOLD {
            return None;
        }

        // TODO: Step 4. If the transport layer specifies a character encoding, and it is supported, return that
        // encoding with the confidence certain.
        if let Some(encoding_hint_from_content_type) = self.encoding_hint_from_content_type {
            log::debug!(
                "Inferred encoding to be {} from the Content-Type header",
                encoding_hint_from_content_type.name()
            );
            return Some(encoding_hint_from_content_type);
        }

        // Step 5. Optionally, prescan the byte stream to determine its encoding, with the end condition
        // being when the user agent decides that scanning further bytes would not be efficient.
        // NOTE: According to the spec, we should always try to get an xml encoding right after failing
        // to prescan the byte stream
        let bytes_to_prescan =
            &self.buffered_bytes[..Self::BUFFER_THRESHOLD.min(self.buffered_bytes.len())];
        if let Some(encoding) = prescan_the_byte_stream_to_determine_the_encoding(bytes_to_prescan)
            .or_else(|| get_xml_encoding(bytes_to_prescan))
        {
            log::debug!(
                "Prescanning the byte stream determined that the encoding is {}",
                encoding.name()
            );
            return Some(encoding);
        }

        // TODO: Step 6. If the HTML parser for which this algorithm is being run is associated with a Document d
        // whose container document is non-null, then:
        // Step 6.1 Let parentDocument be d's container document.
        // Step 6.2 If parentDocument's origin is same origin with d's origin and parentDocument's character encoding
        // is not UTF-16BE/LE, then return parentDocument's character encoding, with the confidence tentative.
        if let Some(encoding) = self.encoding_of_container_document {
            if encoding != UTF_16LE && encoding != UTF_16BE {
                log::debug!(
                    "Inferred encoding to be that of the container document, which is {}",
                    encoding.name()
                );
                return Some(encoding);
            }
        }

        // Step 7. Otherwise, if the user agent has information on the likely encoding for this page, e.g.
        // based on the encoding of the page when it was last visited, then return that encoding,
        // with the confidence tentative.
        // NOTE: We have no such information.

        // Step 8. The user agent may attempt to autodetect the character encoding from applying frequency analysis
        // or other algorithms to the data stream.
        let mut encoding_detector = chardetng::EncodingDetector::new();
        encoding_detector.feed(&self.buffered_bytes, is_at_end_of_file == AtEndOfFile::Yes);
        let url = document.url();
        let tld = url
            .as_url()
            .domain()
            .and_then(|domain| domain.rsplit('.').next())
            .map(|tld| tld.as_bytes());
        let (guessed_encoding, is_probably_right) = encoding_detector.guess_assess(tld, true);
        if is_probably_right {
            log::debug!(
                "chardetng determined that the document encoding is {}",
                guessed_encoding.name()
            );
            return Some(guessed_encoding);
        }

        // Step 9. Otherwise, return an implementation-defined or user-specified default character encoding,
        // with the confidence tentative.
        // TODO: The spec has a cool table here for determining an appropriate fallback encoding based on the
        // user locale. Use it!
        log::debug!("Failed to determine encoding of byte stream, falling back to UTF-8");
        Some(UTF_8)
    }

    fn finish(&mut self, document: &Document) -> &'static Encoding {
        self.determine_the_character_encoding(document, false, AtEndOfFile::Yes)
            .expect("Should always return character encoding when we're not allowed to wait")
    }
}

impl NetworkDecoderState {
    pub(super) fn new(
        encoding_hint_from_content_type: Option<&'static Encoding>,
        encoding_of_container_document: Option<&'static Encoding>,
    ) -> Self {
        Self::Detecting(DetectingState {
            encoding_hint_from_content_type,
            encoding_of_container_document,
            start_timestamp: Instant::now(),
            attempted_bom_sniffing: false,
            buffered_bytes: vec![],
        })
    }

    /// Feeds the network decoder a chunk of bytes.
    ///
    /// If a new encoding is detected, then the encoding of `document` is updated appropriately.
    ///
    /// The decoded bytes are returned to the caller. Note that there is not necessarily a 1:1
    /// relation between `chunk` and the return value. In the beginning, the decoder will buffer
    /// bytes and return `None`, then later it will flush them and return a large `StrTendril` all
    /// at once.
    pub(super) fn push(&mut self, chunk: &[u8], document: &Document) -> Option<StrTendril> {
        match self {
            Self::Detecting(encoding_detector) => {
                if let Some(encoding) = encoding_detector.buffer(chunk, document, AtEndOfFile::No) {
                    document.set_encoding(encoding);
                    let buffered_bytes = mem::take(&mut encoding_detector.buffered_bytes);
                    *self = Self::Decoding(DecodingState {
                        decoder: Some(LossyDecoder::new_encoding_rs(
                            encoding,
                            NetworkSink::default(),
                        )),
                        encoding,
                    });
                    return self.push(&buffered_bytes, document);
                }

                None
            },
            Self::Decoding(network_decoder) => {
                let decoder = network_decoder
                    .decoder
                    .as_mut()
                    .expect("Can't push after call to finish()");
                decoder.process(ByteTendril::from(chunk));
                Some(std::mem::take(&mut decoder.inner_sink_mut().output))
            },
        }
    }

    pub(super) fn finish(&mut self, document: &Document) -> StrTendril {
        match self {
            Self::Detecting(encoding_detector) => {
                let encoding = encoding_detector.finish(document);
                document.set_encoding(encoding);
                let buffered_bytes = mem::take(&mut encoding_detector.buffered_bytes);
                let mut decoder = LossyDecoder::new_encoding_rs(encoding, NetworkSink::default());
                decoder.process(ByteTendril::from(&*buffered_bytes));
                *self = Self::Decoding(DecodingState {
                    // Important to set `None` here to indicate that we're done decoding
                    decoder: None,
                    encoding,
                });
                let mut chunk = std::mem::take(&mut decoder.inner_sink_mut().output);
                chunk.push_tendril(&decoder.finish());
                chunk
            },
            Self::Decoding(network_decoder) => network_decoder
                .decoder
                .take()
                .map(|decoder| decoder.finish())
                .unwrap_or_default(),
        }
    }

    pub(super) fn is_finished(&self) -> bool {
        match self {
            Self::Detecting(_) => false,
            Self::Decoding(network_decoder) => network_decoder.decoder.is_none(),
        }
    }

    pub(super) fn decoder(&mut self) -> &mut DecodingState {
        match self {
            Self::Detecting(_) => unreachable!("Cannot access decoder before decoding"),
            Self::Decoding(decoder) => decoder,
        }
    }
}

/// An implementor of `TendrilSink` with the sole purpose of buffering decoded data
/// so we can take it later.
#[derive(Default, JSTraceable)]
pub(crate) struct NetworkSink {
    #[no_trace]
    pub(crate) output: StrTendril,
}

impl TendrilSink<UTF8> for NetworkSink {
    type Output = StrTendril;

    fn process(&mut self, tendril: StrTendril) {
        if self.output.is_empty() {
            self.output = tendril;
        } else {
            self.output.push_tendril(&tendril);
        }
    }

    fn error(&mut self, _desc: Cow<'static, str>) {}

    fn finish(self) -> Self::Output {
        self.output
    }
}

#[derive(Default)]
struct Attribute {
    name: Vec<u8>,
    value: Vec<u8>,
}

/// <https://html.spec.whatwg.org/multipage/#prescan-a-byte-stream-to-determine-its-encoding>
pub fn prescan_the_byte_stream_to_determine_the_encoding(
    byte_stream: &[u8],
) -> Option<&'static Encoding> {
    // Step 1. Let position be a pointer to a byte in the input byte stream,
    // initially pointing at the first byte.
    let mut position = 0;

    // Step 2. Prescan for UTF-16 XML declarations: If position points to:
    match byte_stream {
        // A sequence of bytes starting with: 0x3C, 0x0, 0x3F, 0x0, 0x78, 0x0
        // (case-sensitive UTF-16 little-endian '<?x')
        [0x3C, 0x0, 0x3F, 0x0, 0x78, 0x0, ..] => {
            // Return UTF-16LE.
            return Some(UTF_16LE);
        },

        // A sequence of bytes starting with: 0x0, 0x3C, 0x0, 0x3F, 0x0, 0x78
        // (case-sensitive UTF-16 big-endian '<?x')
        [0x0, 0x3C, 0x0, 0x3F, 0x0, 0x78, ..] => {
            // Return UTF-16BE.
            return Some(UTF_16BE);
        },
        _ => {},
    }

    loop {
        // Step 3. Loop: If position points to:
        let remaining_byte_stream = byte_stream.get(position..)?;

        // A sequence of bytes starting with: 0x3C 0x21 0x2D 0x2D (`<!--`)
        if remaining_byte_stream.starts_with(b"<!--") {
            // Advance the position pointer so that it points at the first 0x3E byte which is preceded by two 0x2D bytes
            // (i.e. at the end of an ASCII '-->' sequence) and comes after the 0x3C byte that was found.
            // (The two 0x2D bytes can be the same as those in the '<!--' sequence.)
            // NOTE: This is not very efficient, but likely not an issue...
            position += remaining_byte_stream
                .windows(3)
                .position(|window| window == b"-->")?;
        }
        // A sequence of bytes starting with: 0x3C, 0x4D or 0x6D, 0x45 or 0x65, 0x54 or 0x74, 0x41 or 0x61,
        // and one of 0x09, 0x0A, 0x0C, 0x0D, 0x20, 0x2F (case-insensitive ASCII '<meta' followed by a space or slash)
        else if remaining_byte_stream
            .get(..b"<meta ".len())
            .is_some_and(|candidate| {
                candidate[..b"<meta".len()].eq_ignore_ascii_case(b"<meta") &&
                    candidate.last().is_some_and(|byte| {
                        matches!(byte, 0x09 | 0x0A | 0x0C | 0x0D | 0x20 | 0x2F)
                    })
            })
        {
            // Step 1. Advance the position pointer so that it points at the next 0x09, 0x0A, 0x0C, 0x0D, 0x20,
            // or 0x2F byte (the one in sequence of characters matched above).
            position += b"<meta".len();

            // Step 2. Let attribute list be an empty list of strings.
            // NOTE: This is used to track which attributes we have already seen. As there are only
            // three attributes that we care about, we instead use three booleans.
            let mut have_seen_http_equiv_attribute = false;
            let mut have_seen_content_attribute = false;
            let mut have_seen_charset_attribute = false;

            // Step 3. Let got pragma be false.
            let mut got_pragma = false;

            // Step 4. Let need pragma be null.
            let mut need_pragma = None;

            // Step 5. Let charset be the null value (which, for the purposes of this algorithm,
            // is distinct from an unrecognized encoding or the empty string).
            let mut charset = None;

            // Step 6. Attributes: Get an attribute and its value. If no attribute was sniffed,
            // then jump to the processing step below.
            while let Some(attribute) = get_an_attribute(byte_stream, &mut position) {
                // Step 7 If the attribute's name is already in attribute list,
                // then return to the step labeled attributes.
                // Step 8. Add the attribute's name to attribute list.
                // NOTE: This happens in the match arms below
                // Step 9. Run the appropriate step from the following list, if one applies:
                match attribute.name.as_slice() {
                    // If the attribute's name is "http-equiv"
                    b"http-equiv" if !have_seen_http_equiv_attribute => {
                        have_seen_http_equiv_attribute = true;

                        // If the attribute's value is "content-type", then set got pragma to true.
                        if attribute.value == b"content-type" {
                            got_pragma = true;
                        }
                    },
                    // If the attribute's name is "content"
                    b"content" if !have_seen_content_attribute => {
                        have_seen_content_attribute = true;

                        // Apply the algorithm for extracting a character encoding from a meta element,
                        // giving the attribute's value as the string to parse. If a character encoding
                        // is returned, and if charset is still set to null, let charset be the encoding
                        // returned, and set need pragma to true.
                        if charset.is_none() {
                            if let Some(extracted_charset) =
                                extract_a_character_encoding_from_a_meta_element(&attribute.value)
                            {
                                need_pragma = Some(true);
                                charset = Some(extracted_charset);
                            }
                        }
                    },
                    // If the attribute's name is "charset"
                    b"charset" if !have_seen_charset_attribute => {
                        have_seen_charset_attribute = true;

                        // Let charset be the result of getting an encoding from the attribute's value,
                        // and set need pragma to false.
                        if let Some(extracted_charset) = Encoding::for_label(&attribute.value) {
                            charset = Some(extracted_charset);
                        }

                        need_pragma = Some(false);
                    },
                    _ => {},
                }

                // Step 10. Return to the step labeled attributes.
            }

            // Step 11. Processing: If need pragma is null, then jump to the step below labeled next byte.
            if let Some(need_pragma) = need_pragma {
                // Step 12. If need pragma is true but got pragma is false,
                // then jump to the step below labeled next byte.
                if !need_pragma || got_pragma {
                    // Step 13. If charset is UTF-16BE/LE, then set charset to UTF-8.
                    if charset.is_some_and(|charset| charset == UTF_16BE || charset == UTF_16LE) {
                        charset = Some(UTF_8);
                    }
                    // Step 14. If charset is x-user-defined, then set charset to windows-1252.
                    else if charset.is_some_and(|charset| charset == X_USER_DEFINED) {
                        charset = Some(WINDOWS_1252);
                    }

                    // Step 15. Return charset.
                    return charset;
                }
            }
        }
        // A sequence of bytes starting with a 0x3C byte (<), optionally a 0x2F byte (/),
        // and finally a byte in the range 0x41-0x5A or 0x61-0x7A (A-Z or a-z)
        else if *remaining_byte_stream.first()? == b'<' &&
            remaining_byte_stream
                .get(1)
                .filter(|byte| **byte != b'=')
                .or(remaining_byte_stream.get(2))?
                .is_ascii_alphabetic()
        {
            // Step 1. Advance the position pointer so that it points at the next 0x09 (HT),
            // 0x0A (LF), 0x0C (FF), 0x0D (CR), 0x20 (SP), or 0x3E (>) byte.
            position += remaining_byte_stream
                .iter()
                .position(|byte| byte.is_ascii_whitespace() || *byte == b'>')?;

            // Step 2. Repeatedly get an attribute until no further attributes can be found,
            // then jump to the step below labeled next byte.
            while get_an_attribute(byte_stream, &mut position).is_some() {}
        }
        // A sequence of bytes starting with: 0x3C 0x21 (`<!`)
        // A sequence of bytes starting with: 0x3C 0x2F (`</`)
        // A sequence of bytes starting with: 0x3C 0x3F (`<?`)
        else if remaining_byte_stream.starts_with(b"<!") ||
            remaining_byte_stream.starts_with(b"</") ||
            remaining_byte_stream.starts_with(b"<?")
        {
            // Advance the position pointer so that it points at the first 0x3E byte (>) that comes after the 0x3C byte that was found.
            position += remaining_byte_stream
                .iter()
                .position(|byte| *byte == b'>')?;
        }
        // Any other byte
        else {
            // Do nothing with that byte.
        }

        // Next byte: Move position so it points at the next byte in the input byte stream,
        // and return to the step above labeled loop.
        position += 1;
    }
}

/// <https://html.spec.whatwg.org/multipage/#concept-get-attributes-when-sniffing>
fn get_an_attribute(input: &[u8], position: &mut usize) -> Option<Attribute> {
    // NOTE: If we reach the end of the input during parsing then we return "None"
    // (because there obviously is no attribute). The caller will then also run
    // out of bytes and invoke "get an xml encoding" as mandated by the spec.

    // Step 1. If the byte at position is one of 0x09 (HT), 0x0A (LF), 0x0C (FF), 0x0D (CR),
    // 0x20 (SP), or 0x2F (/), then advance position to the next byte and redo this step.
    *position += &input[*position..]
        .iter()
        .position(|b| !matches!(b, 0x09 | 0x0A | 0x0C | 0x0D | 0x20 | 0x2F))?;

    // Step 2. If the byte at position is 0x3E (>), then abort the get an attribute algorithm.
    // There isn't one.
    if input[*position] == 0x3E {
        return None;
    }

    // Step 3. Otherwise, the byte at position is the start of the attribute name.
    // Let attribute name and attribute value be the empty string.
    let mut attribute = Attribute::default();
    let mut have_spaces = false;
    loop {
        // Step 4. Process the byte at position as follows:
        match *input.get(*position)? {
            // If it is 0x3D (=), and the attribute name is longer than the empty string
            b'=' if !attribute.name.is_empty() => {
                // Advance position to the next byte and jump to the step below labeled value.
                *position += 1;
                break;
            },

            // If it is 0x09 (HT), 0x0A (LF), 0x0C (FF), 0x0D (CR), or 0x20 (SP)
            0x09 | 0x0A | 0x0C | 0x0D | 0x20 => {
                // Jump to the step below labeled spaces.
                have_spaces = true;
                break;
            },

            // If it is 0x2F (/) or 0x3E (>)
            b'/' | b'>' => {
                // Abort the get an attribute algorithm.
                // The attribute's name is the value of attribute name, its value is the empty string.
                return Some(attribute);
            },

            // If it is in the range 0x41 (A) to 0x5A (Z)
            byte @ (b'A'..=b'Z') => {
                // Append the code point b+0x20 to attribute name (where b is the value of the byte at position).
                // (This converts the input to lowercase.)
                attribute.name.push(byte + 0x20);
            },

            // Anything else
            byte => {
                // Append the code point with the same value as the byte at position to attribute name.
                // (It doesn't actually matter how bytes outside the ASCII range are handled here, since only
                // ASCII bytes can contribute to the detection of a character encoding.)
                attribute.name.push(byte);
            },
        }

        // Step 5. Advance position to the next byte and return to the previous step.
        *position += 1;
    }

    if have_spaces {
        // Step 6. Spaces: If the byte at position is one of 0x09 (HT), 0x0A (LF), 0x0C (FF), 0x0D (CR),
        // or 0x20 (SP), then advance position to the next byte, then, repeat this step.
        *position += &input[*position..]
            .iter()
            .position(|b| !b.is_ascii_whitespace())?;

        // Step 7. If the byte at position is not 0x3D (=), abort the get an attribute algorithm.
        // The attribute's name is the value of attribute name, its value is the empty string.
        if input[*position] != b'=' {
            return Some(attribute);
        }

        // Step 8. Advance position past the 0x3D (=) byte.
        *position += 1;
    }

    // Step 9. Value: If the byte at position is one of 0x09 (HT), 0x0A (LF), 0x0C (FF), 0x0D (CR), or 0x20 (SP),
    // then advance position to the next byte, then, repeat this step.
    *position += &input[*position..]
        .iter()
        .position(|b| !b.is_ascii_whitespace())?;

    // Step 10. Process the byte at position as follows:
    match input[*position] {
        // If it is 0x22 (") or 0x27 (')
        b @ (b'"' | b'\'') => {
            // Step 1. Let b be the value of the byte at position.
            // NOTE: We already have b.
            loop {
                // Step 2. Quote loop: Advance position to the next byte.
                *position += 1;

                // Step 3. If the value of the byte at position is the value of b, then advance position to the next byte
                // and abort the "get an attribute" algorithm. The attribute's name is the value of attribute name, and
                // its value is the value of attribute value.
                let byte_at_position = *input.get(*position)?;
                if byte_at_position == b {
                    *position += 1;
                    return Some(attribute);
                }
                // Step 4. Otherwise, if the value of the byte at position is in the range 0x41 (A) to 0x5A (Z),
                // then append a code point to attribute value whose value is 0x20 more than the value of the byte
                // at position.
                else if byte_at_position.is_ascii_uppercase() {
                    attribute.value.push(byte_at_position + 0x20);
                }
                // Step 5. Otherwise, append a code point to attribute value whose value is the same
                // as the value of the byte at position.
                else {
                    attribute.value.push(byte_at_position);
                }

                // Step 6. Return to the step above labeled quote loop.
            }
        },

        // If it is 0x3E (>)
        b'>' => {
            // Abort the get an attribute algorithm. The attribute's name is the value of attribute name,
            // its value is the empty string.
            return Some(attribute);
        },

        // If it is in the range 0x41 (A) to 0x5A (Z)
        b @ (b'A'..=b'Z') => {
            // Append a code point b+0x20 to attribute value (where b is the value of the byte at position).
            // Advance position to the next byte.
            attribute.value.push(b + 0x20);
            *position += 1;
        },

        // Anything else
        b => {
            // Append a code point with the same value as the byte at position to attribute value.
            // Advance position to the next byte.
            attribute.value.push(b);
            *position += 1
        },
    }

    loop {
        // Step 11. Process the byte at position as follows:
        match *input.get(*position)? {
            // If it is 0x09 (HT), 0x0A (LF), 0x0C (FF), 0x0D (CR), 0x20 (SP), or 0x3E (>)
            0x09 | 0x0A | 0x0C | 0x0D | 0x20 | 0x3E => {
                // Abort the get an attribute algorithm. The attribute's name is the value of attribute name and
                // its value is the value of attribute value.
                return Some(attribute);
            },

            // If it is in the range 0x41 (A) to 0x5A (Z)
            byte if byte.is_ascii_uppercase() => {
                // Append a code point b+0x20 to attribute value (where b is the value of the byte at position).
                attribute.value.push(byte + 0x20);
            },

            // Anything else
            byte => {
                // Append a code point with the same value as the byte at position to attribute value.
                attribute.value.push(byte);
            },
        }

        // Step 12. Advance position to the next byte and return to the previous step.
        *position += 1;
    }
}

/// <https://html.spec.whatwg.org/multipage/#algorithm-for-extracting-a-character-encoding-from-a-meta-element>
fn extract_a_character_encoding_from_a_meta_element(input: &[u8]) -> Option<&'static Encoding> {
    // Step 1. Let position be a pointer into s, initially pointing at the start of the string.
    let mut position = 0;

    loop {
        // Step 2. Loop: Find the first seven characters in s after position that are an ASCII case-insensitive
        // match for the word "charset". If no such match is found, return nothing.
        // NOTE: In our case, the attribute value always comes from "get_an_attribute" and is already lowercased.
        position += input[position..]
            .windows(7)
            .position(|window| window == b"charset")? +
            b"charset".len();

        // Step 3. Skip any ASCII whitespace that immediately follow the word "charset" (there might not be any).
        position += &input[position..]
            .iter()
            .position(|byte| !byte.is_ascii_whitespace())?;

        // Step 4. If the next character is not a U+003D EQUALS SIGN (=), then move position to point just before
        // that next character, and jump back to the step labeled loop.
        // NOTE: This is phrased very oddly, because position is already pointing to that character.
        if *input.get(position)? == b'=' {
            position += 1;
            break;
        }
    }

    // Step 5. Skip any ASCII whitespace that immediately follow the equals sign (there might not be any).
    position += &input[position..]
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())?;

    // Step 6. Process the next character as follows:
    let next_character = input.get(position)?;

    // If it is a U+0022 QUOTATION MARK character (") and there is a later U+0022 QUOTATION MARK character (") in s
    // If it is a U+0027 APOSTROPHE character (') and there is a later U+0027 APOSTROPHE character (') in s
    if matches!(*next_character, b'"' | b'\'') {
        // Return the result of getting an encoding from the substring that is between
        // this character and the next earliest occurrence of this character.
        let remaining = input.get(position + 1..)?;
        let end = remaining.iter().position(|byte| byte == next_character)?;
        Encoding::for_label(&remaining[..end])
    }
    // If it is an unmatched U+0022 QUOTATION MARK character (")
    // If it is an unmatched U+0027 APOSTROPHE character (')
    // If there is no next character
    // NOTE: All of these cases are already covered above

    // Otherwise
    else {
        // Return the result of getting an encoding from the substring that consists of this character up
        // to but not including the first ASCII whitespace or U+003B SEMICOLON character (;), or the end of s,
        // whichever comes first.
        let remaining = input.get(position..)?;
        let end = remaining
            .iter()
            .position(|byte| byte.is_ascii_whitespace() || *byte == b';')
            .unwrap_or(remaining.len());

        Encoding::for_label(&remaining[..end])
    }
}

/// <https://html.spec.whatwg.org/multipage/#concept-get-xml-encoding-when-sniffing>
pub fn get_xml_encoding(input: &[u8]) -> Option<&'static Encoding> {
    // Step 1. Let encodingPosition be a pointer to the start of the stream.
    // NOTE: We don't need this variable yet.
    // Step 2. If encodingPosition does not point to the start of a byte sequence 0x3C, 0x3F, 0x78,
    // 0x6D, 0x6C (`<?xml`), then return failure.
    if !input.starts_with(b"<?xml") {
        return None;
    }

    // Step 3. Let xmlDeclarationEnd be a pointer to the next byte in the input byte stream which is 0x3E (>).
    // If there is no such byte, then return failure.
    // NOTE: The spec does not use this variable but the intention is clear.
    let xml_declaration_end = input.iter().position(|byte| *byte == b'>')?;
    let input = &input[..xml_declaration_end];

    // Step 4. Set encodingPosition to the position of the first occurrence of the subsequence of bytes 0x65, 0x6E,
    // 0x63, 0x6F, 0x64, 0x69, 0x6E, 0x67 (`encoding`) at or after the current encodingPosition. If there is no
    // such sequence, then return failure.
    let mut encoding_position = input
        .windows(b"encoding".len())
        .position(|window| window == b"encoding")?;

    // Step 5. Advance encodingPosition past the 0x67 (g) byte.
    encoding_position += b"encoding".len();

    // Step 6. While the byte at encodingPosition is less than or equal to 0x20 (i.e., it is either an
    // ASCII space or control character), advance encodingPosition to the next byte.
    while *input.get(encoding_position)? <= 0x20 {
        encoding_position += 1;
    }

    // Step 7. If the byte at encodingPosition is not 0x3D (=), then return failure.
    if *input.get(encoding_position)? != b'=' {
        return None;
    }

    // Step 8. Advance encodingPosition to the next byte.
    encoding_position += 1;

    // Step 9. While the byte at encodingPosition is less than or equal to 0x20 (i.e., it is either an
    // ASCII space or control character), advance encodingPosition to the next byte.
    while *input.get(encoding_position)? <= 0x20 {
        encoding_position += 1;
    }

    // Step 10. Let quoteMark be the byte at encodingPosition.
    let quote_mark = *input.get(encoding_position)?;

    // Step 11. If quoteMark is not either 0x22 (") or 0x27 ('), then return failure.
    if !matches!(quote_mark, b'"' | b'\'') {
        return None;
    }

    // Step 12. Advance encodingPosition to the next byte.
    encoding_position += 1;

    // Step 13. Let encodingEndPosition be the position of the next occurrence of quoteMark at or after
    // encodingPosition. If quoteMark does not occur again, then return failure.
    let encoding_end_position = input[encoding_position..]
        .iter()
        .position(|byte| *byte == quote_mark)?;

    // Step 14. Let potentialEncoding be the sequence of the bytes between encodingPosition
    // (inclusive) and encodingEndPosition (exclusive).
    let potential_encoding = &input[encoding_position..][..encoding_end_position];

    // Step 15. If potentialEncoding contains one or more bytes whose byte value is 0x20 or below,
    // then return failure.
    if potential_encoding.iter().any(|byte| *byte <= 0x20) {
        return None;
    }

    // Step 16. Let encoding be the result of getting an encoding given potentialEncoding isomorphic decoded.
    let encoding = Encoding::for_label(potential_encoding)?;

    // Step 17. If the encoding is UTF-16BE/LE, then change it to UTF-8.
    // Step 18. Return encoding.
    if encoding == UTF_16BE || encoding == UTF_16LE {
        Some(UTF_8)
    } else {
        Some(encoding)
    }
}

#[derive(PartialEq)]
enum AtEndOfFile {
    Yes,
    No,
}
