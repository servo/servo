//use dom::bindings::clientrect::ClientRect;
use dom::bindings::utils::WrapperCache;

pub struct ClientRect {
    wrapper: WrapperCache,
    top: f32,
    bottom: f32,
    left: f32,
    right: f32,
}

pub impl ClientRect {
    fn new(top: f32, bottom: f32, left: f32, right: f32) -> @mut ClientRect {
        let rect = @mut ClientRect {
            top: top, bottom: bottom, left: left, right: right,
            wrapper: WrapperCache::new()
        };
        rect.init_wrapper();
        rect
    }

    fn Top(&self) -> f32 {
        self.top
    }

    fn Bottom(&self) -> f32 {
        self.bottom
    }

    fn Left(&self) -> f32 {
        self.left
    }

    fn Right(&self) -> f32 {
        self.right
    }

    fn Width(&self) -> f32 {
        f32::abs(self.right - self.left)
    }

    fn Height(&self) -> f32 {
        f32::abs(self.bottom - self.top)
    }
}
