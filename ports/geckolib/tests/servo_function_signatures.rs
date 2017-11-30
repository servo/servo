/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unused)]

use self::glue::*;
use style::gecko_bindings::bindings;
use style::gecko_properties::*;

include!(concat!(env!("OUT_DIR"), "/check_bindings.rs"));

#[path = "../../../ports/geckolib/error_reporter.rs"]
mod error_reporter;

#[path = "../../../ports/geckolib/stylesheet_loader.rs"]
mod stylesheet_loader;

#[allow(non_snake_case, unused_unsafe, private_no_mangle_fns)]
mod glue {
    // this module pretends to be glue.rs, with the safe functions swapped for unsafe ones. This is
    // a hack to compensate for the fact that `fn` types cannot coerce to `unsafe fn` types. The
    // imports are populated with the same things so the type assertion should be equivalent
    use geckoservo::*;
    include!(concat!(env!("OUT_DIR"), "/glue.rs"));
}
