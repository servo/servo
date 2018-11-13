// Copyright 2014 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Interface to the measurements in the task_basic_info struct, gathered by
//! invoking `task_info()` with the `TASK_BASIC_INFO` flavor.

use std::os::raw::c_int;

#[allow(non_camel_case_types)]
type size_t = usize;

/// Obtains task_basic_info::virtual_size.
pub fn virtual_size() -> Option<usize> {
    let mut virtual_size: size_t = 0;
    let rv = unsafe {
        TaskBasicInfoVirtualSize(&mut virtual_size)
    };
    if rv == 0 { Some(virtual_size as usize) } else { None }
}

/// Obtains task_basic_info::resident_size.
pub fn resident_size() -> Option<usize> {
    let mut resident_size: size_t = 0;
    let rv = unsafe {
        TaskBasicInfoResidentSize(&mut resident_size)
    };
    if rv == 0 { Some(resident_size as usize) } else { None }
}

extern {
    fn TaskBasicInfoVirtualSize(virtual_size: *mut size_t) -> c_int;
    fn TaskBasicInfoResidentSize(resident_size: *mut size_t) -> c_int;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_stuff() {
        // In theory these can fail to return a value, but in practice they
        // don't unless something really bizarre has happened with the OS. So
        // assume they succeed. The returned values are non-deterministic, but
        // check they're non-zero as a basic sanity test.
        assert!(virtual_size().unwrap() > 0);
        assert!(resident_size().unwrap() > 0);
    }
}
