use dom_struct::dom_struct;
use js::gc::HandleObject;
use script_bindings::codegen::GenericBindings::ManagedMediaSourceBinding::ManagedMediaSourceMethods;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;

use crate::dom::mediasource::MediaSource;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct ManagedMediaSource {
    inner: MediaSource,
}

impl ManagedMediaSourceMethods<crate::DomTypeHolder> for ManagedMediaSource {
    fn Constructor(
        _global: &Window,
        _proto: Option<HandleObject>,
        _can_gc: CanGc,
    ) -> DomRoot<ManagedMediaSource> {
        todo!()
    }

    fn Streaming(&self) -> bool {
        todo!()
    }

    event_handler!(on_start_streaming, GetOnstartstreaming, SetOnstartstreaming);
    event_handler!(on_end_streaming, GetOnendstreaming, SetOnendstreaming);
}
