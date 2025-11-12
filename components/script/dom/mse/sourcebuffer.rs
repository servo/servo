use dom_struct::dom_struct;
use script_bindings::codegen::GenericBindings::SourceBufferBinding::{
    AppendMode, SourceBufferMethods,
};
use script_bindings::codegen::GenericUnionTypes::ArrayBufferViewOrArrayBuffer;
use script_bindings::domstring::DOMString;
use script_bindings::num::Finite;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;

use crate::dom::audiotracklist::AudioTrackList;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::texttracklist::TextTrackList;
use crate::dom::timeranges::TimeRanges;
use crate::dom::videotracklist::VideoTrackList;

#[dom_struct]
pub(crate) struct SourceBuffer {
    event_target: EventTarget,
    #[ignore_malloc_size_of = "servo_media"]
    #[no_trace]
    inner: DomRefCell<Box<dyn servo_media::mse::SourceBuffer>>,
}

impl SourceBuffer {
    pub(crate) fn new_inherited(
        inner: DomRefCell<Box<dyn servo_media::mse::SourceBuffer>>,
    ) -> Self {
        let this = Self {
            event_target: EventTarget::new_inherited(),
            inner,
        };
        this
    }
    pub(crate) fn new(
        global: &GlobalScope,
        inner: DomRefCell<Box<dyn servo_media::mse::SourceBuffer>>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited(inner)), global, can_gc)
    }

    pub(crate) fn inner(&self) -> std::cell::RefMut<'_, Box<dyn servo_media::mse::SourceBuffer>> {
        self.inner.borrow_mut()
    }
}

impl SourceBufferMethods<crate::DomTypeHolder> for SourceBuffer {
    fn Mode(&self) -> AppendMode {
        match self.inner.borrow().append_mode() {
            servo_media::mse::AppendMode::Segments => AppendMode::Segments,
            servo_media::mse::AppendMode::Sequence => AppendMode::Sequence,
        }
    }

    fn SetMode(&self, value: AppendMode) {
        match value {
            AppendMode::Segments => self
                .inner
                .borrow_mut()
                .set_append_mode(servo_media::mse::AppendMode::Segments),
            AppendMode::Sequence => self
                .inner
                .borrow_mut()
                .set_append_mode(servo_media::mse::AppendMode::Sequence),
        }
    }

    fn Updating(&self) -> bool {
        self.inner.borrow().updating()
    }

    fn Buffered(&self) -> DomRoot<TimeRanges> {
        todo!()
    }

    fn TimestampOffset(&self) -> Finite<f64> {
        Finite::new(self.inner.borrow().timestamp_offset().unwrap()).unwrap()
    }

    fn SetTimestampOffset(&self, value: Finite<f64>) {
        self.inner.borrow().set_timestamp_offset(*value);
    }

    fn AudioTracks(&self) -> DomRoot<AudioTrackList> {
        todo!()
    }

    fn VideoTracks(&self) -> DomRoot<VideoTrackList> {
        todo!()
    }

    fn TextTracks(&self) -> DomRoot<TextTrackList> {
        todo!()
    }

    fn AppendWindowStart(&self) -> Finite<f64> {
        Finite::new(self.inner.borrow().append_window_start().unwrap()).unwrap()
    }

    fn SetAppendWindowStart(&self, value: Finite<f64>) {
        self.inner.borrow().set_append_window_start(*value);
    }

    fn AppendWindowEnd(&self) -> f64 {
        self.inner.borrow().append_window_end().unwrap()
    }

    fn SetAppendWindowEnd(&self, value: f64) {
        self.inner.borrow().set_append_window_end(value);
    }

    fn AppendBuffer(&self, data: ArrayBufferViewOrArrayBuffer) {
        let vec = match data {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };
        self.inner.borrow_mut().append_buffer(vec);
    }

    fn Abort(&self) {
        self.inner.borrow().abort();
    }

    fn ChangeType(&self, _type_: DOMString) {
        todo!()
    }

    fn Remove(&self, start: Finite<f64>, end: f64) {
        self.inner.borrow_mut().remove(*start, end);
    }

    event_handler!(on_update_start, GetOnupdatestart, SetOnupdatestart);
    event_handler!(on_update, GetOnupdate, SetOnupdate);
    event_handler!(on_update_end, GetOnupdateend, SetOnupdateend);
    event_handler!(on_error, GetOnerror, SetOnerror);
    event_handler!(on_abort, GetOnabort, SetOnabort);
}
