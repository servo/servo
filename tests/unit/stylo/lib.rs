/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate app_units;
extern crate atomic_refcell;
extern crate cssparser;
extern crate env_logger;
extern crate euclid;
extern crate geckoservo;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate num_cpus;
extern crate parking_lot;
extern crate rayon;
extern crate selectors;
extern crate servo_url;
#[macro_use] extern crate style;
extern crate style_traits;

mod sanity_checks;

#[path = "../../../ports/geckolib/stylesheet_loader.rs"]
mod stylesheet_loader;

mod servo_function_signatures;

