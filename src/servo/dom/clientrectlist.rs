use dom::clientrect::ClientRect;
use dom::bindings::utils::WrapperCache;

pub struct ClientRectList {
    wrapper: WrapperCache,
    rects: ~[(f32, f32, f32, f32)]
}

pub impl ClientRectList {
    fn new() -> @mut ClientRectList {
        let list = @mut ClientRectList {
            wrapper: WrapperCache::new(),
            rects: ~[(5.6, 80.2, 3.7, 4.8), (800.1, 8001.1, -50.000001, -45.01)]
        };
        list.init_wrapper();
        list
    }

    fn Length(&self) -> u32 {
        self.rects.len() as u32
    }

    fn Item(&self, index: u32) -> Option<@mut ClientRect> {
        if index < self.rects.len() as u32 {
            let (top, bottom, left, right) = self.rects[index];
            Some(ClientRect::new(top, bottom, left, right))
        } else {
            None
        }
    }

    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<@mut ClientRect> {
        *found = index < self.rects.len() as u32;
        self.Item(index)
    }
}