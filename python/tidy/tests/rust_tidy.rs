/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use    app_units::Au;
use azure::azure_hl::{AntialiasMode, Color,
ColorPattern, CompositionOp};
use euclid::size::Size2D;
use azure::azure::AzIntSize;

use std;

mod paint_context;
pub mod display_list;
mod test::{
};

extern crate webrender_traits;
extern crate style_traits;

impl test {

    fn test_fun(y:f32)->f32{
        let x=5;
        x = x-1;
        x = x*x;
        let z = match y {
            1 =>2,
            2 => 1,
        };
        let z = &Vec<T>;
    }

    fn test_fun2(y : &String, z : &Vec<f32>) -> f32 {
        1
    }
}
