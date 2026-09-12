use std::{cell::RefCell, rc::Rc};

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::{
    codegen::GenericBindings::{
        EventHandlerBinding::EventHandlerNonNull,
        MediaStreamRecordingBinding::{
            BitrateMode, MediaRecorderMethods, MediaRecorderOptions, RecordingState,
        },
    },
    error::{Error, Fallible},
    reflector::reflect_dom_object_with_cx,
    root::{Dom, DomRoot},
    str::DOMString,
};

use crate::dom::{
    Window,
    bindings::reflector::DomGlobal,
    types::{EventTarget, MediaStream},
};

/// <https://www.w3.org/TR/mediastream-recording/#list-of-synchronously-exposed-codec-identifiers>
const SYNCHRONOUSLY_EXPOSED_CODEC_IDENTIFIERS: &[&str] = &[
    "vp8", "vp9", "h264", "avc1", "av1", "av01", "hvc1", "hev1", "avc1", "avc3", "opus", "pcm",
];

#[dom_struct]
pub(crate) struct MediaRecorder {
    eventtarget: EventTarget,
    stream: Dom<MediaStream>,
    mime_type: DOMString,
    state: RefCell<RecordingState>,
    audio_bits_per_second: u32,
    video_bits_per_second: u32,
    audio_bitrate_mode: BitrateMode,
}

impl MediaRecorder {
    // TODO: pass whole options
    fn new_inherited(stream: &MediaStream, mime_type: DOMString) -> Self {
        MediaRecorder {
            eventtarget: EventTarget::new_inherited(),
            stream: Dom::from_ref(stream),
            mime_type,
            state: RefCell::new(RecordingState::Inactive),
            audio_bits_per_second: 0,
            video_bits_per_second: 0,
            audio_bitrate_mode: BitrateMode::Variable,
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &Window,
        stream: &MediaStream,
        mime_type: DOMString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_cx(
            Box::new(MediaRecorder::new_inherited(stream, mime_type)),
            global,
            cx,
        )
    }

    /// <https://www.w3.org/TR/mediastream-recording/#abstract-opdef-is-type-supported>
    fn is_type_supported(type_: &DOMString, defer_newer_codecs_check: bool) -> bool {
        // Step 1. if type is empty string, return true (leaving up the choice to UA).
        if type_.is_empty() {
            return true;
        }

        // Step 2. if type is not a valid MIME type stirng, return false.
        // TODO: is there existing check util

        // Step 3. If MediaRecorder does not support the combination of media type/subtype and container
        // specified in type, return false.
        // TODO: how to check

        // Step 4. Let codecStrings be the list result of strictly splitting on "," the string after codecs=
        // if present in type, or an empty list otherwise.
        let codec_strings: Vec<String> = type_
            .str()
            .split_once("codecs=")
            .unwrap()
            .1
            .split(',')
            .map(|s| s.to_string())
            .collect();

        // Step 5. If codecStrings contains more than one audio codec or more than one video codec, return false.
        // TODO: how to check codec type

        // Step 6. Let codecIdentifiers be an empty list.
        let mut codec_identifiers = vec![];

        // Step 7. For each codecString in codecStrings, run the following steps:
        for codec_string in codec_strings {
            // Step 7.1. Let codecIdentifier be the ASCII lowercase of the first part of strictly splitting codecString on ".".
            let code_identifier = codec_string.split('.').next().unwrap().to_ascii_lowercase();
            // Step 7.2. Append codecIdentifier to codecIdentifiers.
            codec_identifiers.push(code_identifier);
        }

        // Step 8. For each codecIdentifier in codecIdentifiers that is synchronously exposed, run the following step:
        for codec_identifier in codec_identifiers.iter() {
            // Step 8.1. If the MediaRecorder does not support codecIdentifier in combination with the media type/subtype
            // and container specified in type, then return false.
            // TODO: define support
        }

        // Step 9. If any codecIdentifier in codecIdentifiers is not synchronously exposed, return deferNewerCodecsCheck.
        if codec_identifiers.iter().any(is_synchronously_expoesd) {
            return defer_newer_codecs_check;
        }

        // Step 10. Return true.
        return true;

        /// <https://www.w3.org/TR/mediastream-recording/#abstract-opdef-is-synchronously-exposed>
        fn is_synchronously_expoesd(code_identifier: &String) -> bool {
            // Step 1. Return true if any item in the list of synchronously exposed codec identifiers
            // is an exact match for codecIdentifier, otherwise false.
            SYNCHRONOUSLY_EXPOSED_CODEC_IDENTIFIERS.contains(&code_identifier.as_str())
        }
    }

    /// <https://www.w3.org/TR/mediastream-recording/#abstract-opdef-inactivate-the-recorder>
    fn inactivate_recorder(&self) {
        // Step 1. Set recorder’s mimeType attribute to the value of the [[ConstrainedMimeType]] slot.
        // TODO
        // Step 2. Set recorder’s state attribute to inactive.
        // TODO
        // Step 3. If recorder’s [[ConstrainedBitsPerSecond]] slot is not undefined, set recorder’s videoBitsPerSecond and audioBitsPerSecond attributes to values the User Agent deems reasonable for the respective media types, such that the sum of videoBitsPerSecond and audioBitsPerSecond is close to the value of recorder’s [[ConstrainedBitsPerSecond]] slot.
        // TODO
    }
}

impl MediaRecorderMethods<crate::DomTypeHolder> for MediaRecorder {
    fn Stream(&self) -> DomRoot<MediaStream> {
        DomRoot::from_ref(&self.stream)
    }

    fn MimeType(&self) -> DOMString {
        self.mime_type.clone()
    }

    fn State(&self) -> RecordingState {
        *self.state.borrow()
    }

    fn GetOnstart(
        &self,
        cx: &mut JSContext,
    ) -> Option<Rc<EventHandlerNonNull<crate::DomTypeHolder>>> {
        todo!()
    }

    fn SetOnstart(
        &self,
        cx: &mut JSContext,
        value: Option<Rc<EventHandlerNonNull<crate::DomTypeHolder>>>,
    ) {
        todo!()
    }

    fn GetOnstop(
        &self,
        cx: &mut JSContext,
    ) -> Option<Rc<EventHandlerNonNull<crate::DomTypeHolder>>> {
        todo!()
    }

    fn SetOnstop(
        &self,
        cx: &mut JSContext,
        value: Option<Rc<EventHandlerNonNull<crate::DomTypeHolder>>>,
    ) {
        todo!()
    }

    fn GetOndataavailable(
        &self,
        cx: &mut JSContext,
    ) -> Option<Rc<EventHandlerNonNull<crate::DomTypeHolder>>> {
        todo!()
    }

    fn SetOndataavailable(
        &self,
        cx: &mut JSContext,
        value: Option<Rc<EventHandlerNonNull<crate::DomTypeHolder>>>,
    ) {
        todo!()
    }

    fn GetOnpause(
        &self,
        cx: &mut JSContext,
    ) -> Option<Rc<EventHandlerNonNull<crate::DomTypeHolder>>> {
        todo!()
    }

    fn SetOnpause(
        &self,
        cx: &mut JSContext,
        value: Option<Rc<EventHandlerNonNull<crate::DomTypeHolder>>>,
    ) {
        todo!()
    }

    fn GetOnresume(
        &self,
        cx: &mut JSContext,
    ) -> Option<Rc<EventHandlerNonNull<crate::DomTypeHolder>>> {
        todo!()
    }

    fn SetOnresume(
        &self,
        cx: &mut JSContext,
        value: Option<Rc<EventHandlerNonNull<crate::DomTypeHolder>>>,
    ) {
        todo!()
    }

    fn GetOnerror(
        &self,
        cx: &mut JSContext,
    ) -> Option<Rc<EventHandlerNonNull<crate::DomTypeHolder>>> {
        todo!()
    }

    fn SetOnerror(
        &self,
        cx: &mut JSContext,
        value: Option<Rc<EventHandlerNonNull<crate::DomTypeHolder>>>,
    ) {
        todo!()
    }

    fn VideoBitsPerSecond(&self) -> u32 {
        self.video_bits_per_second
    }

    fn AudioBitsPerSecond(&self) -> u32 {
        self.audio_bits_per_second
    }

    fn AudioBitrateMode(&self) -> BitrateMode {
        self.audio_bitrate_mode
    }

    fn Start(&self, timeslice: Option<u32>) -> Fallible<()> {
        // Step 1. Let recorder be the MediaRecorder object on which the method was invoked. SKIP
        // Step 2. Let timeslice be the method’s first argument, if provided, or undefined. SKIP

        // Step 3. Let stream be the value of recorder’s stream attribute.
        let stream = &self.stream;

        // Step 4. Let tracks be the set of live tracks in stream’s track set.
        // TODO: live
        let tracks = stream.get_tracks();

        // TODO
        // Step 5. If the value of recorder’s state attribute is not inactive, throw an InvalidStateError DOMException and abort these steps.
        // TODO
        // Step 6. If the isolation properties of stream disallow access from recorder, throw a SecurityError DOMException and abort these steps.
        // TODO

        // Step 7. If stream is inactive, throw a NotSupportedError DOMException and abort these steps.
        // TODO: check inactive
        if false {
            return Err(Error::NotSupported(Some("stream is inactive".into())));
        }

        // Step 8. If the [[ConstrainedMimeType]] slot specifies a media type, container, or codec, then constrain the configuration of recorder to the media type, container, and codec specified in the [[ConstrainedMimeType]] slot.
        // TODO

        // Step 9. If recorder’s [[ConstrainedBitsPerSecond]] slot is not null, set recorder’s videoBitsPerSecond and audioBitsPerSecond attributes to values the User Agent deems reasonable for the respective media types, for recording all tracks in tracks, such that the sum of videoBitsPerSecond and audioBitsPerSecond is close to the value of recorder’s [[ConstrainedBitsPerSecond]] slot.
        // TODO

        // Step 10. Let videoBitrate be the value of recorder’s videoBitsPerSecond attribute, and constrain the configuration of recorder to target an aggregate bitrate of videoBitrate bits per second for all video tracks recorder will be recording. videoBitrate is a hint for the encoder and the value might be surpassed, not achieved, or only be achieved over a long period of time.
        // TODO

        // Step 11. Let audioBitrate be the value of recorder’s audioBitsPerSecond attribute, and constrain the configuration of recorder to target an aggregate bitrate of audioBitrate bits per second for all audio tracks recorder will be recording. audioBitrate is a hint for the encoder and the value might be surpassed, not achieved, or only be achieved over a long period of time.
        // TODO

        // Step 12. Let videoKeyFrameIntervalDuration be recorder.[[VideoKeyFrameIntervalDuration]], and let videoKeyFrameIntervalCount be recorder.[[VideoKeyFrameIntervalCount]]. The UA SHOULD constrain the configuration of recorder so that the video encoder follows the below rules:
        // Step 12.1. If videoKeyFrameIntervalDuration is not null and videoKeyFrameIntervalCount is null, the video encoder produces a keyframe on the first frame arriving after videoKeyFrameIntervalDuration milliseconds elapsed since the last key frame.
        // Step 12.2. If videoKeyFrameIntervalCount is not null and videoKeyFrameIntervalDuration is null, the video encoder produces a keyframe on the first frame arriving after videoKeyFrameIntervalCount frames passed since the last key frame.
        // Step 12.3. If both videoKeyFrameIntervalDuration and videoKeyFrameIntervalCount are not null, then throw a NotSupportedError DOMException and abort these steps.
        // Step 12.4. If both videoKeyFrameIntervalDuration and videoKeyFrameIntervalCount are null, the User Agent may emit key frames as it deems fit.

        // Step 13. Constrain the configuration of recorder to encode using the BitrateMode specified by the value of recorder’s audioBitrateMode attribute for all audio tracks recorder will be recording.
        // TODO

        // Step 14. For each track in tracks, if the User Agent cannot record the track using the current configuration, then throw a NotSupportedError DOMException and abort these steps.
        // TODO

        // Step 15. Set recorder’s state to recording, and run the following steps in parallel:
        *self.state.borrow_mut() = RecordingState::Recording;
        // TODO
        // Step 15.1. If the container and codecs to use for the recording have not yet been fully specified, the User Agent specifies them in recorder’s current configuration. The User Agent MAY take the sources of the tracks in tracks into account when deciding which container and codecs to use.
        // Step 15.2. If the User Agent does not support the specified combination of media type/subtype, codecs and container, then it MUST abort the remaining steps and queue a task, using the DOM manipulation task source, that runs the following steps:
        // Step 15.2.1. Inactivate the recorder with recorder.
        // Step 15.2.2. Fire an error event named NotSupportedError at recorder.
        // Step 15.2.3. Fire an event named stop at recorder.
        // Step 15.3. Start recording all tracks in tracks using the recorder’s current configuration and gather the data into a Blob blob. Queue a task, using the DOM manipulation task source, to run the following steps:
        // Step 15.3.1. Let extendedMimeType be the value of recorder’s [[ConstrainedMimeType]] slot.
        // Step 15.3.2. Modify extendedMimeType by adding media type, subtype and codecs parameter reflecting the configuration used by the MediaRecorder to record all tracks in tracks, if not already present. This MAY include the profiles parameter [RFC6381] or further codec-specific parameters.
        // Step 15.3.3. Set recorder’s mimeType attribute to extendedMimeType.
        // Step 15.3.4. Fire an event named start at recorder.
        // Step 15.4. If at any point stream’s isolation properties change so that MediaRecorder is no longer allowed access to it, the UA MUST stop gathering data, discard any data that it has gathered, and queue a task, using the DOM manipulation task source, that runs the following steps:
        // Step 15.4.1. Inactivate the recorder with recorder.
        // Step 15.4.2. Fire an error event named SecurityError at recorder.
        // Step 15.4.3. Fire a blob event named dataavailable at recorder with blob.
        // Step 15.4.4. Fire an event named stop at recorder.
        // Step 15.5. If at any point, a track is added to or removed from stream’s track set, the UA MUST stop gathering data, and queue a task, using the DOM manipulation task source, that runs the following steps:
        // Step 15.5.1. Inactivate the recorder with recorder.
        // Step 15.5.2. Fire an error event named InvalidModificationError at recorder.
        // Step 15.5.3. Fire a blob event named dataavailable at recorder with blob.
        // Step 15.5.4. Fire an event named stop at recorder.
        // Step 15.6. If the UA at any point is unable to continue gathering data for reasons other than isolation properties or stream’s track set, it MUST stop gathering data, and queue a task, using the DOM manipulation task source, that runs the following steps:
        // Step 15.6.1. Inactivate the recorder with recorder.
        // Step 15.6.2. Fire an error event named UnknownError at recorder.
        // Step 15.6.3. Fire a blob event named dataavailable at recorder with blob.
        // Step 15.6.4. Fire an event named stop at recorder.
        // Step 15.7. If timeslice is not undefined, then once a minimum of timeslice milliseconds of data have been collected, or some minimum time slice imposed by the UA, whichever is greater, start gathering data into a new Blob blob, and queue a task, using the DOM manipulation task source, that fires a blob event named dataavailable at recorder with blob.
        // Step 15.8. If all recorded tracks become ended, then stop gathering data, and queue a task, using the DOM manipulation task source, that runs the following steps:
        // Step 15.8.1. Inactivate the recorder with recorder.
        // Step 15.8.2. Fire a blob event named dataavailable at recorder with blob.
        // Step 15.8.3. Fire an event named stop at recorder.
        Ok(())
    }

    fn Stop(&self) {
        // Step 1. Let recorder be the MediaRecorder object on which the method was invoked. SKIP

        // Step 2. If recorder’s state attribute is inactive, abort these steps.
        if self.state.borrow() == RecordingState::Inactive {
            return;
        }

        // Step 3. Inactivate the recorder with recorder.
        self.inactivate_recorder();

        // Step 4. Queue a task, using the DOM manipulation task source, that runs the following steps:
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(pause: move |cx| {
            // Step 4.1. Stop gathering data.
            // TODO
            // Step 4.2. Let blob be the Blob of collected data so far, then fire a blob event named dataavailable at recorder with blob.
            // TODO
            // Step 4.3. Fire an event named stop at recorder.
            // TODO
                }));

        // Step 5. return undefined. SKIP
    }

    /// <https://www.w3.org/TR/mediastream-recording/#dom-mediarecorder-pause>
    fn Pause(&self) -> Fallible<()> {
        let mut state = self.state.borrow_mut();

        // Step 1. If state is inactive, throw an InvalidStateError DOMException and abort these steps.
        if *state == RecordingState::Inactive {
            return Err(Error::InvalidState(Some(
                "pause when state is inactive".into(),
            )));
        }
        // Step 2. If state is paused, abort these steps.
        if *state == RecordingState::Paused {
            return Ok(());
        }

        // Step 3. Set state to paused, and queue a task, using the DOM manipulation task source,
        // that runs the following steps:
        *state = RecordingState::Paused;
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(pause: move |cx| {
            // Step 3.1. Stop gathering data into blob (but keep it available so that recording can be resumed in the future).
            // TODO
            // Step 3.2. Let target be the MediaRecorder context object. Fire an event named pause at target.
            // TODO: Context::new(self)
            }));

        // Step 4. return undefined.
        Ok(())
    }

    /// <https://www.w3.org/TR/mediastream-recording/#dom-mediarecorder-resume>
    fn Resume(&self) -> Fallible<()> {
        let mut state = self.state.borrow_mut();

        // Step 1. If state is inactive, throw an InvalidStateError DOMException and abort these steps.
        if *state == RecordingState::Inactive {
            return Err(Error::InvalidState(Some(
                "resume when state is inactive".into(),
            )));
        }

        // Step 2. If state is recording, abort these steps.
        if *state == RecordingState::Recording {
            return Ok(());
        }

        // Step 3. Set state to recording, and queue a task, using the DOM manipulation task source, that runs the following steps:
        *state = RecordingState::Paused;
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(resume: move |cx| {
            // Step 3.1. Resume (or continue) gathering data into the current blob.
            // TODO
            // Step 3.2. Let target be the MediaRecorder context object. Fire an event named resume at target.
            // TODO
                }));

        // Step 4. return undefined.
        Ok(())
    }

    /// <https://www.w3.org/TR/mediastream-recording/#dom-mediarecorder-requestdata>
    fn RequestData(&self) -> Fallible<()> {
        // Step 1. If state is inactive throw an InvalidStateError DOMException and terminate these steps.
        // Otherwise the UA MUST queue a task, using the DOM manipulation task source, that runs the following steps:
        if *self.state.borrow() == RecordingState::Inactive {
            return Err(Error::InvalidState(Some(
                "requestData when state is inactive".into(),
            )));
        }
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(request_data: move |cx| {
            // Step 1.1. Let blob be the Blob of collected data so far and let target be the MediaRecorder context object, then fire a blob event named dataavailable at target with blob. (Note that blob will be empty if no data has been gathered yet.)
            // TODO
            // Step 1.2. Create a new Blob and gather subsequent data into it.
            // TODO
                }));

        // Step 2. return undefined.
        Ok(())
    }

    /// <https://www.w3.org/TR/mediastream-recording/#dom-mediarecorder-istypesupported>
    fn IsTypeSupported(global: &Window, type_: DOMString) -> bool {
        Self::is_type_supported(&type_, false)
    }

    /// <https://www.w3.org/TR/mediastream-recording/#dom-mediarecorder-mediarecorder>
    fn Constructor(
        cx: &mut JSContext,
        global: &Window,
        proto: Option<js::gc::HandleObject>,
        stream: &MediaStream,
        options: &MediaRecorderOptions,
    ) -> Fallible<DomRoot<Self>> {
        // Step 1. Let stream be the constructor’s first argument. SKIP
        // Step 2. Let options be the constructor’s second argument. SKIP

        // Step 3. Let type be options’ mimeType.
        let type_ = &options.mimeType;
        // Step 4. If invoking is type supported with type and the value true returns false,
        // throw a NotSupportedError DOMException and abort these steps.
        if !Self::is_type_supported(type_, true) {
            return Err(Error::NotSupported(Some(
                "mimeType is not supported".into(),
            )));
        }

        // Step 5-16. let recorder and initialize.
        let recorder = Self::new(cx, global, stream, options.mimeType.clone());
        // TODO: detailed steps

        // Step 17. Return recorder
        Ok(recorder)
    }
}
