pub trait ClientRect {
    fn Top() -> f32;
    fn Bottom() -> f32;
    fn Left() -> f32;
    fn Right() -> f32;
    fn Width() -> f32;
    fn Height() -> f32;
}

struct ClientRectImpl {
    top: f32,
    bottom: f32,
    left: f32,
    right: f32,
}

impl ClientRectImpl: ClientRect {
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