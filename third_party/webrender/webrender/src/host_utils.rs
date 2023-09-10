#[cfg(feature = "gecko")]
mod utils {
    use std::ffi::CString;
    extern "C" {
        fn gecko_profiler_register_thread(name: *const ::std::os::raw::c_char);
        fn gecko_profiler_unregister_thread();
    }
    pub fn thread_started(thread_name: &str) {
        let name = CString::new(thread_name).unwrap();
        unsafe {
            // gecko_profiler_register_thread copies the passed name here.
            gecko_profiler_register_thread(name.as_ptr());
        }
    }
    pub fn thread_stopped() {
        unsafe { gecko_profiler_unregister_thread(); }
    }
}

#[cfg(not(feature = "gecko"))]
mod utils {
    pub fn thread_started(_: &str) { }
    pub fn thread_stopped() { }
}

pub use utils::*;
