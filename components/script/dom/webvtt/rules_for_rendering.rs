/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;
use script_bindings::dom::UnrootedDom;
use script_bindings::root::DomRoot;
use script_bindings::str::DOMString;

use crate::dom::bindings::codegen::Bindings::TextTrackBinding::{TextTrackMethods, TextTrackMode};
use crate::dom::html::htmlmediaelement::HTMLMediaElement;
use crate::dom::html::htmlvideoelement::HTMLVideoElement;
use crate::dom::texttrack::TextTrack;
use crate::dom::texttrackcue::TextTrackCue;
use crate::script_bindings::inheritance::Castable;

#[derive(PartialEq)]
pub(crate) enum ShouldResetRenderingControls {
    Yes,
    No,
}

/// <https://html.spec.whatwg.org/multipage/#rules-for-updating-the-text-track-rendering>
#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum RulesForUpdatingTheTextTrackRendering {
    /// <https://w3c.github.io/webvtt/#rules-for-updating-the-display-of-webvtt-text-tracks>
    WebVTT,
}

impl RulesForUpdatingTheTextTrackRendering {
    pub(crate) fn run(
        &self,
        cx: &mut JSContext,
        media_element: &HTMLMediaElement,
        text_tracks: Vec<DomRoot<TextTrack>>,
        _language: Option<DOMString>,
        _reset: ShouldResetRenderingControls,
    ) {
        debug_assert!(*self == RulesForUpdatingTheTextTrackRendering::WebVTT);
        // https://w3c.github.io/webvtt/#rules-for-updating-the-display-of-webvtt-text-tracks
        // Step 2. Let video be the media element or other playback mechanism.
        if !media_element.is::<HTMLVideoElement>() {
            // Step 1. If the media element is an audio element,
            // or is another playback mechanism with no rendering area,
            // abort these steps.
            return;
        };

        // Step 3. Let output be an empty list of absolutely positioned CSS block boxes.
        // Step 4. If the user agent is exposing a user interface for video,
        // add to output one or more completely transparent positioned CSS block boxes
        // that cover the same region as the user interface.
        //
        // All of these are handled by modifying the DOM, which in turn will invalidate
        // the layout of the video element

        // Step 5. If the last time these rules were run,
        // the user agent was not exposing a user interface for video,
        // but now it is, optionally let reset be true.
        // Otherwise, let reset be false.
        //
        // Passed in as argument

        // Step 6. Let tracks be the subset of video’s list of text tracks
        // that have as their rules for updating the text track rendering
        // these rules for updating the display of WebVTT text tracks,
        // and whose text track mode is showing.
        let tracks: Vec<DomRoot<TextTrack>> = text_tracks
            .into_iter()
            .filter(|text_track| {
                let is_showing = text_track.Mode() == TextTrackMode::Showing;
                debug_assert!(
                    !is_showing ||
                        text_track
                            .rules_for_updating_the_text_track_rendering()
                            .is_some_and(
                                |rules| rules == RulesForUpdatingTheTextTrackRendering::WebVTT
                            )
                );
                is_showing
            })
            .collect();

        // Step 7. Let cues be an empty list of text track cues.
        // Step 8. For each track track in tracks,
        // append to cues all the cues from track’s list of cues
        // that have their text track cue active flag set.
        let _cues: Vec<DomRoot<TextTrackCue>> = tracks
            .iter()
            .flat_map(|track| {
                track
                    .cues(cx.no_gc())
                    .into_iter()
                    .filter(|cue| cue.is_active())
                    .collect::<Vec<UnrootedDom<'_, TextTrackCue>>>()
            })
            .map(|cue| cue.as_rooted())
            .collect();
    }
}
