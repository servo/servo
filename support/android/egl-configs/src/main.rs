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
    assert!(num_config >= 0);

    let mut configs = Vec::with_capacity(num_config as usize);
    let mut num_config2 = -1;
    check!(GetConfigs(display, configs.as_mut_ptr(), num_config, &mut num_config2));
    assert_eq!(num_config, num_config2);
    configs.set_len(num_config as usize);

    for (i, &config) in configs.iter().enumerate() {
        println!("Config #{}", i + 1);

        macro_rules! to_pairs {
            ($($name: ident)*) => {
                 &[ $( (stringify!($name), $name) ),* ]
            }
        }
        // https://www.khronos.org/registry/EGL/sdk/docs/man/html/eglGetConfigAttrib.xhtml#description
        for &(attr_name, attr) in to_pairs! [
            ALPHA_SIZE
            ALPHA_MASK_SIZE
            BIND_TO_TEXTURE_RGB
            BIND_TO_TEXTURE_RGBA
            BLUE_SIZE
            BUFFER_SIZE
            COLOR_BUFFER_TYPE
            CONFIG_CAVEAT
            CONFIG_ID
            CONFORMANT
            DEPTH_SIZE
            GREEN_SIZE
            LEVEL
            LUMINANCE_SIZE
            MAX_PBUFFER_WIDTH
            MAX_PBUFFER_HEIGHT
            MAX_PBUFFER_PIXELS
            MAX_SWAP_INTERVAL
            MIN_SWAP_INTERVAL
            NATIVE_RENDERABLE
            NATIVE_VISUAL_ID
            NATIVE_VISUAL_TYPE
            RED_SIZE
            RENDERABLE_TYPE
            SAMPLE_BUFFERS
            SAMPLES
            STENCIL_SIZE
            SURFACE_TYPE
            TRANSPARENT_TYPE
            TRANSPARENT_RED_VALUE
            TRANSPARENT_GREEN_VALUE
            TRANSPARENT_BLUE_VALUE
        ] {
            let mut value = -1;
            check!(GetConfigAttrib(display, config, attr as i32, &mut value));
            println!("    {} = {}", attr_name, value)
        }
    }
}

unsafe fn check(result: EGLBoolean, function_name: &str) {
    check_error();
    assert_eq!(result, TRUE, "{} failed", function_name);
}

unsafe fn check_error() {
    let status = GetError();
    if status != SUCCESS as i32 {
        panic!("Error: 0x{:x}", status);
    }
}
