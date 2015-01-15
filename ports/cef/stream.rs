/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use interfaces::{cef_read_handler_t, cef_stream_reader_t, cef_stream_writer_t};
use interfaces::{cef_write_handler_t};
use types::cef_string_t;

use libc;

cef_stub_static_method_impls! {
    fn cef_stream_reader_create_for_file(_file_name: *const cef_string_t)
                                         -> *mut cef_stream_reader_t
    fn cef_stream_reader_create_for_data(_data: *mut (), _size: libc::size_t)
                                         -> *mut cef_stream_reader_t
    fn cef_stream_reader_create_for_handler(_handler: *mut cef_read_handler_t)
                                            -> *mut cef_stream_reader_t
    fn cef_stream_writer_create_for_file(_file_name: *const cef_string_t)
                                         -> *mut cef_stream_writer_t
    fn cef_stream_writer_create_for_handler(_handler: *mut cef_write_handler_t)
                                            -> *mut cef_stream_writer_t
}

