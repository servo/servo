/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use libc::{calloc, c_int, size_t};
use std::mem;
use std::str;
use std::c_vec::CVec;
use string::{cef_string_userfree_utf16_alloc, cef_string_utf16_set};
use types::{cef_command_line_t, cef_string_t, cef_string_userfree_t, cef_string_utf16_t};

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
        (*cl).cl.base.size = mem::size_of::<cef_command_line_t>() as u64;
        cl
    }
}

pub fn command_line_init(argc: c_int, argv: *const *const u8) {
    unsafe {
        let mut a: Vec<String> = vec!();
        for i in range(0u, argc as uint) {
            a.push(str::raw::from_c_str(*argv.offset(i as int) as *const i8));
        }
        let cl = command_line_new();
        (*cl).argc = argc;
        (*cl).argv = a;
        (*cl).cl.get_switch_value = command_line_get_switch_value;
        GLOBAL_CMDLINE = Some(cl);
    }
}

#[no_mangle]
pub extern "C" fn command_line_get_switch_value(cmd: *mut cef_command_line_t, name: *const cef_string_t) -> *mut cef_string_userfree_t {
    if cmd.is_null() || name.is_null() {
        return 0 as *mut cef_string_userfree_t;
    }
    unsafe {
        //technically cef_string_t can be any type of character size
        //but the default cef callback uses utf16, so I'm jumping on board the SS Copy
        let cl: *mut command_line_t = mem::transmute(cmd);
        let cs: *const cef_string_utf16_t = mem::transmute(name);
        let opt = String::from_utf16(CVec::new((*cs).str, (*cs).length as uint).as_slice()).unwrap();
            //debug!("opt: {}", opt);
        for s in (*cl).argv.iter() {
            let o = s.as_slice().trim_left_chars('-');
            //debug!("arg: {}", o);
            if o.as_slice().starts_with(opt.as_slice()) {
                let string = cef_string_userfree_utf16_alloc() as *mut cef_string_utf16_t;
                let arg = o.slice_from(opt.len() + 1).as_bytes();
                arg.with_c_str(|c_str| {
                    cef_string_utf16_set(mem::transmute(c_str), arg.len() as u64, string, 1);
                });
                return string as *mut cef_string_userfree_t
            }
        }
    }
    return 0 as *mut cef_string_userfree_t;
}

#[no_mangle]
pub extern "C" fn cef_command_line_create() -> *mut cef_command_line_t {
        unsafe {
            let cl = command_line_new();
            (*cl).cl.get_switch_value = command_line_get_switch_value;
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
