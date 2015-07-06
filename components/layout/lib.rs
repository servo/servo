/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(append)]
#![feature(arc_unique)]
#![feature(box_syntax)]
#![feature(filling_drop)]
#![feature(hashmap_hasher)]
#![feature(heap_api)]
#![feature(mpsc_select)]
#![feature(plugin)]
#![feature(raw)]
#![feature(slice_chars)]
#![feature(str_char)]
#![feature(unsafe_no_drop_flag)]

#![deny(unsafe_code)]
#![allow(unrooted_must_root)]

#![plugin(string_cache_plugin)]
#![plugin(plugins)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate bitflags;

#[macro_use]
#[no_link]
extern crate plugins as servo_plugins;
extern crate net_traits;
#[macro_use]
extern crate profile_traits;

#[macro_use]
extern crate util;

extern crate rustc_serialize;
extern crate azure;
extern crate canvas_traits;
extern crate clock_ticks;
extern crate cssparser;
extern crate encoding;
extern crate fnv;
extern crate euclid;
extern crate gfx;
extern crate gfx_traits;
extern crate layout_traits;
extern crate libc;
extern crate msg;
extern crate script;
extern crate script_traits;
extern crate selectors;
extern crate smallvec;
extern crate string_cache;
extern crate style;
extern crate url;

// Listed first because of macro definitions
pub mod layout_debug;

pub mod animation;
pub mod block;
pub mod construct;
pub mod context;
pub mod data;
pub mod display_list_builder;
pub mod floats;
pub mod flow;
pub mod flow_list;
pub mod flow_ref;
pub mod fragment;
pub mod generated_content;
pub mod layout_task;
pub mod incremental;
pub mod inline;
pub mod list_item;
pub mod model;
pub mod multicol;
pub mod opaque_node;
pub mod parallel;
pub mod sequential;
pub mod table_wrapper;
pub mod table;
pub mod table_caption;
pub mod table_colgroup;
pub mod table_rowgroup;
pub mod table_row;
pub mod table_cell;
pub mod text;
pub mod traversal;
pub mod wrapper;

pub mod css {
    pub mod matching;
    pub mod node_style;
}
