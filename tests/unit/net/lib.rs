/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(plugin)]
#![plugin(plugins)]

extern crate cookie as cookie_rs;
extern crate devtools_traits;
extern crate flate2;
extern crate hyper;
extern crate ipc_channel;
extern crate msg;
extern crate net;
extern crate net_traits;
extern crate time;
extern crate unicase;
extern crate url;
extern crate util;

#[cfg(test)] mod cookie;
#[cfg(test)] mod data_loader;
#[cfg(test)] mod fetch;
#[cfg(test)] mod mime_classifier;
#[cfg(test)] mod resource_thread;
#[cfg(test)] mod hsts;
#[cfg(test)] mod http_loader;
