/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

extern crate app_units;
extern crate atomic_refcell;
#[macro_use]
extern crate bitflags;
extern crate canvas_traits;
extern crate euclid;
extern crate fnv;
extern crate gfx;
extern crate gfx_traits;
#[macro_use] extern crate html5ever;
extern crate ipc_channel;
extern crate libc;
#[macro_use]
extern crate log;
extern crate malloc_size_of;
extern crate msg;
extern crate net_traits;
extern crate ordered_float;
extern crate parking_lot;
extern crate profile_traits;
#[macro_use]
extern crate range;
extern crate rayon;
extern crate script_layout_interface;
extern crate script_traits;
#[macro_use] extern crate serde;
extern crate serde_json;
extern crate servo_arc;
extern crate servo_atoms;
extern crate servo_config;
extern crate servo_geometry;
extern crate servo_url;
extern crate smallvec;
extern crate style;
extern crate style_traits;
extern crate unicode_bidi;
extern crate unicode_script;
extern crate webrender_api;

#[macro_use]
pub mod layout_debug;

pub mod animation;
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
pub mod opaque_node;
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
pub mod traversal;
pub mod wrapper;

// For unit tests:
pub use fragment::Fragment;
pub use fragment::SpecificFragmentInfo;
pub use self::data::LayoutData;

// We can't use servo_arc for everything in layout, because the Flow stuff uses
// weak references.
use servo_arc::Arc as ServoArc;
