/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use interfaces::cef_command_line_t;

use libc::{calloc, c_int, c_char, size_t};
use std::ffi;
use std::mem;
use std::slice;
use std::str;
use string as cef_string;
use string::cef_string_utf16_set;
use types::{cef_string_t, cef_string_userfree_t, cef_string_utf16_t};

type command_line_t = command_line;
struct command_line {
    pub cl: cef_command_line_t,
    pub argc: c_int,
    pub argv: Vec<String>,
}

static mut GLOBAL_CMDLINE: Option<*mut command_line_t> = None;

fn command_line_new() -> *mut command_line_t {
    unsafe {
        let cl = calloc(1, mem::size_of::<command_line>() as size_t) as *mut command_line_t;
        (*cl).cl.base.size = mem::size_of::<cef_command_line_t>() as size_t;
        cl
    }
}

pub fn command_line_init(argc: c_int, argv: *const *const u8) {
    unsafe {
        let args = slice::from_raw_parts(argv, argc as usize);
        let a = args.iter().map(|&arg| {
            let slice = ffi::CStr::from_ptr(arg as *const c_char);
            str::from_utf8(slice.to_bytes()).unwrap().to_owned()
        }).collect();
        let cl = command_line_new();
        (*cl).argc = argc;
        (*cl).argv = a;
        (*cl).cl.get_switch_value = Some(command_line_get_switch_value as extern "C" fn(*mut cef_command_line_t, *const cef_string_t) -> cef_string_userfree_t);
        GLOBAL_CMDLINE = Some(cl);
    }
}

#[no_mangle]
pub extern "C" fn command_line_get_switch_value(cmd: *mut cef_command_line_t, name: *const cef_string_t) -> cef_string_userfree_t {
    if cmd.is_null() || name.is_null() {
        return cef_string::empty_utf16_userfree_string()
    }
    unsafe {
        //technically cef_string_t can be any type of character size
        //but the default cef callback uses utf16, so I'm jumping on board the SS Copy
        let cl: *mut command_line_t = mem::transmute(cmd);
        let cs: *const cef_string_utf16_t = mem::transmute(name);
        let buf = (*cs).str as *const _;
        let slice = slice::from_raw_parts(buf, (*cs).length as usize);
        let opt = String::from_utf16(slice).unwrap();
            //debug!("opt: {}", opt);
        for s in &(*cl).argv {
            let o = s.trim_left_matches('-');
            //debug!("arg: {}", o);
            if o.starts_with(&opt) {
                let mut string = mem::uninitialized();
                let arg = o[opt.len() + 1..].as_bytes();
                let c_str = ffi::CString::new(arg).unwrap();
                cef_string_utf16_set(c_str.as_bytes().as_ptr() as *const _,
                                     arg.len() as size_t,
                                     &mut string,
                                     1);
                return cef_string::string_to_userfree_string(string)
            }
        }
    }
    return cef_string::empty_utf16_userfree_string()
}

#[no_mangle]
pub extern "C" fn cef_command_line_create() -> *mut cef_command_line_t {
        unsafe {
            let cl = command_line_new();
            (*cl).cl.get_switch_value = Some(command_line_get_switch_value as extern "C" fn(*mut cef_command_line_t, *const cef_string_t) -> cef_string_userfree_t);
            mem::transmute(cl)
        }
}

#[no_mangle]
pub extern "C" fn cef_command_line_get_global() -> *mut cef_command_line_t {
    unsafe {
        match GLOBAL_CMDLINE {
            Some(scl) => {
                mem::transmute(scl)
            },
            None => {
                0 as *mut cef_command_line_t
            }
        }
    }
}

cef_stub_static_method_impls! {
    fn cef_command_line_create_command_line() -> *mut cef_command_line_t
    fn cef_command_line_get_global_command_line() -> *mut cef_command_line_t
}
