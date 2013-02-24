use dom::bindings::utils::{CacheableWrapper, WrapperCache};
use dom::bindings::ClientRectBinding;
use js::jsapi::{JSObject, JSContext};

pub trait ClientRect {
    fn Top() -> f32;
    fn Bottom() -> f32;
    fn Left() -> f32;
    fn Right() -> f32;
    fn Width() -> f32;
    fn Height() -> f32;
}

pub struct ClientRectImpl {
    mut wrapper: ~WrapperCache,
    top: f32,
    bottom: f32,
    left: f32,
    right: f32,
}

pub impl ClientRect for ClientRectImpl {
    fn Top() -> f32 {
        self.top
    }

    fn Bottom() -> f32 {
        self.bottom
    }

    fn Left() -> f32 {
        self.left
    }

    fn Right() -> f32 {
        self.right
    }

    fn Width() -> f32 {
        f32::abs(self.right - self.left)
    }

    fn Height() -> f32 {
        f32::abs(self.bottom - self.top)
    }
}

pub impl CacheableWrapper for ClientRectImpl {
    fn get_wrapper(@self) -> *JSObject {
        unsafe { cast::transmute(self.wrapper.wrapper) }
    }

    fn set_wrapper(@self, wrapper: *JSObject) {
        unsafe { self.wrapper.wrapper = cast::transmute(wrapper); }
    }

    fn wrap_object(@self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        ClientRectBinding::Wrap(cx, scope, self, &mut unused)
    }
}