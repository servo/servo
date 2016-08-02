/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Tests for parsing and serialization of values/properties

use cssparser::Parser;

fn parse<T, F: Fn(&mut Parser) -> Result<T, ()>>(f: F, s: &str) -> Result<T, ()> {
    let mut parser = Parser::new(s);
    f(&mut parser)
}

// This is a macro so that the file/line information
// is preserved in the panic
macro_rules! assert_roundtrip {
    ($fun:expr, $input:expr, $output:expr) => {
        let parsed = $crate::parsing::parse($fun, $input)
                        .expect(&format!("Failed to parse {}", $input));
        let mut serialized = String::new();
        ::cssparser::ToCss::to_css(&parsed, &mut serialized)
                        .expect("Failed to serialize");
        assert_eq!(serialized, $output);
    }
}


mod basic_shape;
mod position;