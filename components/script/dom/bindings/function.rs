/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Defines a macro `native_fn!` to create a JavaScript function from a Rust function pointer.
/// # Example
/// ```
/// let js_function: Rc<Function> = native_fn!(my_rust_function, c"myFunction", 2, 0);
/// ```
#[macro_export]
macro_rules! native_fn {
    ($cx:expr, $call:expr, $name:expr, $nargs:expr, $flags:expr) => {{
        let fun_obj = $crate::native_raw_obj_fn!($cx, $call, $name, $nargs, $flags);
        let cx = $cx.into();
        #[expect(unsafe_code)]
        unsafe {
            Function::new(cx, fun_obj)
        }
    }};
}

/// Defines a macro `native_raw_obj_fn!` to create a raw JavaScript function object.
/// # Example
/// ```
/// let raw_function_obj: *mut JSObject = native_raw_obj_fn!(cx, my_rust_function, c"myFunction", 2, 0);
/// ```
#[macro_export]
macro_rules! native_raw_obj_fn {
    ($cx:expr, $call:expr, $name:expr, $nargs:expr, $flags:expr) => {{
        #[expect(unsafe_code)]
        #[allow(clippy::macro_metavars_in_unsafe)]
        unsafe extern "C" fn wrapper(cx: *mut JSContext, argc: u32, vp: *mut JSVal) -> bool {
            let mut cx = unsafe {
                // SAFETY: We are in SM hook
                js::context::JSContext::from_ptr(
                    std::ptr::NonNull::new(cx).expect("JSContext is not null in SM hook"),
                )
            };
            let call_args = unsafe { CallArgs::from_vp(vp, argc) };
            $call(&mut cx, call_args)
        }
        #[expect(unsafe_code)]
        #[allow(clippy::macro_metavars_in_unsafe)]
        unsafe {
            let name: &std::ffi::CStr = $name;
            let raw_fun = js::jsapi::JS_NewFunction(
                $cx.raw_cx(),
                Some(wrapper),
                $nargs,
                $flags,
                name.as_ptr() as *const std::ffi::c_char,
            );
            assert!(!raw_fun.is_null());
            js::jsapi::JS_GetFunctionObject(raw_fun)
        }
    }};
}
