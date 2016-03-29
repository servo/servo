/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(as_unsafe_cell)]
#![feature(box_patterns)]
#![feature(box_syntax)]
#![feature(custom_derive)]
#![feature(mpsc_select)]
#![feature(nonzero)]
#![feature(plugin)]
#![feature(raw)]
#![feature(step_by)]
#![feature(str_char)]
#![feature(unsafe_no_drop_flag)]

#![deny(unsafe_code)]

#![plugin(heapsize_plugin)]
#![plugin(plugins)]

extern crate app_units;
extern crate azure;
#[macro_use]
extern crate bitflags;
extern crate canvas_traits;
extern crate core;
extern crate cssparser;
extern crate euclid;
extern crate fnv;
extern crate gfx;
extern crate gfx_traits;
extern crate heapsize;
extern crate ipc_channel;
extern crate layout_traits;
extern crate libc;
#[macro_use]
extern crate log;
extern crate msg;
extern crate net_traits;
#[macro_use]
#[no_link]
extern crate plugins as servo_plugins;
#[macro_use]
extern crate profile_traits;
#[macro_use]
extern crate range;
extern crate rustc_serialize;
extern crate script;
extern crate script_traits;
extern crate selectors;
extern crate serde;
extern crate serde_json;
extern crate smallvec;
#[macro_use(atom, ns)] extern crate string_cache;
extern crate style;
extern crate style_traits;
extern crate time;
extern crate unicode_bidi;
extern crate unicode_script;
extern crate url;
extern crate util;
extern crate webrender_traits;

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
pub mod layout_thread;
mod list_item;
mod model;
mod multicol;
mod opaque_node;
mod parallel;
mod persistent_list;
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
mod webrender_helpers;
mod wrapper;

// For unit tests:
pub use fragment::Fragment;
