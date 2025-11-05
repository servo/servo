use dom_struct::dom_struct;
use js::gc::HandleObject;
use script_bindings::codegen::GenericBindings::BufferedChangeEventBinding::{
    BufferedChangeEventInit, BufferedChangeEventMethods,
};
use script_bindings::domstring::DOMString;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;

use crate::dom::bindings::codegen::DomTypeHolder::DomTypeHolder;
use crate::dom::event::Event;
use crate::dom::timeranges::TimeRanges;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct BufferedChangeEvent {
    event: Event,
}

impl BufferedChangeEvent {
    #[expect(dead_code)]
    pub(crate) fn new_inherited() -> Self {
        Self {
            event: Event::new_inherited(),
        }
    }
}

impl BufferedChangeEventMethods<crate::DomTypeHolder> for BufferedChangeEvent {
    fn Constructor(
        _global: &Window,
        _proto: Option<HandleObject>,
        _can_gc: CanGc,
        _type_: DOMString,
        _event_init_dict: &BufferedChangeEventInit<DomTypeHolder>,
    ) -> DomRoot<BufferedChangeEvent> {
        todo!()
    }

    fn AddedRanges(&self) -> DomRoot<TimeRanges> {
        todo!()
    }

    fn RemovedRanges(&self) -> DomRoot<TimeRanges> {
        todo!()
    }

    fn IsTrusted(&self) -> bool {
        todo!()
    }
}
