/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(plugin)]
#![plugin(plugins)]


fn main() {
    let x = "foo".to_string();
    //~^ ERROR str.to_owned() is more efficient than str.to_string()

    let x = &x[..];
    x.to_string();
    //~^ ERROR str.to_owned() is more efficient than str.to_string()
}
