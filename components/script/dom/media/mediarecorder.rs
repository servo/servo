use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::{
    codegen::GenericBindings::{
        EventHandlerBinding::EventHandlerNonNull,
        MediaStreamRecordingBinding::{
            BitrateMode, MediaRecorderMethods, MediaRecorderOptions, RecordingState,
        },
    },
    root::DomRoot,
    str::DOMString,
};

use crate::dom::{
    Window,
    types::{EventTarget, MediaStream},
};

#[dom_struct]
pub(crate) struct MediaRecorder {
    eventtarget: EventTarget,
}

impl MediaRecorderMethods<crate::DomTypeHolder> for MediaRecorder {
    fn Stream(&self) -> DomRoot<MediaStream> {
        todo!()
    }

    fn MimeType(&self) -> DOMString {
        todo!()
    }

    fn State(&self) -> RecordingState {
        todo!()
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
        todo!()
    }

    fn AudioBitsPerSecond(&self) -> u32 {
        todo!()
    }

    fn AudioBitrateMode(&self) -> BitrateMode {
        todo!()
    }

    fn Start(&self, timeslice: Option<u32>) {
        todo!()
    }

    fn Stop(&self) {
        todo!()
    }

    fn Pause(&self) {
        todo!()
    }

    fn Resume(&self) {
        todo!()
    }

    fn RequestData(&self) {
        todo!()
    }

    fn IsTypeSupported(global: &Window, type_: DOMString) -> bool {
        todo!()
    }

    fn Constructor(
        cx: &mut JSContext,
        global: &Window,
        proto: Option<js::gc::HandleObject>,
        stream: &MediaStream,
        options: &MediaRecorderOptions,
    ) -> DomRoot<MediaRecorder> {
        todo!()
    }
}
