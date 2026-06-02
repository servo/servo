/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This is a module, but this comment shouldn't trigger an error.
//* This is also a module doc comment *//
//This comment should trigger an error, because of missing space.

use    app_units::Au;
use raqote::{GradientStop, Source, SolidSource};
use raqote::{Source, SolidSource},
ColorPattern, CompositionOp};
use euclid::Size2D;
use raqote::Gradient;
use raqote::{Gradient};


mod paint_context;
pub mod display_list;
mod test::{
};

extern crate webrender_api;
extern crate style_traits;

#[foo = "bar,baz"]
#[derive(Copy,Debug, ComputeSquaredDistance)]
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
        struct Member {
            member_name:"Foo"
            member_id:5
        }
    }

    fn test_fun2(y : &String, z : &Vec<f32>, r: &Root<isize>, s: &DomRoot<isize>) -> () {
        let x = true;
        x
            && x;
        if x {
             ;
        }
        else {
             ;
        }
    }

    type Text_Fun3 = fn( i32) -> i32;

    fn test_fun3<Text_Fun3>( y: Text_Fun3) {
        let (x, y) = (1, 2) // Should not trigger
        test_fun( x);
        test_fun (y);
    }

    // Should not be triggered
    macro_rules! test_macro ( ( $( $fun:ident = $flag:ident ; )* ) => ());

    let var
        = "val";

    fn test_fun4()
       {
     }
    let var = if true {
          "true"
      } else { // Should not trigger
          "false"
      } // Should not trigger

    if  true { // Double space after keyword
        42
    } else {
        let xif = 42 in {  xif  } // Should not trigger
    }

    //Comment with no space following it, should trigger an error.
    let a = 3;//This should cause a problem.
    //~^ This is a compile-test comment and shouldn't cause a problem.
    // This comment has a URL, but it shouldn't trigger an error http://servo.org.
    // This comment has a URL, but it shouldn't trigger an error http://servo.org.
    /* This is another style of comment, but shouldn't trigger an error. */
}
