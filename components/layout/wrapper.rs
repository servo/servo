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

use atomic_refcell::{AtomicRef, AtomicRefMut};
use core::nonzero::NonZero;
use data::{LayoutData, LayoutDataFlags, StyleAndLayoutData};
use script_layout_interface::{OpaqueStyleAndLayoutData, StyleData};
use script_layout_interface::wrapper_traits::{LayoutNode, ThreadSafeLayoutElement, ThreadSafeLayoutNode};
use script_layout_interface::wrapper_traits::GetLayoutData;
use style::computed_values::content::{self, ContentItem};
use style::dom::{NodeInfo, TNode};
use style::selector_parser::RestyleDamage;

pub unsafe fn drop_style_and_layout_data(data: OpaqueStyleAndLayoutData) {
    let ptr: *mut StyleData = data.ptr.get();
    let non_opaque: *mut StyleAndLayoutData = ptr as *mut _;
    let _ = Box::from_raw(non_opaque);
}

pub trait LayoutNodeLayoutData {
    /// Similar to borrow_data*, but returns the full PersistentLayoutData rather
    /// than only the style::data::ElementData.
    fn borrow_layout_data(&self) -> Option<AtomicRef<LayoutData>>;
    fn mutate_layout_data(&self) -> Option<AtomicRefMut<LayoutData>>;
    fn flow_debug_id(self) -> usize;
}

impl<T: GetLayoutData> LayoutNodeLayoutData for T {
    fn borrow_layout_data(&self) -> Option<AtomicRef<LayoutData>> {
        self.get_raw_data().map(|d| d.layout_data.borrow())
    }

    fn mutate_layout_data(&self) -> Option<AtomicRefMut<LayoutData>> {
        self.get_raw_data().map(|d| d.layout_data.borrow_mut())
    }

    fn flow_debug_id(self) -> usize {
        self.borrow_layout_data().map_or(0, |d| d.flow_construction_result.debug_id())
    }
}

pub trait GetRawData {
    fn get_raw_data(&self) -> Option<&StyleAndLayoutData>;
}

impl<T: GetLayoutData> GetRawData for T {
    fn get_raw_data(&self) -> Option<&StyleAndLayoutData> {
        self.get_style_and_layout_data().map(|opaque| {
            let container = opaque.ptr.get() as *mut StyleAndLayoutData;
            unsafe { &*container }
        })
    }
}

pub trait LayoutNodeHelpers {
    fn initialize_data(&self);
    fn clear_data(&self);
}

impl<T: LayoutNode> LayoutNodeHelpers for T {
    fn initialize_data(&self) {
        if self.get_raw_data().is_none() {
            let ptr: *mut StyleAndLayoutData =
                Box::into_raw(Box::new(StyleAndLayoutData::new()));
            let opaque = OpaqueStyleAndLayoutData {
                ptr: unsafe { NonZero::new(ptr as *mut StyleData) }
            };
            unsafe { self.init_style_and_layout_data(opaque) };
        };
    }

    fn clear_data(&self) {
        if self.get_raw_data().is_some() {
            unsafe { drop_style_and_layout_data(self.take_style_and_layout_data()) };
        }
    }
}

pub trait ThreadSafeLayoutNodeHelpers {
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

    /// The RestyleDamage from any restyling, or RestyleDamage::rebuild_and_reflow() if this
    /// is the first time layout is visiting this node. We implement this here, rather than
    /// with the rest of the wrapper layer, because we need layout code to determine whether
    /// layout has visited the node.
    fn restyle_damage(self) -> RestyleDamage;
}

impl<T: ThreadSafeLayoutNode> ThreadSafeLayoutNodeHelpers for T {
    fn flags(self) -> LayoutDataFlags {
            self.borrow_layout_data().as_ref().unwrap().flags
    }

    fn insert_flags(self, new_flags: LayoutDataFlags) {
        self.mutate_layout_data().unwrap().flags.insert(new_flags);
    }

    fn remove_flags(self, flags: LayoutDataFlags) {
        self.mutate_layout_data().unwrap().flags.remove(flags);
    }

    fn text_content(&self) -> TextContent {
        if self.get_pseudo_element_type().is_replaced_content() {
            let style = self.as_element().unwrap().resolved_style();

            return match style.as_ref().get_counters().content {
                content::T::Items(ref value) if !value.is_empty() => {
                    TextContent::GeneratedContent((*value).clone())
                }
                _ => TextContent::GeneratedContent(vec![]),
            };
        }

        return TextContent::Text(self.node_text_content());
    }

    fn restyle_damage(self) -> RestyleDamage {
        // We need the underlying node to potentially access the parent in the
        // case of text nodes. This is safe as long as we don't let the parent
        // escape and never access its descendants.
        let mut node = unsafe { self.unsafe_get() };

        // If this is a text node, use the parent element, since that's what
        // controls our style.
        if node.is_text_node() {
            node = node.parent_node().unwrap();
            debug_assert!(node.is_element());
        }

        let damage = {
            let data = node.get_raw_data().unwrap();
            if let Some(r) = data.style_data.element_data.borrow().get_restyle() {
                // We're reflowing a node that just got a restyle, and so the
                // damage has been computed and stored in the RestyleData.
                r.damage
            } else if !data.layout_data.borrow().flags.contains(::data::HAS_BEEN_TRAVERSED) {
                // We're reflowing a node that was styled for the first time and
                // has never been visited by layout. Return rebuild_and_reflow,
                // because that's what the code expects.
                RestyleDamage::rebuild_and_reflow()
            } else {
                // We're reflowing a node whose style data didn't change, but whose
                // layout may change due to changes in ancestors or descendants.
                RestyleDamage::empty()
            }
        };

        damage
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
