// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use euclid::Point2D;
use script::test::area::{Area, Shape};

#[test]
fn garbage_input() {
    assert!(Area::parse(";.,()8.2", Shape::Circle).is_none())
}

#[test]
fn no_case_matching_input() {
    assert!(Area::parse("8.2, 10.2", Shape::Circle).is_none())
}

#[test]
fn delimiter_input() {
    assert!(Area::parse(";,  ;,", Shape::Circle).is_none())
}

// Area::Circle tests
#[test]
fn valid_circle_inputs() {
    assert_eq!(Area::parse("10.2, 3.4, 5.2", Shape::Circle),
               Some(Area::Circle { left: 10.2, top: 3.4, radius: 5.2 }));
}

#[test]
fn valid_negative_circle_inputs() {
    assert_eq!(Area::parse("-10.2, -3.4, 5.2", Shape::Circle),
               Some(Area::Circle { left: -10.2, top: -3.4, radius: 5.2 }));
}

#[test]
fn invalid_negative_circle_radius() {
    assert!(Area::parse("-10.2, -3.4, -5.2", Shape::Circle).is_none());
}

// Area::Rectangle tests
#[test]
fn rectangle_valid_input() {
    assert_eq!(Area::parse("5.2, 1.1, 10.2, 3.4", Shape::Rectangle),
               Some(Area::Rectangle { top_left: (5.2, 1.1),
               bottom_right: (10.2, 3.4) }));
}

#[test]
fn rectangle_valid_negative_input() {
    assert_eq!(Area::parse("-10.2, -3.4, -5.2, -1.1", Shape::Rectangle),
               Some(Area::Rectangle { top_left: (-10.2, -3.4),
               bottom_right: (-5.2, -1.1) }));
}

#[test]
fn rectangle_invalid_input() {
    assert_eq!(Area::parse("5.2, 4.3, 10.2, 1.1.2", Shape::Rectangle),
               Some(Area::Rectangle { top_left: (5.2, 0.0),
               bottom_right: (10.2, 4.3) }));
}

#[test]
fn rectangle_unordered_input() {
    assert_eq!(Area::parse("5.2, 1.1, 10.2, 4.3", Shape::Rectangle),
               Some(Area::Rectangle { top_left: (5.2, 1.1),
               bottom_right: (10.2, 4.3) }));
}

// Area::Polygon tests
#[test]
fn polygon_six_points_valid_input() {
    assert_eq!(Area::parse("1.1, 1.1, 6.1, 1.1, 3.1, 3.1", Shape::Polygon),
               Some(Area::Polygon { points: vec![1.1, 1.1, 6.1, 1.1, 3.1, 3.1] }));
}

#[test]
fn polygon_six_points_valid_negative_input() {
    assert_eq!(Area::parse("1.1, -1.1, 6.1, -1.1, 3.1, -3.1", Shape::Polygon),
               Some(Area::Polygon { points: vec![1.1, -1.1, 6.1, -1.1, 3.1, -3.1] }));
}

#[test]
fn polygon_six_points_invalid_input() {
    assert_eq!(Area::parse(";1.1,  1.1,'; 6.1,(*^() 1.1, 3.1, 3.1, 100.1 %$,;", Shape::Polygon),
               Some(Area::Polygon { points: vec![1.1, 1.1, 6.1, 1.1, 3.1, 3.1] }));
}

#[test]
fn polygon_eight_points_invalid_input() {
    assert_eq!(Area::parse("1.1, -1.1, 6.1, -1.1, 1.1, -3.1, 6.1, -3.1.2, 12.1", Shape::Polygon),
               Some(Area::Polygon { points: vec![1.1, -1.1, 6.1, -1.1, 1.1, -3.1, 6.1, 0.0] }));
}

#[test]
fn test_hit_test_circle() {
   let circ1 = Area::Circle { left: 20.0, top: 10.0, radius: 5.0 };
   assert!(!circ1.hit_test(&Point2D::new(10.0, 20.0)));
   let circ2 = Area::Circle { left: 10.0, top: 10.0, radius: 5.0 };
   assert!(circ2.hit_test(&Point2D::new(10.0, 12.0)));
}

#[test]
fn test_hit_test_rectangle() {
   let rect1 = Area::Rectangle { top_left: (1.0, 7.0), bottom_right: (15.0, 10.0) };
   assert!(!rect1.hit_test(&Point2D::new(10.0, 5.0)));
   let rect2 = Area::Rectangle { top_left: (8.0, 10.0), bottom_right: (20.0, 12.0) };
   assert!(rect2.hit_test(&Point2D::new(10.0, 12.0)));
}

#[test]
fn test_hit_test_polygon() {
   let poly1 = Area::Polygon { points: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0] };
   assert!(!poly1.hit_test(&Point2D::new(10.0, 5.0)));
   let poly2 = Area::Polygon { points: vec![7.0, 7.5, 8.2, 9.0, 11.0, 12.0] };
   assert!(!poly2.hit_test(&Point2D::new(10.0, 5.0)));
}
