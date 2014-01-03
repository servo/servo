/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::task;

pub fn spawn_named<S: IntoSendStr>(name: S, f: proc()) {
    let mut builder = task::task();
    builder.name(name);
    builder.spawn(f);
}
