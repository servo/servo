/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(alloc)]
#![feature(box_syntax)]
#![feature(collections)]
#![feature(core)]
#![feature(hash)]
#![feature(int_uint)]
#![feature(io)]
#![feature(libc)]
#![feature(path)]
#![feature(plugin)]
#![feature(rustc_private)]
#![feature(std_misc)]
#![feature(thread_local)]
#![feature(unicode)]
#![feature(unsafe_destructor)]

#![deny(unsafe_blocks)]
#![allow(unrooted_must_root)]
#![allow(missing_copy_implementations)]

#[macro_use]
extern crate log;

#[macro_use]extern crate bitflags;
extern crate azure;
extern crate cssparser;
extern crate canvas;
extern crate geom;
extern crate gfx;
extern crate layout_traits;
extern crate script;
extern crate script_traits;
extern crate "serialize" as rustc_serialize;
extern crate serialize;
extern crate png;
extern crate style;
#[macro_use]
#[no_link] #[plugin]
extern crate "plugins" as servo_plugins;
extern crate net;
extern crate msg;
#[macro_use]
extern crate "util" as servo_util;

#[no_link] #[macro_use] #[plugin]
extern crate string_cache_macros;
extern crate string_cache;

extern crate collections;
extern crate encoding;
extern crate libc;
extern crate url;

// Listed first because of macro definitions
pub mod layout_debug;

pub mod block;
pub mod construct;
pub mod context;
pub mod display_list_builder;
pub mod floats;
pub mod flow;
pub mod flow_list;
pub mod flow_ref;
pub mod fragment;
pub mod layout_task;
pub mod inline;
pub mod list_item;
pub mod model;
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
pub mod util;
pub mod incremental;
pub mod wrapper;

pub mod css {
    pub mod matching;
    pub mod node_style;
}
