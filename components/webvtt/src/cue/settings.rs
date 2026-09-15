/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/// <https://w3c.github.io/webvtt/#webvtt-cue-writing-direction>
#[derive(Debug, Default, PartialEq)]
pub enum WebVttWritingDirection {
    /// <https://w3c.github.io/webvtt/#webvtt-cue-horizontal-writing-direction>
    #[default]
    Horizontal,
    /// <https://w3c.github.io/webvtt/#webvtt-cue-vertical-growing-left-writing-direction>
    VerticalGrowingLeft,
    /// <https://w3c.github.io/webvtt/#webvtt-cue-vertical-growing-right-writing-direction>
    VerticalGrowingRight,
}

/// <https://w3c.github.io/webvtt/#webvtt-cue-text-alignment>
#[derive(Debug, Default, PartialEq)]
pub enum WebVttTextAlignment {
    /// <https://w3c.github.io/webvtt/#webvtt-cue-start-alignment>
    Start,
    /// <https://w3c.github.io/webvtt/#webvtt-cue-center-alignment>
    #[default]
    Center,
    /// <https://w3c.github.io/webvtt/#webvtt-cue-end-alignment>
    End,
    /// <https://w3c.github.io/webvtt/#webvtt-cue-left-alignment>
    Left,
    /// <https://w3c.github.io/webvtt/#webvtt-cue-right-alignment>
    Right,
}

/// <https://w3c.github.io/webvtt/#webvtt-cue-position-alignment>
#[derive(Debug, Default, PartialEq)]
pub enum WebVttPositionAlignment {
    /// <https://w3c.github.io/webvtt/#webvtt-cue-position-line-left-alignment>
    LineLeft,
    /// <https://w3c.github.io/webvtt/#webvtt-cue-position-center-alignment>
    Center,
    /// <https://w3c.github.io/webvtt/#webvtt-cue-position-line-right-alignment>
    LineRight,
    /// <https://w3c.github.io/webvtt/#webvtt-cue-position-automatic-alignment>
    #[default]
    Auto,
}

/// <https://w3c.github.io/webvtt/#webvtt-cue-position>
#[derive(Clone, Debug, Default, PartialEq)]
pub enum WebVttLineAndPositionSetting {
    Double(f64),
    #[default]
    Auto,
}

/// <https://w3c.github.io/webvtt/#webvtt-cue-line-alignment>
#[derive(Debug, Default, PartialEq)]
pub enum WebVttLineAlignment {
    /// <https://w3c.github.io/webvtt/#webvtt-cue-line-start-alignment>
    #[default]
    Start,
    /// <https://w3c.github.io/webvtt/#webvtt-cue-line-center-alignment>
    Center,
    /// <https://w3c.github.io/webvtt/#webvtt-cue-line-end-alignment>
    End,
}

/// <https://w3c.github.io/webvtt/#webvtt-cue-snap-to-lines-flag>
/// This is an enum, since the default value is `true`
#[derive(Debug, Default, PartialEq)]
pub enum WebVttSnapToLines {
    #[default]
    Yes,
    No,
}

impl From<bool> for WebVttSnapToLines {
    fn from(boolean: bool) -> Self {
        match boolean {
            true => WebVttSnapToLines::Yes,
            false => WebVttSnapToLines::No,
        }
    }
}

/// <https://w3c.github.io/webvtt/#webvtt-cue-size>
/// This is a struct, since the default value is 100
#[derive(Debug, PartialEq)]
pub struct WebVttCueSize(pub f64);

impl Default for WebVttCueSize {
    fn default() -> Self {
        Self(100.)
    }
}

/// <https://w3c.github.io/webvtt/#webvtt-cue>
#[derive(Debug, Default, PartialEq)]
pub struct WebVttCue {
    /// <https://html.spec.whatwg.org/multipage/#text-track-cue-identifier>
    pub identifier: String,
    /// <https://html.spec.whatwg.org/multipage/#text-track-cue-start-time>
    pub start_time: f64,
    /// <https://html.spec.whatwg.org/multipage/#text-track-cue-end-time>
    pub end_time: f64,
    /// <https://w3c.github.io/webvtt/#cue-text>
    pub text: String,
    /// <https://w3c.github.io/webvtt/#webvtt-cue-writing-direction>
    pub writing_direction: WebVttWritingDirection,
    /// <https://w3c.github.io/webvtt/#webvtt-cue-text-alignment>
    pub text_alignment: WebVttTextAlignment,
    /// <https://w3c.github.io/webvtt/#webvtt-cue-position-alignment>
    pub position_alignment: WebVttPositionAlignment,
    /// <https://w3c.github.io/webvtt/#webvtt-cue-position>
    pub position: WebVttLineAndPositionSetting,
    /// <https://w3c.github.io/webvtt/#webvtt-cue-line-alignment>
    pub line_alignment: WebVttLineAlignment,
    /// <https://w3c.github.io/webvtt/#webvtt-cue-line>
    pub line: WebVttLineAndPositionSetting,
    /// <https://w3c.github.io/webvtt/#webvtt-cue-snap-to-lines-flag>
    pub snap_to_lines: WebVttSnapToLines,
    /// <https://w3c.github.io/webvtt/#webvtt-cue-size>
    pub size: WebVttCueSize,
}
