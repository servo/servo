use dom_struct::dom_struct;
use js::{context::JSContext, gc::HandleObject};
use script_bindings::{
    codegen::GenericBindings::MediaStreamRecordingBinding::{BlobEventInit, BlobEventMethods},
    root::DomRoot,
    str::DOMString,
};

use crate::dom::{Event, Window, blob::Blob};

#[dom_struct]
pub(crate) struct BlobEvent {
    event: Event,
}

impl BlobEventMethods<crate::DomTypeHolder> for BlobEvent {
    fn Data(&self) -> DomRoot<Blob> {
        todo!()
    }

    fn Timecode(&self) -> script_bindings::num::Finite<f64> {
        todo!()
    }

    fn IsTrusted(&self) -> bool {
        todo!()
    }

    fn Constructor(
        cx: &mut JSContext,
        global: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        eventInitDict: &BlobEventInit<crate::DomTypeHolder>,
    ) -> DomRoot<BlobEvent> {
        todo!()
    }
}
