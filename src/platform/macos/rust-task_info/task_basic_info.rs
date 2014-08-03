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

use libc::{c_int,uint64_t};

/// Obtains task_basic_info::virtual_size.
pub fn virtual_size() -> Option<u64> {
    let mut virtual_size: u64 = 0;
    let mut rv;
    unsafe {
        rv = TaskBasicInfoVirtualSize(&mut virtual_size);
    }
    if rv == 0 { Some(virtual_size) } else { None }
}

/// Obtains task_basic_info::resident_size.
pub fn resident_size() -> Option<u64> {
    let mut resident_size: u64 = 0;
    let mut rv;
    unsafe {
        rv = TaskBasicInfoResidentSize(&mut resident_size);
    }
    if rv == 0 { Some(resident_size) } else { None }
}

#[link(name = "task_info", kind = "static")]
extern {
    fn TaskBasicInfoVirtualSize(virtual_size: *mut uint64_t) -> c_int;
    fn TaskBasicInfoResidentSize(resident_size: *mut uint64_t) -> c_int;
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

