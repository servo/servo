mod ffi;
use ffi::*;

fn main() {
    unsafe {
        run()
    }
}

unsafe fn run() {
    let _display = GetDisplay(DEFAULT_DISPLAY as *mut _);
}
