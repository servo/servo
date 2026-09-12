use dom_struct::dom_struct;
use js::{context::JSContext, gc::HandleObject};
use script_bindings::{
    codegen::GenericBindings::{
        EventBinding::EventMethods,
        MediaStreamRecordingBinding::{BlobEventInit, BlobEventMethods},
    },
    inheritance::Castable,
    num::Finite,
    reflector::reflect_dom_object_with_proto,
    root::{Dom, DomRoot},
    str::DOMString,
};
use style::Atom;

use crate::dom::{Event, Window, blob::Blob};

#[dom_struct]
pub(crate) struct BlobEvent {
    event: Event,
    data: Dom<Blob>,
    timecode: Option<Finite<f64>>,
}

impl BlobEvent {
    fn new_inherited(init: &BlobEventInit<crate::DomTypeHolder>) -> BlobEvent {
        BlobEvent {
            event: Event::new_inherited(),
            data: Dom::from_ref(&init.data),
            timecode: init.timecode,
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        type_: Atom,
        init: &BlobEventInit<crate::DomTypeHolder>,
    ) -> DomRoot<BlobEvent> {
        BlobEvent::new_with_proto(cx, window, None, type_, init)
    }

    fn new_with_proto(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        type_: Atom,
        init: &BlobEventInit<crate::DomTypeHolder>,
    ) -> DomRoot<BlobEvent> {
        let blob_event = reflect_dom_object_with_proto(
            cx,
            Box::new(BlobEvent::new_inherited(init)),
            window,
            proto,
        );
        {
            let event = blob_event.upcast::<Event>();
            event.init_event(type_, init.parent.bubbles, init.parent.cancelable);
        }
        blob_event
    }
}

impl BlobEventMethods<crate::DomTypeHolder> for BlobEvent {
    /// <https://www.w3.org/TR/mediastream-recording/#dom-blobevent-blobevent>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        eventInitDict: &BlobEventInit<crate::DomTypeHolder>,
    ) -> DomRoot<BlobEvent> {
        BlobEvent::new_with_proto(cx, window, proto, Atom::from(type_), eventInitDict)
    }

    fn Data(&self) -> DomRoot<Blob> {
        DomRoot::from_ref(&self.data)
    }

    fn Timecode(&self) -> Finite<f64> {
        self.timecode.unwrap_or_default()
    }

    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
