/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, ParserInput};
use euclid::{Scale, Size2D};
use servo_arc::Arc;
use style::applicable_declarations::CascadePriority;
use style::context::QuirksMode;
use style::custom_properties::{
    CustomPropertiesBuilder, CustomPropertiesMap, Name, SpecifiedValue,
};
use style::media_queries::{Device, MediaType};
use style::properties::{CustomDeclaration, CustomDeclarationValue};
use style::rule_tree::CascadeLevel;
use style::stylesheets::layer_rule::LayerOrder;
use test::{self, Bencher};

fn cascade(
    name_and_value: &[(&str, &str)],
    inherited: Option<&Arc<CustomPropertiesMap>>,
) -> Option<Arc<CustomPropertiesMap>> {
    let declarations = name_and_value
        .iter()
        .map(|&(name, value)| {
            let mut input = ParserInput::new(value);
            let mut parser = Parser::new(&mut input);
            let name = Name::from(name);
            let value = CustomDeclarationValue::Value(SpecifiedValue::parse(&mut parser).unwrap());
            CustomDeclaration { name, value }
        })
        .collect::<Vec<_>>();

    let device = Device::new(
        MediaType::screen(),
        QuirksMode::NoQuirks,
        Size2D::new(800., 600.),
        Scale::new(1.0),
    );
    let mut builder = CustomPropertiesBuilder::new(inherited, &device);

    for declaration in &declarations {
        builder.cascade(
            declaration,
            CascadePriority::new(CascadeLevel::same_tree_author_normal(), LayerOrder::root()),
        );
    }

    builder.build()
}

#[bench]
fn cascade_custom_simple(b: &mut Bencher) {
    b.iter(|| {
        let parent = cascade(&[("foo", "10px"), ("bar", "100px")], None);

        test::black_box(cascade(
            &[("baz", "calc(40em + 4px)"), ("bazz", "calc(30em + 4px)")],
            parent.as_ref(),
        ))
    })
}
