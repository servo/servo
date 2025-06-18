/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;

use encoding_rs::{EncoderResult, Encoding, UTF_8};

/// This is equivalent to [Encoding::encode], except nonmappable code points are handled
/// according to the url specification, which expects nonmappable code points to be wrapped in `%26%23` and
/// `%3B` (see [percent encode after encoding](https://url.spec.whatwg.org/#string-percent-encode-after-encoding)).
pub fn encode_as_url_query_string<'a>(
    mut string: &'a str,
    encoding: &'static Encoding,
) -> Cow<'a, [u8]> {
    let output_encoding = encoding.output_encoding();
    if output_encoding == UTF_8 {
        return Cow::Borrowed(string.as_bytes());
    }

    let bytes = string.as_bytes();
    let valid_up_to = if output_encoding == encoding_rs::ISO_2022_JP {
        Encoding::iso_2022_jp_ascii_valid_up_to(bytes)
    } else {
        Encoding::ascii_valid_up_to(bytes)
    };

    if valid_up_to == bytes.len() {
        // All the bytes are already correctly encoded - we don't need to do anything!
        return Cow::Borrowed(bytes);
    }

    let mut encoder = encoding.new_encoder();
    let mut output = Vec::with_capacity(
        encoder
            .max_buffer_length_from_utf8_if_no_unmappables(string.len())
            .expect("string size would overflow `usize`"),
    );
    loop {
        match encoder.encode_from_utf8_to_vec_without_replacement(string, &mut output, true) {
            (EncoderResult::InputEmpty, _) => break,
            (EncoderResult::OutputFull, consumed) => {
                output.reserve(
                    encoder
                        .max_buffer_length_from_utf8_if_no_unmappables(string.len())
                        .expect("string size would overflow `usize`"),
                );
                string = &string[consumed..];
            },
            (EncoderResult::Unmappable(character), consumed) => {
                use std::io::Write;
                write!(&mut output, "%26%23{}%3B", character as u32).unwrap();
                string = &string[consumed..];
            },
        };
    }

    Cow::Owned(output)
}
