use dom_struct::dom_struct;
use js::context::JSContext;
use js::gc::HandleObject;
use script_bindings::codegen::GenericBindings::EventBinding::EventMethods;
use script_bindings::codegen::GenericBindings::MediaStreamRecordingBinding::{
    BlobEventInit, BlobEventMethods,
};
use script_bindings::inheritance::Castable;
use script_bindings::num::Finite;
use script_bindings::reflector::reflect_dom_object_with_proto;
use script_bindings::root::{Dom, DomRoot};
use script_bindings::str::DOMString;
use style::Atom;

use crate::dom::blob::Blob;
use crate::dom::{Event, EventBubbles, EventCancelable, Window};

#[dom_struct]
pub(crate) struct BlobEvent {
    event: Event,
    data: Dom<Blob>,
    timecode: Finite<f64>,
}

impl BlobEvent {
    fn new_inherited(data: &DomRoot<Blob>, timecode: Option<Finite<f64>>) -> BlobEvent {
        BlobEvent {
            event: Event::new_inherited(),
            data: Dom::from_ref(&data),
            timecode: timecode.unwrap_or_default(),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        data: &DomRoot<Blob>,
        timecode: Option<Finite<f64>>,
    ) -> DomRoot<BlobEvent> {
        BlobEvent::new_with_proto(cx, window, None, type_, bubbles, cancelable, data, timecode)
    }

    fn new_with_proto(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        data: &DomRoot<Blob>,
        timecode: Option<Finite<f64>>,
    ) -> DomRoot<BlobEvent> {
        let blob_event = reflect_dom_object_with_proto(
            cx,
            Box::new(BlobEvent::new_inherited(data, timecode)),
            window,
            proto,
        );
        {
            let event = blob_event.upcast::<Event>();
            event.init_event(type_, bubbles.into(), cancelable.into());
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
        BlobEvent::new_with_proto(
            cx,
            window,
            proto,
            Atom::from(type_),
            eventInitDict.parent.bubbles.into(),
            eventInitDict.parent.cancelable.into(),
            &eventInitDict.data,
            eventInitDict.timecode,
        )
    }

    fn Data(&self) -> DomRoot<Blob> {
        DomRoot::from_ref(&self.data)
    }

    fn Timecode(&self) -> Finite<f64> {
        self.timecode
    }

    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
