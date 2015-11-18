/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(cell_extras)]
#![feature(custom_derive)]
#![feature(hashmap_hasher)]
#![feature(mpsc_select)]
#![feature(plugin)]
#![feature(raw)]
#![feature(step_by)]
#![feature(str_char)]
#![feature(unsafe_no_drop_flag)]

#![deny(unsafe_code)]

#![plugin(string_cache_plugin)]
#![plugin(plugins)]

extern crate app_units;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;
#[macro_use]
extern crate profile_traits;
#[macro_use]
#[no_link]
extern crate plugins as servo_plugins;
#[macro_use]
extern crate util;
extern crate azure;
extern crate canvas_traits;
extern crate clock_ticks;
extern crate cssparser;
extern crate encoding;
extern crate euclid;
extern crate fnv;
extern crate gfx;
extern crate gfx_traits;
extern crate ipc_channel;
extern crate layout_traits;
extern crate libc;
extern crate msg;
extern crate net_traits;
extern crate rustc_serialize;
extern crate script;
extern crate script_traits;
#[macro_use(state_pseudo_classes)] extern crate selectors;
extern crate serde;
extern crate serde_json;
extern crate smallvec;
extern crate string_cache;
extern crate style;
extern crate unicode_bidi;
extern crate unicode_script;
extern crate url;

#[macro_use]
mod layout_debug;

mod animation;
mod block;
mod construct;
mod context;
mod data;
mod display_list_builder;
mod flex;
mod floats;
mod flow;
mod flow_list;
mod flow_ref;
mod fragment;
mod generated_content;
mod incremental;
mod inline;
pub mod layout_task;
mod list_item;
mod model;
mod multicol;
mod opaque_node;
mod parallel;
mod query;
mod sequential;
mod table;
mod table_caption;
mod table_cell;
mod table_colgroup;
mod table_row;
mod table_rowgroup;
mod table_wrapper;
mod text;
mod traversal;
mod wrapper;

mod css {
    pub mod matching;
}
