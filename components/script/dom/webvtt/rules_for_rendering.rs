/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;
use script_bindings::inheritance::Castable;
use script_bindings::root::DomRoot;
use script_bindings::str::DOMString;

use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::TextTrackBinding::{TextTrackMethods, TextTrackMode};
use crate::dom::bindings::codegen::Bindings::VTTCueBinding::VTTCueMethods;
use crate::dom::html::htmlmediaelement::HTMLMediaElement;
use crate::dom::html::htmlvideoelement::HTMLVideoElement;
use crate::dom::node::Node;
use crate::dom::node::node::NodeTraits;
use crate::dom::texttrack::TextTrack;
use crate::dom::webvtt::vttcue::VTTCue;

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
        reset: ShouldResetRenderingControls,
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
        let cue_container = media_element.cue_container(cx);
        let cue_container = cue_container.upcast::<Node>();

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
                            ) ||
                        text_track.cues(cx.no_gc()).is_empty()
                );
                is_showing
            })
            .collect();

        // Step 7. Let cues be an empty list of text track cues.
        // Step 8. For each track track in tracks,
        // append to cues all the cues from track’s list of cues
        // that have their text track cue active flag set.
        //
        // Note we handle the active flag in the subsequent loop,
        // to avoid iterating multiple times through the list
        let cues: Vec<_> = tracks
            .iter()
            .flat_map(|track| track.cues(cx.no_gc()))
            .map(|cue| cue.as_rooted())
            .collect();

        let document = media_element.owner_document();

        for cue in cues {
            // Step 13. If reset is false, then, for each WebVTT cue cue in cues:
            // if cue’s text track cue display state has a set of CSS boxes, then:
            // > If cue’s WebVTT cue region is not null,
            // > add those boxes to that region’s box and remove cue from cues.
            // > Otherwise, add those boxes to output and remove cue from cues.
            //
            // This is written as a CSS spec, but it essentially means we should only
            // reset a cue if either the controls are added/removed (in which case
            // the size of the viewport has changed) or if the cue is no longer active.
            if let Some(current_display_state) = cue.display_state() {
                if reset == ShouldResetRenderingControls::No && cue.is_active() {
                    continue;
                }

                current_display_state.remove_self(cx);
                cue.set_display_state(None);
            }

            // Step 14. For each WebVTT cue cue in cues that has not yet
            // had corresponding CSS boxes added to output,
            // in text track cue order, run the following substeps:
            if !cue.is_active() {
                continue;
            }

            // Step 14.1. Let nodes be the list of WebVTT Node Objects obtained
            // by applying the WebVTT cue text parsing rules,
            // with the fallback language language if provided, to the cue’s cue text.
            let nodes = cue
                .downcast::<VTTCue>()
                .expect("Can only have WebVTT cues")
                .GetCueAsHTML(cx);
            // Step 14.2. If cue’s WebVTT cue region is null, run the following substeps:
            // TODO
            // Step 14.2.1. Apply WebVTT cue settings to obtain CSS boxes boxes from nodes.
            //
            // We model boxes as a node that we add the document fragment to. Note that since
            // `nodes` is a document fragment, `appendChild` would have added all its children
            // rather than the fragment itself. That's why we use this wrapper `boxes` to keep
            // track of cues so that we can later remove it.
            let boxes = document.create_element(cx, "span");
            let _ = boxes.upcast::<Node>().AppendChild(cx, nodes.upcast());
            // Step 14.2.2. Let cue’s text track cue display state have the CSS boxes in boxes.
            cue.set_display_state(Some(boxes.upcast()));
            // Step 14.2.3. Add the CSS boxes in boxes to output.
            let _ = cue_container.AppendChild(cx, boxes.upcast());
        }
    }
}
