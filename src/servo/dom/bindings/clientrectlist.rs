use content::content_task::task_from_context;
use dom::bindings::clientrect::{ClientRect, ClientRectImpl};
use dom::bindings::codegen::ClientRectListBinding;
use dom::bindings::utils::{WrapperCache, CacheableWrapper, BindingObject, OpaqueBindingReference};
use dom::window::Window;
use dom::bindings::window::Window;
use js::jsapi::{JSObject, JSContext};

pub trait ClientRectList {
    fn Length(&self) -> u32;
    fn Item(&self, index: u32) -> Option<~ClientRectImpl>;
    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<~ClientRectImpl>;
}

pub struct ClientRectListImpl {
    wrapper: WrapperCache,
    rects: ~[(f32, f32, f32, f32)]
}

impl ClientRectList for ClientRectListImpl {
    fn Length(&self) -> u32 {
        self.rects.len() as u32
    }

    fn Item(&self, index: u32) -> Option<~ClientRectImpl> {
        if index < self.rects.len() as u32 {
            let (top, bottom, left, right) = self.rects[index];
            Some(~ClientRect(top, bottom, left, right))
        } else {
            None
        }
    }

    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<~ClientRectImpl> {
        *found = index < self.rects.len() as u32;
        self.Item(index)
    }
}

impl ClientRectListImpl {
    static fn new() -> ClientRectListImpl {
        ClientRectListImpl {
            wrapper: WrapperCache::new(),
            rects: ~[(5.6, 80.2, 3.7, 4.8), (800.1, 8001.1, -50.000001, -45.01)]
        }
    }
}

pub impl CacheableWrapper for ClientRectListImpl {
    fn get_wrappercache(&self) -> &WrapperCache {
        unsafe { cast::transmute(&self.wrapper) }
    }

    fn wrap_object_unique(~self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        ClientRectListBinding::Wrap(cx, scope, self, &mut unused)
    }

    fn wrap_object_shared(@self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        fail!(~"nyi")
    }
}

impl BindingObject for ClientRectListImpl {
    fn GetParentObject(&self, cx: *JSContext) -> OpaqueBindingReference {
        let content = task_from_context(cx);
        unsafe { OpaqueBindingReference(Right((*content).window.get() as @CacheableWrapper)) }
    }
}