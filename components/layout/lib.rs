/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

pub mod layout_debug;

mod block;
pub mod construct;
pub mod context;
pub mod data;
pub mod display_list;
mod flex;
mod floats;
pub mod flow;
mod flow_list;
pub mod flow_ref;
mod fragment;
mod generated_content;
pub mod incremental;
mod inline;
mod linked_list;
mod list_item;
mod model;
mod multicol;
pub mod parallel;
mod persistent_list;
pub mod query;
pub mod sequential;
mod table;
mod table_caption;
mod table_cell;
mod table_colgroup;
mod table_row;
mod table_rowgroup;
mod table_wrapper;
mod text;
mod text_run;
pub mod traversal;
pub mod wrapper;

// For unit tests:
// We can't use servo_arc for everything in layout, because the Flow stuff uses
// weak references.
use servo_arc::Arc as ServoArc;

pub use self::data::LayoutData;
pub use crate::fragment::{Fragment, SpecificFragmentInfo};
