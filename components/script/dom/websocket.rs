use dom::bindings::global::GlobalRef;
use dom::bindings::js::Temporary;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::error::Fallible;
use dom::bindings::codegen::Bindings::WebSocketBinding;

#[jstraceable]
pub enum WebSocketType {
    WebSocketTypeId,
    FileTypeId
}

#[dom_struct]
pub struct WebSocket {
    reflector_: Reflector,
    type_: WebSocketType
}

impl WebSocket{
 pub fn new_inherited() -> WebSocket {
        WebSocket {
            reflector_: Reflector::new(),
            type_: WebSocketTypeId
        }
    }

    pub fn new(global: &GlobalRef) -> Temporary<WebSocket> {
        reflect_dom_object(box WebSocket::new_inherited(),
                           global,
                           WebSocketBinding::Wrap)
    }

    pub fn Constructor(global: &GlobalRef) -> Fallible<Temporary<WebSocket>> {
        Ok(WebSocket::new(global))
    }
}
impl Reflectable for WebSocket {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
