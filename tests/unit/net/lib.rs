/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![cfg_attr(test, feature(alloc))]

extern crate net;
extern crate net_traits;
extern crate url;
extern crate util;

#[cfg(test)] mod cookie;
#[cfg(test)] mod data_loader;
#[cfg(test)] mod mime_classifier;
#[cfg(test)] mod resource_task;
