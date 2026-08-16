/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::reflect_dom_object_with_cx;
use stylo_atoms::Atom;

use crate::dom::audio::audiotrack::AudioTrack;
use crate::dom::audio::audiotracklist::AudioTrackList;
use crate::dom::bindings::buffer_source::get_buffer_source_copy;
use crate::dom::bindings::codegen::Bindings::MediaSourceBinding::ReadyState;
use crate::dom::bindings::codegen::Bindings::SourceBufferBinding::{
    AppendMode, SourceBufferMethods,
};
use crate::dom::bindings::codegen::UnionTypes::ArrayBufferViewOrArrayBuffer;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::media::bytestream::{
    ByteStreamFormat, ByteStreamParser, InitializationSegment, TrackKind,
};
use crate::dom::media::mediasource::MediaSource;
use crate::dom::media::videotrack::VideoTrack;
use crate::dom::media::videotracklist::VideoTrackList;
use crate::dom::timeranges::{TimeRanges, TimeRangesContainer};
use crate::dom::webvtt::texttracklist::TextTrackList;

/// How far apart two intervals may be and still count as covering a continuous stretch of
/// media.
///
/// Media segments are authored back to back, but the duration a byte stream reports for
/// the last frame of a segment rarely lands exactly on the timestamp the next segment
/// starts at. The specification allows a fudge factor of twice the maximum frame duration
/// when deciding whether coded frames are continuous, which keeps `buffered` from
/// reporting gaps that playback does not actually have.
///
/// <https://w3c.github.io/media-source/#coded-frame-group>
const CODED_FRAME_GROUP_FUDGE: f64 = 0.05;

/// Adds `[start, end)` to `ranges`, closing a gap smaller than the fudge factor rather
/// than leaving the interval detached from what is already buffered.
fn add_continuous_range(ranges: &mut TimeRangesContainer, start: f64, end: f64) {
    let mut start = start;
    for index in 0..ranges.len() {
        if let Ok(existing_end) = ranges.end(index) &&
            existing_end < start &&
            start - existing_end <= CODED_FRAME_GROUP_FUDGE
        {
            start = existing_end;
            break;
        }
    }
    let _ = ranges.add(start, end);
}

/// <https://w3c.github.io/media-source/#sourcebuffer>
#[dom_struct]
pub(crate) struct SourceBuffer {
    eventtarget: EventTarget,

    /// The media source this object belongs to, cleared once it is removed from the
    /// `sourceBuffers` attribute of its parent media source.
    media_source: MutNullableDom<MediaSource>,

    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-mode>
    mode: Cell<AppendMode>,

    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-updating>
    updating: Cell<bool>,

    /// Stored and returned as the specification requires, but not yet applied to the
    /// coded frames, which reach the media backend with the timestamps they were
    /// authored with.
    ///
    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-timestampoffset>
    timestamp_offset: Cell<f64>,

    /// Stored and returned as the specification requires, but not yet applied: no coded
    /// frame is dropped for falling outside of the window.
    ///
    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-appendwindowstart>
    append_window_start: Cell<f64>,

    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-appendwindowend>
    append_window_end: Cell<f64>,

    /// The ranges each track of this source buffer holds coded frames for, keyed by the
    /// track identifier the byte stream uses.
    ///
    /// <https://w3c.github.io/media-source/#track-buffer>
    track_buffers: DomRefCell<Vec<(u32, TimeRangesContainer)>>,

    /// The largest presentation timestamp a coded frame of this source buffer has, used
    /// by the duration change algorithm.
    highest_presentation_timestamp: Cell<f64>,

    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-audiotracks>
    audio_tracks: Dom<AudioTrackList>,

    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-videotracks>
    video_tracks: Dom<VideoTrackList>,

    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-texttracks>
    text_tracks: Dom<TextTrackList>,

    /// <https://w3c.github.io/media-source/#dfn-input-buffer>
    input_buffer: DomRefCell<Vec<u8>>,

    parser: DomRefCell<ByteStreamParser>,

    /// Bumped whenever an in flight append or range removal is aborted, so that the task
    /// running it knows to stop.
    generation: Cell<u32>,

    /// <https://w3c.github.io/media-source/#dfn-first-initialization-segment-received-flag>
    first_initialization_segment_received: Cell<bool>,
}

impl SourceBuffer {
    fn new_inherited(
        media_source: &MediaSource,
        format: ByteStreamFormat,
        audio_tracks: &AudioTrackList,
        video_tracks: &VideoTrackList,
        text_tracks: &TextTrackList,
    ) -> SourceBuffer {
        SourceBuffer {
            eventtarget: EventTarget::new_inherited(),
            media_source: MutNullableDom::new(Some(media_source)),
            mode: Cell::new(AppendMode::Segments),
            updating: Cell::new(false),
            timestamp_offset: Cell::new(0.),
            append_window_start: Cell::new(0.),
            append_window_end: Cell::new(f64::INFINITY),
            track_buffers: DomRefCell::new(Vec::new()),
            highest_presentation_timestamp: Cell::new(f64::NAN),
            audio_tracks: Dom::from_ref(audio_tracks),
            video_tracks: Dom::from_ref(video_tracks),
            text_tracks: Dom::from_ref(text_tracks),
            input_buffer: DomRefCell::new(Vec::new()),
            parser: DomRefCell::new(ByteStreamParser::new(format)),
            generation: Cell::new(0),
            first_initialization_segment_received: Cell::new(false),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        media_source: &MediaSource,
        format: ByteStreamFormat,
    ) -> DomRoot<SourceBuffer> {
        let global = media_source.global();
        let window = global.as_window();
        let audio_tracks = AudioTrackList::new(cx, window, &[], None);
        let video_tracks = VideoTrackList::new(cx, window, &[], None);
        let text_tracks = TextTrackList::new(cx, window, &[]);

        reflect_dom_object_with_cx(
            Box::new(SourceBuffer::new_inherited(
                media_source,
                format,
                &audio_tracks,
                &video_tracks,
                &text_tracks,
            )),
            window,
            cx,
        )
    }

    pub(crate) fn updating(&self) -> bool {
        self.updating.get()
    }

    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-buffered>
    ///
    /// Steps 3 to 6: the intersection of the track buffer ranges of every track, since a
    /// frame is only playable where every track has one.
    pub(crate) fn buffered_ranges(&self) -> TimeRangesContainer {
        let track_buffers = self.track_buffers.borrow();
        let mut ranges = match track_buffers.first() {
            Some((_, ranges)) => ranges.clone(),
            None => return TimeRangesContainer::default(),
        };
        for (_, track_ranges) in track_buffers.iter().skip(1) {
            ranges = ranges.intersection(track_ranges);
        }
        ranges
    }

    /// The largest end time across the track buffers, which the duration change algorithm
    /// uses rather than the intersection.
    ///
    /// <https://w3c.github.io/media-source/#dfn-duration-change>
    pub(crate) fn highest_track_end_time(&self) -> Option<f64> {
        self.track_buffers
            .borrow()
            .iter()
            .filter_map(|(_, ranges)| ranges.end_time())
            .reduce(f64::max)
    }

    /// Records the interval a media segment covers against the track buffers it belongs
    /// to, creating an entry for a track the initialization segment did not declare.
    fn add_to_track_buffers(&self, track: Option<u32>, start: f64, end: f64) {
        let mut track_buffers = self.track_buffers.borrow_mut();
        match track {
            Some(id) => {
                if let Some((_, ranges)) = track_buffers.iter_mut().find(|(key, _)| *key == id) {
                    add_continuous_range(ranges, start, end);
                } else {
                    let mut ranges = TimeRangesContainer::default();
                    let _ = ranges.add(start, end);
                    track_buffers.push((id, ranges));
                }
            },
            // A segment that interleaves every track covers them all at once.
            None => {
                if track_buffers.is_empty() {
                    let mut ranges = TimeRangesContainer::default();
                    let _ = ranges.add(start, end);
                    track_buffers.push((0, ranges));
                } else {
                    for (_, ranges) in track_buffers.iter_mut() {
                        add_continuous_range(ranges, start, end);
                    }
                }
            },
        }
    }

    pub(crate) fn highest_presentation_timestamp(&self) -> Option<f64> {
        let timestamp = self.highest_presentation_timestamp.get();
        (!timestamp.is_nan()).then_some(timestamp)
    }

    /// Whether this source buffer belongs in `activeSourceBuffers`.
    ///
    /// <https://w3c.github.io/media-source/#dfn-active-track-flag>
    pub(crate) fn is_active(&self) -> bool {
        self.audio_tracks.enabled_index().is_some() || self.video_tracks.selected_index().is_some()
    }

    /// Detaches this source buffer from its media source, which the specification
    /// describes as removing it from the `sourceBuffers` attribute.
    pub(crate) fn detach(&self) {
        self.abort_append();
        self.audio_tracks.clear();
        self.video_tracks.clear();
        self.track_buffers.borrow_mut().clear();
        self.media_source.set(None);
        self.input_buffer.borrow_mut().clear();
    }

    /// Aborts an in flight append, firing the events the specification asks for when a
    /// source buffer is removed or `abort()` is called while updating.
    pub(crate) fn abort_append(&self) {
        if !self.updating.get() {
            return;
        }

        // Step 2.1. Abort the buffer append algorithm if it is running.
        self.generation.set(self.generation.get().wrapping_add(1));
        self.input_buffer.borrow_mut().clear();

        // Step 2.2. Set the updating attribute to false.
        self.updating.set(false);

        // Step 2.3. Queue a task to fire an event named abort at this SourceBuffer object.
        self.queue_event(atom!("abort"));

        // Step 2.4. Queue a task to fire an event named updateend at this SourceBuffer
        // object.
        self.queue_event(Atom::from("updateend"));
    }

    fn parent_media_source(&self) -> Option<DomRoot<MediaSource>> {
        self.media_source.get()
    }

    /// The specification tests for removal from `sourceBuffers` in almost every method.
    fn require_attached(&self) -> Fallible<DomRoot<MediaSource>> {
        self.media_source.get().ok_or_else(|| {
            Error::InvalidState(Some(
                "The source buffer has been removed from the media source".into(),
            ))
        })
    }

    /// <https://w3c.github.io/media-source/#dfn-prepare-append>
    fn prepare_append(&self) -> ErrorResult {
        // Step 1. If the SourceBuffer has been removed from the sourceBuffers attribute of
        // the parent media source then throw an InvalidStateError exception and abort
        // these steps.
        let media_source = self.require_attached()?;

        // Step 2. If the updating attribute equals true, then throw an InvalidStateError
        // exception and abort these steps.
        if self.updating.get() {
            return Err(Error::InvalidState(Some(
                "The source buffer is still updating".into(),
            )));
        }

        // Step 3. Let recent element error be determined by the media element error.
        // Step 4. If recent element error is true, then throw an InvalidStateError
        // exception and abort these steps.
        if let Some(media_element) = media_source.media_element() &&
            media_element.in_error_state()
        {
            return Err(Error::InvalidState(Some(
                "The media element is in an error state".into(),
            )));
        }

        // Step 5. If the readyState attribute of the parent media source is in the "ended"
        // state then run the following steps.
        Self::reopen_media_source(&media_source);

        // Steps 6-8 evict coded frames and report a full buffer. The media backend keeps
        // its own queue bounded, so there is nothing to evict here.

        Ok(())
    }

    /// Moves the parent media source back to `"open"`, which several methods do when they
    /// are called while it is `"ended"`.
    fn reopen_media_source(media_source: &MediaSource) {
        if media_source.ready_state() == ReadyState::Ended {
            media_source.reopen();
        }
    }

    /// <https://w3c.github.io/media-source/#sourcebuffer-segment-parser-loop>
    fn buffer_append(&self, cx: &mut JSContext, generation: u32) {
        let data = std::mem::take(&mut *self.input_buffer.borrow_mut());

        let parsed = self.parser.borrow_mut().parse(&data);
        let Ok(parsed) = parsed else {
            // Step 2. If the input buffer contains bytes that violate the byte stream
            // format specification, then run the append error algorithm and abort this
            // algorithm.
            self.append_error(cx);
            return;
        };

        if let Some(initialization) = parsed.initialization &&
            !self.initialization_segment_received(cx, initialization)
        {
            return;
        }

        // The coded frames are handed to the media backend in the order they were
        // appended.
        if !data.is_empty() &&
            let Some(media_element) = self
                .parent_media_source()
                .and_then(|media_source| media_source.media_element())
        {
            media_element.append_media_source_data(data);
        }

        // Step 6. Add the coded frames to the track buffers.
        //
        // Coded frame processing is not implemented, so the frames reach the media backend
        // exactly as they were appended: the timestamp offset does not move them along the
        // media timeline and the append window does not drop any of them. The track buffer
        // ranges therefore record where the media actually is, rather than where applying
        // those attributes would have put it, so that `buffered` never disagrees with what
        // playback can reach.
        let mut highest = self.highest_presentation_timestamp.get();
        for segment_range in parsed.media_ranges {
            let start = segment_range.range.start;
            let end = segment_range.range.end;
            if end <= start {
                continue;
            }
            self.add_to_track_buffers(segment_range.track, start, end);
            highest = if highest.is_nan() {
                start
            } else {
                highest.max(start)
            };
        }
        self.highest_presentation_timestamp.set(highest);

        if generation != self.generation.get() {
            return;
        }

        if let Some(media_source) = self.parent_media_source() {
            media_source.media_data_appended();
        }

        // Step 7. Set the updating attribute to false.
        self.updating.set(false);

        // Step 8. Queue a task to fire an event named update at this SourceBuffer object.
        self.queue_event(Atom::from("update"));

        // Step 9. Queue a task to fire an event named updateend at this SourceBuffer
        // object.
        self.queue_event(Atom::from("updateend"));
    }

    /// <https://w3c.github.io/media-source/#dfn-initialization-segment-received>
    ///
    /// Returns whether the segment was accepted.
    fn initialization_segment_received(
        &self,
        cx: &mut JSContext,
        initialization: InitializationSegment,
    ) -> bool {
        let Some(media_source) = self.parent_media_source() else {
            return false;
        };

        // Step 1. Update the duration attribute if it currently equals NaN.
        if media_source.duration().is_nan() {
            let duration = initialization.duration.unwrap_or(f64::INFINITY);
            let _ = media_source.duration_change(duration);
        }

        // Step 2. If the initialization segment has no audio, video, or text tracks, then
        // run the append error algorithm and abort these steps.
        if initialization.tracks.is_empty() {
            self.append_error(cx);
            return false;
        }

        // Step 3. If the first initialization segment received flag is true, then verify
        // that the segment describes the same tracks as the first one did.
        if self.first_initialization_segment_received.get() {
            let audio = initialization
                .tracks
                .iter()
                .filter(|track| track.kind == TrackKind::Audio)
                .count();
            let video = initialization
                .tracks
                .iter()
                .filter(|track| track.kind == TrackKind::Video)
                .count();
            if audio != self.audio_tracks.len() || video != self.video_tracks.len() {
                self.append_error(cx);
                return false;
            }
            return true;
        }

        // Step 5. If the first initialization segment received flag is false, then create
        // the track objects the segment describes.
        let global = self.global();
        let window = global.as_window();
        for track in &initialization.tracks {
            let id = DOMString::from(track.id.to_string());
            match track.kind {
                TrackKind::Audio => {
                    let kind = if self.audio_tracks.len() == 0 {
                        DOMString::from("main")
                    } else {
                        DOMString::new()
                    };
                    let audio_track = AudioTrack::new(
                        cx,
                        window,
                        id,
                        kind,
                        DOMString::new(),
                        DOMString::new(),
                        Some(&self.audio_tracks),
                    );
                    self.audio_tracks.add(&audio_track);

                    // Step 5.2.7. If audioTracks.length equals 1, then set the enabled
                    // property on the new AudioTrack object to true and set active track
                    // flag to true.
                    if self.audio_tracks.len() == 1 {
                        self.audio_tracks.set_enabled(0, true);
                    }
                },
                TrackKind::Video => {
                    let kind = if self.video_tracks.len() == 0 {
                        DOMString::from("main")
                    } else {
                        DOMString::new()
                    };
                    let video_track = VideoTrack::new(
                        cx,
                        window,
                        id,
                        kind,
                        DOMString::new(),
                        DOMString::new(),
                        Some(&self.video_tracks),
                    );
                    self.video_tracks.add(&video_track);

                    // Step 5.3.7. If videoTracks.length equals 1, then set the selected
                    // property on the new VideoTrack object to true and set active track
                    // flag to true.
                    if self.video_tracks.len() == 1 {
                        self.video_tracks.set_selected(0, true);
                    }
                },
                // Text tracks of a media segment carry cues we have no way of surfacing
                // yet, so they are not exposed.
                TrackKind::Text => {},
            }
        }

        // Step 5.4. Create a track buffer to store coded frames for each track.
        {
            let mut track_buffers = self.track_buffers.borrow_mut();
            for track in &initialization.tracks {
                if track.kind != TrackKind::Text &&
                    !track_buffers.iter().any(|(id, _)| *id == track.id)
                {
                    track_buffers.push((track.id, TimeRangesContainer::default()));
                }
            }
        }

        // Step 5.6. Set first initialization segment received flag to true.
        self.first_initialization_segment_received.set(true);

        // Step 6. If the active track flag equals true, then add this SourceBuffer to
        // activeSourceBuffers.
        media_source.update_active_source_buffers();

        // Step 7. Set the HTMLMediaElement.readyState attribute to HAVE_METADATA.
        if let Some(media_element) = media_source.media_element() {
            media_element.media_source_metadata_received();
        }

        true
    }

    /// <https://w3c.github.io/media-source/#dfn-append-error>
    fn append_error(&self, cx: &mut JSContext) {
        // Step 1. Run the reset parser state algorithm.
        self.reset_parser_state();

        // Step 2. Set the updating attribute to false.
        self.updating.set(false);

        // Step 3. Queue a task to fire an event named error at this SourceBuffer object.
        self.queue_event(atom!("error"));

        // Step 4. Queue a task to fire an event named updateend at this SourceBuffer
        // object.
        self.queue_event(Atom::from("updateend"));

        // Step 5. Run the end of stream algorithm with the error parameter set to
        // "decode".
        if let Some(media_source) = self.parent_media_source() {
            media_source.end_of_stream_with_decode_error(cx);
        }
    }

    /// <https://w3c.github.io/media-source/#sourcebuffer-reset-parser-state>
    fn reset_parser_state(&self) {
        self.input_buffer.borrow_mut().clear();
        self.parser.borrow_mut().reset();
    }

    /// <https://w3c.github.io/media-source/#dfn-range-removal>
    ///
    /// The interval stops being reported as buffered, but the media backend has already
    /// been handed those frames and cannot drop them, so appending over a removed range
    /// is not supported yet.
    fn range_removal(&self, start: f64, end: f64, generation: u32) {
        // Step 3. Remove the media data from the track buffers.
        for (_, ranges) in self.track_buffers.borrow_mut().iter_mut() {
            ranges.remove(start, end);
        }

        if generation != self.generation.get() {
            return;
        }

        if let Some(media_source) = self.parent_media_source() {
            media_source.media_data_appended();
        }

        // Step 4. Set the updating attribute to false.
        self.updating.set(false);

        // Step 5. Queue a task to fire an event named update at this SourceBuffer object.
        self.queue_event(Atom::from("update"));

        // Step 6. Queue a task to fire an event named updateend at this SourceBuffer
        // object.
        self.queue_event(Atom::from("updateend"));
    }

    /// Whether a structure is half parsed, which the specification tracks as the
    /// `PARSING_MEDIA_SEGMENT` append state.
    fn is_parsing_media_segment(&self) -> bool {
        self.parser.borrow().is_parsing_segment()
    }

    fn queue_event(&self, name: Atom) {
        let this = Trusted::new(self);
        self.global()
            .task_manager()
            .media_element_task_source()
            .queue(task!(fire_source_buffer_event: move |cx| {
                let this = this.root();
                this.upcast::<EventTarget>().fire_event(cx, name);
            }));
    }
}

impl SourceBufferMethods<crate::DomTypeHolder> for SourceBuffer {
    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-mode>
    fn Mode(&self) -> AppendMode {
        self.mode.get()
    }

    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-mode>
    fn SetMode(&self, value: AppendMode) -> ErrorResult {
        // Step 1. If this object has been removed from the sourceBuffers attribute of the
        // parent media source, then throw an InvalidStateError exception and abort these
        // steps.
        let media_source = self.require_attached()?;

        // Step 2. If the updating attribute equals true, then throw an InvalidStateError
        // exception and abort these steps.
        if self.updating.get() {
            return Err(Error::InvalidState(Some(
                "The source buffer is still updating".into(),
            )));
        }

        // Step 3. Let new mode equal the new value being assigned to this attribute.
        // Step 4. If generate timestamps flag equals true and new mode equals "segments",
        // then throw a TypeError exception and abort these steps.
        // Neither byte stream format we support sets the generate timestamps flag.

        // Step 5. If the readyState attribute of the parent media source is in the "ended"
        // state then run the following steps.
        Self::reopen_media_source(&media_source);

        // Step 6. If the append state equals PARSING_MEDIA_SEGMENT, then throw an
        // InvalidStateError and abort these steps.
        if self.is_parsing_media_segment() {
            return Err(Error::InvalidState(Some(
                "A media segment is still being parsed".into(),
            )));
        }

        // Step 7. If the new mode equals "sequence", then set the group start timestamp to
        // the group end timestamp.
        // Step 8. Update the attribute to new mode.
        self.mode.set(value);

        Ok(())
    }

    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-updating>
    fn Updating(&self) -> bool {
        self.updating.get()
    }

    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-buffered>
    fn GetBuffered(&self, cx: &mut JSContext) -> Fallible<DomRoot<TimeRanges>> {
        // Step 1. If this object has been removed from the sourceBuffers attribute of the
        // parent media source then throw an InvalidStateError exception and abort these
        // steps.
        self.require_attached()?;

        // Steps 2-6 compute the intersection of the track buffer ranges, which is what
        // this source buffer keeps.
        let global = self.global();
        Ok(TimeRanges::new(
            cx,
            global.as_window(),
            self.buffered_ranges(),
        ))
    }

    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-timestampoffset>
    fn TimestampOffset(&self) -> Finite<f64> {
        Finite::wrap(self.timestamp_offset.get())
    }

    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-timestampoffset>
    fn SetTimestampOffset(&self, value: Finite<f64>) -> ErrorResult {
        // Step 2. If this object has been removed from the sourceBuffers attribute of the
        // parent media source, then throw an InvalidStateError exception and abort these
        // steps.
        let media_source = self.require_attached()?;

        // Step 3. If the updating attribute equals true, then throw an InvalidStateError
        // exception and abort these steps.
        if self.updating.get() {
            return Err(Error::InvalidState(Some(
                "The source buffer is still updating".into(),
            )));
        }

        // Step 4. If the readyState attribute of the parent media source is in the "ended"
        // state then run the following steps.
        Self::reopen_media_source(&media_source);

        // Step 5. If the append state equals PARSING_MEDIA_SEGMENT, then throw an
        // InvalidStateError and abort these steps.
        if self.is_parsing_media_segment() {
            return Err(Error::InvalidState(Some(
                "A media segment is still being parsed".into(),
            )));
        }

        // Step 6. If the mode attribute equals "sequence", then set the group start
        // timestamp to new timestamp offset.
        // Step 7. Update the attribute to new timestamp offset.
        self.timestamp_offset.set(*value);

        Ok(())
    }

    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-audiotracks>
    fn AudioTracks(&self) -> DomRoot<AudioTrackList> {
        DomRoot::from_ref(&*self.audio_tracks)
    }

    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-videotracks>
    fn VideoTracks(&self) -> DomRoot<VideoTrackList> {
        DomRoot::from_ref(&*self.video_tracks)
    }

    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-texttracks>
    fn TextTracks(&self) -> DomRoot<TextTrackList> {
        DomRoot::from_ref(&*self.text_tracks)
    }

    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-appendwindowstart>
    fn AppendWindowStart(&self) -> Finite<f64> {
        Finite::wrap(self.append_window_start.get())
    }

    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-appendwindowstart>
    fn SetAppendWindowStart(&self, value: Finite<f64>) -> ErrorResult {
        // Step 1. If this object has been removed from the sourceBuffers attribute of the
        // parent media source, then throw an InvalidStateError exception and abort these
        // steps.
        self.require_attached()?;

        // Step 2. If the updating attribute equals true, then throw an InvalidStateError
        // exception and abort these steps.
        if self.updating.get() {
            return Err(Error::InvalidState(Some(
                "The source buffer is still updating".into(),
            )));
        }

        // Step 3. If the new value is less than 0 or greater than or equal to
        // appendWindowEnd then throw a TypeError exception and abort these steps.
        if *value < 0. || *value >= self.append_window_end.get() {
            return Err(Error::Type(
                c"The append window start is out of range".to_owned(),
            ));
        }

        // Step 4. Update the attribute to the new value.
        self.append_window_start.set(*value);

        Ok(())
    }

    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-appendwindowend>
    fn AppendWindowEnd(&self) -> f64 {
        self.append_window_end.get()
    }

    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-appendwindowend>
    fn SetAppendWindowEnd(&self, value: f64) -> ErrorResult {
        // Step 1. If this object has been removed from the sourceBuffers attribute of the
        // parent media source, then throw an InvalidStateError exception and abort these
        // steps.
        self.require_attached()?;

        // Step 2. If the updating attribute equals true, then throw an InvalidStateError
        // exception and abort these steps.
        if self.updating.get() {
            return Err(Error::InvalidState(Some(
                "The source buffer is still updating".into(),
            )));
        }

        // Step 3. If the new value equals NaN, then throw a TypeError and abort these
        // steps.
        // Step 4. If the new value is less than or equal to appendWindowStart then throw a
        // TypeError exception and abort these steps.
        if value.is_nan() || value <= self.append_window_start.get() {
            return Err(Error::Type(
                c"The append window end is out of range".to_owned(),
            ));
        }

        // Step 5. Update the attribute to the new value.
        self.append_window_end.set(value);

        Ok(())
    }

    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-appendbuffer>
    fn AppendBuffer(&self, data: ArrayBufferViewOrArrayBuffer) -> ErrorResult {
        // Step 1. Run the prepare append algorithm.
        self.prepare_append()?;

        // Step 2. Add data to the end of the input buffer.
        let bytes = match &data {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => {
                get_buffer_source_copy(view.into())
            },
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => {
                get_buffer_source_copy(buffer.into())
            },
        };
        self.input_buffer.borrow_mut().extend_from_slice(&bytes);

        // Step 3. Set the updating attribute to true.
        self.updating.set(true);

        // Step 4. Queue a task to fire an event named updatestart at this SourceBuffer
        // object.
        // Step 5. Asynchronously run the buffer append algorithm.
        let this = Trusted::new(self);
        let generation = self.generation.get();
        self.global()
            .task_manager()
            .media_element_task_source()
            .queue(task!(source_buffer_append: move |cx| {
                let this = this.root();
                if generation != this.generation.get() {
                    return;
                }
                this.upcast::<EventTarget>().fire_event(cx, Atom::from("updatestart"));
                this.buffer_append(cx, generation);
            }));

        Ok(())
    }

    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-abort>
    fn Abort(&self) -> ErrorResult {
        // Step 1. If this object has been removed from the sourceBuffers attribute of the
        // parent media source then throw an InvalidStateError exception and abort these
        // steps.
        let media_source = self.require_attached()?;

        // Step 2. If the readyState attribute of the parent media source is not in the
        // "open" state then throw an InvalidStateError exception and abort these steps.
        if media_source.ready_state() != ReadyState::Open {
            return Err(Error::InvalidState(Some(
                "The media source is not open".into(),
            )));
        }

        // Step 3. If the range removal algorithm is running, then throw an
        // InvalidStateError exception and abort these steps.
        // Range removal completes within the task that starts it, so it is never running
        // when script regains control.

        // Step 4. If the updating attribute equals true, then run the following steps.
        self.abort_append();

        // Step 5. Run the reset parser state algorithm.
        self.reset_parser_state();

        // Step 6. Set appendWindowStart to the presentation start time.
        self.append_window_start.set(0.);

        // Step 7. Set appendWindowEnd to positive Infinity.
        self.append_window_end.set(f64::INFINITY);

        Ok(())
    }

    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-changetype>
    fn ChangeType(&self, type_: DOMString) -> ErrorResult {
        // Step 1. If type is an empty string then throw a TypeError exception and abort
        // these steps.
        if type_.is_empty() {
            return Err(Error::Type(c"The type must not be empty".to_owned()));
        }

        // Step 2. If this object has been removed from the sourceBuffers attribute of the
        // parent media source, then throw an InvalidStateError exception and abort these
        // steps.
        let media_source = self.require_attached()?;

        // Step 3. If the updating attribute equals true, then throw an InvalidStateError
        // exception and abort these steps.
        if self.updating.get() {
            return Err(Error::InvalidState(Some(
                "The source buffer is still updating".into(),
            )));
        }

        // Step 4. If type contains a MIME type that is not supported, or contains a MIME
        // type that is not supported with the types specified (currently or previously) of
        // SourceBuffer objects in the sourceBuffers attribute of the parent media source,
        // then throw a NotSupportedError exception and abort these steps.
        let format = MediaSource::byte_stream_format(&type_.str()).ok_or_else(|| {
            Error::NotSupported(Some(format!("The type \"{type_}\" is not supported")))
        })?;

        // The media backend is handed a single byte stream, so it cannot follow a change
        // of container format part way through.
        if self.parser.borrow().format() != format {
            return Err(Error::NotSupported(Some(
                "Changing the byte stream format of a source buffer is not supported".into(),
            )));
        }

        // Step 5. If the readyState attribute of the parent media source is in the "ended"
        // state then run the following steps.
        Self::reopen_media_source(&media_source);

        // Step 6. Run the reset parser state algorithm.
        self.reset_parser_state();

        // Steps 7-9. Update the generate timestamps flag, the append state, and the type.

        Ok(())
    }

    /// <https://w3c.github.io/media-source/#dom-sourcebuffer-remove>
    fn Remove(&self, start: Finite<f64>, end: f64) -> ErrorResult {
        // Step 1. If this object has been removed from the sourceBuffers attribute of the
        // parent media source then throw an InvalidStateError exception and abort these
        // steps.
        let media_source = self.require_attached()?;

        // Step 2. If the updating attribute equals true, then throw an InvalidStateError
        // exception and abort these steps.
        if self.updating.get() {
            return Err(Error::InvalidState(Some(
                "The source buffer is still updating".into(),
            )));
        }

        // Step 3. If duration equals NaN, then throw a TypeError exception and abort these
        // steps.
        let duration = media_source.duration();
        if duration.is_nan() {
            return Err(Error::Type(
                c"The media source has no known duration".to_owned(),
            ));
        }

        // Step 4. If start is negative or greater than duration, then throw a TypeError
        // exception and abort these steps.
        let start = *start;
        if start < 0. || start > duration {
            return Err(Error::Type(
                c"The start is negative or past the duration".to_owned(),
            ));
        }

        // Step 5. If end is less than or equal to start or end equals NaN, then throw a
        // TypeError exception and abort these steps.
        if end <= start || end.is_nan() {
            return Err(Error::Type(
                c"The end is not greater than the start".to_owned(),
            ));
        }

        // Step 6. If the readyState attribute of the parent media source is in the "ended"
        // state then run the following steps.
        Self::reopen_media_source(&media_source);

        // Step 7. Run the range removal algorithm with start and end as the start and end
        // of the removal range.
        //
        // Step 7.1 of that algorithm sets updating to true, and step 7.2 queues the
        // updatestart event before the removal itself runs.
        self.updating.set(true);

        let this = Trusted::new(self);
        let generation = self.generation.get();
        self.global()
            .task_manager()
            .media_element_task_source()
            .queue(task!(source_buffer_range_removal: move |cx| {
                let this = this.root();
                if generation != this.generation.get() {
                    return;
                }
                this.upcast::<EventTarget>().fire_event(cx, Atom::from("updatestart"));
                this.range_removal(start, end, generation);
            }));

        Ok(())
    }

    // <https://w3c.github.io/media-source/#dom-sourcebuffer-onupdatestart>
    event_handler!(updatestart, GetOnupdatestart, SetOnupdatestart);

    // <https://w3c.github.io/media-source/#dom-sourcebuffer-onupdate>
    event_handler!(update, GetOnupdate, SetOnupdate);

    // <https://w3c.github.io/media-source/#dom-sourcebuffer-onupdateend>
    event_handler!(updateend, GetOnupdateend, SetOnupdateend);

    // <https://w3c.github.io/media-source/#dom-sourcebuffer-onerror>
    event_handler!(error, GetOnerror, SetOnerror);

    // <https://w3c.github.io/media-source/#dom-sourcebuffer-onabort>
    event_handler!(abort, GetOnabort, SetOnabort);
}
