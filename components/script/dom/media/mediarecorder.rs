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

    fn Start(&self, timeslice: Option<u32>) {
        todo!()
    }

    fn Stop(&self) {
        todo!()
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

    fn RequestData(&self) {
        todo!()
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
