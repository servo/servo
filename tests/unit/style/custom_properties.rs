/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, ParserInput};
use servo_arc::Arc;
use style::custom_properties::{self, Name, SpecifiedValue, CustomPropertiesMap};
use style::properties::DeclaredValue;
use test::{self, Bencher};

fn cascade(
    name_and_value: &[(&str, &str)],
    inherited: Option<&Arc<CustomPropertiesMap>>,
) -> Option<Arc<CustomPropertiesMap>> {
    let values = name_and_value.iter().map(|&(name, value)| {
        let mut input = ParserInput::new(value);
        let mut parser = Parser::new(&mut input);
        (Name::from(name), SpecifiedValue::parse(&mut parser).unwrap())
    }).collect::<Vec<_>>();

    let mut custom_properties = None;
    let mut seen = Default::default();
    for &(ref name, ref val) in &values {
        custom_properties::cascade(
            &mut custom_properties,
            inherited,
            &mut seen,
            name,
            DeclaredValue::Value(val)
        )
    }

    custom_properties::finish_cascade(custom_properties, inherited)
}

#[bench]
fn cascade_custom_simple(b: &mut Bencher) {
    b.iter(|| {
        let parent = cascade(&[
            ("foo", "10px"),
            ("bar", "100px"),
        ], None);

        test::black_box(cascade(&[
            ("baz", "calc(40em + 4px)"),
            ("bazz", "calc(30em + 4px)"),
        ], parent.as_ref()))
    })
}
