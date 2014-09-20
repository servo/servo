/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![comment = "The Servo Parallel Browser Project"]
#![license = "MPL"]

#![feature(globs, macro_rules, phase, thread_local, unsafe_destructor)]

#![deny(unused_imports, unused_variable)]
#![allow(unrooted_must_root)]

#[phase(plugin, link)]
extern crate log;

extern crate debug;

extern crate geom;
extern crate gfx;
extern crate layout_traits;
extern crate script;
extern crate script_traits;
extern crate serialize;
extern crate style;
#[phase(plugin)]
extern crate "macros" as servo_macros;
extern crate "net" as servo_net;
extern crate "msg" as servo_msg;
#[phase(plugin, link)]
extern crate "util" as servo_util;

extern crate collections;
extern crate encoding;
extern crate green;
extern crate libc;
extern crate sync;
extern crate url;

// Listed first because of macro definitions
pub mod layout_debug;

pub mod block;
pub mod construct;
pub mod context;
pub mod floats;
pub mod flow;
pub mod flow_list;
pub mod flow_ref;
pub mod fragment;
pub mod layout_task;
pub mod inline;
pub mod model;
pub mod parallel;
pub mod table_wrapper;
pub mod table;
pub mod table_caption;
pub mod table_colgroup;
pub mod table_rowgroup;
pub mod table_row;
pub mod table_cell;
pub mod text;
pub mod util;
pub mod incremental;
pub mod wrapper;

pub mod css {
    mod node_util;

    pub mod matching;
    pub mod node_style;
}
