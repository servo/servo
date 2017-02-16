/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate script;

use script::test::TrustedPromise;

fn cloneable<T: Clone>() {
}

fn main() {
    cloneable::<TrustedPromise>();
    //~^ ERROR the trait bound `script::test::TrustedPromise: std::clone::Clone` is not satisfied
}
