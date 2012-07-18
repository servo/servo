trait methods<T> {
    unsafe fn +(idx: uint) -> *T;
    unsafe fn [](idx: uint) -> T;
}

impl methods<T: copy> of methods<T> for *T {
    unsafe fn +(idx: uint) -> *T {
        ptr::offset(self, idx)
    }
    unsafe fn [](idx: uint) -> T {
        *(self + idx)
    }
}
