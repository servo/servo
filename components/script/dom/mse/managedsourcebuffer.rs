use dom_struct::dom_struct;
use script_bindings::codegen::GenericBindings::ManagedSourceBufferBinding::ManagedSourceBufferMethods;

use crate::dom::sourcebuffer::SourceBuffer;

#[dom_struct]
pub(crate) struct ManagedSourceBuffer {
    inner: SourceBuffer,
}

impl ManagedSourceBufferMethods<crate::DomTypeHolder> for ManagedSourceBuffer {
    event_handler!(on_buffered_change, GetOnbufferedchange, SetOnbufferedchange);
}
