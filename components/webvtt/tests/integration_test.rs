/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use servo_webvtt::shared_test_setup::{compute_result_in_seconds, parser_with_dummy_sink};
use servo_webvtt::{
    WebVttCue, WebVttCueSize, WebVttLineAndPositionSetting, WebVttParserError, WebVttSnapToLines,
    WebVttTextAlignment, WebVttWritingDirection,
};

macro_rules! include_vtt_file {
    ($file:expr $(,)?) => {
        include_str!(concat!("../../../tests/wpt/tests/html/semantics/embedded-content/media-elements/track/track-element/resources/", $file))
    };
}

#[test]
fn test_simple_cue_file() {
    let parser = parser_with_dummy_sink();
    assert_eq!(
        parser.parse_sync(&mut (), include_vtt_file!("track.vtt")),
        Ok(())
    );
    assert_eq!(
        *parser.sink.collected_cues.borrow(),
        vec![WebVttCue {
            start_time: compute_result_in_seconds(0., 0., 0., 0.),
            end_time: compute_result_in_seconds(0., 0., 1., 0.),
            text: "test".into(),
            ..Default::default()
        }]
    )
}

#[test]
fn test_multiple_different_ids() {
    let parser = parser_with_dummy_sink();
    assert_eq!(
        parser.parse_sync(&mut (), include_vtt_file!("cue-id.vtt")),
        Ok(())
    );
    assert_eq!(
        *parser.sink.collected_cues.borrow(),
        vec![
            WebVttCue {
                identifier: "random_id".into(),
                start_time: compute_result_in_seconds(0., 0., 0., 0.),
                end_time: compute_result_in_seconds(0., 0., 30., 500.),
                text: "Bear is Coming!!!!!".into(),
                ..Default::default()
            },
            WebVttCue {
                identifier: "another random identifier".into(),
                start_time: compute_result_in_seconds(0., 0., 31., 0.),
                end_time: compute_result_in_seconds(0., 1., 0., 500.),
                text: "I said Bear is coming!!!!".into(),
                ..Default::default()
            },
            WebVttCue {
                identifier: "identifier--too".into(),
                start_time: compute_result_in_seconds(0., 1., 1., 0.),
                end_time: compute_result_in_seconds(0., 2., 0., 500.),
                text: "I said Bear is coming now!!!!".into(),
                ..Default::default()
            },
            WebVttCue {
                identifier: "identifier--too".into(),
                start_time: compute_result_in_seconds(0., 2., 1., 0.),
                end_time: compute_result_in_seconds(0., 3., 0., 500.),
                text: "Duplicate identifier".into(),
                ..Default::default()
            }
        ]
    )
}

#[test]
fn test_too_short_input() {
    let parser = parser_with_dummy_sink();
    assert_eq!(
        parser.parse_sync(&mut (), include_vtt_file!("header-too-short.vtt")),
        Err(WebVttParserError::InvalidHeader)
    );
}

#[test]
fn test_header_exactly_six_characters() {
    let parser = parser_with_dummy_sink();
    assert_eq!(
        parser.parse_sync(&mut (), include_vtt_file!("header-empty-after.vtt")),
        Ok(())
    );
}

#[test]
fn test_header_only_newlines_after() {
    let parser = parser_with_dummy_sink();
    assert_eq!(
        parser.parse_sync(&mut (), include_vtt_file!("header-newlines-after.vtt")),
        Ok(())
    );
}

#[test]
fn test_invalid_header_exactly_six_characters() {
    let parser = parser_with_dummy_sink();
    assert_eq!(
        parser.parse_sync(&mut (), include_vtt_file!("header-invalid-equal.vtt")),
        Err(WebVttParserError::InvalidHeader)
    );
}

#[test]
fn test_invalid_header_with_cues() {
    let parser = parser_with_dummy_sink();
    assert_eq!(
        parser.parse_sync(&mut (), include_vtt_file!("no-webvtt.vtt")),
        Err(WebVttParserError::InvalidHeader)
    );
}

#[test]
fn test_ignore_bom_character() {
    let parser = parser_with_dummy_sink();
    assert_eq!(
        parser.parse_sync(&mut (), include_vtt_file!("bom.vtt")),
        Ok(())
    );
}

#[test]
fn test_parses_vertical_alignment() {
    let parser = parser_with_dummy_sink();
    assert_eq!(
        parser.parse_sync(&mut (), include_vtt_file!("valign.vtt")),
        Ok(())
    );
    assert_eq!(
        *parser.sink.collected_cues.borrow(),
        vec![
            WebVttCue {
                identifier: "1".into(),
                start_time: compute_result_in_seconds(0., 0., 0., 0.),
                end_time: compute_result_in_seconds(0., 0., 30., 500.),
                text: "Bear is Coming!!!!!
Renders on the right side of the video viewport, middle aligned,
top to bottom, growing left."
                    .into(),
                writing_direction: WebVttWritingDirection::VerticalGrowingLeft,
                ..Default::default()
            },
            WebVttCue {
                identifier: "2".into(),
                start_time: compute_result_in_seconds(0., 0., 31., 0.),
                end_time: compute_result_in_seconds(0., 1., 0., 500.),
                text: "I said Bear is coming!!!!
Renders on the left side of the video viewport, middle aligned,
top to bottom, growing right."
                    .into(),
                writing_direction: WebVttWritingDirection::VerticalGrowingRight,
                ..Default::default()
            },
            WebVttCue {
                identifier: "3".into(),
                start_time: compute_result_in_seconds(0., 1., 1., 0.),
                end_time: compute_result_in_seconds(0., 2., 0., 500.),
                text: "I said Bear is coming now!!!!
Renders on the right side of the video viewport, top aligned both
for the cue box and the text within, text from top to bottom, growing left."
                    .into(),
                writing_direction: WebVttWritingDirection::VerticalGrowingLeft,
                text_alignment: WebVttTextAlignment::Start,
                position: WebVttLineAndPositionSetting::Double(0.),
                ..Default::default()
            },
        ]
    )
}

#[test]
fn test_parses_text_alignment() {
    let parser = parser_with_dummy_sink();
    assert_eq!(
        parser.parse_sync(&mut (), include_vtt_file!("alignment.vtt")),
        Ok(())
    );
    assert_eq!(
        *parser.sink.collected_cues.borrow(),
        vec![
            WebVttCue {
                identifier: "1".into(),
                start_time: compute_result_in_seconds(0., 0., 0., 0.),
                end_time: compute_result_in_seconds(0., 0., 30., 500.),
                text: "Bear is Coming!!!!!
Start align."
                    .into(),
                text_alignment: WebVttTextAlignment::Start,
                ..Default::default()
            },
            WebVttCue {
                identifier: "2".into(),
                start_time: compute_result_in_seconds(0., 0., 31., 0.),
                end_time: compute_result_in_seconds(0., 1., 0., 500.),
                text: "I said Bear is coming!!!!
Middle align."
                    .into(),
                text_alignment: WebVttTextAlignment::Center,
                ..Default::default()
            },
            WebVttCue {
                identifier: "3".into(),
                start_time: compute_result_in_seconds(0., 1., 1., 0.),
                end_time: compute_result_in_seconds(0., 2., 0., 500.),
                text: "I said Bear is coming now!!!!
End align."
                    .into(),
                text_alignment: WebVttTextAlignment::End,
                ..Default::default()
            },
            WebVttCue {
                identifier: "4".into(),
                start_time: compute_result_in_seconds(0., 2., 1., 0.),
                end_time: compute_result_in_seconds(100., 20., 0., 500.),
                text: "I said Bear is coming now!!!!
Default is middle alignment."
                    .into(),
                text_alignment: WebVttTextAlignment::Center,
                ..Default::default()
            },
        ]
    )
}

#[test]
fn test_parses_bad_text_alignment() {
    let parser = parser_with_dummy_sink();
    assert_eq!(
        parser.parse_sync(&mut (), include_vtt_file!("alignment-bad.vtt")),
        Ok(())
    );
    assert_eq!(
        *parser.sink.collected_cues.borrow(),
        vec![
            WebVttCue {
                identifier: "1".into(),
                start_time: compute_result_in_seconds(0., 0., 0., 0.),
                end_time: compute_result_in_seconds(0., 0., 30., 500.),
                text: "Bear is Coming!!!!!
Erroneous alignment value -> middle."
                    .into(),
                ..Default::default()
            },
            WebVttCue {
                identifier: "2".into(),
                start_time: compute_result_in_seconds(0., 0., 31., 0.),
                end_time: compute_result_in_seconds(0., 1., 0., 500.),
                text: "I said Bear is coming!!!!".into(),
                ..Default::default()
            },
            WebVttCue {
                identifier: "3".into(),
                start_time: compute_result_in_seconds(0., 1., 1., 0.),
                end_time: compute_result_in_seconds(0., 2., 0., 500.),
                text: "I said Bear is coming now!!!!".into(),
                ..Default::default()
            },
            WebVttCue {
                identifier: "4".into(),
                start_time: compute_result_in_seconds(0., 2., 1., 0.),
                end_time: compute_result_in_seconds(100., 20., 0., 500.),
                text: "I said Bear is coming now!!!!
Erroneous alignment value -> middle."
                    .into(),
                ..Default::default()
            },
        ]
    )
}

#[test]
fn test_parses_line_position() {
    let parser = parser_with_dummy_sink();
    assert_eq!(
        parser.parse_sync(&mut (), include_vtt_file!("line-position.vtt")),
        Ok(())
    );
    assert_eq!(
        *parser.sink.collected_cues.borrow(),
        vec![
            WebVttCue {
                identifier: "1".into(),
                start_time: compute_result_in_seconds(0., 0., 0., 0.),
                end_time: compute_result_in_seconds(0., 0., 15., 0.),
                text: "Bear is Coming!!!!!
Positioning on the top of the viewport, in the middle."
                    .into(),
                line: WebVttLineAndPositionSetting::Double(0.),
                snap_to_lines: WebVttSnapToLines::No,
                ..Default::default()
            },
            WebVttCue {
                identifier: "".into(),
                start_time: compute_result_in_seconds(0., 0., 15., 500.),
                end_time: compute_result_in_seconds(0., 0., 30., 500.),
                text: "Bear is Coming!!!!!
This is line 0.
Positioning on the top of the viewport, in the middle."
                    .into(),
                line: WebVttLineAndPositionSetting::Double(0.),
                snap_to_lines: WebVttSnapToLines::Yes,
                ..Default::default()
            },
            WebVttCue {
                identifier: "2".into(),
                start_time: compute_result_in_seconds(0., 0., 31., 0.),
                end_time: compute_result_in_seconds(0., 0., 45., 500.),
                text: "I said Bear is coming!!!!
Positioning on the center of the video."
                    .into(),
                line: WebVttLineAndPositionSetting::Double(50.),
                snap_to_lines: WebVttSnapToLines::No,
                ..Default::default()
            },
            WebVttCue {
                identifier: "".into(),
                start_time: compute_result_in_seconds(0., 0., 46., 0.),
                end_time: compute_result_in_seconds(0., 1., 0., 500.),
                text: "I said Bear is coming!!!!
This is line 6 from the top of the video viewport."
                    .into(),
                line: WebVttLineAndPositionSetting::Double(5.),
                snap_to_lines: WebVttSnapToLines::Yes,
                ..Default::default()
            },
            WebVttCue {
                identifier: "3".into(),
                start_time: compute_result_in_seconds(0., 1., 1., 0.),
                end_time: compute_result_in_seconds(0., 1., 30., 0.),
                text: "I said Bear is coming now!!!!
Positioning on the bottom middle."
                    .into(),
                line: WebVttLineAndPositionSetting::Double(100.),
                snap_to_lines: WebVttSnapToLines::No,
                ..Default::default()
            },
            WebVttCue {
                identifier: "".into(),
                start_time: compute_result_in_seconds(0., 1., 31., 0.),
                end_time: compute_result_in_seconds(0., 2., 0., 500.),
                text: "I said Bear is coming now!!!!
This is the first line at the bottom of the video viewport.
Positioning on the bottom middle. Only 1 line shows."
                    .into(),
                line: WebVttLineAndPositionSetting::Double(-1.),
                snap_to_lines: WebVttSnapToLines::Yes,
                ..Default::default()
            },
            WebVttCue {
                identifier: "".into(),
                start_time: compute_result_in_seconds(0., 2., 1., 0.),
                end_time: compute_result_in_seconds(0., 2., 30., 0.),
                text: "I said Bear is coming now!!!!
This is legal,
even though the line will likely not be within the video viewport."
                    .into(),
                line: WebVttLineAndPositionSetting::Double(500.),
                snap_to_lines: WebVttSnapToLines::Yes,
                ..Default::default()
            },
        ]
    )
}

#[test]
fn test_parses_size() {
    let parser = parser_with_dummy_sink();
    assert_eq!(
        parser.parse_sync(&mut (), include_vtt_file!("cue-size.vtt")),
        Ok(())
    );
    assert_eq!(
        *parser.sink.collected_cues.borrow(),
        vec![
            WebVttCue {
                identifier: "1".into(),
                start_time: compute_result_in_seconds(0., 0., 0., 0.),
                end_time: compute_result_in_seconds(0., 0., 30., 500.),
                text: "Bear is Coming!!!!!
Box for the cue is 100% of the video viewport width,
exemplified through background color,
even if the text needs less."
                    .into(),
                size: WebVttCueSize(100.),
                ..Default::default()
            },
            WebVttCue {
                identifier: "2".into(),
                start_time: compute_result_in_seconds(0., 0., 31., 0.),
                end_time: compute_result_in_seconds(0., 1., 0., 500.),
                text: "I said Bear is coming!!!!
Box for the cue is 10% of the video viewport width, which will mean that automatic line wrapping will happen."
                    .into(),
                size: WebVttCueSize(10.),
                ..Default::default()
            },
            WebVttCue {
                identifier: "3".into(),
                start_time: compute_result_in_seconds(0., 1., 1., 0.),
                end_time: compute_result_in_seconds(0., 2., 0., 500.),
                text: "I said Bear is coming now!!!!
Cue text box size of 0 is acceptable, even if not visible."
                    .into(),
                size: WebVttCueSize(0.),
                ..Default::default()
            },
        ]
    )
}

#[test]
fn test_parses_bad_size() {
    let parser = parser_with_dummy_sink();
    assert_eq!(
        parser.parse_sync(&mut (), include_vtt_file!("cue-size-bad.vtt")),
        Ok(())
    );
    assert_eq!(
        *parser.sink.collected_cues.borrow(),
        vec![
            WebVttCue {
                identifier: "1".into(),
                start_time: compute_result_in_seconds(0., 0., 0., 0.),
                end_time: compute_result_in_seconds(0., 0., 30., 500.),
                text: "Bear is Coming!!!!!
Cue size setting doesn't parse and is ignored."
                    .into(),
                ..Default::default()
            },
            WebVttCue {
                identifier: "2".into(),
                start_time: compute_result_in_seconds(0., 0., 31., 0.),
                end_time: compute_result_in_seconds(0., 1., 0., 500.),
                text: "I said Bear is coming!!!!
Negative cue size setting is not acceptable and is ignored."
                    .into(),
                ..Default::default()
            },
            WebVttCue {
                identifier: "3".into(),
                start_time: compute_result_in_seconds(0., 1., 1., 0.),
                end_time: compute_result_in_seconds(0., 2., 0., 500.),
                text: "I said Bear is coming now!!!!
Cue size beyond 100% is not acceptable and is ignored."
                    .into(),
                ..Default::default()
            },
        ]
    )
}
