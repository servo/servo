use ffi::*;
use ffi::types::*;
use std::ptr::null_mut;

mod ffi;

fn main() {
    unsafe {
        run()
    }
}

macro_rules! check {
    ($name: ident ( $($arg: expr),* )) => {
        check($name( $($arg),* ), stringify!($name))
    }
}

unsafe fn run() {
    let display = GetDisplay(DEFAULT_DISPLAY as *mut _);
    assert!(!display.is_null());

    check!(Initialize(display, null_mut(), null_mut()));

    let mut num_config = -1;
    check!(GetConfigs(display, null_mut(), 0, &mut num_config));
    println!("Got {} configs", num_config);
}

unsafe fn check(result: EGLBoolean, function_name: &str) {
    check_error();
    assert_eq!(result, TRUE, "{} failed", function_name);
}

unsafe fn check_error() {
    let status = GetError();
    if status != SUCCESS as i32 {
        println!("Error: 0x{:x}", status);
    }
}
