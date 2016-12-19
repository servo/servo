/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use interfaces::{CefV8Value};
use interfaces::{cef_app_t, CefApp, cef_drag_data_t, cef_post_data_element_t, cef_v8value_t, CefPostDataElement};
use interfaces::{cef_dialog_handler_t, cef_focus_handler_t};
use interfaces::{cef_download_handler_t, cef_drag_handler_t, cef_context_menu_handler_t};
use interfaces::{cef_geolocation_handler_t, cef_jsdialog_handler_t, cef_keyboard_handler_t};
use interfaces::{cef_load_handler_t, cef_request_handler_t};
use types::{cef_base_t, cef_browser_settings_t, CefBrowserSettings, cef_color_model_t};
use types::{cef_context_menu_edit_state_flags_t};
use types::{cef_context_menu_media_state_flags_t};
use types::{cef_context_menu_media_type_t, cef_context_menu_type_flags_t, cef_cookie_t, cef_cursor_info_t, CefCursorInfo, cef_cursor_type_t};
use types::{cef_dom_document_type_t, cef_dom_node_type_t};
use types::{cef_drag_operations_mask_t, cef_draggable_region_t, cef_duplex_mode_t};
use types::{cef_errorcode_t, cef_event_flags_t, cef_event_handle_t};
use types::{cef_file_dialog_mode_t, cef_focus_source_t};
use types::{cef_geoposition_t};
use types::{cef_jsdialog_type_t};
use types::{cef_key_event};
use types::{cef_menu_item_type_t, cef_mouse_button_type_t};
use types::{cef_mouse_event, cef_navigation_type_t};
use types::{cef_page_range_t, cef_paint_element_type_t, cef_point_t, cef_postdataelement_type_t};
use types::{cef_pdf_print_settings_t};
use types::{cef_popup_features_t, cef_process_id_t};
use types::{cef_rect_t, cef_request_context_settings_t, CefRequestContextSettings};
use types::{cef_resource_type_t, cef_return_value_t};
use types::{cef_screen_info_t, CefScreenInfo, cef_size_t, cef_string_t, cef_string_userfree_t};
use types::{cef_string_list_t, cef_string_map_t, cef_string_multimap_t, cef_string_utf16};
use types::{cef_termination_status_t, cef_text_input_context_t, cef_thread_id_t};
use types::{cef_time_t, cef_transition_type_t, cef_urlrequest_status_t};
use types::{cef_v8_accesscontrol_t, cef_v8_propertyattribute_t, cef_value_type_t};
use types::{cef_window_info_t, cef_window_open_disposition_t, cef_xml_encoding_type_t, cef_xml_node_type_t};

use libc::{self, c_char, c_int, c_ushort, c_void};
use std::collections::HashMap;
use std::mem;
use std::ptr;
use std::slice;

pub trait CefWrap<CObject> {
    fn to_c(rust_object: Self) -> CObject;
    unsafe fn to_rust(c_object: CObject) -> Self;
}

macro_rules! cef_noop_wrapper(
    ($ty:ty) => (
        impl CefWrap<$ty> for $ty {
            fn to_c(rust_object: $ty) -> $ty {
                rust_object
            }
            unsafe fn to_rust(c_object: $ty) -> $ty {
                c_object
            }
        }
    )
);

macro_rules! cef_pointer_wrapper(
    ($ty:ty) => (
        impl<'a> CefWrap<*const $ty> for &'a $ty {
            fn to_c(rust_object: &'a $ty) -> *const $ty {
                rust_object
            }
            unsafe fn to_rust(c_object: *const $ty) -> &'a $ty {
                &*c_object
            }
        }
        impl<'a> CefWrap<*mut $ty> for &'a mut $ty {
            fn to_c(rust_object: &'a mut $ty) -> *mut $ty {
                rust_object
            }
            unsafe fn to_rust(c_object: *mut $ty) -> &'a mut $ty {
                &mut *c_object
            }
        }
        cef_noop_wrapper!(*const $ty);
        cef_noop_wrapper!(*mut $ty);
    )
);

macro_rules! cef_unimplemented_wrapper(
    ($c_type:ty, $rust_type:ty) => (
        impl CefWrap<$c_type> for $rust_type {
            fn to_c(_: $rust_type) -> $c_type {
                panic!("unimplemented CEF type conversion: {}", stringify!($c_type))
            }
            unsafe fn to_rust(_: $c_type) -> $rust_type {
                panic!("unimplemented CEF type conversion: {}", stringify!($c_type))
            }
        }
    )
);

cef_pointer_wrapper!(());
cef_pointer_wrapper!(*mut ());
cef_pointer_wrapper!(*mut c_void);
cef_pointer_wrapper!(c_void);
cef_pointer_wrapper!(cef_app_t);
cef_pointer_wrapper!(cef_base_t);
cef_pointer_wrapper!(cef_browser_settings_t);
cef_pointer_wrapper!(cef_cookie_t);
cef_pointer_wrapper!(cef_cursor_info_t);
cef_pointer_wrapper!(cef_draggable_region_t);
cef_pointer_wrapper!(cef_geoposition_t);
cef_pointer_wrapper!(cef_key_event);
cef_pointer_wrapper!(cef_mouse_event);
cef_pointer_wrapper!(cef_page_range_t);
cef_pointer_wrapper!(cef_pdf_print_settings_t);
cef_pointer_wrapper!(cef_point_t);
cef_pointer_wrapper!(cef_popup_features_t);
cef_pointer_wrapper!(cef_rect_t);
cef_pointer_wrapper!(cef_request_context_settings_t);
cef_pointer_wrapper!(cef_screen_info_t);
cef_pointer_wrapper!(cef_size_t);
cef_pointer_wrapper!(cef_time_t);
cef_pointer_wrapper!(cef_window_info_t);
cef_pointer_wrapper!(i32);
cef_pointer_wrapper!(i64);
cef_pointer_wrapper!(u32);
cef_pointer_wrapper!(u64);
cef_pointer_wrapper!(usize);

cef_noop_wrapper!(());
cef_noop_wrapper!(*const cef_geolocation_handler_t);
cef_noop_wrapper!(*const cef_string_utf16);
cef_noop_wrapper!(*mut cef_context_menu_handler_t);
cef_noop_wrapper!(*mut cef_dialog_handler_t);
cef_noop_wrapper!(*mut cef_download_handler_t);
cef_noop_wrapper!(*mut cef_drag_data_t);
cef_noop_wrapper!(*mut cef_drag_handler_t);
cef_noop_wrapper!(*mut cef_event_handle_t);
cef_noop_wrapper!(*mut cef_focus_handler_t);
cef_noop_wrapper!(*mut cef_geolocation_handler_t);
cef_noop_wrapper!(*mut cef_jsdialog_handler_t);
cef_noop_wrapper!(*mut cef_keyboard_handler_t);
cef_noop_wrapper!(*mut cef_load_handler_t);
cef_noop_wrapper!(*mut cef_request_handler_t);
cef_noop_wrapper!(*mut cef_string_utf16);
cef_noop_wrapper!(c_int);
cef_noop_wrapper!(CefApp);
cef_noop_wrapper!(CefBrowserSettings);
cef_noop_wrapper!(CefScreenInfo);
cef_noop_wrapper!(CefRequestContextSettings);
cef_noop_wrapper!(CefCursorInfo);
cef_noop_wrapper!(cef_color_model_t);
cef_noop_wrapper!(cef_context_menu_edit_state_flags_t);
cef_noop_wrapper!(cef_context_menu_media_state_flags_t);
cef_noop_wrapper!(cef_context_menu_media_type_t);
cef_noop_wrapper!(cef_context_menu_type_flags_t);
cef_noop_wrapper!(cef_cursor_type_t);
cef_noop_wrapper!(cef_dom_document_type_t);
cef_noop_wrapper!(cef_dom_node_type_t);
cef_noop_wrapper!(cef_drag_operations_mask_t);
cef_noop_wrapper!(cef_duplex_mode_t);
cef_noop_wrapper!(cef_errorcode_t);
cef_noop_wrapper!(cef_event_flags_t);
cef_noop_wrapper!(cef_event_handle_t);
cef_noop_wrapper!(cef_file_dialog_mode_t);
cef_noop_wrapper!(cef_focus_source_t);
cef_noop_wrapper!(cef_jsdialog_handler_t);
cef_noop_wrapper!(cef_jsdialog_type_t);
cef_noop_wrapper!(cef_key_event);
cef_noop_wrapper!(cef_menu_item_type_t);
cef_noop_wrapper!(cef_mouse_button_type_t);
cef_noop_wrapper!(cef_navigation_type_t);
cef_noop_wrapper!(cef_paint_element_type_t);
cef_noop_wrapper!(cef_postdataelement_type_t);
cef_noop_wrapper!(cef_process_id_t);
cef_noop_wrapper!(cef_resource_type_t);
cef_noop_wrapper!(cef_return_value_t);
cef_noop_wrapper!(cef_size_t);
cef_noop_wrapper!(cef_termination_status_t);
cef_noop_wrapper!(cef_text_input_context_t);
cef_noop_wrapper!(cef_thread_id_t);
cef_noop_wrapper!(cef_time_t);
cef_noop_wrapper!(cef_transition_type_t);
cef_noop_wrapper!(cef_urlrequest_status_t);
cef_noop_wrapper!(cef_v8_accesscontrol_t);
cef_noop_wrapper!(cef_v8_propertyattribute_t);
cef_noop_wrapper!(cef_value_type_t);
cef_noop_wrapper!(cef_window_open_disposition_t);
cef_noop_wrapper!(cef_xml_encoding_type_t);
cef_noop_wrapper!(cef_xml_node_type_t);
cef_noop_wrapper!(f64);
cef_noop_wrapper!(i64);
cef_noop_wrapper!(u32);
cef_noop_wrapper!(u64);
cef_noop_wrapper!(usize);
cef_noop_wrapper!(cef_string_list_t);

cef_unimplemented_wrapper!(*const *mut cef_v8value_t, *const CefV8Value);
cef_unimplemented_wrapper!(*mut *mut cef_post_data_element_t, *mut CefPostDataElement);
cef_unimplemented_wrapper!(cef_string_map_t, HashMap<String,String>);
cef_unimplemented_wrapper!(cef_string_multimap_t, HashMap<String,Vec<String>>);
cef_unimplemented_wrapper!(cef_string_t, String);

impl<'a> CefWrap<*const cef_string_t> for &'a [u16] {
    fn to_c(buffer: &'a [u16]) -> *const cef_string_t {
        unsafe {
            let ptr = libc::malloc(((buffer.len() + 1) * 2)) as *mut c_ushort;
            ptr::copy(buffer.as_ptr(), ptr, buffer.len());
            *ptr.offset(buffer.len() as isize) = 0;

            // FIXME(pcwalton): This leaks!! We should instead have the caller pass some scratch
            // stack space to create the object in. What a botch.
            Box::into_raw(box cef_string_utf16 {
                str: ptr,
                length: buffer.len(),
                dtor: Some(free_boxed_utf16_string as extern "C" fn(*mut c_ushort)),
            }) as *const _
        }
    }
    unsafe fn to_rust(cef_string: *const cef_string_t) -> &'a [u16] {
        slice::from_raw_parts((*cef_string).str, (*cef_string).length as usize)
    }
}

extern "C" fn free_boxed_utf16_string(string: *mut c_ushort) {
    unsafe {
        libc::free(string as *mut c_void)
    }
}

impl<'a> CefWrap<*mut cef_string_t> for &'a mut [u16] {
    fn to_c(_: &'a mut [u16]) -> *mut cef_string_t {
        panic!("unimplemented CEF type conversion: &'a str")
    }
    unsafe fn to_rust(_: *mut cef_string_t) -> &'a mut [u16] {
        panic!("unimplemented CEF type conversion: *mut cef_string_t")
    }
}

// FIXME(pcwalton): This is pretty bogus, but it's only used for `init_from_argv`, which should
// never be called by Rust programs anyway. We should fix the wrapper generation though.
impl<'a,'b> CefWrap<*const *const c_char> for &'a &'b str {
    fn to_c(_: &'a &'b str) -> *const *const c_char {
        panic!("unimplemented CEF type conversion: &'a &'b str")
    }
    unsafe fn to_rust(_: *const *const c_char) -> &'a &'b str {
        panic!("unimplemented CEF type conversion: *const *const cef_string_t")
    }
}

impl<'a,'b> CefWrap<*mut *const c_char> for &'a mut &'b str {
    fn to_c(_: &'a mut &'b str) -> *mut *const c_char {
        panic!("unimplemented CEF type conversion: &'a mut &'b str")
    }
    unsafe fn to_rust(_: *mut *const c_char) -> &'a mut &'b str {
        panic!("unimplemented CEF type conversion: *mut *const c_char")
    }
}

impl<'a> CefWrap<cef_string_userfree_t> for String {
    fn to_c(string: String) -> cef_string_userfree_t {
        let utf16_chars: Vec<u16> = string.encode_utf16().collect();

        let boxed_string;
        unsafe {
            let buffer = libc::malloc((mem::size_of::<c_ushort>() as libc::size_t) *
                                      ((utf16_chars.len() + 1) as libc::size_t)) as *mut u16;
            for (i, ch) in utf16_chars.iter().enumerate() {
                *buffer.offset(i as isize) = *ch
            }
            *buffer.offset(utf16_chars.len() as isize) = 0;

            boxed_string = libc::malloc(mem::size_of::<cef_string_utf16>() as libc::size_t) as
                *mut cef_string_utf16;
            ptr::write(&mut (*boxed_string).str, buffer);
            ptr::write(&mut (*boxed_string).length, utf16_chars.len());
            ptr::write(&mut (*boxed_string).dtor, Some(free_utf16_buffer as extern "C" fn(*mut c_ushort)));
        }
        boxed_string
    }
    unsafe fn to_rust(_: cef_string_userfree_t) -> String {
        panic!("unimplemented CEF type conversion: cef_string_userfree_t")
    }
}

extern "C" fn free_utf16_buffer(buffer: *mut c_ushort) {
    unsafe {
        libc::free(buffer as *mut c_void)
    }
}

impl<'a> CefWrap<cef_string_t> for &'a mut String {
    fn to_c(_: &'a mut String) -> cef_string_t {
        panic!("unimplemented CEF type conversion: &'a mut String");
    }
    unsafe fn to_rust(_: cef_string_t) -> &'a mut String {
        panic!("unimplemented CEF type conversion: cef_string_t");
    }
}

impl<'a> CefWrap<&'a cef_string_list_t> for &'a cef_string_list_t {
    fn to_c(stringlist: &'a cef_string_list_t) -> &'a cef_string_list_t {
        stringlist
    }
    unsafe fn to_rust(_: &'a cef_string_list_t) -> &'a cef_string_list_t {
        panic!("unimplemented CEF type conversion: cef_string_t");
    }
}
