/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use util::thread::spawn_named;

#[test]
fn spawn_named_test() {
    spawn_named("Test".to_owned(), move || {
        println!("I can run!");
    });
}
