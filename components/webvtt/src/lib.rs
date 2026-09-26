/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;
use std::mem;

use html5ever::buffer_queue::{BufferQueue, SetResult};
use html5ever::tendril::StrTendril;
use markup5ever::small_char_set;

mod collectors;
pub mod cue;

use collectors::collect_webvtt_cue_timings_and_settings;

use crate::cue::settings::WebVttCue;

#[derive(Debug, PartialEq)]
pub enum WebVttParserError {
    InvalidHeader,
}

impl std::fmt::Display for WebVttParserError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebVttParserError::InvalidHeader => write!(formatter, "Invalid WebVTT header in file"),
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
enum ParserState {
    #[default]
    FileTag,
    WhitespaceAfterFileTag,
    BeforeNewlineAfterFileTag,
    BeforeHeader,
    InBlockLoop,
    AfterBlockLoop,
    Region,
    Finished,
}

pub trait WebVttParserSink<Context> {
    fn consume_cue(&self, cx: &mut Context, cue: WebVttCue);
}

#[derive(Default)]
pub struct IncrementalWebVTTParser<Context, Sink: WebVttParserSink<Context>> {
    phantom: PhantomData<Context>,
    pub sink: Sink,
    buffer: BufferQueue,

    // Checkpoint values
    seen_cue: bool,
    seen_eof: bool,
    seen_arrow: bool,

    // State values
    in_header: bool,
    line_count: u32,
    state: ParserState,

    // Storage values
    current_line_in_block: StrTendril,
    current_buffer_in_block: String,
    lines_from_previous_position: StrTendril,

    current_cue_in_block: Option<WebVttCue>,
}

pub type ParserUpdate = Result<(), WebVttParserError>;

impl<Context, Sink> IncrementalWebVTTParser<Context, Sink>
where
    Sink: WebVttParserSink<Context>,
{
    /// <https://w3c.github.io/webvtt/#webvtt-parser-algorithm>
    pub fn new(sink: Sink) -> Self {
        Self {
            sink,
            phantom: Default::default(),
            buffer: Default::default(),
            seen_cue: Default::default(),
            seen_eof: Default::default(),
            seen_arrow: Default::default(),
            in_header: Default::default(),
            line_count: Default::default(),
            state: Default::default(),
            current_line_in_block: Default::default(),
            current_buffer_in_block: Default::default(),
            current_cue_in_block: Default::default(),
            lines_from_previous_position: Default::default(),
        }
    }

    pub fn end(&mut self, cx: &mut Context) -> ParserUpdate {
        self.seen_eof = true;
        self.step(cx)
    }

    pub fn parse_sync(&mut self, cx: &mut Context, input: &str) -> ParserUpdate {
        self.seen_eof = true;
        self.parse(cx, input)
    }

    /// <https://w3c.github.io/webvtt/#webvtt-parser-algorithm>
    pub fn parse(&mut self, cx: &mut Context, input: &str) -> ParserUpdate {
        // Step 1. Let input be the string being parsed, after conversion to Unicode,
        // and with the following transformations applied:
        // > Replace all U+0000 NULL characters by U+FFFD REPLACEMENT CHARACTERs.
        // > Replace each U+000D CARRIAGE RETURN U+000A LINE FEED (CRLF) character pair
        // > by a single U+000A LINE FEED (LF) character.
        // > Replace all remaining U+000D CARRIAGE RETURN characters by U+000A LINE FEED (LF) characters.
        // TODO
        // Step 2. Let position be a pointer into input, initially pointing at the start of the string.
        // In an incremental WebVTT parser, when this algorithm (or further algorithms that it uses)
        // moves the position pointer, the user agent must wait until appropriate further characters
        // from the byte stream have been added to input before moving the pointer,
        // so that the algorithm never reads past the end of the input string.
        // Once the byte stream has ended, and all characters have been added to input,
        // then the position pointer may, when so instructed by the algorithms,
        // be moved past the end of input.
        self.buffer.push_back(StrTendril::from(input));
        self.step(cx)
    }

    fn step(&mut self, cx: &mut Context) -> ParserUpdate {
        loop {
            let current_state = self.state;
            match current_state {
                // https://w3c.github.io/webvtt/#webvtt-parser-algorithm
                ParserState::FileTag => {
                    // https://w3c.github.io/webvtt/#webvtt-file-body
                    // > An optional U+FEFF BYTE ORDER MARK (BOM) character.
                    if self.buffer.peek().is_some_and(|c| c == '\u{FEFF}') {
                        let _ = self.buffer.next();
                    }
                    let Some(input) = self.buffer.eat("WEBVTT", u8::eq) else {
                        // Step 4. If input is less than six characters long, then abort these steps.
                        // The file does not start with the correct WebVTT file signature
                        // and was therefore not successfully processed.
                        if self.seen_eof {
                            return Err(WebVttParserError::InvalidHeader);
                        }
                        return Ok(());
                    };
                    // Step 5. If input is exactly six characters long but does not exactly equal "WEBVTT",
                    // then abort these steps. The file does not start with the correct WebVTT
                    // file signature and was therefore not successfully processed.
                    // Step 6. If input is more than six characters long but the first six characters
                    // do not exactly equal "WEBVTT", or the seventh character is not a U+0020 SPACE character,
                    // a U+0009 CHARACTER TABULATION (tab) character, or a U+000A LINE FEED (LF) character,
                    // then abort these steps. The file does not start with the correct WebVTT file signature
                    // and was therefore not successfully processed.
                    //
                    // We check the first part of this step here
                    if !input {
                        return Err(WebVttParserError::InvalidHeader);
                    }
                    self.state = ParserState::WhitespaceAfterFileTag;
                    continue;
                },
                // https://w3c.github.io/webvtt/#webvtt-parser-algorithm
                ParserState::WhitespaceAfterFileTag => {
                    let Some(seventh) = self.buffer.peek() else {
                        // Input is exactly six characters and is a valid header
                        if self.seen_eof {
                            self.state = ParserState::Finished;
                            continue;
                        }
                        return Ok(());
                    };
                    // Step 6. If input is more than six characters long but the first six characters
                    // do not exactly equal "WEBVTT", or the seventh character is not a U+0020 SPACE character,
                    // a U+0009 CHARACTER TABULATION (tab) character, or a U+000A LINE FEED (LF) character,
                    // then abort these steps. The file does not start with the correct WebVTT file signature
                    // and was therefore not successfully processed.
                    //
                    // We check the second part of this step here
                    if !matches!(seventh, '\u{0020}' | '\u{0009}' | '\u{000A}') {
                        return Err(WebVttParserError::InvalidHeader);
                    }
                    self.state = ParserState::BeforeNewlineAfterFileTag;
                    continue;
                },
                // https://w3c.github.io/webvtt/#webvtt-parser-algorithm
                ParserState::BeforeNewlineAfterFileTag => {
                    // Step 7. collect a sequence of code points that are not U+000A LINE FEED (LF) characters.
                    let Some(current_char) = self.buffer.next() else {
                        // Step 8. If position is past the end of input, then abort these steps.
                        // The file was successfully processed, but it contains no useful data and so
                        // no WebVTT cues were added to output.
                        if self.seen_eof {
                            self.state = ParserState::Finished;
                            continue;
                        }
                        return Ok(());
                    };
                    // Step 9. The character indicated by position is a U+000A LINE FEED (LF) character.
                    // Advance position to the next character in input.
                    if current_char == '\u{000A}' {
                        self.state = ParserState::BeforeHeader;
                    }
                },
                // https://w3c.github.io/webvtt/#webvtt-parser-algorithm
                ParserState::BeforeHeader => {
                    let Some(current_char) = self.buffer.peek() else {
                        // Step 10. If position is past the end of input, then abort these steps.
                        // The file was successfully processed, but it contains no useful data and
                        // so no WebVTT cues were added to output.
                        if self.seen_eof {
                            self.state = ParserState::Finished;
                            continue;
                        }
                        return Ok(());
                    };
                    // Step 11. Header: If the character indicated by position is not
                    // a U+000A LINE FEED (LF) character,
                    // then collect a WebVTT block with the in header flag set.
                    // Otherwise, advance position to the next character in input.
                    if current_char != '\u{000A}' {
                        self.in_header = true;
                        self.start_collecting_webvtt_block();
                    } else {
                        self.buffer.next();
                        self.state = ParserState::Region;
                    }
                },
                // https://w3c.github.io/webvtt/#collect-a-webvtt-block
                ParserState::InBlockLoop => {
                    let Some(current_char) =
                        self.buffer.pop_except_from(small_char_set!('\u{000A}'))
                    else {
                        // Step 11.3. If position is past the end of input, let seen EOF be true.
                        // Otherwise, the character indicated by position is a U+000A LINE FEED (LF) character;
                        // advance position to the next character in input.
                        //
                        // We check the first part of this step here
                        // Step 11.7. If seen EOF is true, break out of loop.
                        if self.seen_eof {
                            // It depends when we see the EOF whether there is still content in the buffer or not.
                            // In the case that the EOF is at the end of a line (e.g. no `\n` in between), we
                            // should copy the current line to the buffer. The buffer is then populated with the
                            // line as usual, so that the last line of a file can still be the cue text.
                            if !self.current_line_in_block.is_empty() {
                                if !self.current_buffer_in_block.is_empty() {
                                    self.current_buffer_in_block.push('\n');
                                }
                                self.current_buffer_in_block
                                    .push_str(&self.current_line_in_block);
                            }
                            self.state = ParserState::AfterBlockLoop;
                            continue;
                        }
                        return Ok(());
                    };
                    match current_char {
                        // Step 11.3. If position is past the end of input, let seen EOF be true.
                        // Otherwise, the character indicated by position is a U+000A LINE FEED (LF) character;
                        // advance position to the next character in input.
                        //
                        // We check the second part of this step here
                        SetResult::FromSet('\u{000A}') => {
                            self.lines_from_previous_position.push_char('\u{000A}');
                            // Step 11.2. Increment line count by 1.
                            self.line_count += 1;
                            // Step 11.4. If line contains the three-character substring "-->"
                            // (U+002D HYPHEN-MINUS, U+002D HYPHEN-MINUS, U+003E GREATER-THAN SIGN),
                            // then run these substeps:
                            if self.current_line_in_block.contains("-->") {
                                // Step 11.4.1. If in header is not set and at least
                                // one of the following conditions are true:
                                if !self.in_header &&
                                    (
                                        // line count is 1
                                        self.line_count == 1
                                    // line count is 2 and seen arrow is false
                                    || (self.line_count == 2 && !self.seen_arrow)
                                    )
                                {
                                    // Step 11.4.1.1. Let seen arrow be true.
                                    self.seen_arrow = true;
                                    // Step 11.4.1.2. Let previous position be position.
                                    self.lines_from_previous_position.clear();
                                    // Step 11.4.1.3. Cue creation: Let cue be a new WebVTT cue and initialize it as follows:
                                    // Step 11.4.1.3.1. Let cue’s text track cue identifier be buffer.
                                    let identifier = self.current_buffer_in_block.clone();
                                    // Step 11.4.1.4. Collect WebVTT cue timings and settings from line using regions for cue.
                                    // If that fails, let cue be null.
                                    // Otherwise, let buffer be the empty string and let seen cue be true.
                                    let cue = collect_webvtt_cue_timings_and_settings(
                                        identifier,
                                        &self.current_line_in_block,
                                    );
                                    let has_cue = cue.is_some();
                                    self.current_cue_in_block = cue;
                                    if has_cue {
                                        self.current_buffer_in_block.clear();
                                        self.current_line_in_block.clear();
                                        self.seen_cue = true;
                                    }
                                } else {
                                    // Otherwise, let position be previous position and break out of loop.
                                    self.state = ParserState::AfterBlockLoop;
                                    self.buffer.push_front(std::mem::take(
                                        &mut self.lines_from_previous_position,
                                    ));
                                }
                                continue;
                            } else if self.current_line_in_block.is_empty() {
                                // Step 11.5. Otherwise, if line is the empty string, break out of loop.
                                self.state = ParserState::AfterBlockLoop;
                            } else {
                                // Step 11.6. Otherwise, run these substeps:
                                // Step 11.6.1. If in header is not set and line count is 2, run these substeps:
                                // TODO
                                // Step 11.6.2. If buffer is not the empty string,
                                // append a U+000A LINE FEED (LF) character to buffer.
                                if !self.current_buffer_in_block.is_empty() {
                                    self.current_buffer_in_block.push('\u{000A}');
                                }
                                // Step 11.6.3. Append line to buffer.
                                self.current_buffer_in_block
                                    .push_str(&self.current_line_in_block);
                                // Step 11.6.4. Let previous position be position.
                                self.current_line_in_block.clear();
                                self.lines_from_previous_position.clear();
                            }
                            continue;
                        },
                        // Step 11.1. collect a sequence of code points that are not U+000A LINE FEED (LF) characters.
                        // Let line be those characters, if any.
                        SetResult::NotFromSet(current_tendril) => {
                            self.lines_from_previous_position
                                .push_tendril(&current_tendril);
                            self.current_line_in_block.push_tendril(&current_tendril);
                        },
                        _ => {
                            unreachable!();
                        },
                    }
                },
                ParserState::AfterBlockLoop => {
                    // https://w3c.github.io/webvtt/#collect-a-webvtt-block
                    // Step 12. If cue is not null, let the cue text of cue be buffer, and return cue.
                    // https://w3c.github.io/webvtt/#webvtt-parser-algorithm
                    // Step 14.2. If block is a WebVTT cue, add block to the text track list of cues output.
                    if let Some(mut cue) = self.current_cue_in_block.take() {
                        cue.text = self.current_buffer_in_block.clone();
                        self.sink.consume_cue(cx, cue);
                    }
                    // Step 14.3. Otherwise, if block is a CSS style sheet, add block to stylesheets.
                    // TODO
                    // Step 14.4. Otherwise, if block is a WebVTT region object, add block to regions.
                    // TODO
                    // Step 14.5. collect a sequence of code points that are U+000A LINE FEED (LF) characters.
                    let Some(current_char) = self.buffer.peek() else {
                        if self.seen_eof {
                            self.state = ParserState::Finished;
                            continue;
                        }
                        return Ok(());
                    };
                    if current_char == '\u{000A}' {
                        // Since we don't change the state here, it means that if the next character is
                        // also a newline, we re-enter this block and consume it again. Therefore, we
                        // only consume one-by-one.
                        let _ = self.buffer.next();
                    } else {
                        // If we were in the header block, then we should proceed with the next step
                        // which is collecting a region in step 12. Otherwise, we are in the general loop of
                        // step 14.

                        if mem::take(&mut self.in_header) {
                            self.state = ParserState::Region;
                        } else {
                            self.start_collecting_webvtt_block();
                        }
                    }
                },
                // https://w3c.github.io/webvtt/#webvtt-parser-algorithm
                ParserState::Region => {
                    // Step 12. collect a sequence of code points that are U+000A LINE FEED (LF) characters.
                    // TODO
                    self.start_collecting_webvtt_block();
                },
                ParserState::Finished => {
                    // Step 15. End: The file has ended. Abort these steps. The WebVTT parser has finished.
                    // The file was successfully processed.
                    return Ok(());
                },
            }
        }
    }

    /// <https://w3c.github.io/webvtt/#collect-a-webvtt-block>
    fn start_collecting_webvtt_block(&mut self) {
        // Step 2. Let line count be zero.
        self.line_count = 0;
        // Step 3. Let previous position be position.
        self.lines_from_previous_position.clear();
        // Step 4. Let line be the empty string.
        self.current_line_in_block.clear();
        // Step 5. Let buffer be the empty string.
        self.current_buffer_in_block.clear();
        // Step 7. Let seen arrow be false.
        self.seen_arrow = false;
        // Step 8. Let cue be null.
        self.current_cue_in_block = None;
        self.state = ParserState::InBlockLoop;
    }
}

#[cfg(any(test, feature = "test-util"))]
pub mod shared_test_setup;

#[cfg(test)]
mod tests {
    use crate::WebVttParserError;
    use crate::shared_test_setup::{compute_result_in_seconds, parser_with_dummy_sink};

    #[test]
    fn test_header_in_two_chunks() {
        let mut parser = parser_with_dummy_sink();
        assert_eq!(parser.parse(&mut (), "WEB"), Ok(()));
        assert_eq!(parser.parse(&mut (), "VTT"), Ok(()));
        assert_eq!(parser.end(&mut ()), Ok(()));
    }

    #[test]
    fn test_invalid_header_in_two_chunks() {
        let mut parser = parser_with_dummy_sink();
        assert_eq!(parser.parse(&mut (), "WEB"), Ok(()));
        assert_eq!(
            parser.parse(&mut (), "NOT"),
            Err(WebVttParserError::InvalidHeader)
        );
    }

    #[test]
    fn test_valid_space_character_after_header() {
        let mut parser = parser_with_dummy_sink();
        assert_eq!(parser.parse_sync(&mut (), "WEBVTT "), Ok(()));
    }

    #[test]
    fn test_no_space_character_after_header_multiple_chunks() {
        let mut parser = parser_with_dummy_sink();
        assert_eq!(parser.parse(&mut (), "WEB"), Ok(()));
        assert_eq!(parser.parse(&mut (), "VTT"), Ok(()));
        assert_eq!(
            parser.parse(&mut (), "2"),
            Err(WebVttParserError::InvalidHeader)
        );
    }

    mod cue_settings {
        use crate::tests::compute_result_in_seconds;
        use crate::{WebVttCue, collect_webvtt_cue_timings_and_settings};

        #[test]
        fn test_parses_cue_correctly() {
            assert_eq!(
                collect_webvtt_cue_timings_and_settings(
                    Default::default(),
                    "01:10:03.000 --> 02:20:23.000"
                ),
                Some(WebVttCue {
                    start_time: compute_result_in_seconds(1., 10., 3., 0.),
                    end_time: compute_result_in_seconds(2., 20., 23., 0.),
                    ..Default::default()
                })
            );
        }

        #[test]
        fn test_does_not_require_whitespace_around_arrow() {
            assert_eq!(
                collect_webvtt_cue_timings_and_settings(
                    Default::default(),
                    "01:10:03.000-->02:20:23.000"
                ),
                Some(WebVttCue {
                    start_time: compute_result_in_seconds(1., 10., 3., 0.),
                    end_time: compute_result_in_seconds(2., 20., 23., 0.),
                    ..Default::default()
                })
            );
        }

        #[test]
        fn test_can_handle_tabs_around_arrow() {
            assert_eq!(
                collect_webvtt_cue_timings_and_settings(
                    Default::default(),
                    "01:10:03.000\t-->\t02:20:23.000"
                ),
                Some(WebVttCue {
                    start_time: compute_result_in_seconds(1., 10., 3., 0.),
                    end_time: compute_result_in_seconds(2., 20., 23., 0.),
                    ..Default::default()
                })
            );
        }

        #[test]
        fn test_arrow_too_short_is_invalid() {
            assert_eq!(
                collect_webvtt_cue_timings_and_settings(
                    Default::default(),
                    "01:10:03.000 -> t02:20:23.000"
                ),
                None
            );
        }

        #[test]
        fn test_arrow_too_long_is_invalid() {
            assert_eq!(
                collect_webvtt_cue_timings_and_settings(
                    Default::default(),
                    "01:10:03.000 ---> t02:20:23.000"
                ),
                None
            );
        }

        #[test]
        fn test_skips_whitespace_at_start() {
            assert_eq!(
                collect_webvtt_cue_timings_and_settings(
                    Default::default(),
                    "  01:10:03.000 --> 02:20:23.000"
                ),
                Some(WebVttCue {
                    start_time: compute_result_in_seconds(1., 10., 3., 0.),
                    end_time: compute_result_in_seconds(2., 20., 23., 0.),
                    ..Default::default()
                })
            );
        }
    }
}
