use content::content_task::task_from_context;
use dom::bindings::utils::{CacheableWrapper, WrapperCache, BindingObject, OpaqueBindingReference};
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
    wrapper: WrapperCache,
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

pub fn ClientRect(top: f32, bottom: f32, left: f32, right: f32) -> ClientRectImpl {
    ClientRectImpl {
        top: top, bottom: bottom, left: left, right: right,
        wrapper: WrapperCache::new()
    }
}

pub impl CacheableWrapper for ClientRectImpl {
    fn get_wrappercache(&self) -> &WrapperCache {
        unsafe { cast::transmute(&self.wrapper) }
    }

    fn wrap_object_unique(~self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        ClientRectBinding::Wrap(cx, scope, self, &mut unused)
    }

    fn wrap_object_shared(@self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        fail!(~"nyi")
    }
}

impl BindingObject for ClientRectImpl {
    fn GetParentObject(&self, cx: *JSContext) -> OpaqueBindingReference {
        let content = task_from_context(cx);
        unsafe { OpaqueBindingReference(Right((*content).window.get() as @CacheableWrapper)) }
    }
}