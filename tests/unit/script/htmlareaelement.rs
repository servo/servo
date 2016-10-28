// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


use euclid::point::Point2D;
use script::dom::htmlareaelement::Area;
fn inspect_circle (a: Area, vec: Vec<f32>) {
    match a {
        Area::Circle { left, top, radius } => { assert_eq! (left, vec[0]);
            assert_eq! (top, vec[1]);
            assert_eq! (radius, vec[2]);
        },
        _ => { assert!(false); },
    }
}
fn inspect_rectangle (a: Area, vec: Vec<f32>) {
    match a {
        Area::Rectangle { left_l, top_t, left_r, top_b } => {
            assert!(left_l < left_r);
            assert!(top_t < top_b);
            assert_eq! (left_l, vec[0]);
            assert_eq! (top_t, vec[1]);
            assert_eq! (left_r, vec[2]);
            assert_eq! (top_b, vec[3]);
        },
        _ => { assert!(false); },
    }
}
fn inspect_polygon (a: Area, vec: Vec<f32>, size: i32) {
    match a {
        Area::Polygon { points } => {
            assert_eq! (size, points.len() as i32);
            assert! (points.len() % 2 == 0);
            assert! (points.len() >= 6);
            let mut index = 0;
            for x in vec {
                assert_eq! (x, points[index]);
                index = index + 1;
            }
        },

        _ => { assert!(false); },
    }
}

fn inspect_default (a: Area) {
    match a {
        Area::Default => { assert!(true); },

        _=> { assert!(false); },
    }
}
/*Pathological tests*/

#[test]
fn garbage_input ()
{
    let area = Area::get_area (";.,()8.2".to_string());
    inspect_default (area);
}


#[test]
fn no_case_matching_input ()
{
    let area = Area::get_area ("8.2, 10.2".to_string());
    inspect_default (area);
}

#[test]
fn delemiter_input ()
{
    let area = Area::get_area (";,  ;,".to_string());
    inspect_default (area);
}

/*Area::Circle tests*/
#[test]
fn valid_circle_inputs ()
{
    let input = "10.2, 3.4, 5.2";
    let mut vec = Vec::new ();

    vec.push(10.2);
    vec.push(3.4);
    vec.push(5.2);
    let area = Area::get_area (input.to_string());

    inspect_circle(area, vec);
}

#[test]
fn valid_negative_circle_inputs ()
{
    let input = "-10.2, -3.4, 5.2";
    let mut vec = Vec::new ();

    vec.push(-10.2);
    vec.push(-3.4);
    vec.push(5.2);
    let area = Area::get_area (input.to_string());

    inspect_circle(area, vec);
}

/*Area::Rectangle tests*/
#[test]
fn rectangle_valid_input ()
{
    let input = "5.2, 1.1, 10.2, 3.4";

    let mut vec = Vec::new ();

    vec.push(5.2);
    vec.push(1.1);
    vec.push(10.2);
    vec.push(3.4);

    let area = Area::get_area (input.to_string());

    inspect_rectangle (area, vec);
}

#[test]
fn rectangle_valid_negative_input ()
{
    let input = "-10.2, -3.4, -5.2, -1.1";

    let mut vec = Vec::new ();

    vec.push(-10.2);
    vec.push(-3.4);
    vec.push(-5.2);
    vec.push(-1.1);

    let area = Area::get_area (input.to_string());

    inspect_rectangle (area, vec);
}

#[test]
fn rectangle_invalid_input ()
{
    let input = "5.2, 4.3, 10.2, 1.1.2";

    let mut vec = Vec::new ();

    vec.push(5.2);
    vec.push(0.0);
    vec.push(10.2);
    vec.push(4.3);

    let area = Area::get_area (input.to_string());

    inspect_rectangle (area, vec);
}

#[test]
fn rectangle_unordered_input ()
{
    let input = "5.2, 1.1, 10.2, 4.3";

    let mut vec = Vec::new ();

    vec.push(5.2);
    vec.push(1.1);
    vec.push(10.2);
    vec.push(4.3);

    let area = Area::get_area (input.to_string());

    inspect_rectangle (area, vec);
}

/*Area::Rectangle tests*/
#[test]
fn polygon_six_points_valid_input ()
{
    let input = "1.1, 1.1, 6.1, 1.1, 3.1, 3.1";
    let size = 6;

    let mut vec = Vec::new ();

    vec.push(1.1);
    vec.push(1.1);
    vec.push(6.1);
    vec.push(1.1);
    vec.push(3.1);
    vec.push(3.1);

    let area = Area::get_area (input.to_string());

    inspect_polygon (area, vec, size);
}

#[test]
fn polygon_six_points_valid_negative_input ()
{
    let input = "1.1, -1.1, 6.1, -1.1, 3.1, -3.1";
    let size = 6;

    let mut vec = Vec::new ();

    vec.push(1.1);
    vec.push(-1.1);
    vec.push(6.1);
    vec.push(-1.1);
    vec.push(3.1);
    vec.push(-3.1);

    let area = Area::get_area (input.to_string());

    inspect_polygon (area, vec, size);
}


#[test]
fn polygon_six_points_invalid_input ()
{
    let input = ";1.1,  1.1,'; 6.1,(*^() 1.1, 3.1, 3.1, 100.1 %$,;";
    let size = 6;

    let mut vec = Vec::new ();

    vec.push(1.1);
    vec.push(1.1);
    vec.push(6.1);
    vec.push(1.1);
    vec.push(3.1);
    vec.push(3.1);

    let area = Area::get_area (input.to_string());

    inspect_polygon (area, vec, size);
}

#[test]
fn polygon_eight_points_invalid_input ()
{
    let input = "1.1, -1.1, 6.1, -1.1, 1.1, -3.1, 6.1, -3.1.2, 12.1";
    let size = 8;

    let mut vec = Vec::new ();

    vec.push(1.1);
    vec.push(-1.1);
    vec.push(6.1);
    vec.push(-1.1);
    vec.push(1.1);
    vec.push(-3.1);
    vec.push(6.1);
    vec.push(0.0);

    let area = Area::get_area (input.to_string());

    inspect_polygon (area, vec, size);
}
#[test]
fn test_hit_test_circle() {
   let p = Point2D::new(10.0, 20.0);
   assert_eq!(Area::Circle { left: 20.0, top: 10.0, radius: 5.0 }.hit_test(p), false);
   let q = Point2D::new(10.0, 12.0);
   assert_eq!(Area::Circle { left: 10.0, top: 10.0, radius: 5.0 }.hit_test(q), true);
}

#[test]
fn test_hit_test_rectangle() {
   let p = Point2D::new(10.0, 5.0);
   assert_eq!(Area::Rectangle { left_l: 1.0, top_t: 7.0, left_r: 15.0, top_b: 10.0 }.hit_test(p), false);
   let q = Point2D::new(10.0, 12.0);
   assert_eq!(Area::Rectangle { left_l: 8.0, top_t: 10.0, left_r: 20.0, top_b: 12.0 }.hit_test(q), true);
}

#[test]
fn test_hit_test_polygon() {
   let v = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
   let p = Point2D::new(10.0, 5.0);
   assert_eq!(Area::Polygon { points: v }.hit_test(p), false);
   let w = vec![7.0, 7.5, 8.2, 9.0, 11.0, 12.0];
   let q = Point2D::new(10.0, 5.0);
   assert_eq!(Area::Polygon { points: w }.hit_test(q), false);
}
