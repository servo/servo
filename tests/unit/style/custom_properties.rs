/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, ParserInput};
use euclid::{Scale, Size2D};
use servo_arc::Arc;
use style::applicable_declarations::CascadePriority;
use style::context::QuirksMode;
use style::custom_properties::{
    ComputedCustomProperties, CustomPropertiesBuilder, DeferFontRelativeCustomPropertyResolution,
    Name, SpecifiedValue,
};
use style::font_metrics::FontMetrics;
use style::media_queries::{Device, MediaType};
use style::properties::style_structs::Font;
use style::properties::{ComputedValues, CustomDeclaration, CustomDeclarationValue, StyleBuilder};
use style::rule_cache::RuleCacheConditions;
use style::rule_tree::CascadeLevel;
use style::servo::media_queries::FontMetricsProvider;
use style::stylesheets::container_rule::ContainerSizeQuery;
use style::stylesheets::layer_rule::LayerOrder;
use style::stylesheets::UrlExtraData;
use style::stylist::Stylist;
use style::values::computed::font::GenericFontFamily;
use style::values::computed::{Context, Length};
use test::{self, Bencher};
use url::Url;

#[derive(Debug)]
struct DummyMetricsProvider;

impl FontMetricsProvider for DummyMetricsProvider {
    fn query_font_metrics(
        &self,
        _vertical: bool,
        _font: &Font,
        _base_size: Length,
        _in_media_query: bool,
        _retrieve_math_scales: bool,
    ) -> FontMetrics {
        Default::default()
    }
    fn base_size_for_generic(&self, _: GenericFontFamily) -> Length {
        Length::new(16.)
    }
}

fn cascade(
    name_and_value: &[(&str, &str)],
    inherited: &ComputedCustomProperties,
) -> ComputedCustomProperties {
    let dummy_url_data = UrlExtraData::from(Url::parse("about:blank").unwrap());
    let declarations = name_and_value
        .iter()
        .map(|&(name, value)| {
            let mut input = ParserInput::new(value);
            let mut parser = Parser::new(&mut input);
            let name = Name::from(name);
            let value = CustomDeclarationValue::Value(Arc::new(
                SpecifiedValue::parse(&mut parser, &dummy_url_data).unwrap(),
            ));
            CustomDeclaration { name, value }
        })
        .collect::<Vec<_>>();

    let initial_style = ComputedValues::initial_values_with_font_override(Font::initial_values());
    let device = Device::new(
        MediaType::screen(),
        QuirksMode::NoQuirks,
        Size2D::new(800., 600.),
        Scale::new(1.0),
        Box::new(DummyMetricsProvider),
        initial_style,
    );
    let stylist = Stylist::new(device, QuirksMode::NoQuirks);
    let mut builder = StyleBuilder::new(stylist.device(), Some(&stylist), None, None, None, false);
    builder.custom_properties = inherited.clone();
    let mut rule_cache_conditions = RuleCacheConditions::default();
    let mut context = Context::new(
        builder,
        stylist.quirks_mode(),
        &mut rule_cache_conditions,
        ContainerSizeQuery::none(),
    );
    let mut builder = CustomPropertiesBuilder::new(&stylist, &mut context);

    for declaration in &declarations {
        builder.cascade(
            declaration,
            CascadePriority::new(CascadeLevel::same_tree_author_normal(), LayerOrder::root()),
        );
    }

    builder.build(DeferFontRelativeCustomPropertyResolution::No);
    context.builder.custom_properties
}

#[bench]
fn cascade_custom_simple(b: &mut Bencher) {
    b.iter(|| {
        let parent = cascade(
            &[("foo", "10px"), ("bar", "100px")],
            &ComputedCustomProperties::default(),
        );

        test::black_box(cascade(
            &[("baz", "calc(40em + 4px)"), ("bazz", "calc(30em + 4px)")],
            &parent,
        ))
    })
}
