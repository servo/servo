/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS library requires that DOM nodes be convertable to *c_void through this trait
extern mod netsurfcss;

use dom::node::{AbstractNode, LayoutView};

use core::cast;

// FIXME: Rust #3908. rust-css can't reexport VoidPtrLike
use css::node_void_ptr::netsurfcss::util::VoidPtrLike;

impl VoidPtrLike for AbstractNode<LayoutView> {
    fn from_void_ptr(node: *libc::c_void) -> AbstractNode<LayoutView> {
        assert!(node.is_not_null());
        unsafe {
            cast::transmute(node)
        }
    }

    fn to_void_ptr(&self) -> *libc::c_void {
        unsafe {
            cast::transmute(*self)
        }
    }
}
