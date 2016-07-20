/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A safe wrapper for DOM nodes that prevents layout from mutating the DOM, from letting DOM nodes
//! escape, and from generally doing anything that it isn't supposed to. This is accomplished via
//! a simple whitelist of allowed operations, along with some lifetime magic to prevent nodes from
//! escaping.
//!
//! As a security wrapper is only as good as its whitelist, be careful when adding operations to
//! this list. The cardinal rules are:
//!
//! 1. Layout is not allowed to mutate the DOM.
//!
//! 2. Layout is not allowed to see anything with `LayoutJS` in the name, because it could hang
//!    onto these objects and cause use-after-free.
//!
//! When implementing wrapper functions, be careful that you do not touch the borrow flags, or you
//! will race and cause spurious thread failure. (Note that I do not believe these races are
//! exploitable, but they'll result in brokenness nonetheless.)
//!
//! Rules of the road for this file:
//!
//! * Do not call any methods on DOM nodes without checking to see whether they use borrow flags.
//!
//!   o Instead of `get_attr()`, use `.get_attr_val_for_layout()`.
//!
//!   o Instead of `html_element_in_html_document()`, use
//!     `html_element_in_html_document_for_layout()`.

#![allow(unsafe_code)]

use core::nonzero::NonZero;
use data::{LayoutDataFlags, PrivateLayoutData};
use script_layout_interface::wrapper_traits::{LayoutNode, ThreadSafeLayoutNode};
use script_layout_interface::{OpaqueStyleAndLayoutData, PartialStyleAndLayoutData};
use style::computed_values::content::{self, ContentItem};
use style::refcell::{Ref, RefCell, RefMut};

pub type NonOpaqueStyleAndLayoutData = *mut RefCell<PrivateLayoutData>;

pub trait LayoutNodeLayoutData {
    /// Similar to borrow_data*, but returns the full PrivateLayoutData rather
    /// than only the PrivateStyleData.
    unsafe fn borrow_layout_data_unchecked(&self) -> Option<*const PrivateLayoutData>;
    fn borrow_layout_data(&self) -> Option<Ref<PrivateLayoutData>>;
    fn mutate_layout_data(&self) -> Option<RefMut<PrivateLayoutData>>;
    fn initialize_data(self);
    fn flow_debug_id(self) -> usize;
}

impl<T: LayoutNode> LayoutNodeLayoutData for T {
    unsafe fn borrow_layout_data_unchecked(&self) -> Option<*const PrivateLayoutData> {
        self.get_style_and_layout_data().map(|opaque| {
            let container = *opaque.ptr as NonOpaqueStyleAndLayoutData;
            &(*(*container).as_unsafe_cell().get()) as *const PrivateLayoutData
        })
    }

    fn borrow_layout_data(&self) -> Option<Ref<PrivateLayoutData>> {
        unsafe {
            self.get_style_and_layout_data().map(|opaque| {
                let container = *opaque.ptr as NonOpaqueStyleAndLayoutData;
                (*container).borrow()
            })
        }
    }

    fn mutate_layout_data(&self) -> Option<RefMut<PrivateLayoutData>> {
        unsafe {
            self.get_style_and_layout_data().map(|opaque| {
                let container = *opaque.ptr as NonOpaqueStyleAndLayoutData;
                (*container).borrow_mut()
            })
        }
    }

    fn initialize_data(self) {
        if unsafe { self.borrow_data_unchecked() }.is_none() {
            let ptr: NonOpaqueStyleAndLayoutData =
                Box::into_raw(box RefCell::new(PrivateLayoutData::new()));
            let opaque = OpaqueStyleAndLayoutData {
                ptr: unsafe { NonZero::new(ptr as *mut RefCell<PartialStyleAndLayoutData>) }
            };
            self.init_style_and_layout_data(opaque);
        }
    }

    fn flow_debug_id(self) -> usize {
        self.borrow_layout_data().map_or(0, |d| d.flow_construction_result.debug_id())
    }
}

pub trait ThreadSafeLayoutNodeHelpers {
    fn flow_debug_id(self) -> usize;

    unsafe fn borrow_layout_data_unchecked(&self) -> Option<*const PrivateLayoutData>;

    /// Borrows the layout data immutably. Fails on a conflicting borrow.
    ///
    /// TODO(pcwalton): Make this private. It will let us avoid borrow flag checks in some cases.
    #[inline(always)]
    fn borrow_layout_data(&self) -> Option<Ref<PrivateLayoutData>>;

    /// Borrows the layout data mutably. Fails on a conflicting borrow.
    ///
    /// TODO(pcwalton): Make this private. It will let us avoid borrow flag checks in some cases.
    #[inline(always)]
    fn mutate_layout_data(&self) -> Option<RefMut<PrivateLayoutData>>;

    /// Returns the layout data flags for this node.
    fn flags(self) -> LayoutDataFlags;

    /// Adds the given flags to this node.
    fn insert_flags(self, new_flags: LayoutDataFlags);

    /// Removes the given flags from this node.
    fn remove_flags(self, flags: LayoutDataFlags);

    /// If this is a text node, generated content, or a form element, copies out
    /// its content. Otherwise, panics.
    ///
    /// FIXME(pcwalton): This might have too much copying and/or allocation. Profile this.
    fn text_content(&self) -> TextContent;
}

impl<T: ThreadSafeLayoutNode> ThreadSafeLayoutNodeHelpers for T {
    fn flow_debug_id(self) -> usize {
        self.borrow_layout_data().map_or(0, |d| d.flow_construction_result.debug_id())
    }

    unsafe fn borrow_layout_data_unchecked(&self) -> Option<*const PrivateLayoutData> {
        self.get_style_and_layout_data().map(|opaque| {
            let container = *opaque.ptr as NonOpaqueStyleAndLayoutData;
            &(*(*container).as_unsafe_cell().get()) as *const PrivateLayoutData
        })
    }

    fn borrow_layout_data(&self) -> Option<Ref<PrivateLayoutData>> {
        unsafe {
            self.get_style_and_layout_data().map(|opaque| {
                let container = *opaque.ptr as NonOpaqueStyleAndLayoutData;
                (*container).borrow()
            })
        }
    }

    fn mutate_layout_data(&self) -> Option<RefMut<PrivateLayoutData>> {
        unsafe {
            self.get_style_and_layout_data().map(|opaque| {
                let container = *opaque.ptr as NonOpaqueStyleAndLayoutData;
                (*container).borrow_mut()
            })
        }
    }

    fn flags(self) -> LayoutDataFlags {
        unsafe {
            (*self.borrow_layout_data_unchecked().unwrap()).flags
        }
    }

    fn insert_flags(self, new_flags: LayoutDataFlags) {
        self.mutate_layout_data().unwrap().flags.insert(new_flags);
    }

    fn remove_flags(self, flags: LayoutDataFlags) {
        self.mutate_layout_data().unwrap().flags.remove(flags);
    }

    fn text_content(&self) -> TextContent {
        if self.get_pseudo_element_type().is_replaced_content() {
            let style = self.resolved_style();

            return match style.as_ref().get_counters().content {
                content::T::Content(ref value) if !value.is_empty() => {
                    TextContent::GeneratedContent((*value).clone())
                }
                _ => TextContent::GeneratedContent(vec![]),
            };
        }

        return TextContent::Text(self.node_text_content());
    }
}

pub enum TextContent {
    Text(String),
    GeneratedContent(Vec<ContentItem>),
}

impl TextContent {
    pub fn is_empty(&self) -> bool {
        match *self {
            TextContent::Text(_) => false,
            TextContent::GeneratedContent(ref content) => content.is_empty(),
        }
    }
}
