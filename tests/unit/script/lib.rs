/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(plugin)]
#![plugin(plugins)]

use app_units::Au;
use cssparser::{Parser, SourcePosition};
use script::dom::htmlimageelement::{parse_a_sizes_attribute, Size};
use style::media_queries::*;
use style::values::specified;
use util::str::DOMString;

#[test]
fn some_parse_sizes_test() {
    let result = parse_a_sizes_attribute(DOMString::from("(min-width: 900px) 1000px,
            (max-width: 900px) and (min-width: 400px) 50em,
            100vw           "),
            None);
    assert_eq!(result.len(), 3);
}

#[test]
fn some_parse_sizes_1_test() {
    let mut result = parse_a_sizes_attribute(DOMString::from("(min-width: 900px) 1000px,
            (max-width: 900px) and (min-width: 400px) 50em,
            100vw     "),
            None);
    result.pop();
    let mut component_secondlast = result.pop();
    if component_secondlast.is_some() {
        let component_query = component_secondlast.unwrap().query;
        if component_query.is_some() {
            let component_query_expr = component_query.unwrap().expressions;
            assert_eq!(component_query_expr.len() , 2);
        }
    }
}

#[test]
fn some_parse_sizes_2_test() {
    let mut result = parse_a_sizes_attribute(DOMString::from("(min-width: 900px) 1000px,
            (max-width: 900px) and (min-width: 400px) 50em,
            100vw     "),
            None);
    result.pop();
    result.pop();
    let mut component_first = result.pop();
    if component_first.is_some() {
      let component_query = component_first.unwrap().query;
          if component_query.is_some() {
                let component_query_expr = component_query.unwrap().expressions;
                match component_query_expr[0] {
                   Expression::Width(Range::Min(w)) => assert!(w == specified::Length::Absolute(Au::from_px(900))),
            _ => panic!("wrong expression type"),
            }
    }
    }
}

#[test]
fn some_parse_sizes_3_test() {
    let mut result = parse_a_sizes_attribute(DOMString::from("(min-width: 900px) 1000px,
            (max-width: 900px) and (min-width: 400px) 50em      ,
            100vw     "),
            None);
    result.pop();
    let mut component_secondlast = result.pop();
    if component_secondlast.is_some() {
      let component_query = component_secondlast.unwrap().query;
          if component_query.is_some() {
                let component_query_expr = component_query.unwrap().expressions;
                match component_query_expr[0] {
                   Expression::Width(Range::Max(w)) => assert!(w == specified::Length::Absolute(Au::from_px(900))),
            _ => panic!("wrong expression type"),
            }
                match component_query_expr[1] {
            Expression::Width(Range::Min(w)) => assert!(w == specified::Length::Absolute(Au::from_px(400))),
            _ => panic!("wrong expression type"),
        }
    }
    }
}
extern crate app_units;
extern crate cssparser;
extern crate msg;
extern crate script;
extern crate style;
extern crate url;
extern crate util;

#[cfg(test)] mod origin;
#[cfg(all(test, target_pointer_width = "64"))] mod size_of;
#[cfg(test)] mod textinput;
#[cfg(test)] mod dom {
    mod bindings;
    mod blob;
    mod xmlhttprequest;
}
