impl methods<T: copy> for *T {
    unsafe fn +(idx: uint) -> *T {
        ptr::offset(self, idx)
    }
    unsafe fn [](idx: uint) -> T {
        *(self + idx)
    }
}
