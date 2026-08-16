/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::ops::Range;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use mime::Mime;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::reflect_dom_object_with_proto;
use servo_media::{ServoMedia, SupportsMediaType};
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::MediaSourceBinding::{
    EndOfStreamError, MediaSourceMethods, ReadyState,
};
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::html::htmlmediaelement::HTMLMediaElement;
use crate::dom::media::bytestream::ByteStreamFormat;
use crate::dom::media::sourcebuffer::SourceBuffer;
use crate::dom::media::sourcebufferlist::SourceBufferList;
use crate::dom::timeranges::TimeRangesContainer;
use crate::dom::window::Window;

/// <https://w3c.github.io/media-source/#mediasource>
#[dom_struct]
pub(crate) struct MediaSource {
    eventtarget: EventTarget,

    /// <https://w3c.github.io/media-source/#dom-mediasource-sourcebuffers>
    source_buffers: Dom<SourceBufferList>,

    /// <https://w3c.github.io/media-source/#dom-mediasource-activesourcebuffers>
    active_source_buffers: Dom<SourceBufferList>,

    /// <https://w3c.github.io/media-source/#dom-mediasource-readystate>
    ready_state: Cell<ReadyState>,

    /// <https://w3c.github.io/media-source/#dom-mediasource-duration>
    duration: Cell<f64>,

    /// The media element this media source is attached to, if any.
    ///
    /// <https://w3c.github.io/media-source/#dfn-attaching-to-a-media-element>
    media_element: MutNullableDom<HTMLMediaElement>,

    /// <https://w3c.github.io/media-source/#dfn-live-seekable-range>
    #[no_trace]
    live_seekable_range: DomRefCell<Option<Range<f64>>>,

    /// <https://w3c.github.io/media-source/#dfn-has-ever-been-attached>
    has_ever_been_attached: Cell<bool>,
}

impl MediaSource {
    fn new_inherited(
        source_buffers: &SourceBufferList,
        active_source_buffers: &SourceBufferList,
    ) -> MediaSource {
        MediaSource {
            eventtarget: EventTarget::new_inherited(),
            source_buffers: Dom::from_ref(source_buffers),
            active_source_buffers: Dom::from_ref(active_source_buffers),
            // Step 2. Set the readyState attribute to "closed".
            ready_state: Cell::new(ReadyState::Closed),
            // Step 3. Set the duration attribute to NaN.
            duration: Cell::new(f64::NAN),
            media_element: Default::default(),
            live_seekable_range: DomRefCell::new(None),
            has_ever_been_attached: Cell::new(false),
        }
    }

    /// <https://w3c.github.io/media-source/#dom-mediasource-mediasource>
    pub(crate) fn new_with_proto(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
    ) -> DomRoot<MediaSource> {
        let window = global.as_window();

        // Step 1. Set the live seekable range to an empty TimeRanges object.
        // Step 4. Set the sourceBuffers attribute to a new empty SourceBufferList object.
        let source_buffers = SourceBufferList::new(cx, window);

        // Step 5. Set the activeSourceBuffers attribute to a new empty SourceBufferList
        // object.
        let active_source_buffers = SourceBufferList::new(cx, window);

        reflect_dom_object_with_proto(
            cx,
            Box::new(MediaSource::new_inherited(
                &source_buffers,
                &active_source_buffers,
            )),
            global,
            proto,
        )
    }

    pub(crate) fn ready_state(&self) -> ReadyState {
        self.ready_state.get()
    }

    pub(crate) fn duration(&self) -> f64 {
        self.duration.get()
    }

    pub(crate) fn media_element(&self) -> Option<DomRoot<HTMLMediaElement>> {
        self.media_element.get()
    }

    /// <https://w3c.github.io/media-source/#dfn-attaching-to-a-media-element>
    ///
    /// Returns whether the media source could be attached. A media source that has
    /// already been attached, or that is not in the `"closed"` ready state, makes the
    /// resource fetch algorithm fail.
    pub(crate) fn attach(&self, cx: &mut JSContext, media_element: &HTMLMediaElement) -> bool {
        // If readyState is NOT set to "closed", then run the "If the media data cannot be
        // fetched at all, due to network errors, causing the user agent to give up trying
        // to fetch the resource" steps of the resource fetch algorithm.
        if self.ready_state.get() != ReadyState::Closed || self.has_ever_been_attached.get() {
            return false;
        }

        self.has_ever_been_attached.set(true);
        self.media_element.set(Some(media_element));

        // Set the media element's delaying-the-load-event flag to false.
        media_element.delay_load_event(false, cx);

        // Set the readyState attribute to "open".
        self.ready_state.set(ReadyState::Open);

        // Queue a task to fire an event named sourceopen at the MediaSource.
        self.queue_event(Atom::from("sourceopen"));

        true
    }

    /// <https://w3c.github.io/media-source/#dfn-detaching-from-a-media-element>
    pub(crate) fn detach(&self) {
        if self.media_element.get().is_none() {
            return;
        }

        // Step 1. Set the readyState attribute to "closed".
        self.ready_state.set(ReadyState::Closed);

        // Step 2. Update duration to NaN.
        self.duration.set(f64::NAN);

        // Step 3. Remove all the SourceBuffer objects from activeSourceBuffers.
        // Step 4. Queue a task to fire an event named removesourcebuffer at
        // activeSourceBuffers.
        let had_active_source_buffers = !self.active_source_buffers.is_empty();
        self.active_source_buffers.clear();
        if had_active_source_buffers {
            self.active_source_buffers.queue_removesourcebuffer_event();
        }

        // Step 5. Remove all the SourceBuffer objects from sourceBuffers.
        // Step 6. Queue a task to fire an event named removesourcebuffer at sourceBuffers.
        let had_source_buffers = !self.source_buffers.is_empty();
        for source_buffer in self.source_buffers.buffers() {
            source_buffer.detach();
        }
        self.source_buffers.clear();
        if had_source_buffers {
            self.source_buffers.queue_removesourcebuffer_event();
        }

        // Step 7. Queue a task to fire an event named sourceclose at the MediaSource.
        self.queue_event(Atom::from("sourceclose"));

        self.media_element.set(None);
    }

    /// Recomputes `activeSourceBuffers`, which the specification keeps in the same order
    /// as `sourceBuffers`.
    ///
    /// <https://w3c.github.io/media-source/#dom-mediasource-activesourcebuffers>
    pub(crate) fn update_active_source_buffers(&self) {
        let active: Vec<_> = self
            .source_buffers
            .buffers()
            .into_iter()
            .filter(|source_buffer| source_buffer.is_active())
            .collect();
        self.active_source_buffers.update(&active);
    }

    /// The largest end time across the track buffer ranges of all source buffers.
    ///
    /// <https://w3c.github.io/media-source/#dfn-duration-change>
    fn highest_end_time(&self) -> Option<f64> {
        self.source_buffers
            .buffers()
            .into_iter()
            .filter_map(|source_buffer| source_buffer.highest_track_end_time())
            .reduce(f64::max)
    }

    /// <https://w3c.github.io/media-source/#dfn-duration-change>
    pub(crate) fn duration_change(&self, new_duration: f64) -> ErrorResult {
        // Step 1. If the current value of duration is equal to new duration, then return.
        if self.duration.get() == new_duration ||
            (self.duration.get().is_nan() && new_duration.is_nan())
        {
            return Ok(());
        }

        // Step 2. If new duration is less than the highest starting presentation timestamp
        // of any buffered coded frames for all SourceBuffer objects in sourceBuffers, then
        // throw an InvalidStateError exception and abort these steps.
        let highest_start = self
            .source_buffers
            .buffers()
            .into_iter()
            .filter_map(|source_buffer| source_buffer.highest_presentation_timestamp())
            .reduce(f64::max);
        if let Some(highest_start) = highest_start &&
            new_duration < highest_start
        {
            return Err(Error::InvalidState(Some(
                "The new duration is less than the highest buffered presentation timestamp".into(),
            )));
        }

        // Step 3. Let highest end time be the largest track buffer ranges end time across
        // all the track buffers across all SourceBuffer objects in sourceBuffers.
        // Step 4. If new duration is less than highest end time, then update new duration
        // to equal highest end time.
        let new_duration = match self.highest_end_time() {
            Some(highest_end_time) => f64::max(new_duration, highest_end_time),
            None => new_duration,
        };

        // Step 5. Update duration to new duration.
        self.duration.set(new_duration);

        // Step 6. Update the media duration to new duration and run the HTMLMediaElement
        // duration change algorithm.
        if let Some(media_element) = self.media_element.get() {
            media_element.media_source_duration_changed(new_duration);
        }

        Ok(())
    }

    /// <https://w3c.github.io/media-source/#dfn-end-of-stream>
    fn end_of_stream_algorithm(&self, cx: &mut JSContext, error: Option<EndOfStreamError>) {
        // Step 1. Change the readyState attribute value to "ended".
        self.ready_state.set(ReadyState::Ended);

        // Step 2. Queue a task to fire an event named sourceended at the MediaSource.
        self.queue_event(Atom::from("sourceended"));

        let Some(media_element) = self.media_element.get() else {
            return;
        };

        match error {
            // Step 3. If error is not set:
            None => {
                // Step 3.1. Run the duration change algorithm with new duration set to the
                // largest track buffer ranges end time across all the track buffers across
                // all SourceBuffer objects in sourceBuffers.
                if let Some(highest_end_time) = self.highest_end_time() {
                    let _ = self.duration_change(highest_end_time);
                }

                // Step 3.2. Notify the media element that it now has all of the media data.
                media_element.media_source_end_of_stream();
            },
            // Step 4. If error is set to "network", and step 5 for "decode", run the
            // corresponding media data processing steps of the media element.
            Some(error) => {
                media_element.media_source_error(cx, error);
            },
        }
    }

    /// Moves an `"ended"` media source back to `"open"`, which the specification asks
    /// several `SourceBuffer` methods to do before they mutate anything.
    pub(crate) fn reopen(&self) {
        if self.ready_state.get() != ReadyState::Ended {
            return;
        }

        self.ready_state.set(ReadyState::Open);
        self.queue_event(Atom::from("sourceopen"));
    }

    /// <https://w3c.github.io/media-source/#dfn-append-error>
    ///
    /// Step 5 of the append error algorithm ends the stream with a decode error.
    pub(crate) fn end_of_stream_with_decode_error(&self, cx: &mut JSContext) {
        self.end_of_stream_algorithm(cx, Some(EndOfStreamError::Decode));
    }

    /// Whether an append is in flight on any of the source buffers.
    fn is_updating(&self) -> bool {
        self.source_buffers
            .buffers()
            .into_iter()
            .any(|source_buffer| source_buffer.updating())
    }

    /// <https://w3c.github.io/media-source/#dfn-mediasource-seekable>
    pub(crate) fn seekable(&self) -> TimeRangesContainer {
        let mut ranges = TimeRangesContainer::default();

        // Step 1. Let recent duration be the current value of the duration attribute.
        let duration = self.duration.get();

        // Step 2. If recent duration is NaN, then return an empty TimeRanges object.
        if duration.is_nan() {
            return ranges;
        }

        // Step 3. If recent duration is positive infinity:
        if duration.is_infinite() {
            let buffered = self.media_element_buffered();
            let live_seekable_range = self.live_seekable_range.borrow();

            // Step 3.1. If live seekable range is not empty:
            if let Some(live_seekable_range) = live_seekable_range.as_ref() {
                let start = match buffered.start_time() {
                    Some(start) => f64::min(live_seekable_range.start, start),
                    None => live_seekable_range.start,
                };
                let end = match buffered.end_time() {
                    Some(end) => f64::max(live_seekable_range.end, end),
                    None => live_seekable_range.end,
                };
                let _ = ranges.add(start, end);
                return ranges;
            }

            // Step 3.2. If the HTMLMediaElement.buffered attribute returns an empty
            // TimeRanges object, then return an empty TimeRanges object and abort these
            // steps.
            let Some(end) = buffered.end_time() else {
                return ranges;
            };

            // Step 3.3. Return a single range with a start time of 0 and an end time equal
            // to the highest end time reported by the HTMLMediaElement.buffered attribute.
            let _ = ranges.add(0., end);
            return ranges;
        }

        // Step 4. Otherwise return a single range with a start time of 0 and an end time
        // equal to recent duration.
        let _ = ranges.add(0., duration);
        ranges
    }

    /// <https://w3c.github.io/media-source/#htmlmediaelement-extensions>
    ///
    /// The intersection of the buffered ranges of every active source buffer.
    pub(crate) fn media_element_buffered(&self) -> TimeRangesContainer {
        let active_source_buffers = self.active_source_buffers.buffers();

        // Step 1. If activeSourceBuffers.length equals 0 then return an empty TimeRanges
        // object and abort these steps.
        if active_source_buffers.is_empty() {
            return TimeRangesContainer::default();
        }

        // Step 2. Let active ranges be the ranges returned by buffered for each
        // SourceBuffer object in activeSourceBuffers.
        let active_ranges: Vec<_> = active_source_buffers
            .iter()
            .map(|source_buffer| source_buffer.buffered_ranges())
            .collect();

        // Step 3. Let highest end time be the largest range end time in the active ranges.
        let Some(highest_end_time) = active_ranges
            .iter()
            .filter_map(|ranges| ranges.end_time())
            .reduce(f64::max)
        else {
            return TimeRangesContainer::default();
        };

        // Step 4. Let intersection ranges equal a TimeRanges object containing a single
        // range from 0 to highest end time.
        if highest_end_time <= 0. {
            return TimeRangesContainer::default();
        }
        let mut intersection_ranges = TimeRangesContainer::default();
        let _ = intersection_ranges.add(0., highest_end_time);

        // Step 5. For each SourceBuffer object in activeSourceBuffers run the following
        // steps.
        let ended = self.ready_state.get() == ReadyState::Ended;
        for mut source_ranges in active_ranges {
            // Step 5.2. If readyState is "ended", then set the end time on the last range
            // in source ranges to highest end time.
            if ended {
                source_ranges.set_end_time(highest_end_time);
            }

            // Step 5.3. Let new intersection ranges equal the intersection between the
            // intersection ranges and the source ranges.
            intersection_ranges = intersection_ranges.intersection(&source_ranges);
        }

        intersection_ranges
    }

    /// Called by a source buffer once an append made new media data available.
    pub(crate) fn media_data_appended(&self) {
        self.update_active_source_buffers();

        // The media element's duration follows the media source for as long as they are
        // attached, so the buffered ranges are all it has to refresh.
        if let Some(media_element) = self.media_element.get() {
            media_element.media_source_buffered_changed();
        }
    }

    fn queue_event(&self, name: Atom) {
        let this = Trusted::new(self);
        self.global()
            .task_manager()
            .media_element_task_source()
            .queue(task!(fire_media_source_event: move |cx| {
                let this = this.root();
                this.upcast::<EventTarget>().fire_event(cx, name);
            }));
    }

    /// <https://w3c.github.io/media-source/#dom-mediasource-istypesupported>
    ///
    /// Returns the byte stream format `type` maps to, if it is one this implementation
    /// can both parse and hand over to the media backend.
    pub(crate) fn byte_stream_format(type_: &str) -> Option<ByteStreamFormat> {
        // Step 1. If type is an empty string, then return false.
        if type_.is_empty() {
            return None;
        }

        // Step 2. If type does not contain a valid MIME type string, then return false.
        let mime: Mime = type_.parse().ok()?;

        // Step 3. If type contains a media type or media subtype that the MediaSource does
        // not support, then return false.
        let format = match mime.subtype().as_str() {
            "mp4" => ByteStreamFormat::IsoBmff,
            "webm" => ByteStreamFormat::WebM,
            _ => return None,
        };
        let audio_only = match mime.type_().as_str() {
            "audio" => true,
            "video" => false,
            _ => return None,
        };

        // Step 4. If type contains a media type, media subtype, or codec that is not
        // supported by the MediaSource, then return false.
        //
        // The registry for each byte stream format lists the codecs it may carry, and a
        // codec that produces video can never appear in an audio container.
        let codecs = mime.get_param("codecs")?;
        let codecs: Vec<&str> = codecs
            .as_str()
            .split(',')
            .map(|codec| codec.trim())
            .collect();
        if codecs.is_empty() || codecs.iter().any(|codec| codec.is_empty()) {
            return None;
        }
        for codec in &codecs {
            let Some(codec) = CodecFamily::parse(codec) else {
                return None;
            };
            if !codec.is_carried_by(format) {
                return None;
            }
            if audio_only && codec.is_video() {
                return None;
            }
        }

        // Finally, defer to the media backend for the codecs it can actually decode.
        if ServoMedia::get().can_play_type(type_) != SupportsMediaType::Probably {
            return None;
        }

        Some(format)
    }
}

/// The codecs the byte stream format registries allow, grouped by the container they may
/// appear in.
///
/// <https://w3c.github.io/mse-byte-stream-format-registry/>
#[derive(Clone, Copy, PartialEq)]
enum CodecFamily {
    /// A video codec only ISO BMFF may carry.
    IsoBmffVideo,
    /// An audio codec only ISO BMFF may carry.
    IsoBmffAudio,
    /// A video codec only WebM may carry.
    WebMVideo,
    /// An audio codec only WebM may carry.
    WebMAudio,
    /// A video codec both formats may carry.
    SharedVideo,
    /// An audio codec both formats may carry.
    SharedAudio,
}

impl CodecFamily {
    fn parse(codec: &str) -> Option<CodecFamily> {
        // Codec identifiers are matched case insensitively, and everything after the first
        // dot is a profile or level that only the media backend can judge.
        let codec = codec.to_ascii_lowercase();
        let name = codec.split('.').next().unwrap_or_default();

        match name {
            "avc1" | "avc3" | "hev1" | "hvc1" | "mp4v" => Some(CodecFamily::IsoBmffVideo),
            "mp4a" | "ac-3" | "ec-3" => Some(CodecFamily::IsoBmffAudio),
            "vp8" | "vp9" => Some(CodecFamily::WebMVideo),
            "vorbis" => Some(CodecFamily::WebMAudio),
            // VP9 in its ISO BMFF spelling, and AV1, are registered for both formats.
            "vp09" | "av01" => Some(CodecFamily::SharedVideo),
            "opus" | "flac" => Some(CodecFamily::SharedAudio),
            _ => None,
        }
    }

    fn is_carried_by(&self, format: ByteStreamFormat) -> bool {
        match self {
            CodecFamily::IsoBmffVideo | CodecFamily::IsoBmffAudio => {
                format == ByteStreamFormat::IsoBmff
            },
            CodecFamily::WebMVideo | CodecFamily::WebMAudio => format == ByteStreamFormat::WebM,
            CodecFamily::SharedVideo | CodecFamily::SharedAudio => true,
        }
    }

    fn is_video(&self) -> bool {
        matches!(
            self,
            CodecFamily::IsoBmffVideo | CodecFamily::WebMVideo | CodecFamily::SharedVideo
        )
    }
}

impl MediaSourceMethods<crate::DomTypeHolder> for MediaSource {
    /// <https://w3c.github.io/media-source/#dom-mediasource-mediasource>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
    ) -> DomRoot<MediaSource> {
        MediaSource::new_with_proto(cx, &window.global(), proto)
    }

    /// <https://w3c.github.io/media-source/#dom-mediasource-sourcebuffers>
    fn SourceBuffers(&self) -> DomRoot<SourceBufferList> {
        DomRoot::from_ref(&*self.source_buffers)
    }

    /// <https://w3c.github.io/media-source/#dom-mediasource-activesourcebuffers>
    fn ActiveSourceBuffers(&self) -> DomRoot<SourceBufferList> {
        DomRoot::from_ref(&*self.active_source_buffers)
    }

    /// <https://w3c.github.io/media-source/#dom-mediasource-readystate>
    fn ReadyState(&self) -> ReadyState {
        self.ready_state.get()
    }

    /// <https://w3c.github.io/media-source/#dom-mediasource-duration>
    fn Duration(&self) -> f64 {
        // If readyState equals "closed" then return NaN and abort these steps.
        if self.ready_state.get() == ReadyState::Closed {
            return f64::NAN;
        }

        self.duration.get()
    }

    /// <https://w3c.github.io/media-source/#dom-mediasource-duration>
    fn SetDuration(&self, value: f64) -> ErrorResult {
        // Step 1. If the value being set is negative or NaN then throw a TypeError
        // exception and abort these steps.
        if value.is_nan() || value < 0. {
            return Err(Error::Type(
                c"The duration must not be negative or NaN".to_owned(),
            ));
        }

        // Step 2. If the readyState attribute is not "open" then throw an
        // InvalidStateError exception and abort these steps.
        if self.ready_state.get() != ReadyState::Open {
            return Err(Error::InvalidState(Some(
                "The media source is not open".into(),
            )));
        }

        // Step 3. If the updating attribute equals true on any SourceBuffer in
        // sourceBuffers, then throw an InvalidStateError exception and abort these steps.
        if self.is_updating() {
            return Err(Error::InvalidState(Some(
                "A source buffer is still updating".into(),
            )));
        }

        // Step 4. Run the duration change algorithm with new duration set to the value
        // being assigned to this attribute.
        self.duration_change(value)
    }

    /// <https://w3c.github.io/media-source/#dom-mediasource-canconstructindedicatedworker>
    fn CanConstructInDedicatedWorker(_window: &Window) -> bool {
        // Media sources are not exposed to dedicated workers yet.
        false
    }

    /// <https://w3c.github.io/media-source/#dom-mediasource-addsourcebuffer>
    fn AddSourceBuffer(
        &self,
        cx: &mut JSContext,
        type_: DOMString,
    ) -> Fallible<DomRoot<SourceBuffer>> {
        // Step 1. If type is an empty string then throw a TypeError exception and abort
        // these steps.
        if type_.is_empty() {
            return Err(Error::Type(c"The type must not be empty".to_owned()));
        }

        // Step 2. If type contains a MIME type that is not supported or contains a MIME
        // type that is not supported with the types specified for the SourceBuffer objects
        // in sourceBuffers, then throw a NotSupportedError exception and abort these steps.
        let Some(format) = MediaSource::byte_stream_format(&type_.str()) else {
            return Err(Error::NotSupported(Some(format!(
                "The type \"{type_}\" is not supported"
            ))));
        };

        // Step 3. If the user agent can't handle any more SourceBuffer objects or if
        // creating a SourceBuffer based on type would result in an unsupported
        // SourceBuffer configuration, then throw a QuotaExceededError exception and abort
        // these steps.
        //
        // Every source buffer feeds the same pipeline, so a second one would interleave
        // its byte stream with the first.
        if !self.source_buffers.is_empty() {
            return Err(Error::QuotaExceeded {
                quota: None,
                requested: None,
            });
        }

        // Step 4. If the readyState attribute is not in the "open" state then throw an
        // InvalidStateError exception and abort these steps.
        if self.ready_state.get() != ReadyState::Open {
            return Err(Error::InvalidState(Some(
                "The media source is not open".into(),
            )));
        }

        // Step 5. Create a new SourceBuffer object and associated resources.
        // Step 6. Set the generate timestamps flag on the new object to the value in the
        // "Generate Timestamps Flag" column of the byte stream format registry entry that
        // is associated with type.
        // Step 7. If the generate timestamps flag equals true set the mode attribute on
        // the new object to "sequence". Otherwise set it to "segments".
        //
        // Neither of the byte stream formats we support sets the generate timestamps flag.
        let source_buffer = SourceBuffer::new(cx, self, format);

        // Step 8. Add the new object to sourceBuffers and queue a task to fire an event
        // named addsourcebuffer at sourceBuffers.
        self.source_buffers.add(&source_buffer);

        // Step 9. Return the new object.
        Ok(source_buffer)
    }

    /// <https://w3c.github.io/media-source/#dom-mediasource-removesourcebuffer>
    fn RemoveSourceBuffer(&self, source_buffer: &SourceBuffer) -> ErrorResult {
        // Step 1. If sourceBuffer specifies an object that is not in sourceBuffers then
        // throw a NotFoundError exception and abort these steps.
        if !self.source_buffers.contains(source_buffer) {
            return Err(Error::NotFound(Some(
                "The source buffer is not in this media source".into(),
            )));
        }

        // Step 2. If the sourceBuffer.updating attribute equals true, then run the
        // following steps.
        source_buffer.abort_append();

        // Step 3. Let SourceBuffer audioTracks list equal the AudioTrackList object
        // returned by sourceBuffer.audioTracks.
        // Steps 4-8, and their video and text track equivalents, remove the tracks of the
        // source buffer from the media element.
        source_buffer.detach();

        // Step 9. If sourceBuffer is in activeSourceBuffers, then remove sourceBuffer from
        // activeSourceBuffers and queue a task to fire an event named removesourcebuffer
        // at the SourceBufferList returned by activeSourceBuffers.
        self.update_active_source_buffers();

        // Step 10. Remove sourceBuffer from sourceBuffers and queue a task to fire an
        // event named removesourcebuffer at the SourceBufferList returned by sourceBuffers.
        self.source_buffers.remove(source_buffer);

        // Step 11. Destroy all resources for sourceBuffer.
        // The source buffer is now unreachable from the media source, so the collector
        // takes care of the rest.

        Ok(())
    }

    /// <https://w3c.github.io/media-source/#dom-mediasource-endofstream>
    fn EndOfStream(&self, cx: &mut JSContext, error: Option<EndOfStreamError>) -> ErrorResult {
        // Step 1. If the readyState attribute is not in the "open" state then throw an
        // InvalidStateError exception and abort these steps.
        if self.ready_state.get() != ReadyState::Open {
            return Err(Error::InvalidState(Some(
                "The media source is not open".into(),
            )));
        }

        // Step 2. If the updating attribute equals true on any SourceBuffer in
        // sourceBuffers, then throw an InvalidStateError exception and abort these steps.
        if self.is_updating() {
            return Err(Error::InvalidState(Some(
                "A source buffer is still updating".into(),
            )));
        }

        // Step 3. Run the end of stream algorithm with the error parameter set to error.
        self.end_of_stream_algorithm(cx, error);

        Ok(())
    }

    /// <https://w3c.github.io/media-source/#dom-mediasource-setliveseekablerange>
    fn SetLiveSeekableRange(&self, start: Finite<f64>, end: Finite<f64>) -> ErrorResult {
        // Step 1. If the readyState attribute is not "open" then throw an
        // InvalidStateError exception and abort these steps.
        if self.ready_state.get() != ReadyState::Open {
            return Err(Error::InvalidState(Some(
                "The media source is not open".into(),
            )));
        }

        // Step 2. If start is negative or greater than end, then throw a TypeError
        // exception and abort these steps.
        let (start, end) = (*start, *end);
        if start < 0. || start > end {
            return Err(Error::Type(
                c"The start must not be negative or greater than the end".to_owned(),
            ));
        }

        // Step 3. Set live seekable range to be a new normalized TimeRanges object
        // containing a single range whose start position is start and end position is end.
        *self.live_seekable_range.borrow_mut() = Some(start..end);

        Ok(())
    }

    /// <https://w3c.github.io/media-source/#dom-mediasource-clearliveseekablerange>
    fn ClearLiveSeekableRange(&self) -> ErrorResult {
        // Step 1. If the readyState attribute is not "open" then throw an
        // InvalidStateError exception and abort these steps.
        if self.ready_state.get() != ReadyState::Open {
            return Err(Error::InvalidState(Some(
                "The media source is not open".into(),
            )));
        }

        // Step 2. If live seekable range contains a range, then set live seekable range to
        // be a new empty TimeRanges object.
        *self.live_seekable_range.borrow_mut() = None;

        Ok(())
    }

    /// <https://w3c.github.io/media-source/#dom-mediasource-istypesupported>
    fn IsTypeSupported(_window: &Window, type_: DOMString) -> bool {
        MediaSource::byte_stream_format(&type_.str()).is_some()
    }

    // <https://w3c.github.io/media-source/#dom-mediasource-onsourceopen>
    event_handler!(sourceopen, GetOnsourceopen, SetOnsourceopen);

    // <https://w3c.github.io/media-source/#dom-mediasource-onsourceended>
    event_handler!(sourceended, GetOnsourceended, SetOnsourceended);

    // <https://w3c.github.io/media-source/#dom-mediasource-onsourceclose>
    event_handler!(sourceclose, GetOnsourceclose, SetOnsourceclose);
}
