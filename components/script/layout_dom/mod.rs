/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

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
//! 2. Layout is not allowed to see anything with `LayoutDom` in the name, because it could hang
//!    onto these objects and cause use-after-free.
//!
//! When implementing wrapper functions, be careful that you do not touch the borrow flags, or you
//! will race and cause spurious thread failure. (Note that I do not believe these races are
//! exploitable, but they'll result in brokenness nonetheless.)

mod iterators;
mod servo_dangerous_style_document;
mod servo_dangerous_style_element;
mod servo_dangerous_style_node;
mod servo_dangerous_style_shadow_root;
mod servo_layout_element;
mod servo_layout_node;

pub use iterators::*;
pub use servo_dangerous_style_document::*;
pub use servo_dangerous_style_element::*;
pub use servo_dangerous_style_node::*;
pub use servo_dangerous_style_shadow_root::*;
pub use servo_layout_element::*;
pub use servo_layout_node::*;
