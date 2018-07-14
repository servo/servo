/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[cfg(test)] extern crate euclid;
#[cfg(test)] extern crate msg;
#[cfg(test)] extern crate script;
#[cfg(test)] extern crate servo_url;
#[cfg(test)] extern crate style;

#[cfg(test)] mod origin;
#[cfg(all(test, target_pointer_width = "64"))] mod size_of;
#[cfg(test)] mod textinput;
#[cfg(test)] mod headers;
#[cfg(test)] mod htmlareaelement;
#[cfg(test)] mod htmlimageelement;

/**
```compile_fail,E0277
extern crate script;

fn cloneable<T: Clone>() {}

fn main() {
    cloneable::<script::test::TrustedPromise>();
}
```
*/
pub fn trustedpromise_does_not_impl_clone() {}
