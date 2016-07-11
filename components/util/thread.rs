/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::thread;

pub fn spawn_named<F>(name: String, f: F)
    where F: FnOnce() + Send + 'static
{
    thread::Builder::new().name(name).spawn(f).expect("Thread spawn failed");
}
