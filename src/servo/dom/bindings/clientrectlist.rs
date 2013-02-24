use content::content_task::task_from_context;
use dom::bindings::clientrect::ClientRectImpl;
use dom::bindings::ClientRectListBinding;
use dom::bindings::utils::{WrapperCache, CacheableWrapper, BindingObject};
use js::jsapi::{JSObject, JSContext};

pub trait ClientRectList {
    fn Length(&self) -> u32;
    fn Item(&self, index: u32) -> Option<@ClientRectImpl>;
}

pub struct ClientRectListImpl {
    mut wrapper: ~WrapperCache
}

impl ClientRectList for ClientRectListImpl {
    fn Length(&self) -> u32 {
        0
    }

    fn Item(&self, index: u32) -> Option<@ClientRectImpl> {
        None
    }
}

pub impl CacheableWrapper for ClientRectListImpl {
    fn get_wrapper(@self) -> *JSObject {
        unsafe { cast::transmute(self.wrapper.wrapper) }
    }

    fn set_wrapper(@self, wrapper: *JSObject) {
        unsafe { self.wrapper.wrapper = cast::transmute(wrapper); }
    }

    fn wrap_object(@self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        ClientRectListBinding::Wrap(cx, scope, self, &mut unused)
    }
}

pub impl BindingObject for ClientRectListImpl {
    fn GetParentObject(@self, cx: *JSContext) -> @CacheableWrapper {
        let content = task_from_context(cx);
        unsafe { (*content).window.get() as @CacheableWrapper }
    }
}